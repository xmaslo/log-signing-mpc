use std::fs::File;
use std::io::{Write};
use std::ops::Deref;
use std::path::Path;
use std::pin::Pin;
use anyhow::{anyhow, Error, Result};
use curv::elliptic::curves::Secp256k1;
use futures::{Sink, Stream, StreamExt};
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{Keygen, ProtocolMessage, LocalKey};
use round_based::{AsyncProtocol, Msg};

use std::sync::{Arc};
use futures::stream::Fuse;
use tokio::sync::RwLock;


const THRESHOLD: u16 = 2;
const NUMBER_OF_PARTIES: u16 = 3;

// pub struct KeyGenerator
// {
//     index: u16,
//     receiving_stream: Arc<RwLock<Pin<Box<dyn Stream<Item=Msg<ProtocolMessage>> + Send + Sync>>>>,
//     outgoing_sink: Arc<RwLock<Pin<Box<dyn Sink<Msg<ProtocolMessage>, Error=Error> + Send + Sync>>>>,
// }
//
// impl KeyGenerator
// {
//     pub fn new(
//         i: u16,
//         rs: Pin<Box<dyn Stream<Item=Msg<ProtocolMessage>> + Send + Sync>>,
//         os: Pin<Box<dyn Sink<Msg<ProtocolMessage>, Error=Error> + Send + Sync>>,
//     ) -> KeyGenerator {
//         KeyGenerator {
//             index: i,
//             receiving_stream: Arc::new(RwLock::new(rs)),
//             outgoing_sink: Arc::new(RwLock::new(os)),
//         }
//     }
//
//     pub async fn run(&self, file_name: &Path) {
//         // let incoming = self.receiving_stream.read().await.fuse();
//         // let outgoing = self.outgoing_sink.write().await
//         //     .deref().as_ref().get_ref();
//
//         // let keygen: Keygen = Keygen::new(self.index, THRESHOLD, NUMBER_OF_PARTIES).unwrap();
//         // let results: Result<LocalKey<Secp256k1>, Error> = AsyncProtocol::new(keygen, incoming, outgoing)
//         //     .run()
//         //     .await
//         //     .map_err(|e| anyhow!("protocol execution terminated with error: {}", e));
//         //
//         // let local_key: LocalKey<Secp256k1> = results.unwrap();
//         //
//         // Self::generate_file(file_name, &local_key);
//     }
//
//     fn generate_file(file_name: &Path, result: &LocalKey<Secp256k1>) -> usize {
//         println!("Writing generated key into {:?}", file_name);
//         if file_name.exists() {
//             println!("{:?} already exists. Removing it...", file_name);
//         }
//         let open_result = File::create(file_name);
//         let mut file = match open_result {
//             Ok(f) => f,
//             Err(error) => panic!("Problem with opening file {:?}", error)
//         };
//
//         let output = serde_json::to_vec_pretty(result);
//
//         let write_result = file.write(output.unwrap().as_ref()).unwrap();
//
//         write_result
//     }
// }

pub async fn generate_keys(file_name: &Path,
                           index: u16,
                           receiving_stream: Pin<&mut Fuse<(impl Stream<Item=Result<Msg<ProtocolMessage>>> + Sized)>>,
                           outgoing_sink: Pin<&mut (impl Sink<Msg<ProtocolMessage>, Error=Error> + Sized)>
) {
    let keygen: Keygen = Keygen::new(index, THRESHOLD, NUMBER_OF_PARTIES).unwrap();
    let results: Result<LocalKey<Secp256k1>, Error> = AsyncProtocol::new(keygen, receiving_stream, outgoing_sink)
        .run()
        .await
        .map_err(|e| anyhow!("protocol execution terminated with error: {}", e));

    let local_key: LocalKey<Secp256k1> = results.unwrap();

    generate_file(file_name, &local_key);
}

fn generate_file(file_name: &Path, result: &LocalKey<Secp256k1>) -> usize {
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
}