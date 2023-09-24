use std::{sync::Arc};
use futures::{
    channel::mpsc::SendError,
    Sink, SinkExt, Stream, StreamExt,
};
use tokio::sync::RwLock;
use reqwest::Client;
use anyhow::Result;

pub struct Room {
    server_id: u16,
    room_id: u16,
    receiving_sink: Arc<RwLock<Box<dyn Sink<String, Error = SendError> + Send + Sync + Unpin>>>,
    outgoing_stream: Arc<RwLock<Box<dyn Stream<Item = Result<String>> + Send + Sync + Unpin>>>,
    client: Client,
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
        let mut counter = 0;

        loop {
            match self.outgoing_stream.write().await.next().await {
                Some(Ok(message)) => {
                    counter += 1;
                    println!("Sending: {}  in round {}\n", message, counter);
                    for url in server_urls {
                        let endpoint = format!("https://{}/receive_broadcast/{}", url, self.room_id);
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

    // called by receive broadcast endpoint to receive messages from other servers
    pub async fn receive(&self, message: String) {
        let msg_value: serde_json::Value = serde_json::from_str(&message).unwrap();
        let receiver: Option<u16> = msg_value["receiver"].as_u64().map(|r| r as u16);

        // Filter out messages based on the receiver ID
        if let Some(receiver_id) = receiver {
            if receiver_id != self.server_id {
                println!("Filtered out a message meant for {}",
                         receiver_id);
                return;
            }
        }

        println!("Received message {}", message);

        let mut guard = self.receiving_sink.write().await;
        let sink = guard.as_mut();

        if let Err(e) = sink.send(message).await {
            eprintln!("Failed to forward received message to sink: {:?}", e);
        }
    }
}