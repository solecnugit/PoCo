use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LookupMap;
use near_sdk::AccountId;

use super::InternalUserProfile;
use super::UserProfile;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserManager {
    user_map: LookupMap<AccountId, InternalUserProfile>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            user_map: LookupMap::new(b"u"),
        }
    }

    #[inline]
    pub fn get_user_profile(&self, account: &AccountId) -> UserProfile {
        self.user_map
            .get(account)
            .map(|e| e.into())
            .unwrap_or_default()
    }

    #[inline]
    pub fn set_user_endpoint(&mut self, account: &AccountId, endpoint: &String) {
        let mut profile = self.user_map.get(&account).unwrap_or_default();

        profile.set_endpoint(endpoint);

        self.user_map.insert(account, &profile);
    }

    #[inline]
    pub fn get_user_endpoint(&self, account: &AccountId) -> Option<String> {
        self.user_map
            .get(account)
            .map(|e| e.get_endpoint())
            .flatten()
    }
}
