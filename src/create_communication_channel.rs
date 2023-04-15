use std::{
    collections::HashMap,
    sync::Arc,
    fs::File,
    io::BufReader,
};
use futures::{
    channel::mpsc::SendError,
    Sink, SinkExt, Stream, StreamExt,
};
use tokio::sync::RwLock;
use rocket::{
    Data, State,
    http::Status,
    tokio::io::AsyncReadExt,
    data::ToByteUnit,
};
use reqwest::{Client, Certificate, Identity, tls};

use rustls::{ClientConfig, RootCertStore, Certificate as RustlsCertificate};
use rustls_pemfile::{certs};
use round_based::Msg;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use tokio::spawn;
use crate::rocket_instances::SharedDb;


// This function creates the communication channels between the servers
// The messages sent to the outgoing sink will be received by other servers in their receiving_stream
// And vice versa, the messages sent by other servers to their outgoing sink will be received by this server in its receiving_stream
// To enable this functionality, it is necessary to:
//  1. add the db to the rocket state
//  3. Mount the receive_broadcast endpoint to the rocket instance


// Handling the messages received from the other servers
#[rocket::post("/receive_broadcast/<room_id>", data = "<data>")]
pub async fn receive_broadcast(db: &State<SharedDb>,
                               room_id: u16,
                               data: Data<'_>) -> Result<Status, std::io::Error> {
    let mut buffer = Vec::new();
    let data_length = data.open(1.mebibytes()).read_to_end(&mut buffer).await?;

    println!("Received data length: {} bytes", data_length);

    let message = String::from_utf8(buffer).unwrap_or_else(|_| String::from("Invalid UTF-8"));

    if let Some(room) = db.get_room(room_id).await {
        room.receive(message).await;
    }

    Ok(Status::Ok)
}

pub fn create_tls_config(server_id: u16) -> Client {
    // Load CA certificate
    let ca_cert = File::open("certs/ca_cert.pem");

    let mut client = Client::builder()
        .use_rustls_tls()
        .danger_accept_invalid_certs(true)
        ;

    if let Ok(ca_cert) = ca_cert {
        let mut ca_cert_reader = BufReader::new(ca_cert);
        if let Ok(ca_certs) = certs(&mut ca_cert_reader) {
            for ca_cert in ca_certs {
                let cert = Certificate::from_der(&*ca_cert);
                client = client.add_root_certificate(cert.unwrap());
            }
        }
    }

    // // Load public certificates
    // let public_cert = File::open(format!("certs/public/cert_{}.pem", server_id))?;
    // let public_cert_reader = BufReader::new(public_cert);
    //
    // let public_certs = certs(public_cert_reader).unwrap();

    client.build().unwrap()

}

pub struct Db {
    client: Client,
    rooms: RwLock<HashMap<String, Arc<Room>>>
}

pub struct Room {
    server_id: u16,
    room_id: u16,
    receiving_sink: Arc<RwLock<Box<dyn Sink<String, Error = SendError> + Send + Sync + Unpin>>>,
    outgoing_stream: Arc<RwLock<Box<dyn Stream<Item = Result<String>> + Send + Sync + Unpin>>>,
    client: Client,
}

impl Db {
    pub fn empty(server_id: u16) -> Self {

        Self {
            rooms: RwLock::new(HashMap::new()),
            client: create_tls_config(server_id),
        }
    }

    pub async fn create_room<SerializableMessage: Serialize + DeserializeOwned>(
        &self, server_id: u16, room_id: u16, server_urls: Vec<String>) -> (
        impl Stream<Item = Result<Msg<SerializableMessage>>>,
        impl Sink<Msg<SerializableMessage>, Error = anyhow::Error>,
    ) {
        // if room already exists, delete it first
        self.delete_room(room_id).await;

        let (receiving_sink, mut receiving_stream) = futures::channel::mpsc::unbounded();
        let (outgoing_sink, mut outgoing_stream) = futures::channel::mpsc::unbounded();

        let room = Room::new(server_id, room_id, Box::new(receiving_sink),
                             Box::new(outgoing_stream), self.client.clone());

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
            let _ = sink.send(Ok(serialized)).await.map_err(|err| anyhow::Error::from(err));
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

    pub async fn test_tls(&self, room_id: u16, server_urls: Vec<String>) {
        if let Some(room) = self.get_room(room_id).await {
            room.test_tls(&server_urls).await;
        }
    }

    pub async fn get_room(&self, room_id: u16) -> Option<Arc<Room>> {
        self.rooms.read().await.get(&room_id.to_string()).cloned()
    }

    pub async fn delete_room(&self, room_id: u16) {
        self.rooms.write().await.remove(&room_id.to_string());
    }

}


impl Room {
    pub fn new(
        server_id: u16,
        room_id: u16,
        sink: Box<dyn Sink<String, Error = SendError> + Send + Sync + Unpin>,
        stream: Box<dyn Stream<Item = Result<String>> + Send + Sync + Unpin>,
        client: Client,
    ) -> Self {
        Self {
            server_id,
            room_id,
            receiving_sink: Arc::new(RwLock::new(sink)),
            outgoing_stream: Arc::new(RwLock::new(stream)),
            client,
        }
    }

    pub async fn init_room(&self, server_urls: &Vec<String>) {

        // In updated better to sleep in key_gen after the initialization, as the key_gen is called after the init_room
        // and not possibly concurrently
        // thread::sleep(Duration::from_secs(15));


        let mut counter = 0;

        loop {
            match self.outgoing_stream.write().await.next().await {
                Some(Ok(message)) => {
                    counter += 1;
                    println!("Sending: {}  in round {}\n", message, counter);
                    for url in server_urls {
                        let endpoint = format!("https://{}/receive_broadcast/{}", url, self.room_id); // Include room_id in the URL
                        println!("Sending: {} to {}", message, url);
                        match self.client.post(&endpoint).body(message.clone()).send().await {
                            Ok(_response) => {
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

    pub async fn test_tls(&self, server_urls: &Vec<String>) {
        for url in server_urls {
            let endpoint = format!("https://{}/receive_broadcast/{}", url, self.room_id); // Include room_id in the URL
            println!("Sending: {} to {}", "test", url);
            match self.client.post(&endpoint).body("Test").send().await {
                Ok(_response) => {
                    println!("Successfully sent message to {}", url);
                }
                Err(e) => {
                    eprintln!("Error sending message to {}: {}", url, e);
                }
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