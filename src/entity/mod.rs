//! 实体表：struct 定义、DDL、迁移与仓储。

pub(crate) mod models;
pub(crate) mod repositories;
mod schema;
mod sql_backend;

#[cfg(test)]
pub(crate) mod test_util;

pub use models::{Account, AccountAuth, CasbinRulePolicy};
pub(crate) use repositories::casbin_rule::trim_rule;
pub use repositories::{AccountRepository, CasbinRuleRepository};
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

    #[tokio::test]
    async fn migrate_creates_all_tables() {
        let pool = test_util::migrated_pool("memdb_entity").await;

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
