use futures::{Sink, SinkExt, Stream, StreamExt};

use std::sync::{Arc};
use rocket::http::Status;
use rocket::State;
use tokio::sync::{RwLock};

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

// Handling the sending messages from this server
#[rocket::post("/init_room")]
async fn init_room(room: &State<Room>) -> Status {
    room.init_room().await;
    Status::Ok
}



// Handling the messages received from the other servers
#[rocket::post("/receive_broadcast", data = "<message>")]
async fn receive_broadcast(room: &State<Room>, message: String) -> Status {
    room.receive(message).await;
    Status::Ok
}

struct Room {
    receiving_sink: Arc<RwLock<Box<dyn Sink<String, Error = futures::channel::mpsc::SendError> + Send + Sync + Unpin>>>,
    outgoing_stream: Arc<RwLock<Box<dyn Stream<Item = String> + Send + Sync + Unpin>>>,
    client: reqwest::Client,
}

// TODO: Add the logic of multiple groups
impl Room {
    pub fn new(sink: Box<dyn Sink<String, Error = futures::channel::mpsc::SendError> + Send + Sync + Unpin>,
               stream: Box<dyn Stream<Item = String> + Send + Sync + Unpin>,) -> Self {

        Self {
            receiving_sink: Arc::new(RwLock::new(sink)),
            outgoing_stream: Arc::new(RwLock::new(stream)),
            client: reqwest::Client::new(),
        }
    }

    pub async fn init_room(&self) {
        loop {
            match self.outgoing_stream.write().await.next().await {
                Some(message) => {
                    let message = format!("{} Resended", message);
                    println!("Sending: {}", message);
                    self.client.post("http://localhost:8000/receive_broadcast").body(message).send().await.unwrap();
                }
                None => break,
            }
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
    let (receiving_sink, mut receiving_stream) = futures::channel::mpsc::unbounded();
    let (outgoing_sink, mut outgoing_stream) = futures::channel::mpsc::unbounded();


    // The receiving_stream will be passed to the multisig library to work with it
    tokio::spawn(async move {
        while let Some(message) = receiving_stream.next().await {
            println!("Received: {}", message);
        }
    });

    rocket::build()
        .mount("/", rocket::routes![receive_broadcast, send_broadcast, init_room])
        .manage(Room::new(Box::new(receiving_sink), Box::new(outgoing_stream)))
        // The outgoing_sink will be passed to the multisig library to work with it
        .manage(OutgoingSink::new(Box::new(outgoing_sink)))
        .launch()
        .await?;

    Ok(())
}