mod create_communication_channel;
mod key_generation;
mod signing;
mod check_timestamp;

use std::sync::{Mutex};
use std::thread;
use std::time::Duration;
use create_communication_channel::{receive_broadcast};

use futures::{StreamExt};
use anyhow::{Result};

use rocket::data::{ByteUnit, Limits};

use rocket::http::Status;
use rocket::State;
use crate::create_communication_channel::Db;
use crate::key_generation::generate_keys;

use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{ProtocolMessage};
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::sign::{OfflineProtocolMessage, PartialSignature};
use crate::check_timestamp::verify_timestamp_10_minute_window;
use crate::signing::KeyGenerator;


#[rocket::post("/key_gen/<room_id>", data = "<data>")]
async fn key_gen(
    db: &State<Db>,
    server_id: &State<ServerIdState>,
    data: String,
    room_id: u16,
) -> Status {

    let urls = data.split(',').map(|s| s.to_string()).collect();
    let server_id = server_id.server_id.lock().unwrap().clone();

    let (receiving_stream, outgoing_sink) =
        db.create_room::<ProtocolMessage>(server_id, room_id, urls).await;

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    generate_keys(server_id, receiving_stream, outgoing_sink).await;

    Status::Ok
}

#[rocket::post("/sign/<room_id>", data = "<data>")]
async fn sign(
    db: &State<Db>,
    server_id: &State<ServerIdState>,
    data: String,
    room_id: u16
) -> Status {
    let server_id = server_id.server_id.lock().unwrap().clone();

    let splitted_data = data.split(',').map(|s| s.to_string()).collect::<Vec<String>>();

    let participant2 = splitted_data[0].as_str().parse::<u16>().unwrap();
    let participants = vec![server_id, participant2];

    let mut url = Vec::new();
    url.push(splitted_data[1].clone());

    let mut hash = splitted_data[2].clone();

    let parsed_unix_seconds = splitted_data[3].clone().parse::<u64>();
    let timestamp = match parsed_unix_seconds {
        Ok(v) => v,
        Err(_) => return Status::BadRequest,
    };

    verify_timestamp_10_minute_window(timestamp);

    hash += &splitted_data[3];

    println!(
        "My ID: {}\n\
         Other server ID: {}\n\
         Other server URL: {}\n\
         Data to sign: {}\n", server_id, participant2, url[0], hash
    );

    let mut kg = KeyGenerator::new(participants.clone(), participants.len(), server_id);
    let server_id = kg.get_different_party_index();

    // No check if the id is not already in use
    let (receiving_stream, outgoing_sink)
            = db.create_room::<OfflineProtocolMessage>(server_id, room_id, url.clone()).await;

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    kg.do_offline_stage(receiving_stream, outgoing_sink).await.unwrap();

    let (receiving_stream, outgoing_sink)
        = db.create_room::<PartialSignature>(server_id, room_id + 1, url).await;

    thread::sleep(Duration::from_secs(2)); // wait for others to finish offline stage

    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    kg.sign_hash(&hash, receiving_stream, outgoing_sink)
        .await
        .expect("Message could not be signed");

    Status::Ok
}

struct ServerIdState{
    server_id: Mutex<u16>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    // id that will be used to filter out messages
    let id = args.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    let port = args.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(8000);

    // Create a figment with the desired configuration
    let figment = rocket::Config::figment()
        .merge(("address", "127.0.0.1"))
        .merge(("port", port))
        .merge(("workers", 4))
        .merge(("log_level", "normal"))
        .merge(("limits", Limits::new().limit("json", ByteUnit::from(1048576 * 1024))));

    let rocket_instance = rocket::custom(figment)
        .mount("/", rocket::routes![receive_broadcast,
            key_gen,
            sign])
        .manage(Db::empty())
        .manage(ServerIdState{server_id: Mutex::new(id)});

    // TODO: I will use these lines when implementing TLS
    // let server_future = tokio::spawn(async { rocket_instance.launch().await });
    // let (server_result) = tokio::join!(server_future);

    // Check the results
    // println!("Rocket server result: {:?}", server_result);

    let _ = rocket_instance.launch().await.expect("TODO: panic message");

    Ok(())
}