mod create_communication_channel;
mod key_generator;

use std::path::Path;
use create_communication_channel::{create_communication_channels, Room, receive_broadcast};

use futures::{Sink, Stream, StreamExt};
use anyhow::{anyhow, Error, Result};

use std::pin::Pin;
use curv::elliptic::curves::Secp256k1;
use futures::stream::Fuse;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{Keygen, LocalKey, ProtocolMessage};
use rocket::data::{ByteUnit, Limits};

use rocket::http::Status;
use rocket::State;
use round_based::{AsyncProtocol, Msg};
use crate::key_generator::generate_keys;

// use crate::key_generator::KeyGenerator;

#[rocket::post("/init_room", data = "<urls>")]
async fn init_room(room: &State<Room>, urls: String) -> Status
{
    let urls = urls.split(',').map(|s| s.to_string()).collect();
    // Necessary step 1, calling the .init_room with correct urls on the rocket managed state
    room.init_room(&urls).await;
    Status::Ok
}
//
// #[rocket::post("/keygen", data = "<file_name>")]
// async fn keygen(kg: &State<KeyGenerator>, file_name: String) -> Status {
//     kg.run(Path::new("file_name"));
//     Status::Ok
// }


async fn run_keygen(
    keygen: Keygen,
    receiving_stream: Pin<&mut Fuse<(impl Stream<Item=Result<Msg<ProtocolMessage>>> + Sized)>>,
    outgoing_sink: Pin<&mut (impl Sink<Msg<ProtocolMessage>, Error=Error> + Sized)>,
) -> Result<LocalKey<Secp256k1>, Error> {
    AsyncProtocol::new(keygen, receiving_stream, outgoing_sink)
        .run()
        .await
        .map_err(|e| anyhow!("protocol execution terminated with error: {}", e))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    // id that will be used to filter out messages
    let id = args.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    let port = args.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(8000);
    let file_name = args.get(3).unwrap();


    let (mut receiving_stream, outgoing_sink, room)
        = create_communication_channels(id);

    let receiving_stream = receiving_stream.fuse();
    tokio::pin!(receiving_stream);
    tokio::pin!(outgoing_sink);


    // let kg = KeyGenerator::new(0, receiving_stream, outgoing_sink);
    // kg.run(Path::new("local-share1.json"));

    // let keygen: Keygen = Keygen::new(id, 2, 3).unwrap();

    // println!("Keygen started: {:?}", keygen);

    let figment = rocket::Config::figment()
        .merge(("address", "127.0.0.1"))
        .merge(("port", port))
        .merge(("workers", 4))
        .merge(("log_level", "normal"))
        .merge(("limits", Limits::new().limit("json", ByteUnit::from(1048576 * 1024))));

    // Replace `let server = rocket::custom(figment)` with the following lines
    let rocket_instance = rocket::custom(figment)
        .mount("/", rocket::routes![receive_broadcast, /*send_broadcast,*/ init_room] /*keygen*/)
        .manage(room);

    // Run the server and the run_keygen function concurrently
    let server_future = tokio::spawn(async { rocket_instance.launch().await });
    let keygen_future = generate_keys(Path::new(file_name), id, receiving_stream, outgoing_sink);

    let (server_result, keygen_result) = tokio::join!(server_future, keygen_future);

    // Check the results
    println!("Rocket server result: {:?}", server_result);
    println!("Keygen result: {:?}", keygen_result);

    Ok(())
}