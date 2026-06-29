//! Tracing subscriber `EnvFilter` layer.

use tracing_subscriber::EnvFilter;

use super::config::LoggerConfig;

/// Build the `EnvFilter` layer from config.
pub fn env_filter(cfg: &LoggerConfig) -> EnvFilter {
    EnvFilter::try_new(&cfg.filter).unwrap_or_else(|_| EnvFilter::new("info"))
}
