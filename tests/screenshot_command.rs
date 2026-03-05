mod common;

use std::fs;
use std::path::PathBuf;

use atspicli::core::command::{CommandExecutor, CommandRequest};
use atspicli::core::execution_context::ExecutionContext;

fn list_temp_artifacts() -> Vec<PathBuf> {
    fs::read_dir(std::env::temp_dir())
        .expect("temp dir should exist")
        .filter_map(|entry| entry.ok().map(|value| value.path()))
        .filter(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name.starts_with("atspicli-"))
                .unwrap_or(false)
        })
        .collect()
}

#[test]
fn test_screenshot_command_writes_output_and_cleans_temp_files() {
    for artifact in list_temp_artifacts() {
        let _ = fs::remove_file(artifact);
    }

    let backend = common::build_backend();
    let executor = CommandExecutor::new(&backend);
    let context = ExecutionContext::new(Some("demo-app".to_string()), Some(1010));

    let output = std::env::temp_dir().join(format!(
        "screenshot-output-{}-{}.txt",
        std::process::id(),
        chrono_like_timestamp()
    ));
    if output.exists() {
        fs::remove_file(&output).expect("remove old output");
    }

    executor
        .execute(
            &context,
            &CommandRequest::Screenshot {
                locator: Some("button[text=Save]".to_string()),
                output: output.to_string_lossy().to_string(),
            },
        )
        .expect("screenshot should succeed");

    assert!(output.exists(), "output screenshot file should exist");
    let data = fs::read_to_string(&output).expect("read screenshot output");
    assert!(data.contains("screenshot"));

    let leftovers = list_temp_artifacts();
    assert!(
        leftovers.is_empty(),
        "expected /tmp/atspicli-* to be cleaned, found: {leftovers:?}"
    );

    fs::remove_file(output).expect("cleanup output");
}

fn chrono_like_timestamp() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default()
}
