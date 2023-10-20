extern crate serde;
extern crate serde_json;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Participant {
    server_id: u16,
    url: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct EndpointSignatureData {
    participants: Vec<Participant>,
    data_to_sign: String,
    timestamp: String,
}

impl EndpointSignatureData {
    pub fn participant_id(&self, n: usize) -> u16 {
        self.participants[n].server_id
    }

    pub fn participant_url(&self, n: usize) -> &str {
        &self.participants[n].url
    }

    pub fn data_to_sign(&self) -> &str {
        &self.data_to_sign
    }
    pub fn timestamp(&self) -> &str {
        &self.timestamp
    }

    pub fn get_ids(&self) -> Vec<u16> {
        let mut res: Vec<u16> = Vec::new();
        for participant in &self.participants {
            res.push(participant.server_id);
        }

        res
    }

    pub fn get_urls(&self) -> Vec<String> {
        let mut res: Vec<String> = Vec::new();
        for participant in &self.participants {
            res.push(participant.url.clone());
        }

        res
    }
}

// Define a test module
#[cfg(test)]
mod tests {
    use super::*;

    fn get_testing_data() -> String {
        let data = r#"
        {
            "participants": [
                {
                    "server_id": 1,
                    "url": "http://127.0.0.1:3001"
                },
                {
                    "server_id": 2,
                    "url": "http://127.0.0.1:3002"
                }
            ],
            "data_to_sign": "7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d",
            "timestamp": "16816533390"
        }
        "#;

        data.to_string()
    }

    #[test]
    fn test_parse_json_data() {
        let json_str = get_testing_data();


        if let Ok(json_data) = serde_json::from_str::<EndpointSignatureData>(json_str.as_str()) {
            assert_eq!(json_data.participants.len(), 2);
            assert_eq!(json_data.participants[0].server_id, 1);
            assert_eq!(json_data.participants[1].server_id, 2);
            assert_eq!(json_data.data_to_sign, "7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d");
            assert_eq!(json_data.timestamp, "16816533390");
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn test_ids() {
        let json_str = get_testing_data();

        if let Ok(json_data) = serde_json::from_str::<EndpointSignatureData>(json_str.as_str()) {
            let ids = json_data.get_ids();
            assert_eq!(ids[0], 1);
            assert_eq!(ids[1], 2);
        }
        else {
            assert!(false);
        }
    }

    #[test]
    fn test_urls() {
        let json_str = get_testing_data();

        if let Ok(json_data) = serde_json::from_str::<EndpointSignatureData>(json_str.as_str()) {
            let urls = json_data.get_urls();
            assert_eq!(urls[0], "http://127.0.0.1:3001");
            assert_eq!(urls[1], "http://127.0.0.1:3002");
        }
        else {
            assert!(false);
        }
    }
}
