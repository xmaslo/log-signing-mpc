extern crate core;
extern crate hex;

mod create_communication_channel;

mod rocket_instances;
mod utils;
mod mpc_operations;

use sha256;
use std::{
    path::Path,
    sync::{Arc},
    thread,
    time::Duration,
};

use curv::arithmetic::Converter;
use curv::BigInt;

use futures::StreamExt;
use anyhow::Result;

use rocket::{
    data::{ByteUnit, Limits},
    State,
    response::status,
};

use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::{
    keygen::ProtocolMessage,
    sign::{OfflineProtocolMessage, PartialSignature},
};
use tokio::sync::RwLock;

use crate::utils::check_timestamp::verify_timestamp_10_minute_window;
use crate::utils::local_share_utils::read_file;

use crate::{
    create_communication_channel::{Db},
    rocket_instances::{rocket_with_client_auth, rocket_without_client_auth, ServerIdState, SharedDb},
};

#[rocket::post("/key_gen/<room_id>", data = "<data>")]
async fn key_gen(
    db: &State<SharedDb>,
    server_id: &State<ServerIdState>,
    data: String,
    room_id: u16,
) -> Result<&'static str, status::Forbidden<&'static str>> {

    let urls = data.split(',').map(|s| s.to_string()).collect();
    let server_id = *server_id.server_id.lock().unwrap();

    let (receiving_stream, outgoing_sink) =
        db.create_room::<ProtocolMessage>(server_id, room_id, urls).await;

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    let generation_result = mpc_operations::generate_keys(server_id, receiving_stream, outgoing_sink).await;

    let status = match generation_result {
        Ok(_) => "Ok".to_string(),
        Err(e) => e,
    };

    return if status == "Ok" {
        println!("Keys were successfully generated");
        Ok("Keys were successfully generated")
    } else {
        println!("Keys could NOT be generated");
        Err(status::Forbidden(Some("Keys could NOT be generated")))
    }
}

#[rocket::post("/verify", data = "<data>")]
async fn verify(server_id: &State<ServerIdState>, data: String) -> Result<&'static str, status::BadRequest<&'static str>> {
    let split_data = data.split(',').map(|s| s.to_string()).collect::<Vec<String>>();
    let signature_hex = split_data[0].clone();

    let signature = utils::hex_to_string(signature_hex);
    let data = utils::hex_to_string(split_data[1].clone());
    let timestamp = &split_data[2];
    let signed_data = sha256::digest(data + timestamp);

    let (r,s) = mpc_operations::extract_rs(signature.as_str());
    let msg = BigInt::from_bytes(&hex::decode(signed_data).unwrap());

    let server_id = *server_id.server_id.lock().unwrap();
    let local_share_file_name = format!("local-share{}.json", server_id);
    let file_contents = read_file(Path::new(&local_share_file_name));
    match file_contents {
        None => return Err(status::BadRequest(Some("local-share.json is missing. Generate it first with the /keygen endpoint"))),
        _ => {}
    }
    let file_contents = file_contents.unwrap();

    let public_key = mpc_operations::get_public_key(file_contents.as_str());

    return if mpc_operations::check_sig(&r, &s, &msg, &public_key) {
        Ok("Valid signature")
    } else {
        Err(status::BadRequest(Some("Invalid signature")))
    }
}

