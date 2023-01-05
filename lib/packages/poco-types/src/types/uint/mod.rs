use impl_serde::impl_uint_serde;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::{Deserialize, Serialize};
use schemars::JsonSchema;
use uint::construct_uint;

construct_uint! {
    #[derive(BorshDeserialize, BorshSerialize, JsonSchema)]
    #[serde(crate = "near_sdk::serde")]
    pub struct U256(4);
}

impl_uint_serde!(U256, 4);
