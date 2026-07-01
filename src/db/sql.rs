//! SQL 连接池（PostgreSQL / MySQL / SQLite），基于 sqlx `AnyPool`。

use super::db::DbProvider;
use crate::error::AppError;
use crate::util::redact_url;
use sqlx::AnyPool;

#[derive(Clone)]
pub struct SqlDb {
    pool: AnyPool,
    backend: &'static str,
}

impl SqlDb {
    pub async fn connect(url: &str, backend: &'static str) -> Result<Self, AppError> {
        sqlx::any::install_default_drivers();
        crate::info!(backend, url = %redact_url(url), "connecting to database");
        let pool = AnyPool::connect(url)
            .await
            .map_err(|e| AppError::Internal(format!("db connect: {e}")))?;
        Ok(Self { pool, backend })
    }

    /// 底层 sqlx 连接池，供业务层直接执行 SQL。
    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }
}

impl DbProvider for SqlDb {
    fn backend_name(&self) -> &'static str {
        self.backend
    }

    async fn ping(&self) -> Result<(), AppError> {
        sqlx::query("SELECT 1")
            .fetch_one(&self.pool)
            .await
            .map(|_| ())
            .map_err(|e| AppError::Internal(format!("db ping: {e}")))
    }
}
