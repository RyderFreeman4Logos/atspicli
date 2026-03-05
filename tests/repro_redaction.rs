use atspicli::core::redaction::redact_sensitive;

#[test]
fn test_redact_sensitive_leaves_unkeyed_plaintext_unchanged() {
    let secret = "hunter2";
    let redacted = redact_sensitive(secret);
    // `redact_sensitive` is key-pattern-based and intentionally does not classify
    // unstructured plaintext as sensitive by itself.
    assert_eq!(
        redacted, secret,
        "unkeyed plaintext should remain unchanged in this helper"
    );
}
