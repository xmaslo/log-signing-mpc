use std::path::Path;
use std::fs;
use anyhow::Context;
use curv::elliptic::curves::Secp256k1;
use multi_party_ecdsa::protocols::multi_party_ecdsa::gg_2020::state_machine::keygen::{LocalKey};

pub fn read_file(file_name: &Path) -> String {
    fs::read_to_string(file_name)
        .expect("Should have been able to read the file")
}

pub fn file_to_local_key(file_content: &String) -> LocalKey<Secp256k1> {
    let local_share: LocalKey<Secp256k1> = serde_json::from_slice(file_content.as_bytes()).context("parse local share").unwrap();

    local_share
}