mod create_communication_channel;
mod key_generator;

use std::path::Path;
use create_communication_channel::{create_communication_channels, Room, receive_broadcast};

use futures::{Sink, SinkExt, StreamExt};

use std::sync::{Arc};
use std::pin::Pin;

use rocket::http::Status;
use rocket::State;
use rocket::Config;
use round_based::Msg;

use serde::Serialize;

use tokio::sync::RwLock;
use crate::key_generator::KeyGenerator;


#[rocket::post("/send_broadcast", data = "<message>")]
async fn send_broadcast(outgoing_sink: &State<OutgoingSink>, message: String) -> Status {
    outgoing_sink.receive(message).await;
    Status::Ok
}

struct OutgoingSink {
    outgoing_sink: Arc<RwLock<Pin<Box<dyn Sink<Msg<String>, Error=anyhow::Error> + Send + Sync>>>>,
}

impl OutgoingSink {
    pub fn new(sink: Pin<Box<dyn Sink<Msg<String>, Error=anyhow::Error> + Send + Sync>>) -> Self {
        Self {
            outgoing_sink: Arc::new(RwLock::new(sink))
        }
    }

    pub async fn receive(&self, message: String) {
        let msg_value: serde_json::Value = serde_json::from_str(&message).unwrap();
        //TODO: handle error more gracefully
        let receiver = msg_value["receiver"].as_u64().map(|r| r as u16).unwrap();
        let sender = msg_value["sender"].as_u64().map(|r| r as u16).unwrap();
        let body = msg_value["body"].as_str().unwrap().to_string();

        let msg = Msg {
            sender,
            receiver: Some(receiver),
            body,
        };
        let mut guard = self.outgoing_sink.write().await;
        let mut sink = guard.as_mut();
        sink.send(msg).await.unwrap();
    }
}

#[rocket::post("/init_room", data = "<urls>")]
async fn init_room(room: &State<Room>, urls: String) -> Status
{
    let urls = urls.split(',').map(|s| s.to_string()).collect();
    // Necessary step 1, calling the .init_room with correct urls on the rocket managed state
    room.init_room(&urls).await;
    Status::Ok
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    // id that will be used to filter out messages
    let id = args.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(0);
    let port = args.get(2).and_then(|s| s.parse::<u16>().ok()).unwrap_or(8000);


    let (mut receiving_stream, outgoing_sink, room)
        = create_communication_channels(id);

    // The receiving_stream will be passed to the multisig library to work with it instead
    tokio::spawn(async move {
        while let Some(message) = receiving_stream.next().await {
            println!("Received: {:?}", message);
        }
    });

    // The outgoing sink will be passed to the multisig library to work with it instead
    let outgoing_sink_managed = OutgoingSink::new(Box::pin(outgoing_sink));

    // let kg = KeyGenerator::new(0);
    // kg.run(Path::new("file_name"), receiving_stream, outgoing_sink);

    let figment = rocket::Config::figment()
        .merge(("address", "127.0.0.1"))
        .merge(("port", port))
        .merge(("workers", 4))
        .merge(("log_level", "normal"));

    rocket::custom(figment)
        // Necessary step 3
        .mount("/", rocket::routes![receive_broadcast, send_broadcast, init_room])
        // Necessary step 2
        .manage(room)
        .manage(outgoing_sink_managed)
        .launch()
        .await?;

    Ok(())
}