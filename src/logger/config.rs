//! Logger configuration loaded from environment variables.

/// Tracing subscriber settings.
#[derive(Debug, Clone)]
pub struct LoggerConfig {
    /// `EnvFilter` directive, e.g. `info,thumbor=info`.
    pub filter: String,
    /// Whether to print the `target` field in log lines.
    pub show_target: bool,
}

impl Default for LoggerConfig {
    fn default() -> Self {
        Self {
            filter: "info,thumbor=info".into(),
            show_target: false,
        }
    }
}

impl LoggerConfig {
    /// Build config from `RUST_LOG` or `THUMBOR_LOG_LEVEL`.
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(filter) = std::env::var("RUST_LOG") {
            cfg.filter = filter;
            return cfg;
        }
        if let Ok(level) = std::env::var("THUMBOR_LOG_LEVEL") {
            cfg.filter = format!("{level},thumbor={level}");
        }
        cfg
    }
}
