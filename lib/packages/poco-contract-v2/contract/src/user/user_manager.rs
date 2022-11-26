use near_sdk::collections::LazyOption;
use near_sdk::collections::LookupMap;
use near_sdk::AccountId;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserInternalProfile {
    endpoint: LazyOption<String>
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UserProfile {
    endpoint: Option<String>
}

impl Default for UserInternalProfile {
    fn default() -> Self {
        Self { endpoint: LazyOption::new(b"e", None) }
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self { endpoint: None }
    }
}

#[derive(BorshDeserialize, BorshSerialize)]
pub struct UserManager {
    user_map: LookupMap<AccountId, UserInternalProfile>
}

impl UserManager {
    pub fn new() -> Self {
        UserManager { user_map: LookupMap::new(b"u") }
    }

    #[inline]
    pub fn get_user_profile(&self, account: &AccountId) -> UserProfile {
        self.user_map
        .get(account)
        .map(|e| UserProfile { endpoint: e.endpoint.get() })
        .unwrap_or_default()
    }

    #[inline]
    pub fn set_user_endpoint(&mut self, account: &AccountId, endpoint: String) {
        let mut profile = self
        .user_map
        .get(&account)
        .unwrap_or_default();

        profile.endpoint = LazyOption::new(b"e", Some(&endpoint));

        self.user_map.insert(account, &profile);
    }
}