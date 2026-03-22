mod common;

use atspicli::core::command::{CommandExecutor, CommandRequest};
use atspicli::core::execution_context::ExecutionContext;

#[test]
fn test_sensitive_node_policy_blocks_read_and_screenshot() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let get_error = executor
        .execute(
            &context,
            &CommandRequest::Get {
                locator: "input[name=password]".to_string(),
                property: "text".to_string(),
            },
        )
        .expect_err("sensitive get should fail");
    assert_eq!(get_error.exit_code(), 8);

    let snapshot_error = executor
        .execute(
            &context,
            &CommandRequest::Snapshot {
                locator: "input[name=password]".to_string(),
                depth: 3,
            },
        )
        .expect_err("sensitive snapshot should fail");
    assert_eq!(snapshot_error.exit_code(), 8);

    let screenshot_error = executor
        .execute(
            &context,
            &CommandRequest::Screenshot {
                locator: Some("input[name=password]".to_string()),
                output: "/tmp/should-not-exist.png".to_string(),
            },
        )
        .expect_err("sensitive screenshot should fail");
    assert_eq!(screenshot_error.exit_code(), 8);

    let full_screenshot_error = executor
        .execute(
            &context,
            &CommandRequest::Screenshot {
                locator: None,
                output: "/tmp/should-not-exist-full.png".to_string(),
            },
        )
        .expect_err("full screenshot should fail when any node is sensitive");
    assert_eq!(full_screenshot_error.exit_code(), 8);
}
