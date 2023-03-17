use futures::{Sink, SinkExt, Stream, StreamExt};
use futures::channel::mpsc::{SendError};
use tokio::sync::RwLock;
use rocket::State;
use round_based::Msg;

use anyhow::{Context, Result};
use std::sync::Arc;
use rocket::http::Status;
use serde::{de::DeserializeOwned, Serialize, Deserialize};

pub trait SerializableMessage: Serialize + DeserializeOwned + Send + 'static {}

impl<T> SerializableMessage for T where T: Serialize + DeserializeOwned + Send + 'static {}

// This function creates the communication channels between the servers
// The messages sent to the outgoing sink will be received by other servers in their receiving_stream
// And vice versa, the messages sent by other servers to their outgoing sink will be received by this server in its receiving_stream
// To enable this functionality, it is necessary to:
//  1. add the room to the rocket state
//  2. call the init_room function on the rocket managed room state with the addresses of the communication partners
//      2.1 this can be now done by the call to the init_room endpoint
//  3. Mount the receive_broadcast endpoint to the rocket instance
pub fn create_communication_channels<SerializableMessage: Serialize + DeserializeOwned>() -> (
    impl Stream<Item = Result<Msg<SerializableMessage>>>,
    impl Sink<Msg<SerializableMessage>, Error = anyhow::Error>,
    Room,
)
{
    let (receiving_sink, mut receiving_stream) = futures::channel::mpsc::unbounded();
    let (outgoing_sink, mut outgoing_stream) = futures::channel::mpsc::unbounded();

    let room = Room::new(Box::new(receiving_sink), Box::new(outgoing_stream));

    let receiving_stream = receiving_stream.map(move |msg| {
        let msg_value: serde_json::Value = serde_json::from_str(&msg).context("parse message as JSON value")?;
        let sender = msg_value["sender"].as_u64().ok_or(anyhow::Error::msg("Invalid 'sender' field"))? as u16;
        let receiver = msg_value["receiver"].as_u64().map(|r| r as u16);
        let body_value = msg_value["body"].clone();
        let body = SerializableMessage::deserialize(body_value).context("deserialize message")?;

        Ok(Msg {
            sender,
            receiver,
            body,
        })
    });

    let outgoing_sink = futures::sink::unfold(outgoing_sink, |mut sink, message: Msg<SerializableMessage>| async move {
        let serialized = serde_json::to_string(&message).context("serialize message")?;
        sink.send(Ok(serialized)).await.map_err(|err| anyhow::Error::from(err));
        Ok(sink)
    });

    (receiving_stream, outgoing_sink, room)
}

// Handling the messages received from the other servers
#[rocket::post("/receive_broadcast", data = "<message>")]
pub async fn receive_broadcast(room: &State<Room>, message: String) -> Status
{
    room.receive(message).await;
    Status::Ok
}

pub struct Room
{
    receiving_sink: Arc<RwLock<Box<dyn Sink<String, Error = SendError> + Send + Sync + Unpin>>>,
    outgoing_stream: Arc<RwLock<Box<dyn Stream<Item = Result<String>> + Send + Sync + Unpin>>>,
    client: reqwest::Client,
}

impl Room {
    pub fn new(sink: Box<dyn Sink<String, Error = SendError> + Send + Sync + Unpin>,
               stream: Box<dyn Stream<Item = Result<String>> + Send + Sync + Unpin>,) -> Self {

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
                Some(Ok(message)) => {
                    counter += 1;
                    println!("Sending: {}  in round {}\n", message, counter);
                    for url in server_urls {
                        let endpoint = format!("http://{}/receive_broadcast", url);
                        println!("Sending: {} to {}", message, url);
                        self.client.post(&endpoint).body(message.clone()).send().await.unwrap();
                    }
                }
                Some(Err(_)) => break,
                None => break,
            }
        }
    }

    pub async fn receive(&self, message: String) {
        let mut guard = self.receiving_sink.write().await;
        let mut sink = guard.as_mut();

        if let Err(e) = sink.send(message).await {
            eprintln!("Failed to forward received message to sink: {:?}", e);
        }
    }
}