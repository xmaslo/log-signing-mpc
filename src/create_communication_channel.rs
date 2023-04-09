use std::collections::HashMap;
use futures::{Sink, SinkExt, Stream, StreamExt};
use futures::channel::mpsc::{SendError};
use tokio::sync::RwLock;
use rocket::{Data, State};
use round_based::Msg;

use anyhow::{Context, Result};
use std::sync::Arc;
use rocket::http::Status;
use serde::{de::DeserializeOwned, Serialize};
use rocket::tokio::io::AsyncReadExt;

use std::thread;
use std::time::Duration;
use rocket::data::ToByteUnit;
use tokio::spawn;

// This function creates the communication channels between the servers
// The messages sent to the outgoing sink will be received by other servers in their receiving_stream
// And vice versa, the messages sent by other servers to their outgoing sink will be received by this server in its receiving_stream
// To enable this functionality, it is necessary to:
//  1. add the room to the rocket state
//  2. call the init_room function on the rocket managed room state with the addresses of the communication partners
//      2.1 this can be now done by the call to the init_room endpoint
//  3. Mount the receive_broadcast endpoint to the rocket instance
pub fn create_communication_channels<SerializableMessage: Serialize + DeserializeOwned>(server_id: u16, room_id: u16) -> (
    impl Stream<Item = Result<Msg<SerializableMessage>>>,
    impl Sink<Msg<SerializableMessage>, Error = anyhow::Error>,
    Room,
)
{
    let (receiving_sink, mut receiving_stream) = futures::channel::mpsc::unbounded();
    let (outgoing_sink, mut outgoing_stream) = futures::channel::mpsc::unbounded();

    let room = Room::new(server_id, room_id, Box::new(receiving_sink), Box::new(outgoing_stream));

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
#[rocket::post("/receive_broadcast/<room_id>", data = "<data>")]
pub async fn receive_broadcast(db: &State<Db>,
                               room_state: &State<Room>,
                               room_id: u16,
                               data: Data<'_>) -> Result<Status, std::io::Error> {
    let mut buffer = Vec::new();
    let data_length = data.open(1.mebibytes()).read_to_end(&mut buffer).await?;

    println!("Received data length: {} bytes", data_length);

    let message = String::from_utf8(buffer).unwrap_or_else(|_| String::from("Invalid UTF-8"));

    if let Some(room) = db.get_room(room_id).await {
        room.receive(message).await;
    } else {
        room_state.receive(message).await;
    }

    Ok(Status::Ok)
}

pub struct Db {
    rooms: RwLock<HashMap<String, Arc<Room>>>
}

pub struct Room {
    server_id: u16,
    room_id: u16,
    receiving_sink: Arc<RwLock<Box<dyn Sink<String, Error = SendError> + Send + Sync + Unpin>>>,
    outgoing_stream: Arc<RwLock<Box<dyn Stream<Item = Result<String>> + Send + Sync + Unpin>>>,
    client: reqwest::Client,
}

impl Db {
    pub fn empty() -> Self {
        Self {
            rooms: RwLock::new(HashMap::new()),
        }
    }

    pub async fn create_room<SerializableMessage: Serialize + DeserializeOwned>(
        &self, server_id: u16, room_id: u16, server_urls: Vec<String>) -> (
        impl Stream<Item = Result<Msg<SerializableMessage>>>,
        impl Sink<Msg<SerializableMessage>, Error = anyhow::Error>,
    ) {
        let (receiving_sink, mut receiving_stream) = futures::channel::mpsc::unbounded();
        let (outgoing_sink, mut outgoing_stream) = futures::channel::mpsc::unbounded();

        let room = Room::new(server_id, room_id, Box::new(receiving_sink), Box::new(outgoing_stream));

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



        let room = Arc::new(room);
        self.rooms.write().await.insert(room_id.to_string(), Arc::clone(&room));

        let room_clone = Arc::clone(&room);

        spawn(async move {
            room_clone.init_room(&server_urls).await;
        });

        (receiving_stream, outgoing_sink)
    }

    pub async fn get_room(&self, room_id: u16) -> Option<Arc<Room>> {
        self.rooms.read().await.get(&room_id.to_string()).cloned()
    }

    pub async fn insert_room(&self, room_id: u16, room: Arc<Room>) {
        self.rooms.write().await.insert(room_id.to_string(), room);
    }

}


impl Room {
    pub fn new(
        server_id: u16,
        room_id: u16,
        sink: Box<dyn Sink<String, Error = SendError> + Send + Sync + Unpin>,
        stream: Box<dyn Stream<Item = Result<String>> + Send + Sync + Unpin>,
    ) -> Self {
        Self {
            server_id,
            room_id,
            receiving_sink: Arc::new(RwLock::new(sink)),
            outgoing_stream: Arc::new(RwLock::new(stream)),
            client: reqwest::Client::new(),
        }
    }

    pub async fn init_room(&self, server_urls: &Vec<String>) {
        thread::sleep(Duration::from_secs(15));

        let mut counter = 0;

        loop {
            match self.outgoing_stream.write().await.next().await {
                Some(Ok(message)) => {
                    counter += 1;
                    println!("Sending: {}  in round {}\n", message, counter);
                    for url in server_urls {
                        let endpoint = format!("http://{}/receive_broadcast/{}", url, self.room_id); // Include room_id in the URL
                        println!("Sending: {} to {}", message, url);
                        match self.client.post(&endpoint).body(message.clone()).send().await {
                            Ok(response) => {
                                println!("Successfully sent message to {}", url);
                            }
                            Err(e) => {
                                eprintln!("Error sending message to {}: {}", url, e);
                            }
                        }
                    }
                }
                Some(Err(_)) => break,
                None => break,
            }
        }
    }

    pub async fn receive(&self, message: String) {
        let msg_value: serde_json::Value = serde_json::from_str(&message).unwrap();
        //TODO: handle error more gracefully
        let receiver = msg_value["receiver"].as_u64().map(|r| r as u16);

        // Filter out messages based on the receiver ID
        if let Some(receiver_id) = receiver {
            if receiver_id != self.server_id {
                println!("Filtered out the: {} message, originally to {}", msg_value,
                         receiver_id);
                return;
            }
        }

        let mut guard = self.receiving_sink.write().await;
        let mut sink = guard.as_mut();

        if let Err(e) = sink.send(message).await {
            eprintln!("Failed to forward received message to sink: {:?}", e);
        }
    }
}