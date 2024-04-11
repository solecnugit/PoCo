use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
#[borsh(crate = "near_sdk::borsh")]
pub enum MessageType {
    Transaction,
    Prepare,
    PrePrepare,
    Commit,
    RoundChange,
}

#[derive(BorshDeserialize, BorshSerialize, Debug, PartialEq)]
#[borsh(crate = "near_sdk::borsh")]
pub struct Message {
    pub phase: MessageType,
    pub data: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_serialization() {
        // Initialize a Message
        let message = Message {
            phase: MessageType::Transaction,
            data: "Test data".to_string(),
        };

        let mut buffer = Vec::new();

        // Serialize the Message
        BorshSerialize::serialize(&message, &mut buffer).unwrap();

        // Deserialize the Message
        let deserialized_message = Message::try_from_slice(&buffer).expect("Failed to deserialize Message");

        // Check that the deserialized Message is the same as the original
        assert_eq!(message, deserialized_message);
    }
}