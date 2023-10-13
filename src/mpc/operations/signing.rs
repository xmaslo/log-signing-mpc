use std::path::Path;
use std::pin::Pin;
use std::{thread, time};
use std::collections::HashMap;
use std::time::Duration;
use anyhow::{anyhow, Context, Error, Result};
use curv::arithmetic::Converter;
use curv::BigInt;
use curv::elliptic::curves::Secp256k1;
use futures::{Sink, SinkExt, Stream, StreamExt, TryStreamExt};
use futures::stream::Fuse;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::LocalKey;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::sign::{CompletedOfflineStage, OfflineProtocolMessage, OfflineStage, PartialSignature, SignManual};
use round_based::{AsyncProtocol, Msg};
use crate::mpc::utils::local_share_utils::{read_file, file_to_local_key};

/// The structure that holds current state for the offline stage with other parties
pub struct Signer {
    my_index: u16,
    completed_offline_stage: HashMap<u16, Option<CompletedOfflineStage>>,
}

impl Signer {
    pub fn new(mi: u16) -> Signer {
        Signer {
            my_index: mi,
            completed_offline_stage: HashMap::new()
        }
    }

    pub fn add_participant(&mut self, new_participant: u16) -> Result<u16, &'static str> {
        if new_participant == self.my_index {
            return Err("New participant must have different index from current instance");
        }

        if self.completed_offline_stage.contains_key(&new_participant) {
            println!("Participant with that id is already present");
            return Ok(0);
        }

        self.completed_offline_stage.insert(new_participant, None);
        Ok(new_participant)
    }

    pub async fn do_offline_stage(
        &mut self,
        receiving_stream: Pin<&mut Fuse<impl Stream<Item=Result<Msg<OfflineProtocolMessage>>>>>,
        outgoing_sink: Pin<&mut impl Sink<Msg<OfflineProtocolMessage>, Error=Error>>,
        other_party_index: u16
    ) -> Result<(), Error>
    {
        if !self.is_participant_present(other_party_index) {
            return Err(anyhow!("Participant {} is not present", other_party_index));
        }

        let local_share = self.get_local_share();
        if local_share.is_none() {
            return Err(anyhow!("local-share{}.json is missing. Generate it with the /keygen endpoint first.", self.my_index));
        }
        let local_share: LocalKey<Secp256k1> = local_share.unwrap();

        println!("Participants: {}:{}", self.my_index, other_party_index);
        println!("My real index: {}", self.my_index);
        println!("My other index: {}", self.real_to_arbitrary_index(vec![other_party_index]));

        // wait for servers to synchronize
        // TODO: do this synchronization in a better way then sleeping
        let one_second:Duration = time::Duration::from_secs(1);
        thread::sleep(one_second);

        let signing =
            OfflineStage::new(self.real_to_arbitrary_index(vec![other_party_index]),
                              self.get_participants(other_party_index).unwrap(),
                              local_share).unwrap();

        let offline_stage = AsyncProtocol::new(signing, receiving_stream, outgoing_sink)
            .run()
            .await
            .map_err(|e| anyhow!("protocol execution terminated with error: {}", e));

        // self.completed_offline_stage = Some(offline_stage?);
        self.completed_offline_stage.insert(other_party_index, Some(offline_stage?));

        println!("OFFLINE STAGE IS COMPLETED");

        Ok(())
    }

    pub async fn sign_hash(
        &self,
        hash_to_sign: &String,
        receiving_stream: Pin<&mut impl Stream<Item = Result<Msg<PartialSignature>, Error>>>,
        mut outgoing_sink: Pin<&mut (impl Sink<Msg<PartialSignature>, Error=Error> + Sized)>,
        other_party_index: u16
    ) -> Result<String> {
        let (signing, partial_signature) = SignManual::new(
            BigInt::from_bytes(&hex::decode(hash_to_sign).unwrap()),
            self.completed_offline_stage.get(&other_party_index).unwrap().as_ref().unwrap().clone()
        )?;

        outgoing_sink
            .send(Msg {
                sender: self.real_to_arbitrary_index(vec![other_party_index]),
                receiver: None,
                body: partial_signature,
            }).await?;

        let partial_signatures: Vec<_> = receiving_stream
            .take(1)
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

    pub fn real_to_arbitrary_index(&self, other_indeces: Vec<u16>) -> u16 {
        let mut index: u16 = 1;
        for i in other_indeces {
            if self.my_index > i {
                index += 1
            }
        }

        return index
    }

    pub fn completed_offline_stage(&self) -> &HashMap<u16, Option<CompletedOfflineStage>> {
        &self.completed_offline_stage
    }

    pub fn is_participant_present(&self, index: u16) -> bool {
        self.completed_offline_stage().contains_key(&index)
    }

    pub fn is_offline_stage_complete(&self, participant: u16) -> bool {
        let participant_value = self.completed_offline_stage.get(&participant);
        return match participant_value {
            Some(v) => !v.is_none(),
            None => false
        }
    }

    fn get_local_share(&self) -> Option<LocalKey<Secp256k1>> {
        let file_name = format!("local-share{}.json", self.my_index);
        let file_content = read_file(Path::new(file_name.as_str()))?;
        Some(file_to_local_key(&file_content))
    }

    fn get_participants(&self, other_party_index: u16) -> Result<Vec<u16>,&'static str>  {
        if !self.is_participant_present(other_party_index) {
            return Err("Participant is not present");
        }

        let mut participants: Vec<u16> = vec![self.my_index, other_party_index];
        participants.sort(); // both parties my provide indexes in the same order
        Ok(participants)
    }
}

#[cfg(test)]
mod tests {
    use crate::mpc::operations::signing::Signer;

    #[test]
    fn add_participant_valid() {
        let mut s: Signer = Signer::new(1);
        assert_eq!(s.add_participant(2), Ok(2));
        assert!(s.is_participant_present(2));
    }

    #[test]
    fn add_participant_already_present() {
        let mut s: Signer = Signer::new(1);
        s.add_participant(2).unwrap();
        assert_eq!(s.add_participant(2).unwrap(), 0);
        assert!(s.is_participant_present(2));
    }

    #[test]
    fn add_participant_same_as_current_instance() {
        let mut s: Signer = Signer::new(1);
        s.add_participant(1).expect_err("Expected error, Ok returned");
        assert!(!s.is_participant_present(1));
    }

    #[test]
    fn offline_stage_complete_no() {
        let mut s: Signer = Signer::new(1);
        s.add_participant(2).unwrap();

        assert!(!s.is_offline_stage_complete(2));
    }

    #[test]
    fn offline_stage_complete_missing_participant() {
        let s: Signer = Signer::new(1);
        assert!(!s.is_offline_stage_complete(2));
    }

    #[test]
    fn arbitrary_index_conversion() {
        let mut s: Signer = Signer::new(2);
        s.add_participant(1).unwrap();
        s.add_participant(3).unwrap();

        assert_eq!(s.real_to_arbitrary_index(vec![1]), 2);
        assert_eq!(s.real_to_arbitrary_index(vec![3]), 1);

        let mut s: Signer = Signer::new(3);
        s.add_participant(1).unwrap();
        s.add_participant(2).unwrap();

        assert_eq!(s.real_to_arbitrary_index(vec![1]), 2);
        assert_eq!(s.real_to_arbitrary_index(vec![2]), 2);
    }
}