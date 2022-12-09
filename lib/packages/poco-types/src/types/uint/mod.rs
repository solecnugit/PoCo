use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use uint::construct_uint;

construct_uint! {
    #[derive(BorshDeserialize, BorshSerialize, Serialize, Deserialize)]
    #[serde(crate = "near_sdk::serde")]
    pub struct U256(4);
}