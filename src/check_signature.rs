use anyhow::Context;
use curv::{
    arithmetic::traits::Converter,
    elliptic::curves::{secp256_k1::Secp256k1, Point, Scalar},
    BigInt,
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct Signature {
    r: Scalar<Secp256k1>,
    s: Scalar<Secp256k1>,
    recid: u32,
}

pub fn extract_rs(signature: &str) -> (Scalar<Secp256k1>, Scalar<Secp256k1>) {
    let parsed_signature: Signature = serde_json::from_str(signature).context("Parse signature").unwrap();
    (parsed_signature.r, parsed_signature.s)
}

// implementation from https://github.com/ZenGo-X/multi-party-ecdsa/blob/master/examples/common.rs
pub fn check_sig(
    r: &Scalar<Secp256k1>,
    s: &Scalar<Secp256k1>,
    msg: &BigInt,
    pk: &Point<Secp256k1>,
) -> bool {
    use secp256k1::{Message, PublicKey, Signature, SECP256K1};

    let raw_msg = BigInt::to_bytes(msg);
    let mut msg: Vec<u8> = Vec::new(); // padding
    msg.extend(vec![0u8; 32 - raw_msg.len()]);
    msg.extend(raw_msg.iter());

    let msg = Message::from_slice(msg.as_slice()).unwrap();
    let mut raw_pk = pk.to_bytes(false).to_vec();
    if raw_pk.len() == 64 {
        raw_pk.insert(0, 4u8);
    }
    let pk = PublicKey::from_slice(&raw_pk).unwrap();

    let mut compact: Vec<u8> = Vec::new();
    let bytes_r = &r.to_bytes().to_vec();
    compact.extend(vec![0u8; 32 - bytes_r.len()]);
    compact.extend(bytes_r.iter());

    let bytes_s = &s.to_bytes().to_vec();
    compact.extend(vec![0u8; 32 - bytes_s.len()]);
    compact.extend(bytes_s.iter());

    let secp_sig = Signature::from_compact(compact.as_slice()).unwrap();

    return SECP256K1.verify(&msg, &secp_sig, &pk).is_ok();
}

#[cfg(test)]
mod tests {
    use curv::arithmetic::Converter;
    use curv::BigInt;
    use curv::elliptic::curves::{Point, Secp256k1};
    use crate::check_signature::{check_sig, extract_rs};

    #[test]
    fn check_valid_signature() {
        let signature = "{\"r\":{\"curve\":\"secp256k1\",\"scalar\":[10,220,76,129,129,115,200,211,20,231,213,128,218,23,186,111,92,165,38,8,69,209,254,206,204,30,239,226,132,136,230,154]},\"s\":{\"curve\":\"secp256k1\",\"scalar\":[91,75,36,116,47,138,116,142,176,14,240,250,3,184,215,0,168,218,133,14,158,179,170,80,136,117,115,228,189,186,37,149]},\"recid\":0}";
        let (r,s) = extract_rs(signature);

        let str_num = String::from("sign_this_data1681402350");

        let msg = BigInt::from_bytes(str_num.as_bytes());

        let bytes: [u8; 33] = [3, 183, 191, 143, 211, 92, 155, 44, 130, 59, 29, 152, 124, 146, 233, 81, 9, 70, 219, 20, 100, 4, 243, 31, 227, 146, 20, 116, 205, 145, 227, 57, 0];

        let public_key: Point<Secp256k1> = Point::from_bytes(&bytes).unwrap();

        assert!(check_sig(&r, &s, &msg, &public_key));
    }
}