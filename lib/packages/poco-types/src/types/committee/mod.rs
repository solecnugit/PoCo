#[cfg(feature = "all")]
use std::fmt::{Debug, Display};
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::AccountId;

pub mod message;
