//! Tracing-based logger: config, formatter, subscriber init, HTTP middleware, macros.
//!
//! ```text
//! logger
//! ├── mod.rs
//! ├── config.rs      — EnvFilter / level settings
//! ├── formatter.rs   — local time + fmt layer
//! ├── init.rs        — global subscriber bootstrap
//! ├── layer.rs       — tracing EnvFilter layer
//! └── macros.rs      — structured log helpers
//! ```

mod config;
mod formatter;
pub mod init;
mod layer;
#[macro_use]
mod macros;

pub use config::LoggerConfig;
pub use init::{init, init_with};
