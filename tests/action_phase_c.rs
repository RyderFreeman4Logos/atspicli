mod common;

use atspicli::core::command::{CommandExecutor, CommandRequest};
use atspicli::core::execution_context::ExecutionContext;

#[test]
fn test_action_phase_c_scroll_to_success_and_failure() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let ok = executor.execute(
        &context,
        &CommandRequest::ScrollTo {
            locator: "button[text=Save]".to_string(),
        },
    );
    assert!(ok.is_ok());

    let fail = executor.execute(
        &context,
        &CommandRequest::ScrollTo {
            locator: "missing".to_string(),
        },
    );
    assert!(fail.is_err());
}

#[test]
fn test_action_phase_c_scroll_direction_validation() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let ok = executor.execute(
        &context,
        &CommandRequest::Scroll {
            direction: "down".to_string(),
            amount: 3,
        },
    );
    assert!(ok.is_ok());

    let err = executor
        .execute(
            &context,
            &CommandRequest::Scroll {
                direction: "diagonal".to_string(),
                amount: 3,
            },
        )
        .expect_err("invalid direction should fail");
    assert_eq!(err.exit_code(), 9);
}
