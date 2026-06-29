//! Global tracing subscriber bootstrap.

use tracing_subscriber::prelude::*;

use super::config::LoggerConfig;
use super::formatter;
use super::layer;

/// Initialize logging with [`LoggerConfig::from_env`].
pub fn init() {
    init_with(LoggerConfig::from_env());
}

/// Initialize logging with an explicit config.
pub fn init_with(cfg: LoggerConfig) {
    tracing_subscriber::registry()
        .with(layer::env_filter(&cfg))
        .with(formatter::fmt_layer(&cfg))
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_filter_is_info() {
        let cfg = LoggerConfig::default();
        assert!(cfg.filter.contains("info"));
    }
}
