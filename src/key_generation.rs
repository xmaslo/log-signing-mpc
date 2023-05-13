use std::fs::File;
use std::io::{Write};
use std::path::Path;
use std::pin::Pin;
use std::{thread, time};
use std::time::Duration;
use anyhow::{anyhow, Error, Result};
use curv::elliptic::curves::Secp256k1;
use futures::{Sink, Stream};
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{Keygen, ProtocolMessage, LocalKey};
use round_based::{AsyncProtocol, Msg};

use futures::stream::Fuse;

const THRESHOLD: u16 = 1;
const NUMBER_OF_PARTIES: u16 = 3;

fn are_keys_already_generated(index: u16) -> Result<String, String> {
    let file_name: String = format!("local-share{}.json", index);
    let file_path: &Path = Path::new(file_name.as_str());
    if file_path.exists() {
        let error_msg = format!("{} already exists. If you want to generate keys, remove already existing ones.", file_name.clone());
        return Err(error_msg);
    }
    Ok(file_name)
}

pub async fn generate_keys(index: u16,
                           receiving_stream: Pin<&mut Fuse<impl Stream<Item=Result<Msg<ProtocolMessage>>>>>,
                           outgoing_sink: Pin<&mut impl Sink<Msg<ProtocolMessage>, Error=Error>>
) -> Result<(), String> {
    let file = are_keys_already_generated(index);
    let file = match file {
        Ok(f) => f,
        Err(e) => return Err(e),
    };

    // wait for other servers to catch up (if started manually)
    // TODO: do this synchronization in a better way then sleeping
    let five_seconds:Duration = time::Duration::from_secs(5);
    thread::sleep(five_seconds);

    let keygen: Keygen = Keygen::new(index, THRESHOLD, NUMBER_OF_PARTIES).unwrap();
    let results: Result<LocalKey<Secp256k1>, Error> = AsyncProtocol::new(keygen, receiving_stream, outgoing_sink)
        .run()
        .await
        .map_err(|e| anyhow!("protocol execution terminated with error: {}", e));

    let local_key: LocalKey<Secp256k1> = results.unwrap();

    let generation_result = generate_file(&file, &local_key);
    match generation_result {
        Ok(_) => Ok(()),
        Err(_) => Err("Unable to generate file".to_string()),
    }
}

fn generate_file(file_name: &String, result: &LocalKey<Secp256k1>) -> Result<usize> {
    let open_result = File::create(Path::new(file_name));
    let mut file = match open_result {
        Ok(f) => f,
        Err(error) => panic!("Problem with opening file {:?}", error)
    };
    let output = serde_json::to_vec_pretty(result);

    let write_result = file.write(output.unwrap().as_ref())?;

    println!("Generated key written into {}", file_name);

    Ok(write_result)
}