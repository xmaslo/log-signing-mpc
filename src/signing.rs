use std::fs;
use std::path::Path;
use std::pin::Pin;
use anyhow::{anyhow, Context, Error, Result};
use curv::arithmetic::Converter;
use curv::BigInt;
use curv::elliptic::curves::Secp256k1;
use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use futures::stream::Fuse;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{LocalKey};
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::sign::{CompletedOfflineStage, OfflineProtocolMessage, OfflineStage, PartialSignature, SignManual};
use round_based::{AsyncProtocol, Msg};

pub struct KeyGenerator {
    completed_offline_stage: Option<CompletedOfflineStage>
}

impl KeyGenerator {
    pub fn new() -> KeyGenerator {
        KeyGenerator {
            completed_offline_stage: None
        }
    }

    pub async fn do_offline_stage(
        &mut self,
        file_name: &Path,
        party_index: u16,
        participants: Vec<u16>,
        receiving_stream: Pin<&mut Fuse<impl Stream<Item=Result<Msg<OfflineProtocolMessage>>>>>,
        outgoing_sink: Pin<&mut impl Sink<Msg<OfflineProtocolMessage>, Error=Error>>
    ) -> Result<()>
    {
        let local_share = self.read_file(file_name);

        let signing = OfflineStage::new(party_index, participants, local_share).unwrap();

        let offline_stage = AsyncProtocol::new(signing, receiving_stream, outgoing_sink)
            .run()
            .await
            .map_err(|e| anyhow!("protocol execution terminated with error: {}", e));

        self.completed_offline_stage = Some(offline_stage?);

        Ok(())
    }

    pub async fn sign_hash(
        &self,
        hash_to_sign: &String,
        party_index: u16,
        signing_parties_n: usize,
        receiving_stream: Pin<&mut impl Stream<Item = Result<Msg<PartialSignature>, Error>>>,
        mut outgoing_sink: Pin<&mut (impl Sink<Msg<PartialSignature>, Error=Error> + Sized)>
    ) -> Result<String> {
        let (signing, partial_signature) = SignManual::new(
            BigInt::from_bytes(hash_to_sign.as_bytes()),
            self.completed_offline_stage.as_ref().unwrap().clone(),
        )?;

        outgoing_sink
            .send(Msg {
                sender: party_index,
                receiver: None,
                body: partial_signature,
            }).await?;

        let partial_signatures: Vec<_> = receiving_stream
            .take(signing_parties_n - 1)
            .map_ok(|msg| msg.body)
            .try_collect()
            .await?;

        let signature = signing
            .complete(&partial_signatures)
            .context("online stage failed")?;
        let signature = serde_json::to_string(&signature).context("serialize signature").unwrap();
        println!("SIGNATURE:\n{}", signature);

        Ok(signature)
    }

    fn read_file(&self, file_name: &Path) -> LocalKey<Secp256k1> {
        let contents = fs::read(file_name)
            .expect("Should have been able to read the file");

        let local_share = serde_json::from_slice(&contents).context("parse local share").unwrap();

        local_share
    }
}