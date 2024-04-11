use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::store::LookupMap;
use near_sdk::{AccountId, NearToken};
use poco_types::types::user::{InternalUserProfile, UserProfile};

#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct UserManager {
    user_map: LookupMap<AccountId, InternalUserProfile>,
    stake_map: LookupMap<AccountId, NearToken>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            user_map: LookupMap::new(b"user-manager:usermap".to_vec()),
            stake_map: LookupMap::new(b"user-manager:stakemap".to_vec()),
        }
    }

    #[inline]
    pub fn get_user_profile(&self, account: &AccountId) -> UserProfile {
        self.user_map.get(account).map(|e| e.into()).unwrap()
    }

    #[inline]
    pub fn get_user_stake(&self, account: &AccountId) -> NearToken {
        self.stake_map.get(account).unwrap().clone()
    }

    #[inline]
    pub fn set_user_stake(&mut self, account: &AccountId, stake: u128) {
        let stake_token = NearToken::from_yoctonear(stake);
        self.stake_map.insert(account.clone(), stake_token);
    }

    #[inline]
    pub fn set_user_endpoint(&mut self, account: &AccountId, endpoint: String) {
        if self.user_map.contains_key(account) {
            self.user_map
                .get_mut(account)
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_stake() {
        let mut user_manager = UserManager::new();

        let user1 = AccountId::try_from("user1".to_string()).unwrap();
        let user2 = AccountId::try_from("user2".to_string()).unwrap();
        let user3 = AccountId::try_from("user3".to_string()).unwrap();

        // Add some stakes
        user_manager.set_user_stake(&user1, 50_u128);
        user_manager.set_user_stake(&user2, 30_u128);
        user_manager.set_user_stake(&user3, 20_u128);

        // Check the stakes
        assert_eq!(
            user_manager.get_user_stake(&user1),
            NearToken::from_yoctonear(50_u128)
        );
        assert_eq!(
            user_manager.get_user_stake(&user2),
            NearToken::from_yoctonear(30_u128)
        );
        assert_eq!(
            user_manager.get_user_stake(&user3),
            NearToken::from_yoctonear(20_u128)
        );
    }
}
