use std::path::Path;
use std::fs;
use anyhow::Context;
use curv::elliptic::curves::Secp256k1;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{LocalKey};

pub fn read_file(file_name: &Path) -> LocalKey<Secp256k1> {
    let contents = fs::read(file_name)
        .expect("Should have been able to read the file");

    let local_share = serde_json::from_slice(&contents).context("parse local share").unwrap();

    local_share
}