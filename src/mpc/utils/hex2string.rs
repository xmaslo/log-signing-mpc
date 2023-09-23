pub fn hex_to_string(hex_string: String) -> String {
    let signature = hex::decode(hex_string).expect("Decoding failed");
    String::from_utf8(signature).expect("Found invalid UTF-8")
}