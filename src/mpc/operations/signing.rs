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
    offline_stage: HashMap<String, CompletedOfflineStage>,
    threshold: u16,
    n_of_participants: u16,
}

impl Signer {
    pub fn new(mi: u16, t: u16, n: u16) -> Signer {
        Signer {
            my_index: mi,
            offline_stage: HashMap::new(),
            threshold: t,
            n_of_participants: n,
        }
    }

    fn vec_to_string(participants: &Vec<u16>) -> String {
        let mut participants = participants.clone();
        let mut result: String = String::new();
        participants.sort();
        for participant in participants {
            result.push_str( participant.to_string().as_str());
        }

        return result;
    }

    pub async fn do_offline_stage(
        &mut self,
        receiving_stream: Pin<&mut Fuse<impl Stream<Item=Result<Msg<OfflineProtocolMessage>>>>>,
        outgoing_sink: Pin<&mut impl Sink<Msg<OfflineProtocolMessage>, Error=Error>>,
        participants: &Vec<u16>
    ) -> Result<(), Error>
    {
        if !self.are_participants_valid(participants) {
            return Err(anyhow!("Invalid participants provided"));
        }
        let participants_string = Signer::vec_to_string(participants);

        let local_share = self.get_local_share();
        if local_share.is_none() {
            return Err(anyhow!("local-share{}.json is missing. Generate it with the /keygen endpoint first.", self.my_index));
        }
        let local_share: LocalKey<Secp256k1> = local_share.unwrap();

        let arbitrary_index = match self.real_to_arbitrary_index(participants) {
            None => return Err(anyhow!("Invalid participants")),
            Some(ai) => ai
        };

        println!("Participants: {}:{}", self.my_index, participants_string);
        println!("My real index: {}", self.my_index);
        println!("My other index: {}", arbitrary_index);

        // wait for servers to synchronize
        // TODO: do this synchronization in a better way then sleeping
        let one_second:Duration = time::Duration::from_secs(1);
        thread::sleep(one_second);

        let signing =
            OfflineStage::new(arbitrary_index,
                              self.get_participants(&participants).unwrap(),
                              local_share).unwrap();

        let offline_stage = AsyncProtocol::new(signing, receiving_stream, outgoing_sink)
            .run()
            .await
            .map_err(|e| anyhow!("protocol execution terminated with error: {}", e));

        self.offline_stage.insert(participants_string, offline_stage?);

        println!("OFFLINE STAGE IS COMPLETED");

        Ok(())
    }

    pub async fn sign_hash(
        &self,
        hash_to_sign: &String,
        receiving_stream: Pin<&mut impl Stream<Item = Result<Msg<PartialSignature>, Error>>>,
        mut outgoing_sink: Pin<&mut (impl Sink<Msg<PartialSignature>, Error=Error> + Sized)>,
        participants: Vec<u16>
    ) -> Result<String, Error> {
        if !self.are_participants_valid(&participants) {
            return Err(anyhow!("Invalid participants provided"));
        }

        let participants_string = Signer::vec_to_string(&participants);

        let offline_stage = match self.offline_stage.get(participants_string.as_str()) {
            None => return Err(anyhow!("Offline stage not completed")),
            Some(os) => os.clone()
        };

        let (signing, partial_signature) = SignManual::new(
            BigInt::from_bytes(&hex::decode(hash_to_sign).unwrap()),
            offline_stage
        )?;

        let arbitrary_index = match self.real_to_arbitrary_index(&participants) {
            None => return Err(anyhow!("Invalid participants")),
            Some(ai) => ai
        };

        outgoing_sink
            .send(Msg {
                sender: arbitrary_index,
                receiver: None,
                body: partial_signature,
            }).await?;

        let partial_signatures: Vec<_> = receiving_stream
            .take(self.threshold as usize)
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

    pub fn real_to_arbitrary_index(&self, other_indices: &Vec<u16>) -> Option<u16> {
        if !self.are_participants_valid(other_indices) {
            return None;
        }
        let mut index: u16 = 1;
        for i in other_indices {
            if self.my_index > i.clone() {
                index += 1
            }
        }

        return Some(index)
    }

    pub fn is_offline_stage_complete(&self, participants: &Vec<u16>) -> bool {
        let participants_string = Signer::vec_to_string(participants);
        self.offline_stage.contains_key(participants_string.as_str())
    }

    fn get_local_share(&self) -> Option<LocalKey<Secp256k1>> {
        let file_name = format!("local-share{}.json", self.my_index);
        let file_content = read_file(Path::new(file_name.as_str()))?;
        Some(file_to_local_key(&file_content))
    }

    fn get_participants(&self, participants: &Vec<u16>) -> Result<Vec<u16>,&'static str>  {
        let mut p = participants.clone();
        let mut all_participants: Vec<u16> = vec![self.my_index];
        all_participants.append(&mut p);
        all_participants.sort(); // both parties must provide indexes in the same order
        Ok(all_participants)
    }

    fn are_participants_valid(&self, participants: &Vec<u16>) -> bool {
        if participants.len() as u16 != self.threshold {
            return false;
        }

        for participant in participants {
            if participant == &self.my_index ||
                participant > &self.n_of_participants {
                return false;
            }
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use crate::mpc::operations::signing::Signer;

    #[test]
    fn offline_stage_complete_no() {
        let s: Signer = Signer::new(1, 1, 3);

        assert!(!s.is_offline_stage_complete(&vec![2]));
    }

    #[test]
    fn arbitrary_index_conversion() {
        let s: Signer = Signer::new(2, 1, 3);
        assert_eq!(s.real_to_arbitrary_index(&vec![1]), Some(2));
        assert_eq!(s.real_to_arbitrary_index(&vec![3]), Some(1));

        let s: Signer = Signer::new(3, 1, 3);
        assert_eq!(s.real_to_arbitrary_index(&vec![1]), Some(2));
        assert_eq!(s.real_to_arbitrary_index(&vec![2]), Some(2));

        let s: Signer = Signer::new(2, 2, 4);
        assert_eq!(s.real_to_arbitrary_index(&vec![3,4]), Some(1));
        assert_eq!(s.real_to_arbitrary_index(&vec![1,3]), Some(2));
    }
}