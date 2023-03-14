use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};

use std::sync::{
    atomic::{AtomicU16},
    Arc, Mutex,
};
use rocket::data::ToByteUnit;
use rocket::http::Status;
use rocket::State;
use tokio::sync::{RwLock};



// Handling the messages received from the other servers
#[rocket::post("/receive_broadcast", data = "<message>")]
async fn receive_broadcast(room: &State<Room>, message: String) -> Status {
    room.receive(message).await;
    Status::Ok
}

struct Room {
    receiving_sink: Arc<RwLock<Box<dyn Sink<String, Error = futures::channel::mpsc::SendError> + Send + Sync + Unpin>>>,
}

// TODO: Add the logic of multiple groups
impl Room {
    pub fn new(sink: Box<dyn Sink<String, Error = futures::channel::mpsc::SendError> + Send + Sync + Unpin>) -> Self {
        Self {
            receiving_sink: Arc::new(RwLock::new(sink)),
        }
    }

    pub async fn receive(&self, message: String) {
        let mut guard = self.receiving_sink.write().await;
        let mut sink = guard.as_mut();
        sink.send(message).await.unwrap();
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let (sink, mut stream) = futures::channel::mpsc::unbounded();

    // The sink will be passed to the multisig library to work with it
    tokio::spawn(async move {
        while let Some(message) = stream.next().await {
            println!("Received: {}", message);
        }
    });

    rocket::build()
        .mount("/", rocket::routes![receive_broadcast])
        .manage(Room::new(Box::new(sink)))
        .launch()
        .await?;

    Ok(())
}