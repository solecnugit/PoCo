use std::collections::HashSet;
#[cfg(feature = "protocol")]
use near_sdk::{AccountId, store::LookupSet};
#[cfg(feature = "native")]
use near_primitives::types::AccountId;

pub type RegionId = String;

pub struct Region {
    pub id: RegionId,
    #[cfg(feature = "native")]
    pub members: HashSet<AccountId>,
    #[cfg(feature = "protocol")]
    pub members: LookupSet<AccountId>,
}

impl Region {
    #[cfg(feature = "native")]
    pub fn new(region_id: RegionId) -> Self {
        Self {
            id: region_id,
            members: HashSet::new(),
        }
    }

    #[cfg(feature = "protocol")]
    pub fn new(region_id: RegionId) -> Self {
        Self {
            members: LookupSet::new(format!("region:{}:members", region_id).as_bytes().to_vec()),
            id: region_id,
        }
    }

    #[inline]
    pub fn add_member(&mut self, member_id: AccountId) {
        self.members.insert(member_id);
    }

    #[inline]
    pub fn remove_member(&mut self, member_id: AccountId) {
        self.members.remove(&member_id);
    }

    #[inline]
    pub fn is_member(&self, member_id: &AccountId) -> bool {
        self.members.contains(member_id)
    }
}