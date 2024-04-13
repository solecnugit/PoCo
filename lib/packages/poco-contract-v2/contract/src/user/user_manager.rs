use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::UnorderedMap;
use near_sdk::store::LookupMap;
use near_sdk::{AccountId, NearToken};
use poco_types::types::user::{InternalUserProfile, UserProfile};

// for ease of testing
use std::fs::File;
use std::io::{self, BufRead};
use std::path::Path;
use std::collections::HashMap;


#[derive(BorshDeserialize, BorshSerialize)]
#[borsh(crate = "near_sdk::borsh")]
pub struct UserManager {
    user_map: LookupMap<AccountId, InternalUserProfile>,
    stake_map: UnorderedMap<AccountId, NearToken>,
}

impl UserManager {
    pub fn new() -> Self {
        UserManager {
            user_map: LookupMap::new(b"user-manager:usermap".to_vec()),
            stake_map: UnorderedMap::new(b"user-manager:stakemap".to_vec()),
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
        self.stake_map.insert(account, &stake_token);
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

    #[inline]
    pub fn get_stake_map(&self) -> &UnorderedMap<AccountId, NearToken> {
        &self.stake_map
    }

    pub fn load_stake_map_from_file<P>(&mut self, path: P) -> io::Result<HashMap<AccountId, NearToken>>
    where
        P: AsRef<Path>,
    {
        let file = File::open(path)?;
        let reader = io::BufReader::new(file);

        // let mut stake_map = self.stake_map;

        for line in reader.lines() {
            let line = line?;
            let parts: Vec<&str> = line.split(' ').collect();
            let account = AccountId::try_from(parts[0].to_string()).unwrap();
            let stake = parts[1].parse::<u128>().unwrap();

            self.set_user_stake(&account, stake);
        }

        Ok((self.stake_map).to_vec().into_iter().collect())
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
    use std::path::PathBuf;

    #[test]
    fn test_load_stake_map_from_file() {
        // Create a temporary file and write some data to it
        let test_file_path = PathBuf::from("/home/sole/PoCo/lib/packages/poco-contract-v2/contract/temp-file/stake.txt");

        // Create a UserManager instance
        let mut user_manager = UserManager::new();

        println!("{:?}", test_file_path);
        println!("{:?}", user_manager.get_stake_map());

        // Call the method to be tested
        let result = user_manager.load_stake_map_from_file(&test_file_path);

        // Check the result
        assert!(result.is_ok(), "Loading stake map from file failed");

        // Check the contents of the stake map
        let stake_map = user_manager.get_stake_map();
        let accountid = AccountId::try_from("user1".to_string()).unwrap();
        let stake = NearToken::from_yoctonear(1000000000000_u128);
        assert_eq!(stake_map.get(&accountid), Some(stake));
        // assert_eq!(stake_map.get("account2"), Some(&200));
    }

    #[test]
    fn test_insert_unorderedmap() {
        let mut user_manager = UserManager::new();
        let account = AccountId::try_from("user1".to_string()).unwrap();
        let stake = 50_u128;
        // let stake_token = NearToken::from_yoctonear(stake);
        // self.stake_map.insert(account, &stake_token);
        user_manager.set_user_stake(&account, stake);
        let stake = 70_u128;
        println!("{:?}", user_manager.get_user_stake(&account));
        assert_eq!(
            user_manager.get_user_stake(&account),
            NearToken::from_yoctonear(stake)
        );
    }

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
