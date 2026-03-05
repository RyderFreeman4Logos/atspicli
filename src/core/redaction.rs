use regex::Regex;
use std::sync::OnceLock;
use tracing::warn;

const SENSITIVE_KEY_PATTERN: &str = r"password|passwd|secret|token|api[_-]?key|access[_-]?token|refresh[_-]?token|client[_-]?secret";

struct RedactionPatterns {
    query_param: Regex,
    quoted_key_value: Regex,
    key_value: Regex,
    authorization_header: Regex,
    bearer_token: Regex,
}

impl RedactionPatterns {
    fn build() -> std::result::Result<Self, regex::Error> {
        let query_param = Regex::new(&format!(
            r"(?i)([?&](?:{})=)[^&#\s]+",
            SENSITIVE_KEY_PATTERN
        ))?;
        let quoted_key_value = Regex::new(&format!(
            r#"(?i)("(?:{})"\s*:\s*)"[^"]*""#,
            SENSITIVE_KEY_PATTERN
        ))?;
        let key_value = Regex::new(&format!(
            "(?i)\\b({})\\b(\\s*[:=]\\s*)(\"[^\"]*\"|'[^']*'|[^\\s,;&]+)",
            SENSITIVE_KEY_PATTERN
        ))?;
        let authorization_header =
            Regex::new(r"(?i)\b(authorization\s*:\s*(?:bearer|basic)\s+)[^\s,;]+")?;
        let bearer_token = Regex::new(r"(?i)\b(bearer\s+)[A-Za-z0-9._~+/-]+=*")?;

        Ok(Self {
            query_param,
            quoted_key_value,
            key_value,
            authorization_header,
            bearer_token,
        })
    }
}

fn redaction_patterns() -> Option<&'static RedactionPatterns> {
    static PATTERNS: OnceLock<std::result::Result<RedactionPatterns, regex::Error>> =
        OnceLock::new();
    PATTERNS.get_or_init(RedactionPatterns::build).as_ref().ok()
}

pub fn redact_sensitive(input: &str) -> String {
    let Some(patterns) = redaction_patterns() else {
        warn!("Regex patterns failed to compile in redact_sensitive, falling back to full redaction.");
        return "<redacted>".to_string();
    };

    let mut redacted = input.to_string();
    redacted = patterns
        .query_param
        .replace_all(&redacted, "$1<redacted>")
        .into_owned();
    redacted = patterns
        .quoted_key_value
        .replace_all(&redacted, "$1\"<redacted>\"")
        .into_owned();
    redacted = patterns
        .key_value
        .replace_all(&redacted, "$1$2<redacted>")
        .into_owned();
    redacted = patterns
        .authorization_header
        .replace_all(&redacted, "$1<redacted>")
        .into_owned();
    patterns
        .bearer_token
        .replace_all(&redacted, "$1<redacted>")
        .into_owned()
}

#[cfg(test)]
mod tests {
    use super::redact_sensitive;

    #[test]
    fn test_redact_sensitive_masks_known_keys() {
        let input = "user=alice password=hunter2 token=abc123";
        let output = redact_sensitive(input);
        assert!(output.contains("password=<redacted>"));
        assert!(output.contains("token=<redacted>"));
        assert!(!output.contains("hunter2"));
        assert!(!output.contains("abc123"));
    }

    #[test]
    fn test_redact_sensitive_masks_json_and_authorization() {
        let input =
            r#"payload={"password":"hunter2","apiKey":"xyz"} Authorization: Bearer abc.def"#;
        let output = redact_sensitive(input);
        assert!(output.contains(r#""password":"<redacted>""#));
        assert!(output.contains(r#""apiKey":"<redacted>""#));
        assert!(output.contains("Authorization: Bearer <redacted>"));
        assert!(!output.contains("hunter2"));
        assert!(!output.contains("xyz"));
        assert!(!output.contains("abc.def"));
    }

    #[test]
    fn test_redact_sensitive_masks_query_params() {
        let input = "https://example.test/path?token=abc123&page=1&api_key=qwe";
        let output = redact_sensitive(input);
        assert!(output.contains("?token=<redacted>"));
        assert!(output.contains("&api_key=<redacted>"));
        assert!(output.contains("&page=1"));
        assert!(!output.contains("abc123"));
        assert!(!output.contains("qwe"));
    }
}
