pub use format::{pretty_bytes, pretty_gas};
pub use misc::either_of;

mod format;
mod misc;

pub fn convert_account_id_from_sdk_to_primitives(
    account_id: &near_sdk::AccountId,
) -> near_primitives::types::AccountId {
    account_id.as_str().parse().unwrap()
}

pub fn convert_account_id_from_primitives_to_sdk(
    account_id: &near_primitives::types::AccountId,
) -> near_sdk::AccountId {
    account_id.as_str().parse().unwrap()
}