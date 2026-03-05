mod common;

use atspicli::core::command::{CommandExecutor, CommandRequest};
use atspicli::core::execution_context::ExecutionContext;

#[test]
fn test_action_phase_a_click_success_and_failure() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let ok = executor.execute(
        &context,
        &CommandRequest::Click {
            locator: "button[text=Save]".to_string(),
        },
    );
    assert!(ok.is_ok());

    let fail = executor.execute(
        &context,
        &CommandRequest::Click {
            locator: "missing-button".to_string(),
        },
    );
    assert!(fail.is_err());
}

#[test]
fn test_action_phase_a_dblclick_success_and_failure() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let ok = executor.execute(
        &context,
        &CommandRequest::Dblclick {
            locator: "button[text=Save]".to_string(),
        },
    );
    assert!(ok.is_ok());

    let fail = executor.execute(
        &context,
        &CommandRequest::Dblclick {
            locator: "missing-button".to_string(),
        },
    );
    assert!(fail.is_err());
}

#[test]
fn test_action_phase_a_hover_success_and_failure() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let ok = executor.execute(
        &context,
        &CommandRequest::Hover {
            locator: "button[text=Save]".to_string(),
        },
    );
    assert!(ok.is_ok());

    let fail = executor.execute(
        &context,
        &CommandRequest::Hover {
            locator: "missing-button".to_string(),
        },
    );
    assert!(fail.is_err());
}

#[test]
fn test_action_phase_a_focus_success_and_failure() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let ok = executor.execute(
        &context,
        &CommandRequest::Focus {
            locator: "button[text=Save]".to_string(),
        },
    );
    assert!(ok.is_ok());

    let fail = executor.execute(
        &context,
        &CommandRequest::Focus {
            locator: "missing-button".to_string(),
        },
    );
    assert!(fail.is_err());
}

#[test]
fn test_action_phase_a_press_success_and_failure() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let ok = executor.execute(
        &context,
        &CommandRequest::Press {
            key: "Enter".to_string(),
        },
    );
    assert!(ok.is_ok());

    let fail = executor.execute(
        &context,
        &CommandRequest::Press {
            key: "   ".to_string(),
        },
    );
    assert!(fail.is_err());
}
