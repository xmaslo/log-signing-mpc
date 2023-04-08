use std::fs;
use std::path::Path;
use std::pin::Pin;
use anyhow::{anyhow, Context, Error, Result};
use curv::arithmetic::Converter;
use curv::BigInt;
use curv::elliptic::curves::Secp256k1;
use futures::{Sink, SinkExt, Stream, StreamExt, TryFutureExt, TryStreamExt};
use futures::stream::Fuse;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{LocalKey, ProtocolMessage};
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::sign::{CompletedOfflineStage, OfflineProtocolMessage, OfflineStage, PartialSignature, SignManual};
use rocket::http::Status;
use round_based::{AsyncProtocol, Msg};

fn read_file(file_name: &Path) -> LocalKey<Secp256k1> {
    let contents = fs::read(file_name)
        .expect("Should have been able to read the file");

    let local_share = serde_json::from_slice(&contents).context("parse local share").unwrap();

    local_share
}

pub async fn do_offline_stage(
    file_name: &Path,
    party_index: u16,
    participants: Vec<u16>,
    receiving_stream: Pin<&mut Fuse<impl Stream<Item=Result<Msg<OfflineProtocolMessage>>> + Sized>>,
    outgoing_sink: Pin<&mut (impl Sink<Msg<OfflineProtocolMessage>, Error=Error> + Sized)>
) -> CompletedOfflineStage
{
    let local_share = read_file(file_name);

    let signing = OfflineStage::new(party_index, participants, local_share).unwrap();

    let completed_offline_stage = AsyncProtocol::new(signing, receiving_stream, outgoing_sink)
        .run()
        .await
        .map_err(|e| anyhow!("protocol execution terminated with error: {}", e));

    completed_offline_stage.unwrap()
}

pub async fn sign_hash(hash_to_sign: &str,
                 completed_offline_stage: CompletedOfflineStage,
                 party_index: u16,
                 number_of_parties: usize,
                 receiving_stream: Pin<&mut impl Stream<Item = Result<Msg<PartialSignature>, Error>>>,
                 mut outgoing_sink: Pin<&mut (impl Sink<Msg<PartialSignature>, Error=Error> + Sized)>
) -> Result<()> {
    let (signing, partial_signature) = SignManual::new(
        BigInt::from_bytes(hash_to_sign.as_bytes()),
        completed_offline_stage,
    )?;

    outgoing_sink
        .send(Msg {
        sender: party_index,
        receiver: None,
        body: partial_signature,
    }).await?;

    // receiving_stream.take(number_of_parties - 1);

    let partial_signatures: Vec<_> = receiving_stream
        .take(number_of_parties - 1)
        .map_ok(|msg| msg.body)
        .try_collect()
        .await?;

    let signature = signing
        .complete(&partial_signatures)
        .context("online stage failed")?;
    let signature = serde_json::to_string(&signature).context("serialize signature").unwrap();
    println!("{}", signature);

    Ok(())
}