use std::{
    collections::HashMap,
    sync::Arc,
    fs::File,
    io::BufReader,
    io::Read,
};
use futures::{
    Sink, SinkExt, Stream, StreamExt,
};
use tokio::sync::RwLock;
use reqwest::{Client, Certificate, Identity};

use rustls_pemfile::{certs};
use round_based::Msg;

use anyhow::{Context, Result};
use serde::{de::DeserializeOwned, Serialize};
use tokio::spawn;


use crate::communication::room::Room;


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

    // Load public certificates
    let mut buf = Vec::new();
    let _ = File::open(format!("certs/private/cert_and_key_{}.pem", server_id))
        .unwrap()
        .read_to_end(&mut buf)
        .unwrap();

    let identity = Identity::from_pem(&buf).unwrap();
    client = client.identity(identity);
    client.build().unwrap()

}

pub struct Db {
    client: Client,
    rooms: RwLock<HashMap<String, Arc<Room>>>
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

        let (receiving_sink,  receiving_stream) = futures::channel::mpsc::unbounded();
        let (outgoing_sink, outgoing_stream) = futures::channel::mpsc::unbounded();

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

    pub async fn get_room(&self, room_id: u16) -> Option<Arc<Room>> {
        self.rooms.read().await.get(&room_id.to_string()).cloned()
    }

    pub async fn delete_room(&self, room_id: u16) {
        self.rooms.write().await.remove(&room_id.to_string());
    }
}
