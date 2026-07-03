//! 实体表：struct 定义、DDL 与迁移入口。

mod models;
mod schema;
mod sql_backend;

pub use models::{Account, AccountAuth, CasbinRule, CasbinRulePolicy};
pub use schema::tables;
pub use sql_backend::SqlBackend;

use crate::error::{AppError, AppResult};
use sqlx::AnyPool;

/// 创建/更新全部实体表。
pub async fn migrate(pool: &AnyPool, backend: SqlBackend) -> AppResult<()> {
    for ddl in [schema::accounts(backend), schema::casbin_rule(backend)] {
        sqlx::query(ddl)
            .execute(pool)
            .await
            .map_err(|e| AppError::Internal(format!("entity migrate: {e}")))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn pool() -> AnyPool {
        sqlx::any::install_default_drivers();
        AnyPool::connect("sqlite:file:memdb_entity?mode=memory&cache=shared")
            .await
            .expect("sqlite")
    }

    #[tokio::test]
    async fn migrate_creates_all_tables() {
        let pool = pool().await;
        migrate(&pool, SqlBackend::Sqlite).await.unwrap();

        let (accounts,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='accounts'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        let (casbin,): (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='casbin_rule'",
        )
        .fetch_one(&pool)
        .await
        .unwrap();
        assert_eq!(accounts, 1);
        assert_eq!(casbin, 1);
    }
}
