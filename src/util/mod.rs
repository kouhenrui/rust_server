//! Shared utilities: env parsing helpers and URL redaction for logs.

use std::fmt::Display;
use std::str::FromStr;

/// Parse `value` as `T`; on failure log a warning and return `None`.
pub fn parse_or_warn<T>(value: &str, warn_msg: &str) -> Option<T>
where
    T: FromStr,
    T::Err: Display,
{
    match value.parse::<T>() {
        Ok(v) => Some(v),
        Err(e) => {
            crate::warn!(value = %value, error = %e, "{warn_msg}");
            None
        }
    }
}

/// Redact credentials in URLs for safe logging (`user:***@host` / `:***@host`).
pub fn redact_url(url: &str) -> String {
    if let Some(scheme_end) = url.find("://") {
        let (scheme, rest) = url.split_at(scheme_end + 3);
        if let Some(at) = rest.find('@') {
            let (auth, host_part) = rest.split_at(at + 1);
            let user = auth
                .strip_suffix('@')
                .and_then(|a| a.split(':').next())
                .unwrap_or("");
            let redacted = if auth.contains(':') {
                if user.is_empty() {
                    ":***@".to_string()
                } else {
                    format!("{user}:***@")
                }
            } else {
                format!("{auth}@")
            };
            return format!("{scheme}{redacted}{host_part}");
        }
    }
    url.to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_or_warn_ok() {
        assert_eq!(parse_or_warn::<u16>("8080", "bad port"), Some(8080));
    }

    #[test]
    fn parse_or_warn_none_on_invalid() {
        assert!(parse_or_warn::<u16>("not-a-port", "bad port").is_none());
    }

    #[test]
    fn redact_url_hides_password() {
        let out = redact_url("redis://user:secret@127.0.0.1:6379/0");
        assert!(out.contains("user:***@"));
        assert!(!out.contains("secret"));
    }
}
