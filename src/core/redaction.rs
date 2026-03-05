pub fn redact_sensitive(input: &str) -> String {
    const SENSITIVE_KEYS: [&str; 6] =
        ["password", "passwd", "secret", "token", "apikey", "api_key"];

    input
        .split_whitespace()
        .map(|part| {
            if let Some((key, _value)) = part.split_once('=') {
                if SENSITIVE_KEYS
                    .iter()
                    .any(|sensitive| key.eq_ignore_ascii_case(sensitive))
                {
                    return format!("{key}=<redacted>");
                }
            }
            part.to_string()
        })
        .collect::<Vec<String>>()
        .join(" ")
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
}
