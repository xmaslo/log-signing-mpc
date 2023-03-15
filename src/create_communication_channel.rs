use futures::{Sink, SinkExt, Stream, StreamExt};
use tokio::sync::RwLock;
use rocket::State;


use std::sync::{Arc};
use rocket::http::Status;


// This function creates the communication channels between the servers
// The messages sent to the outgoing sink will be received by other servers in their receiving_stream
// And vice versa, the messages sent by other servers to their outgoing sink will be received by this server in its receiving_stream
// To enable this functionality, it is necessary to:
//  1. add the room to the rocket state
//  2. call the init_room function on the rocket managed room state with the addresses of the communication partners
//      2.1 this can be now done by the call to the init_room endpoint
//  3. Mount the receive_broadcast endpoint to the rocket instance
pub fn create_communication_channels() -> (
    futures::channel::mpsc::UnboundedReceiver<String>,
    futures::channel::mpsc::UnboundedSender<String>,
    Room,
) {
    let (receiving_sink, mut receiving_stream) = futures::channel::mpsc::unbounded();
    let (outgoing_sink, mut outgoing_stream) = futures::channel::mpsc::unbounded();

    let room = Room::new(Box::new(receiving_sink), Box::new(outgoing_stream));

    (receiving_stream, outgoing_sink, room)
}



// Handling the messages received from the other servers
#[rocket::post("/receive_broadcast", data = "<message>")]
pub async fn receive_broadcast(room: &State<Room>, message: String) -> Status {
    room.receive(message).await;
    Status::Ok
}

pub struct Room {
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
