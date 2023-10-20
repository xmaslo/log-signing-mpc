extern crate core;
extern crate hex;

use crate::rocket_instances;

use crate::mpc::operations::{
    check_signature,
    signing,
    key_generation,
};

use crate::mpc::utils::{
    hex2string,
    local_share_utils,
    check_timestamp
};

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
    State,
    response::status,
    http::Status,
    data::ToByteUnit,
    Data,
};

use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::{
    keygen::ProtocolMessage,
    sign::{OfflineProtocolMessage, PartialSignature},
};
use tokio::sync::RwLock;
use tokio::io::AsyncReadExt;

use crate::mpc::utils::parse_signature_json::EndpointSignatureData;

#[rocket::post("/key_gen/<room_id>", data = "<data>")]
pub async fn key_gen(
    db: &State<rocket_instances::SharedDb>,
    config_state: &State<rocket_instances::ServerConfigState>,
    data: String,
    room_id: u16,
) -> Result<&'static str, status::Forbidden<&'static str>> {

    let urls: Vec<String> = data.split(',').map(|s| s.to_string()).collect();
    let mpc_config = config_state.config().lock().unwrap().clone();

    let (receiving_stream, outgoing_sink) =
        db.create_room::<ProtocolMessage>(mpc_config.server_id(), room_id, &urls).await;

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    let generation_result =
        key_generation::generate_keys(mpc_config.server_id(),
                                      receiving_stream,
                                      outgoing_sink,
                                      mpc_config.threshold(),
                                      mpc_config.number_of_parties()).await;

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
pub async fn verify(config_state: &State<rocket_instances::ServerConfigState>,
                    data: String) -> Result<&'static str, status::BadRequest<&'static str>> {
    let split_data = data.split(',').map(|s| s.to_string()).collect::<Vec<String>>();
    let signature_hex = split_data[0].clone();

    let signature = hex2string::hex_to_string(signature_hex);
    let data = hex2string::hex_to_string(split_data[1].clone());
    let timestamp = &split_data[2];
    let signed_data = sha256::digest(data + timestamp);

    let (r,s) = check_signature::extract_rs(signature.as_str());
    let msg = BigInt::from_bytes(&hex::decode(signed_data).unwrap());

    let server_id = config_state.config().lock().unwrap().server_id();
    let local_share_file_name = format!("local-share{}.json", server_id);
    let file_contents = local_share_utils::read_file(Path::new(&local_share_file_name));
    match file_contents {
        None => return Err(status::BadRequest(Some("local-share.json is missing. Generate it first with the /keygen endpoint"))),
        _ => {}
    }
    let file_contents = file_contents.unwrap();

    let public_key = check_signature::get_public_key(file_contents.as_str());

    return if check_signature::check_sig(&r, &s, &msg, &public_key) {
        Ok("Valid signature")
    } else {
        Err(status::BadRequest(Some("Invalid signature")))
    }
}

#[rocket::post("/sign/<room_id>", data = "<data>")]
pub async fn sign(
    db: &State<rocket_instances::SharedDb>,
    config_state: &State<rocket_instances::ServerConfigState>,
    signer: &State<Arc<RwLock<signing::Signer>>>,
    data: String,
    room_id: u16
) -> Result<String, status::BadRequest<&'static str>> {
    let server_id: u16 = config_state.config().lock().unwrap().server_id();

    let esig_data = match serde_json::from_str::<EndpointSignatureData>(data.as_str()) {
        Ok(ed) => ed,
        Err(_) => return Err(status::BadRequest(Some("Unable to parse json data")))
    };

    let original_data = hex2string::hex_to_string(String::from(esig_data.data_to_sign()));

    let timestamp = match esig_data.timestamp().clone().parse::<u64>() {
        Ok(v) => v,
        Err(_) => return Err(status::BadRequest(Some("TIMESTAMP IN BAD FORMAT")))
    };
    if !check_timestamp::verify_timestamp_10_minute_window(timestamp) {
        let too_old_timestamp: &str = "TIMESTAMP IS OLDER THAN 10 MINUTES";
        println!("{}", too_old_timestamp);
        return Err(status::BadRequest(Some(too_old_timestamp)));
    }

    let hash = sha256::digest(original_data + esig_data.timestamp());

    let participant_ids = esig_data.participant_ids();
    let participant_urls = esig_data.participant_urls();

    println!(
        "My ID: {}\n\
         Other server IDs: {:?}\n\
         Other server URLs: {:?}\n\
         Data to sign: {}\n", server_id, &participant_ids, &participant_urls, hash
    );

    if !signer.read().await.is_offline_stage_complete(&participant_ids) {
        let arbitrary_server_id = match signer.read().await.
            real_to_arbitrary_index(&participant_ids) {
            None => return Err(status::BadRequest(Some("Second participant is invalid"))),
            Some(asi) => asi
        };

        let (receiving_stream, outgoing_sink)
            = db.create_room::<OfflineProtocolMessage>(arbitrary_server_id, room_id, &participant_urls).await;

        let receiving_stream = receiving_stream.fuse();
        tokio::pin!(receiving_stream);
        tokio::pin!(outgoing_sink);

        println!("Beginning offline stage");

        let offline_stage_result = signer.write().await.do_offline_stage(receiving_stream, outgoing_sink, &participant_ids).await;
        match offline_stage_result {
            Err(e) => {
                println!("{}", e.to_string());
                return Err(status::BadRequest(Some("Offline stage failed")));
            },
            _ => {}
        }
    }

    let (receiving_stream, outgoing_sink)
        = db.create_room::<PartialSignature>(server_id, room_id, &participant_urls).await;

    thread::sleep(Duration::from_secs(2)); // wait for others to finish offline stage

    println!("Beginning online stage");

    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    let signature = signer.read().await.sign_hash(&hash, receiving_stream, outgoing_sink, participant_ids)
        .await
        .expect("Message could not be signed");

    Ok(signature)
}

// This function creates the communication channels between the servers
// The messages sent to the outgoing sink will be received by other servers in their receiving_stream
// And vice versa, the messages sent by other servers to their outgoing sink will be received by this server in its receiving_stream
#[rocket::post("/receive_broadcast/<room_id>", data = "<data>")]
pub async fn receive_broadcast(db: &State<rocket_instances::SharedDb>,
                               room_id: u16,
                               data: Data<'_>) -> Result<Status, std::io::Error> {
    let mut buffer = Vec::new();
    let data_length = data.open(1.mebibytes()).read_to_end(&mut buffer).await?;

    println!("Received data length: {} bytes", data_length);

    let message = String::from_utf8(buffer).unwrap_or_else(|_| String::from("Invalid UTF-8"));

    if let Some(room) = db.get_room(room_id).await {
        room.receive(message).await;
    }

    Ok(Status::Ok)
}