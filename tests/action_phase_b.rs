mod common;

use atspicli::core::command::{CommandExecutor, CommandRequest};
use atspicli::core::execution_context::ExecutionContext;

#[test]
fn test_action_phase_b_input_focus_failure_fallback() {
    let backend = common::build_backend();
    backend.set_focus_failure("input[name=username]");

    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let result = executor.execute(
        &context,
        &CommandRequest::Input {
            locator: "input[name=username]".to_string(),
            text: "alice".to_string(),
        },
    );
    assert!(result.is_ok(), "input should fallback when focus fails");

    let text = executor
        .execute(
            &context,
            &CommandRequest::Get {
                locator: "input[name=username]".to_string(),
                property: "text".to_string(),
            },
        )
        .expect("get text");
    assert_eq!(text.render().expect("rendered"), "alice");
}

#[test]
fn test_action_phase_b_fill_clears_then_inputs() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let result = executor.execute(
        &context,
        &CommandRequest::Fill {
            locator: "input[name=username]".to_string(),
            text: "bob".to_string(),
        },
    );
    assert!(result.is_ok(), "fill should succeed");

    let events = backend.take_events();
    assert!(
        events
            .iter()
            .any(|event| event == "clear:input[name=username]"),
        "fill should clear before input"
    );
    assert!(
        events
            .iter()
            .any(|event| event == "input:input[name=username]:bob"),
        "fill should write new value after clear"
    );
}
