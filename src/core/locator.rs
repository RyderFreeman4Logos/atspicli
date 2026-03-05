use crate::error::{AtspiCliError, Result};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocatorLimits {
    pub max_length: usize,
    pub max_segments: usize,
    pub max_predicates: usize,
    pub max_nesting: usize,
}

impl Default for LocatorLimits {
    fn default() -> Self {
        Self {
            max_length: 512,
            max_segments: 16,
            max_predicates: 24,
            max_nesting: 4,
        }
    }
}

pub fn validate_locator(locator: &str) -> Result<()> {
    validate_locator_with_limits(locator, LocatorLimits::default())
}

pub fn validate_locator_with_limits(locator: &str, limits: LocatorLimits) -> Result<()> {
    let trimmed = locator.trim();
    if trimmed.is_empty() {
        return Err(AtspiCliError::InvalidLocator(
            "Locator cannot be empty".to_string(),
        ));
    }
    if trimmed.len() > limits.max_length {
        return Err(AtspiCliError::LocatorTooComplex(format!(
            "Locator length {} exceeds limit {}",
            trimmed.len(),
            limits.max_length
        )));
    }

    if trimmed.contains(">>>") {
        return Err(AtspiCliError::InvalidLocator(
            "Invalid combinator sequence '>>>'".to_string(),
        ));
    }
    if trimmed.starts_with('>') || trimmed.ends_with('>') {
        return Err(AtspiCliError::InvalidLocator(
            "Locator cannot start or end with '>'".to_string(),
        ));
    }

    let segments = count_segments(trimmed);
    if segments > limits.max_segments {
        return Err(AtspiCliError::LocatorTooComplex(format!(
            "Locator segments {} exceed limit {}",
            segments, limits.max_segments
        )));
    }

    let nesting = max_parentheses_depth(trimmed)?;
    if nesting > limits.max_nesting {
        return Err(AtspiCliError::LocatorTooComplex(format!(
            "Locator nesting {} exceeds limit {}",
            nesting, limits.max_nesting
        )));
    }

    let predicates = count_predicates(trimmed);
    if predicates > limits.max_predicates {
        return Err(AtspiCliError::LocatorTooComplex(format!(
            "Locator predicates {} exceed limit {}",
            predicates, limits.max_predicates
        )));
    }

    for token in trimmed.split_whitespace() {
        if token.contains(':') && !token.contains(":visible") && !token.contains(":has(") {
            return Err(AtspiCliError::InvalidLocator(format!(
                "Unsupported pseudo selector in token '{token}'"
            )));
        }
    }

    if trimmed.contains("text=") {
        ensure_text_selector_has_value(trimmed, "text=")?;
    }
    if trimmed.contains("text~=") {
        ensure_text_selector_has_value(trimmed, "text~=")?;
    }

    Ok(())
}

fn count_segments(locator: &str) -> usize {
    let mut segments = 1usize;
    let bytes = locator.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        if bytes[i] == b'>' {
            if i + 1 < bytes.len() && bytes[i + 1] == b'>' {
                segments += 1;
                i += 2;
                continue;
            }
            segments += 1;
        }
        i += 1;
    }
    segments
}

fn count_predicates(locator: &str) -> usize {
    locator.matches(":visible").count()
        + locator.matches(":has(").count()
        + locator.matches("text~=").count()
        + locator.matches("text=").count()
}

fn max_parentheses_depth(locator: &str) -> Result<usize> {
    let mut depth = 0usize;
    let mut max_depth = 0usize;
    for ch in locator.chars() {
        match ch {
            '(' => {
                depth += 1;
                if depth > max_depth {
                    max_depth = depth;
                }
            }
            ')' => {
                if depth == 0 {
                    return Err(AtspiCliError::InvalidLocator(
                        "Unbalanced parentheses".to_string(),
                    ));
                }
                depth -= 1;
            }
            _ => {}
        }
    }

    if depth != 0 {
        return Err(AtspiCliError::InvalidLocator(
            "Unbalanced parentheses".to_string(),
        ));
    }
    Ok(max_depth)
}

