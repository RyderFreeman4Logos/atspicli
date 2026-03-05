use atspicli::core::redaction::redact_sensitive;

#[test]
fn test_log_redaction_masks_sensitive_tokens() {
    let _ = std::env::var("RUST_LOG");
    let log_sample = "action=input user=alice password=secret123 token=xyz";
    let redacted = redact_sensitive(log_sample);

    assert!(redacted.contains("password=<redacted>"));
    assert!(redacted.contains("token=<redacted>"));
    assert!(!redacted.contains("secret123"));
    assert!(!redacted.contains("xyz"));
}
