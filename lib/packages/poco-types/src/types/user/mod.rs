use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::serde::Serialize;
use near_sdk::store::{LazyOption, UnorderedMap};
use near_sdk::AccountId;
use schemars::JsonSchema;

use crate::types::uint::U256;

#[derive(BorshDeserialize, BorshSerialize)]
pub struct OnchainUserProfile {
    props: UnorderedMap<String, U256>,
    endpoint: LazyOption<String>,
    region: LazyOption<String>,
}

#[derive(Serialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct UserProperty<'a> {
    pub key: &'a str,
    pub value: &'a U256,
}

#[derive(Serialize, JsonSchema)]
#[serde(crate = "near_sdk::serde")]
pub struct UserProfile<'a> {
    props: Vec<UserProperty<'a>>,
    endpoint: &'a Option<String>,
}

impl OnchainUserProfile {
    #[inline]
    pub fn new(account: &AccountId) -> Self {
        let prefix = account.to_string();
        let props = UnorderedMap::new(format!("{prefix}:props").as_bytes().to_vec());

        OnchainUserProfile {
            props,
            endpoint: LazyOption::new(format!("{prefix}:endpoint").as_bytes().to_vec(), None),
            region: LazyOption::new(format!("{prefix}:region").as_bytes().to_vec(), None),
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
    pub fn unset_endpoint(&mut self) {
        self.endpoint.set(None);
    }

    #[inline]
    pub fn get_region(&self) -> &Option<String> {
        self.region.get()
    }

    #[inline]
    pub fn set_region(&mut self, region: String) {
        self.region.replace(region);
    }

    #[inline]
    pub fn unset_region(&mut self) {
        self.region.set(None)
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

impl<'a, 'b: 'a> From<&'b OnchainUserProfile> for UserProfile<'a> {
    fn from(profile: &'b OnchainUserProfile) -> Self {
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
