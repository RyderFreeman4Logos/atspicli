mod common;

use std::time::Duration;

use atspicli::core::command::CommandBackend;
use atspicli::core::model::NodeDescriptor;

#[test]
fn test_wait_succeeds_for_existing_node() {
    let backend = common::build_backend();
    let result = backend.wait_for("root", Duration::from_secs(1));
    assert!(result.is_ok());
}

#[test]
fn test_wait_times_out_for_missing_node() {
    let backend = common::build_backend();
    let result = backend.wait_for("nonexistent", Duration::from_millis(300));
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("Timeout"));
}

#[test]
fn test_wait_succeeds_for_delayed_node() {
    let backend = common::build_backend();
    let node = NodeDescriptor::new("delayed-button");
    backend.add_delayed_node(node, Duration::from_millis(200));
    let result = backend.wait_for("delayed-button", Duration::from_secs(2));
    assert!(result.is_ok());
}

#[test]
fn test_wait_times_out_when_delay_exceeds_timeout() {
    let backend = common::build_backend();
    let node = NodeDescriptor::new("slow-element");
    backend.add_delayed_node(node, Duration::from_secs(5));
    let result = backend.wait_for("slow-element", Duration::from_millis(300));
    assert!(result.is_err());
}
