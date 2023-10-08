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
}

// Define a test module
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Result;

    #[test]
    fn test_parse_json_data() -> Result<()> {
        let json_str = r#"
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


        let json_data: EndpointSignatureData = serde_json::from_str(json_str)?;

        assert_eq!(json_data.participants.len(), 2);
        assert_eq!(json_data.participants[0].server_id, 1);
        assert_eq!(json_data.participants[1].server_id, 2);
        assert_eq!(json_data.data_to_sign, "7b736f6d652c6172626974726172792c646174612c746f2c7369676e7d");
        assert_eq!(json_data.timestamp, "16816533390");

        Ok(())
    }
}
