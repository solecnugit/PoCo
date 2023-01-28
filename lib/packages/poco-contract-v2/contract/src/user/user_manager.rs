use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use poco_types::types::user::{InternalUserProfile, UserProfile};

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserManager {
    user_map: LookupMap<AccountId, InternalUserProfile>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            user_map: LookupMap::new(b"user-manager:usermap".to_vec()),
        }
    }

    #[inline]
    pub fn get_user_profile(&self, account: &AccountId) -> UserProfile {
        self.user_map.get(account).map(|e| e.into()).unwrap()
    }

    #[inline]
    pub fn set_user_endpoint(&mut self, account: &AccountId, endpoint: String) {
        if self.user_map.contains_key(&account) {
            self.user_map
                .get_mut(&account)
                .unwrap()
                .set_endpoint(endpoint);
        } else {
            let mut profile = InternalUserProfile::new(account);

            profile.set_endpoint(endpoint);

            self.user_map.insert(account.clone(), profile);
        }
    }

    #[inline]
    pub fn get_user_endpoint(&self, account: &AccountId) -> Option<&str> {
        self.user_map
            .get(account)
            .and_then(|e| e.get_endpoint().as_ref().map(|e| e.as_str()))
    }
}

impl Default for UserManager {
    fn default() -> Self {
        Self::new()
    }
}