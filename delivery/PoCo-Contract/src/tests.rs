use near_sdk::test_utils::{accounts, VMContextBuilder};
use near_sdk::testing_env;

use crate::models::task::TranscodingRequirement;
use crate::models::task::TaskStatus;
use crate::MediaTranscodingContract;

fn get_context(predecessor: near_sdk::AccountId) -> VMContextBuilder {
    let mut builder = VMContextBuilder::new();
    builder.predecessor_account_id(predecessor);
    builder
}

#[test]
fn test_new() {
    let context = get_context(accounts(1));
    testing_env!(context.build());
    
    let contract = MediaTranscodingContract::new(None, None, None, None);
    assert_eq!(contract.owner_id, accounts(1));
}

#[test]
fn test_publish_task() {
    let context = get_context(accounts(1));
    testing_env!(context.build());
    
    let mut contract = MediaTranscodingContract::new(None, None, None, None);
    
    let requirements = TranscodingRequirement {
        target_codec: "h264".to_string(),
        target_resolution: "1080p".to_string(),
        target_bitrate: "8000k".to_string(),
        target_framerate: "30".to_string(),
        additional_params: "{}".to_string(),
    };
    
    let task = contract.publish_task("ipfs://source_file".to_string(), requirements);
    
    assert_eq!(task.broadcaster_id, accounts(1));
    assert_eq!(task.status, TaskStatus::Published);
    assert!(contract.get_task(task.task_id).is_some());
}