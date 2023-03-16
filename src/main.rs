mod create_communication_channel;

use create_communication_channel::{create_communication_channels, Room, receive_broadcast};

use futures::{Sink, SinkExt, StreamExt};

use std::sync::{Arc};
use rocket::http::Status;
use rocket::State;
use rocket::Config;
use tokio::sync::RwLock;


#[rocket::post("/send_broadcast", data = "<message>")]
async fn send_broadcast(outgoing_sink: &State<OutgoingSink>, message: String) -> Status {
    outgoing_sink.receive(message).await;
    Status::Ok
}

struct OutgoingSink {
    outgoing_sink: Arc<RwLock<Box<dyn Sink<String, Error=futures::channel::mpsc::SendError> + Send + Sync + Unpin>>>,
}

impl OutgoingSink {
    pub fn new(sink: Box<dyn Sink<String, Error=futures::channel::mpsc::SendError> + Send + Sync + Unpin>) -> Self {
        Self {
            outgoing_sink: Arc::new(RwLock::new(sink))
        }
    }

    pub async fn receive(&self, message: String) {
        let mut guard = self.outgoing_sink.write().await;
        let mut sink = guard.as_mut();
        sink.send(message).await.unwrap();
    }
}

#[rocket::post("/init_room", data = "<urls>")]
async fn init_room(room: &State<Room>, urls: String) -> Status {
    let urls = urls.split(',').map(|s| s.to_string()).collect();
    // Necessary step 1, calling the .init_room with correct urls on the rocket managed state
    room.init_room(&urls).await;
    Status::Ok
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (mut receiving_stream, outgoing_sink, room) = create_communication_channels();

    // The receiving_stream will be passed to the multisig library to work with it instead
    tokio::spawn(async move {
        while let Some(message) = receiving_stream.next().await {
            println!("Received: {}", message);
        }
    });

    // The outgoing sink will be passed to the multisig library to work with it instead
    let outgoing_sink_managed = OutgoingSink::new(Box::new(outgoing_sink));

    let args: Vec<String> = std::env::args().collect();
    let port = args.get(1).and_then(|s| s.parse::<u16>().ok()).unwrap_or(8000);

    rocket::build()
        // Necessary step 3
        .mount("/", rocket::routes![receive_broadcast, send_broadcast, init_room])
        // Necessary step 2
        .manage(room)
        .manage(outgoing_sink_managed)
        .launch()
        .await?;

    Ok(())
}