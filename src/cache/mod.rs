//! 缓存模块：抽象见 [`cache::Cache`]，实现见 [`redis`]、[`memory`]。

pub mod cache;
pub mod config;
pub mod memory;
pub mod redis;

pub use cache::CacheClient as Cache;
pub use config::CacheBackendConfig;
