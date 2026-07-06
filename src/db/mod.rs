//! 数据库连接提供者：抽象见 [`client::DbProvider`]，实现见 [`sql`]、[`mongo`]。

pub mod client;
pub mod config;
pub mod mongo;
pub mod sql;

pub use client::DbClient as Db;
pub use client::DbProvider;
pub use config::DbAuth;
pub use config::DbBackendConfig;
