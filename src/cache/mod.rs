//! 缓存模块：抽象见 [`client::Cache`]，实现见 [`redis`]、[`memory`]。

pub mod client;
pub mod config;
pub mod memory;
pub mod redis;

pub use client::CacheClient as Cache;
pub use config::CacheBackendConfig;
