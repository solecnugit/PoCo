use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::LazyOption;
use near_sdk::serde::Serialize;
use near_sdk::store::Vector;
use near_sdk::AccountId;
use schemars::JsonSchema;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InternalUserProfile {
    qos: Vector<u64>,
    endpoint: LazyOption<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct UserProfile {
    qos: Vec<u64>,
    endpoint: Option<String>,
}

impl InternalUserProfile {
    #[inline]
    pub fn new(account: &AccountId) -> Self {
        let prefix = account.to_string();
        let mut qos = Vector::new(format!("{}-qos-vec", prefix).as_bytes().to_vec());

        for _ in 0..64 {
            qos.push(0u64);
        }

        InternalUserProfile {
            qos,
            endpoint: LazyOption::new(format!("{}-endpoint", prefix).as_bytes().to_vec(), None),
        }
    }

    #[inline]
    pub fn get_endpoint(&self) -> Option<String> {
        self.endpoint.get()
    }

    #[inline]
    pub fn set_endpoint(&mut self, endpoint: &String) {
        self.endpoint.replace(endpoint);
    }

    #[inline]
    pub fn get_qos_vec(&self) -> &Vector<u64> {
        &self.qos
    }

    #[inline]
    pub fn get_qos_slot(&self, slot: u32) -> Option<u64> {
        self.qos.get(slot).cloned()
    }

    #[inline]
    pub fn set_qos_slot(&mut self, slot: u32, value: u64) {
        self.qos.replace(slot, value);
    }
}

impl Default for UserProfile {
    fn default() -> Self {
        Self {
            qos: Default::default(),
            endpoint: Default::default(),
        }
    }
}

impl From<&InternalUserProfile> for UserProfile {
    fn from(profile: &InternalUserProfile) -> Self {
        UserProfile {
            qos: profile.qos.iter().map(|e| e.clone()).collect(),
            endpoint: profile.endpoint.get(),
        }
    }
}

impl From<InternalUserProfile> for UserProfile {
    fn from(profile: InternalUserProfile) -> Self {
        (&profile).into()
    }
}
