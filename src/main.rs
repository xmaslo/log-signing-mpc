mod create_communication_channel;
mod key_generation;

use std::path::Path;
use std::sync::{Arc, Mutex};
use create_communication_channel::{create_communication_channels, Room, receive_broadcast};

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


#[rocket::post("/init_room/<room_id>", data = "<urls>")]
async fn init_room(db: &State<Db>, room: &State<Room>, room_id: u16, urls: String) -> Status {
    let urls = urls.split(',').map(|s| s.to_string()).collect();

    if let Some(room) = db.get_room(room_id).await {
        // Necessary step 1, calling the .init_room with correct urls on the rocket managed state
        room.init_room(&urls).await;
    } else {
        room.init_room(&urls).await;
    }
    Status::Ok
}


#[rocket::post("/sign", data = "<data>")]
async fn sign(db: &State<Db>, server_id: &State<ServerIdState>, data: String) -> Status
{

    let urls = data.split(',').map(|s| s.to_string()).collect();

    // TODO: Get id and message from data
    let room_id = 1;
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


    let (receiving_stream, outgoing_sink, room)
        = create_communication_channels(id, 0);

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);

    // Create a figment with the desired configuration
    let figment = rocket::Config::figment()
        .merge(("address", "127.0.0.1"))
        .merge(("port", port))
        .merge(("workers", 4))
        .merge(("log_level", "normal"))
        .merge(("limits", Limits::new().limit("json", ByteUnit::from(1048576 * 1024))));

    // Replace `let server = rocket::custom(figment)` with the following lines
    let rocket_instance = rocket::custom(figment)
        .mount("/", rocket::routes![receive_broadcast, init_room, sign])
        .manage(room)
        .manage(Db::empty())
        .manage(ServerIdState{server_id: Mutex::new(id)});

    let file_name: String = format!("local_share{}.json", id);

    // Run the server and the run_keygen function concurrently
    let server_future = tokio::spawn(async { rocket_instance.launch().await });
    let keygen_future = generate_keys(Path::new(&file_name), id, receiving_stream, outgoing_sink);

    let (server_result, keygen_result) = tokio::join!(server_future, keygen_future);

    // Check the results
    println!("Rocket server result: {:?}", server_result);
    println!("Keygen result: {:?}", keygen_result);

    Ok(())
}