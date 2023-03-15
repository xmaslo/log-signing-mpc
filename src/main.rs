use futures::{Sink, SinkExt, Stream, StreamExt};

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
    room.init_room(&urls).await;
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

    pub async fn init_room(&self, server_urls: &Vec<String>) {
        let mut counter = 0;

        loop {
            match self.outgoing_stream.write().await.next().await {
                Some(message) => {
                    counter += 1;
                    let message = format!("{} Resended {}", message, counter);
                    println!("Sending: {}\n", message);
                    for url in server_urls {
                        let endpoint = format!("http://{}/receive_broadcast", url);
                        println!("Sending: {} to {}", message, url);
                        self.client.post(&endpoint).body(message.clone()).send().await.unwrap();
                    }
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


// The messages sent to the outgoing sink will be received by other servers in their receiving_stream
// And vice versa, the messages sent by other servers to their outgoing sink will be received by this server in its receiving_stream
// To enable this functionality, it is necessary to:
//  1. add the room to the rocket state
//  2. call the init_room function on the rocket managed room state with the addresses of the communication partners
//      2.1 this can be now done by the call to the init_room endpoint
//  3. Mount the receive_broadcast endpoint to the rocket instance
fn create_communication_channels() -> (
    futures::channel::mpsc::UnboundedReceiver<String>,
    futures::channel::mpsc::UnboundedSender<String>,
    Room,
) {
    let (receiving_sink, mut receiving_stream) = futures::channel::mpsc::unbounded();
    let (outgoing_sink, mut outgoing_stream) = futures::channel::mpsc::unbounded();

    let room = Room::new(Box::new(receiving_sink), Box::new(outgoing_stream));

    (receiving_stream, outgoing_sink, room)
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