use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::store::{LazyOption, UnorderedMap};
use near_sdk::AccountId;
use near_sdk::schemars::JsonSchema;

use crate::types::uint::U256;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct InternalUserProfile {
    props: UnorderedMap<String, U256>,
    endpoint: LazyOption<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
#[schemars(crate = "near_sdk::schemars")]
pub struct UserProperty<'a> {
    pub key: &'a str,
    pub value: &'a U256,
}

#[derive(Serialize, JsonSchema)]
#[schemars(crate = "near_sdk::schemars")]
#[serde(crate = "near_sdk::serde")]
pub struct UserProfile<'a> {
    props: Vec<UserProperty<'a>>,
    endpoint: &'a Option<String>,
}

// 问题：在这个Props可能用来放什么东西呢？
// 1. hash签名
// 2. 余额
impl InternalUserProfile {
    #[inline]
    pub fn new(account: &AccountId) -> Self {
        let prefix = account.to_string();
        let props = UnorderedMap::new(format!("{prefix}:props").as_bytes().to_vec());

        InternalUserProfile {
            props,
            endpoint: LazyOption::new(format!("{prefix}:endpoint").as_bytes().to_vec(), None),
        }
    }

    #[inline]
    pub fn get_endpoint(&self) -> &Option<String> {
        self.endpoint.get()
    }

    #[inline]
    pub fn set_endpoint(&mut self, endpoint: String) {
        self.endpoint.replace(endpoint);
    }

    #[inline]
    pub fn get_prop(&self, name: &str) -> Option<&U256> {
        self.props.get(name)
    }

    #[inline]
    pub fn set_prop(&mut self, name: &str, value: &U256) {
        self.props.insert(name.to_string(), *value);
    }
}

impl<'a, 'b: 'a> From<&'b InternalUserProfile> for UserProfile<'a> {
    fn from(profile: &'b InternalUserProfile) -> Self {
        UserProfile {
            props: profile
                .props
                .iter()
                .map(|(key, value)| UserProperty { key, value })
                .collect(),
            endpoint: profile.endpoint.get(),
        }
    }
}
