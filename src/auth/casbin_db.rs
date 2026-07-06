//! Casbin 默认策略种子（业务配置，非 DDL）。

use crate::entity::{CasbinRuleRepository, SqlBackend};
use crate::error::AppResult;
use sqlx::AnyPool;

fn api_route(api_prefix: &str, suffix: &str) -> String {
    format!("{api_prefix}{suffix}")
}

/// Default RBAC policies for the given API prefix.
/// Paths must stay in sync with [`crate::config::Config::api_prefix`].
pub fn default_policies(api_prefix: &str) -> Vec<(&'static str, Vec<String>)> {
    vec![
        ("p", vec!["admin".into(), "/*".into(), "*".into()]),
        ("p", vec!["user".into(), api_route(api_prefix, "/me"), "GET".into()]),
        (
            "p",
            vec!["user".into(), api_route(api_prefix, "/img"), "GET".into()],
        ),
        (
            "p",
            vec![
                "user".into(),
                api_route(api_prefix, "/img"),
                "POST".into(),
            ],
        ),
        (
            "p",
            vec![
                "anonymous".into(),
                api_route(api_prefix, "/health"),
                "GET".into(),
            ],
        ),
        (
            "p",
            vec![
                "anonymous".into(),
                api_route(api_prefix, "/login"),
                "POST".into(),
            ],
        ),
        (
            "p",
            vec![
                "anonymous".into(),
                api_route(api_prefix, "/img"),
                "GET".into(),
            ],
        ),
        (
            "p",
            vec![
                "anonymous".into(),
                api_route(api_prefix, "/img"),
                "POST".into(),
            ],
        ),
        ("g", vec!["testuser".into(), "user".into()]),
    ]
}

/// Insert built-in policies when the table has no rows.
pub async fn seed_if_empty(
    pool: &AnyPool,
    backend: SqlBackend,
    api_prefix: &str,
) -> AppResult<()> {
    if CasbinRuleRepository::count(pool).await? > 0 {
        return Ok(());
    }
    for (ptype, rule) in default_policies(api_prefix) {
        CasbinRuleRepository::insert(pool, backend, ptype, &rule).await?;
    }
    crate::info!("casbin default policies seeded");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::DEFAULT_API_PREFIX;
    use crate::entity::test_util;

    #[tokio::test]
    async fn seed_runs_once() {
        let pool = test_util::migrated_pool("memdb_casbin_seed").await;
        seed_if_empty(&pool, SqlBackend::Sqlite, DEFAULT_API_PREFIX)
            .await
            .unwrap();
        seed_if_empty(&pool, SqlBackend::Sqlite, DEFAULT_API_PREFIX)
            .await
            .unwrap();
        assert_eq!(
            CasbinRuleRepository::count(&pool).await.unwrap(),
            default_policies(DEFAULT_API_PREFIX).len() as i64
        );
    }
}