fn ensure_text_selector_has_value(locator: &str, keyword: &str) -> Result<()> {
    let mut start_index = 0usize;
    while let Some(relative_idx) = locator[start_index..].find(keyword) {
        let idx = start_index + relative_idx + keyword.len();
        let suffix = &locator[idx..];
        if suffix.is_empty() || suffix.starts_with(' ') || suffix.starts_with('>') {
            return Err(AtspiCliError::InvalidLocator(format!(
                "Selector '{keyword}' must include a value"
            )));
        }
        start_index = idx;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{validate_locator, validate_locator_with_limits, LocatorLimits};

    #[test]
    fn test_validate_locator_accepts_simple_role() {
        assert!(validate_locator("button").is_ok());
    }

    #[test]
    fn test_validate_locator_accepts_descendant_chain() {
        assert!(validate_locator("window >> button").is_ok());
    }

    #[test]
    fn test_validate_locator_accepts_child_chain() {
        assert!(validate_locator("window > button").is_ok());
    }

    #[test]
    fn test_validate_locator_accepts_text_exact() {
        assert!(validate_locator("button[text=Save]").is_ok());
    }

    #[test]
    fn test_validate_locator_accepts_text_contains() {
        assert!(validate_locator("button[text~=Save]").is_ok());
    }

    #[test]
    fn test_validate_locator_accepts_visible() {
        assert!(validate_locator("button:visible").is_ok());
    }

    #[test]
    fn test_validate_locator_accepts_has_selector() {
        assert!(validate_locator("list:has(item[text=Done])").is_ok());
    }

    #[test]
    fn test_validate_locator_accepts_mixed_features() {
        assert!(validate_locator("window >> list:has(item[text~=Task]):visible").is_ok());
    }

    #[test]
    fn test_validate_locator_rejects_empty() {
        assert!(validate_locator("   ").is_err());
    }

    #[test]
    fn test_validate_locator_rejects_unbalanced_parentheses_left() {
        assert!(validate_locator("list:has(item").is_err());
    }

    #[test]
    fn test_validate_locator_rejects_unbalanced_parentheses_right() {
        assert!(validate_locator("list:item)").is_err());
    }

    #[test]
    fn test_validate_locator_rejects_unknown_pseudo() {
        assert!(validate_locator("button:focused").is_err());
    }

    #[test]
    fn test_validate_locator_rejects_leading_combinator() {
        assert!(validate_locator("> button").is_err());
    }

    #[test]
    fn test_validate_locator_rejects_trailing_combinator() {
        assert!(validate_locator("button >").is_err());
    }

    #[test]
    fn test_validate_locator_rejects_invalid_triple_combinator() {
        assert!(validate_locator("window >>> button").is_err());
    }

    #[test]
    fn test_validate_locator_rejects_text_without_value() {
        assert!(validate_locator("button[text= ]").is_err());
    }

    #[test]
    fn test_validate_locator_rejects_text_contains_without_value() {
        assert!(validate_locator("button[text~= ]").is_err());
    }

    #[test]
    fn test_validate_locator_limits_length() {
        let limits = LocatorLimits {
            max_length: 8,
            ..LocatorLimits::default()
        };
        assert!(validate_locator_with_limits("very-long-locator", limits).is_err());
    }

    #[test]
    fn test_validate_locator_limits_segments() {
        let limits = LocatorLimits {
            max_segments: 2,
            ..LocatorLimits::default()
        };
        assert!(validate_locator_with_limits("a > b > c", limits).is_err());
    }

    #[test]
    fn test_validate_locator_limits_predicates() {
        let limits = LocatorLimits {
            max_predicates: 1,
            ..LocatorLimits::default()
        };
        assert!(validate_locator_with_limits("button:visible[text=Save]", limits).is_err());
    }

    #[test]
    fn test_validate_locator_limits_nesting() {
        let limits = LocatorLimits {
            max_nesting: 1,
            ..LocatorLimits::default()
        };
        assert!(validate_locator_with_limits("a:has(b:has(c))", limits).is_err());
    }

    #[test]
    fn test_validate_locator_supports_nested_has_with_default_limits() {
        assert!(validate_locator("window:has(list:has(item[text=Open]))").is_ok());
    }
}
