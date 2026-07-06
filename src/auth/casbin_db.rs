//! Casbin 默认策略种子（业务配置，非 DDL）。

use crate::entity::{CasbinRuleRepository, SqlBackend};
use crate::error::AppResult;
use sqlx::AnyPool;

/// Default RBAC policies seeded when the table is empty.
pub const DEFAULT_POLICIES: &[(&str, &[&str])] = &[
    ("p", &["admin", "/*", "*"]),
    ("p", &["user", "/me", "GET"]),
    ("p", &["user", "/img", "GET"]),
    ("p", &["user", "/img", "POST"]),
    ("p", &["anonymous", "/health", "GET"]),
    ("p", &["anonymous", "/login", "POST"]),
    ("p", &["anonymous", "/img", "GET"]),
    ("p", &["anonymous", "/img", "POST"]),
    ("g", &["testuser", "user"]),
];

/// Insert built-in policies when the table has no rows.
pub async fn seed_if_empty(pool: &AnyPool, backend: SqlBackend) -> AppResult<()> {
    if CasbinRuleRepository::count(pool).await? > 0 {
        return Ok(());
    }
    for (ptype, rule) in DEFAULT_POLICIES {
        CasbinRuleRepository::insert_slices(pool, backend, ptype, rule).await?;
    }
    crate::info!("casbin default policies seeded");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::test_util;

    #[tokio::test]
    async fn seed_runs_once() {
        let pool = test_util::migrated_pool("memdb_casbin_seed").await;
        seed_if_empty(&pool, SqlBackend::Sqlite).await.unwrap();
        seed_if_empty(&pool, SqlBackend::Sqlite).await.unwrap();
        assert_eq!(
            CasbinRuleRepository::count(&pool).await.unwrap(),
            DEFAULT_POLICIES.len() as i64
        );
    }
}
