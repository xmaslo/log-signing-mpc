extern crate core;
extern crate hex;

mod create_communication_channel;
mod key_generation;
mod signing;
mod check_timestamp;
mod check_signature;
mod common;
mod rocket_instances;

use sha256::digest;
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
    http::{Status, Header},
    State,
    response::{self, Responder, status, status::Custom},
    Request
};

use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::{
    keygen::ProtocolMessage,
    sign::{OfflineProtocolMessage, PartialSignature},
};

use crate::key_generation::generate_keys;
use crate::check_timestamp::verify_timestamp_10_minute_window;
use crate::check_signature::{check_sig, extract_rs, get_public_key};
use crate::common::read_file;

use crate::{
    create_communication_channel::{Db},
    rocket_instances::{rocket_with_client_auth, rocket_without_client_auth, ServerIdState, SharedDb},
    signing::{Signer},
};

struct Cors<R>(R);

impl<'r, R: Responder<'r, 'static>> Responder<'r, 'static> for Cors<R> {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let mut response = self.0.respond_to(req)?;
        response.set_header(Header::new("Access-Control-Allow-Origin", "*"));
        Ok(response)
    }
}


#[rocket::post("/key_gen/<room_id>", data = "<data>")]
async fn key_gen(
    db: &State<SharedDb>,
    server_id: &State<ServerIdState>,
    data: String,
    room_id: u16,
) -> Custom<Cors<status::Accepted<String>>> {

    let urls = data.split(',').map(|s| s.to_string()).collect();
    let server_id = *server_id.server_id.lock().unwrap();

    let (receiving_stream, outgoing_sink) =
        db.create_room::<ProtocolMessage>(server_id, room_id, urls).await;

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    let generation_result = generate_keys(server_id, receiving_stream, outgoing_sink).await;

    let status = match generation_result {
        Ok(_) => "Ok".to_string(),
        Err(e) => e,
    };

    return if status == "Ok" {
        println!("Keys were successfully generated");
        Custom(Status::Accepted, Cors(status::Accepted(Some(status))))
    } else {
        println!("Keys could NOT be generated");
        Custom(Status::Accepted, Cors(status::Accepted(Some(status))))
    }
}

#[rocket::post("/verify", data = "<data>")]
async fn verify(server_id: &State<ServerIdState>, data: String) -> Custom<Cors<status::Accepted<String>>> {
    let splitted_data = data.split(';').map(|s| s.to_string()).collect::<Vec<String>>();
    let signature = splitted_data[0].clone();
    let signed_data = digest(splitted_data[1].clone() + &splitted_data[2]);
    println!("{}", signed_data);

    let (r,s) = extract_rs(signature.as_str());
    let msg = BigInt::from_bytes(&hex::decode(signed_data).unwrap());

    let server_id = *server_id.server_id.lock().unwrap();
    let local_share_file_name = format!("local-share{}.json", server_id);
    let file_contents = read_file(Path::new(&local_share_file_name));
    let public_key = get_public_key(file_contents.as_str());

    let response = status::Accepted(Some(check_sig(&r, &s, &msg, &public_key).to_string()));

    Custom(Status::Accepted, Cors(response))
}

#[rocket::post("/sign/<room_id>", data = "<data>")]
async fn sign(
    db: &State<SharedDb>,
    server_id: &State<ServerIdState>,
    data: String,
    room_id: u16
) -> Custom<Cors<status::Accepted<String>>> {
    let server_id = *server_id.server_id.lock().unwrap();

    let splitted_data = data.split(',').map(|s| s.to_string()).collect::<Vec<String>>();

    let participant2 = splitted_data[0].as_str().parse::<u16>().unwrap();
    let participants = vec![server_id, participant2];

    let url = vec![splitted_data[1].clone()];

    let file_hash = splitted_data[2].clone();

    let parsed_unix_seconds = splitted_data[3].clone().parse::<u64>();
    let timestamp = match parsed_unix_seconds {
        Ok(v) => v,
        Err(_) => return Custom(Status::BadRequest, Cors(status::Accepted(Some("TIMESTAMP IN BAD FORMAT".to_string())))),
    };

    if !verify_timestamp_10_minute_window(timestamp) {
        let too_old_timestamp: String = "TIMESTAMP IS OLDER THAN 10 MINUTES".to_string();
        println!("{}", too_old_timestamp.as_str());
        return Custom(Status::BadRequest, Cors(status::Accepted(Some(too_old_timestamp))));
    }

    let hash = digest(file_hash + &splitted_data[3]);

    println!(
        "My ID: {}\n\
         Other server ID: {}\n\
         Other server URL: {}\n\
         Data to sign: {}\n", server_id, participant2, url[0], hash
    );

    let mut signer = Signer::new(participants.clone(), participants.len(), server_id);
    let server_id = signer.get_different_party_index();

    // No check if the id is not already in use
    let (receiving_stream, outgoing_sink)
        = db.create_room::<OfflineProtocolMessage>(server_id, room_id, url.clone()).await;

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    signer.do_offline_stage(receiving_stream, outgoing_sink).await.unwrap();

    let (receiving_stream, outgoing_sink)
        = db.create_room::<PartialSignature>(server_id, room_id + 1, url).await;

    thread::sleep(Duration::from_secs(2)); // wait for others to finish offline stage

    thread::sleep(Duration::from_secs(15));

    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    let signature = signer.sign_hash(&hash, receiving_stream, outgoing_sink)
        .await
        .expect("Message could not be signed");

    let response = status::Accepted(Some(signature));

    Custom(Status::Accepted, Cors(response))
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

    // Run the Rocket instances concurrently
    let server_future_protected = tokio::spawn(async { rocket_instance_protected.launch().await });
    let server_future_public = tokio::spawn(async { rocket_instance_public.launch().await });

    let (protected_result, public_result) = tokio::join!(server_future_protected, server_future_public);

    // Check the results
    println!("Protected Rocket server result: {:?}", protected_result);
    println!("Public Rocket server result: {:?}", public_result);

    Ok(())
}