#[rocket::post("/sign/<room_id>", data = "<data>")]
async fn sign(
    db: &State<SharedDb>,
    server_id: &State<ServerIdState>,
    signer: &State<Arc<RwLock<mpc_operations::Signer>>>,
    data: String,
    room_id: u16
) -> Result<String, status::BadRequest<&'static str>> {
    let server_id: u16 = *server_id.server_id.lock().unwrap();
    let split_data: Vec<String> = data.split(',').map(|s| s.to_string()).collect::<Vec<String>>();
    let participant2: u16 = split_data[0].as_str().parse::<u16>().unwrap();
    let url = vec![split_data[1].clone()];
    let data_to_sign_hex = split_data[2].clone();
    let data_to_sign = utils::hex_to_string(data_to_sign_hex);

    let parsed_unix_seconds = split_data[3].clone().parse::<u64>();
    let timestamp = match parsed_unix_seconds {
        Ok(v) => v,
        Err(_) => return Err(status::BadRequest(Some("TIMESTAMP IN BAD FORMAT")))
    };
    if !verify_timestamp_10_minute_window(timestamp) {
        let too_old_timestamp: &str = "TIMESTAMP IS OLDER THAN 10 MINUTES";
        println!("{}", too_old_timestamp);
        return Err(status::BadRequest(Some(too_old_timestamp)));
    }

    let hash = sha256::digest(data_to_sign + &split_data[3]);

    println!(
        "My ID: {}\n\
         Other server ID: {}\n\
         Other server URL: {}\n\
         Data to sign: {}\n", server_id, participant2, url[0], hash
    );

    if !signer.read().await.is_offline_stage_complete(participant2) {
        let participant_result = signer.write().await.add_participant(participant2);
        match participant_result {
            Err(msg) => return Err(status::BadRequest(Some(msg))),
            _ => {}
        };
        let server_id = signer.read().await.convert_my_real_index_to_arbitrary_one(participant2);

        // No check if the id is not already in use
        let (receiving_stream, outgoing_sink)
            = db.create_room::<OfflineProtocolMessage>(server_id, room_id, url.clone()).await;

        let receiving_stream = receiving_stream.fuse();
        tokio::pin!(receiving_stream);
        tokio::pin!(outgoing_sink);

        println!("Beginning offline stage");

        let offline_stage_result = signer.write().await.do_offline_stage(receiving_stream, outgoing_sink, participant2).await;
        match offline_stage_result {
            Err(e) => {
                println!("{}", e.to_string());
                return Err(status::BadRequest(Some("Offline stage failed")));
            },
            _ => {}
        }
    }

    let (receiving_stream, outgoing_sink)
        = db.create_room::<PartialSignature>(server_id, room_id, url).await;

    thread::sleep(Duration::from_secs(2)); // wait for others to finish offline stage

    println!("Beginning online stage");

    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    let signature = signer.read().await.sign_hash(&hash, receiving_stream, outgoing_sink, participant2)
        .await
        .expect("Message could not be signed");

    Ok(signature)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();

    // id that will be used to filter out messages
    let server_id = args.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    let port = args.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(8000);
    let port_mutual_auth = args.get(3).and_then(|s| s.parse::<u16>().ok()).unwrap_or(3000);

    // TODO: might be good idea to adjust for development and production (https://rocket.rs/v0.4/guide/configuration/)
    // Create a figment with the desired configuration
    let figment = rocket::Config::figment()
        .merge(("address", "0.0.0.0"))
        .merge(("workers", 4))
        .merge(("log_level", "normal"))
        .merge(("limits", Limits::new().limit("json", ByteUnit::from(1048576 * 1024))));


    let shared_db = SharedDb(Arc::new(Db::empty(server_id)));

    // Create two Rocket instances with different ports and TLS settings
    let rocket_instance_protected = rocket_with_client_auth(figment.clone(), server_id , shared_db.clone(), port_mutual_auth);
    let rocket_instance_public = rocket_without_client_auth(figment.clone(), server_id, shared_db.clone(), port);

    let signer = Arc::new(RwLock::new(mpc_operations::Signer::new(server_id)));

    let rocket_instance_protected = rocket_instance_protected.manage(signer.clone());
    let rocket_instance_public = rocket_instance_public.manage(signer.clone());

    // Run the Rocket instances concurrently
    let server_future_protected = tokio::spawn(async { rocket_instance_protected.launch().await });
    let server_future_public = tokio::spawn(async { rocket_instance_public.launch().await });

    let (protected_result, public_result) = tokio::join!(server_future_protected, server_future_public);

    // Check the results
    println!("Protected Rocket server result: {:?}", protected_result);
    println!("Public Rocket server result: {:?}", public_result);

    Ok(())
}