//! 数据库连接提供者：抽象见 [`db::DbProvider`]，实现见 [`sql`]、[`mongo`]。

pub mod config;
pub mod db;
pub mod mongo;
pub mod sql;

pub use db::DbClient as Db;
pub use db::DbProvider;
pub use config::DbAuth;
pub use config::DbBackendConfig;
