//! 数据库连接提供者抽象；[`super::sql::SqlDb`]、[`super::mongo::MongoDb`] 为具体实现。
//!
//! 本模块只负责**建立并持有连接**（连接池 / 客户端），不包含 ORM 或 SQL 封装；
//! 业务层通过 [`DbClient::sql_pool`] / [`DbClient::mongo_client`] 直接使用 sqlx / mongodb driver。

use super::config::DbBackendConfig;
use super::mongo::MongoDb;
use super::sql::SqlDb;
use crate::error::AppError;

/// 数据库连接提供者：连接、健康检查、暴露底层 driver 句柄。
#[allow(async_fn_in_trait)]
pub trait DbProvider: Send + Sync {
    fn backend_name(&self) -> &'static str;


    async fn ping(&self) -> Result<(), AppError>;
}

/// 热插拔数据库入口（`mod.rs` 中 re-export 为 [`super::Db`]）。
#[derive(Clone)]
pub enum DbClient {
    Postgres(SqlDb),
    Mysql(SqlDb),
    Sqlite(SqlDb),
    Mongodb(MongoDb),
}

impl DbClient {
    pub async fn connect(config: &DbBackendConfig) -> Result<Self, AppError> {
        match config {
            DbBackendConfig::Postgres(cfg) => {
                let url = cfg.postgres_url()?;
                Ok(Self::Postgres(SqlDb::connect(&url, "postgres").await?))
            }
            DbBackendConfig::Mysql(cfg) => {
                let url = cfg.mysql_url()?;
                Ok(Self::Mysql(SqlDb::connect(&url, "mysql").await?))
            }
            DbBackendConfig::Sqlite(cfg) => {
                let url = cfg.sqlite_url()?;
                Ok(Self::Sqlite(SqlDb::connect(&url, "sqlite").await?))
            }
            DbBackendConfig::Mongodb(cfg) => {
                let url = cfg.mongodb_url()?;
                Ok(Self::Mongodb(MongoDb::connect(&url).await?))
            }
        }
    }

    pub fn backend_name(&self) -> &'static str {
        match self {
            Self::Postgres(b) => b.backend_name(),
            Self::Mysql(b) => b.backend_name(),
            Self::Sqlite(b) => b.backend_name(),
            Self::Mongodb(b) => b.backend_name(),
        }
    }

    pub async fn ping(&self) -> Result<(), AppError> {
        match self {
            Self::Postgres(b) => b.ping().await,
            Self::Mysql(b) => b.ping().await,
            Self::Sqlite(b) => b.ping().await,
            Self::Mongodb(b) => b.ping().await,
        }
    }

    /// SQL 连接池（PostgreSQL / MySQL / SQLite）；可直接用于 sqlx 查询。
    pub fn sql_pool(&self) -> Option<&sqlx::AnyPool> {
        match self {
            Self::Postgres(db) | Self::Mysql(db) | Self::Sqlite(db) => Some(db.pool()),
            _ => None,
        }
    }

    /// MongoDB 客户端；可直接用于 collection 操作。
    pub fn mongo_client(&self) -> Option<&mongodb::Client> {
        match self {
            Self::Mongodb(db) => Some(db.client()),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::config::DbAuth;

    fn sqlite_memory_config() -> DbBackendConfig {
        DbBackendConfig::Sqlite(DbAuth {
            url: Some("sqlite::memory:".into()),
            ..Default::default()
        })
    }

    #[tokio::test]
    async fn sqlite_provider_exposes_pool() {
        let db = DbClient::connect(&sqlite_memory_config()).await.unwrap();
        assert_eq!(db.backend_name(), "sqlite");
        assert!(db.sql_pool().is_some());
        assert!(db.mongo_client().is_none());
        db.ping().await.unwrap();
    }

    #[test]
    fn disabled_backend_is_rejected() {
        std::env::set_var("THUMBOR_DB_BACKEND", "disabled");
        let err = DbBackendConfig::from_env().unwrap_err();
        assert!(err.to_string().contains("not supported"));
    }
}
