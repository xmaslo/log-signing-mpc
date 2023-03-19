use std::fs::File;
use std::io::{Write};
use std::path::Path;
use std::pin::Pin;
use std::rc::Rc;
use std::sync::Arc;
use anyhow::{anyhow, Error};
use curv::elliptic::curves::Secp256k1;
use futures::{Sink, Stream, StreamExt};
use futures::stream::Fuse;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{Keygen, ProtocolMessage, LocalKey};
use round_based::{AsyncProtocol, Msg};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use super::create_communication_channel::{SerializableMessage};

const THRESHOLD: u16 = 2;
const NUMBER_OF_PARTIES: u16 = 3;

/*pub struct KeyGenerator {
    index: u16,
    receiving_stream: Fuse<Arc<dyn Stream<Item=Result<Msg<ProtocolMessage>, Error>> + Send + Sync>>,
    outgoing_sink: dyn Sink<Msg<ProtocolMessage>, Error=Error> + Send + Sync
}*/

// let mut incoming: Pin<&mut Fuse<impl Stream<Item = Result<Msg<ProtocolMessage>, Error>>>>
// let mut outgoing: Pin<&mut (impl Sink<Msg<ProtocolMessage>, Error = Error>)>

/*pub fn new(i: u16,
               rs: Pin<&mut Fuse<impl Stream<Item = Result<Msg<ProtocolMessage>, Error>>>>,
               os: Pin<&mut (impl Sink<Msg<ProtocolMessage>, Error = Error>)>) -> KeyGenerator {
        KeyGenerator {
            index: i,
            receiving_stream: rs,
            outgoing_sink: os
        }
    }*/

pub struct KeyGenerator<S, T>
    where
        S: Stream<Item=Result<Msg<ProtocolMessage>, Error>> + Send + Sync + 'static,
        T: Sink<Msg<ProtocolMessage>, Error=Error> + Send + Sync + 'static,
{
    index: u16,
    receiving_stream: Fuse<S>,
    outgoing_sink: T,
}

impl<S, T> KeyGenerator<S, T>
    where
        S: Stream<Item=Result<Msg<ProtocolMessage>, Error>> + Send + Sync + 'static,
        T: Sink<Msg<ProtocolMessage>, Error=Error> + Send + Sync + 'static,
{
    pub fn new(
        i: u16,
        rs: S,
        os: T,
    ) -> KeyGenerator<S, T> {
        KeyGenerator {
            index: i,
            receiving_stream: rs.fuse(),
            outgoing_sink: os,
        }
    }

    pub async fn run(self, file_name: &Path) -> usize {
        let incoming = self.receiving_stream;
        let outgoing = self.outgoing_sink;
        tokio::pin!(incoming);
        tokio::pin!(outgoing);

        let keygen: Keygen = Keygen::new(self.index, THRESHOLD, NUMBER_OF_PARTIES).unwrap();
        let results: Result<LocalKey<Secp256k1>, Error> = AsyncProtocol::new(keygen, incoming, outgoing)
            .run()
            .await
            .map_err(|e| anyhow!("protocol execution terminated with error: {}", e));

        let local_key: LocalKey<Secp256k1> = results.unwrap();

        println!("Writing generated key into {:?}", file_name);
        if file_name.exists() {
            println!("{:?} already exists. Removing it...", file_name);
        }
        let open_result = File::create(file_name);
        let mut file = match open_result {
            Ok(f) => f,
            Err(error) => panic!("Problem with opening file {:?}", error)
        };

        let output = serde_json::to_vec_pretty(&local_key);

        let write_result = file.write(output.unwrap().as_ref()).unwrap();

        write_result
        // self.generate_file(file_name, &local_key);
    }

    /*fn generate_file(&self, file_name: &Path, result: &LocalKey<Secp256k1>) -> usize {
        println!("Writing generated key into {:?}", file_name);
        if file_name.exists() {
            println!("{:?} already exists. Removing it...", file_name);
        }
        let open_result = File::create(file_name);
        let mut file = match open_result {
            Ok(f) => f,
            Err(error) => panic!("Problem with opening file {:?}", error)
        };

        let output = serde_json::to_vec_pretty(result);

        let write_result = file.write(output.unwrap().as_ref()).unwrap();

        write_result
    }*/
}