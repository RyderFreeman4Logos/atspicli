use atspicli::core::locator::{validate_locator_with_limits, LocatorLimits};

#[test]
fn test_locator_limits_length() {
    let limits = LocatorLimits {
        max_length: 8,
        ..LocatorLimits::default()
    };
    assert!(validate_locator_with_limits("very-long-locator", limits).is_err());
}

#[test]
fn test_locator_limits_nesting() {
    let limits = LocatorLimits {
        max_nesting: 1,
        ..LocatorLimits::default()
    };
    assert!(validate_locator_with_limits("a:has(b:has(c))", limits).is_err());
}

#[test]
fn test_locator_limits_predicate_count() {
    let limits = LocatorLimits {
        max_predicates: 2,
        ..LocatorLimits::default()
    };
    assert!(
        validate_locator_with_limits("a:visible:has(b[text=1]):has(c[text=2])", limits).is_err()
    );
}
