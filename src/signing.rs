use std::path::Path;
use std::pin::Pin;
use std::{thread, time};
use std::time::Duration;
use anyhow::{anyhow, Context, Error, Result};
use curv::arithmetic::Converter;
use curv::BigInt;
use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use futures::stream::Fuse;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::sign::{CompletedOfflineStage, OfflineProtocolMessage, OfflineStage, PartialSignature, SignManual};
use round_based::{AsyncProtocol, Msg};
use crate::common::{read_file, file_to_local_key};

pub struct KeyGenerator {
    participants: Vec<u16>,
    participants_n: usize,
    party_index: u16,
    completed_offline_stage: Option<CompletedOfflineStage>,
}

impl KeyGenerator {
    pub fn new(mut p: Vec<u16>, n: usize, pi: u16) -> KeyGenerator {
        p.sort(); // participants must be specified in the same order by both servers
        KeyGenerator {
            participants: p,
            participants_n: n,
            party_index: pi,
            completed_offline_stage: None
        }
    }

    pub async fn do_offline_stage(
        &mut self,
        receiving_stream: Pin<&mut Fuse<impl Stream<Item=Result<Msg<OfflineProtocolMessage>>>>>,
        outgoing_sink: Pin<&mut impl Sink<Msg<OfflineProtocolMessage>, Error=Error>>
    ) -> Result<()>
    {
        println!("Participants: {}:{}", self.participants[0], self.participants[1]);
        println!("Number of participants: {}", self.participants_n);
        println!("My real index: {}", self.party_index);
        println!("My other index: {}", self.get_different_party_index());

        let file_name = format!("local-share{}.json", self.party_index);

        let file_content = read_file(Path::new(file_name.as_str()));
        let local_share = file_to_local_key(&file_content);

        // wait for servers to synchronize
        // TODO: do this synchronization in a better way then sleeping
        let one_second:Duration = time::Duration::from_secs(1);
        thread::sleep(one_second);
        let signing = OfflineStage::new(self.get_different_party_index(), self.participants.clone(), local_share)?;

        let offline_stage = AsyncProtocol::new(signing, receiving_stream, outgoing_sink)
            .run()
            .await
            .map_err(|e| anyhow!("protocol execution terminated with error: {}", e));

        self.completed_offline_stage = Some(offline_stage?);

        println!("OFFLINE STAGE IS COMPLETED");

        Ok(())
    }

    pub async fn sign_hash(
        &self,
        hash_to_sign: &String,
        receiving_stream: Pin<&mut impl Stream<Item = Result<Msg<PartialSignature>, Error>>>,
        mut outgoing_sink: Pin<&mut (impl Sink<Msg<PartialSignature>, Error=Error> + Sized)>
    ) -> Result<String> {
        let (signing, partial_signature) = SignManual::new(
            BigInt::from_bytes(&hex::decode(hash_to_sign).unwrap()),
            self.completed_offline_stage.as_ref().unwrap().clone(),
        )?;

        outgoing_sink
            .send(Msg {
                sender: self.get_different_party_index(),
                receiver: None,
                body: partial_signature,
            }).await?;

        let partial_signatures: Vec<_> = receiving_stream
            .take(self.participants_n - 1)
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

    // parties participating in signing => their index
    // [1,2] => [1,2]
    // [2,3] => [1,2]
    // [1,2,3] => [1,2,3] note: we do not support signatures of all 3 parties
    pub fn get_different_party_index(&self) -> u16 {
        if self.participants_n == 2
        {
            return if self.party_index == self.participants[0] {
                1
            } else {
                2
            }
        }

        self.party_index
    }
}