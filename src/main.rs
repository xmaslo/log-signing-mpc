mod create_communication_channel;
mod key_generation;

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use create_communication_channel::{Room, receive_broadcast};

use futures::{SinkExt, StreamExt};
use anyhow::{Result};

use rocket::data::{ByteUnit, Limits};

use rocket::http::Status;
use rocket::State;
use crate::create_communication_channel::Db;
use crate::key_generation::generate_keys;

use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{ProtocolMessage};
use round_based::{AsyncProtocol, Msg};
use tokio::spawn;
use serde::{Deserialize, Serialize};


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

    thread::sleep(Duration::from_secs(15));

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    let file_name: String = format!("local_share{}.json", server_id);

    // Run the keygen function
    let keygen_result = generate_keys(Path::new(&file_name), server_id, receiving_stream, outgoing_sink).await;

    // Check the result
    println!("Keygen result: {:?}", keygen_result);

    Status::Ok
}



#[rocket::post("/sign/<room_id>", data = "<data>")]
async fn sign(db: &State<Db>, server_id: &State<ServerIdState>, data: String, room_id: u16) -> Status
{

    let urls = data.split(',').map(|s| s.to_string()).collect();
    let server_id = server_id.server_id.lock().unwrap().clone();

    // No check if the id is not already in use
    let (receiving_stream, outgoing_sink)
            = db.create_room::<Msg<ProtocolMessage>>(server_id, room_id, urls).await;

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

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

    let file_name: String = format!("local_share{}.json", id);

    // TODO: I will use these lines when implementing TLS
    // let server_future = tokio::spawn(async { rocket_instance.launch().await });
    // let (server_result) = tokio::join!(server_future);

    // Check the results
    // println!("Rocket server result: {:?}", server_result);

    let _ = rocket_instance.launch().await.expect("TODO: panic message");

    Ok(())
}