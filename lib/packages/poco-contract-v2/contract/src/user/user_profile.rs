use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::serde::Serialize;

#[derive(BorshDeserialize, BorshSerialize)]
pub(crate) struct InternalUserProfile {
    endpoint: LazyOption<String>,
}

#[derive(Serialize)]
#[serde(crate = "near_sdk::serde")]
pub struct UserProfile {
    endpoint: Option<String>,
}

impl Default for InternalUserProfile {
    fn default() -> Self {
        Self {
            endpoint: LazyOption::new(b"e", None),
        }
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self { endpoint: None }
    }
}

impl InternalUserProfile {
    #[inline]
    pub fn get_endpoint(&self) -> Option<String> {
        self.endpoint.get()
    }

    #[inline]
    pub fn set_endpoint(&mut self, endpoint: &String) {
        self.endpoint.replace(endpoint);
    }
}

impl UserProfile {
    pub fn new() -> Self {
        UserProfile { endpoint: None }
    }
}

impl From<InternalUserProfile> for UserProfile {
    fn from(profile: InternalUserProfile) -> Self {
        UserProfile {
            endpoint: profile.get_endpoint(),
        }
    }
}
