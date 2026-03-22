mod common;

use atspicli::core::command::{CommandExecutor, CommandOutput, CommandRequest};
use atspicli::core::execution_context::ExecutionContext;

#[test]
fn test_readonly_commands_snapshot_get_wait_and_list_apps() {
    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let snapshot = executor
        .execute(
            &context,
            &CommandRequest::Snapshot {
                locator: "button[text=Save]".to_string(),
                depth: 3,
            },
        )
        .expect("snapshot should succeed");
    match snapshot {
        CommandOutput::Text(value) => assert!(value.contains("button[text=Save]")),
        other => panic!("unexpected output: {other:?}"),
    }

    let get = executor
        .execute(
            &context,
            &CommandRequest::Get {
                locator: "button[text=Save]".to_string(),
                property: "role".to_string(),
            },
        )
        .expect("get should succeed");
    assert_eq!(get, CommandOutput::Text("button".to_string()));

    let wait = executor.execute(
        &context,
        &CommandRequest::Wait {
            locator: "root".to_string(),
            timeout_secs: 1,
        },
    );
    assert!(wait.is_ok());

    let apps = executor
        .execute(&context, &CommandRequest::ListApps)
        .expect("list-apps should succeed");
    match apps {
        CommandOutput::AppList(items) => assert_eq!(items.len(), 1),
        other => panic!("unexpected output: {other:?}"),
    }
}
