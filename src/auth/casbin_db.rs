//! Casbin `casbin_rule` 默认策略种子。

use crate::entity::SqlBackend;
use crate::error::{AppError, AppResult};
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
    let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM casbin_rule")
        .fetch_one(pool)
        .await
        .map_err(|e| AppError::Internal(format!("casbin count: {e}")))?;
    if count > 0 {
        return Ok(());
    }
    for (ptype, rule) in DEFAULT_POLICIES {
        insert_rule(pool, backend, ptype, rule).await?;
    }
    crate::info!("casbin default policies seeded");
    Ok(())
}

pub(crate) async fn insert_rule(
    pool: &AnyPool,
    backend: SqlBackend,
    ptype: &str,
    rule: &[&str],
) -> AppResult<()> {
    let cols = pad_rule_slice(rule);
    sqlx::query(backend.casbin_insert_sql())
        .bind(ptype)
        .bind(&cols[0])
        .bind(&cols[1])
        .bind(&cols[2])
        .bind(&cols[3])
        .bind(&cols[4])
        .bind(&cols[5])
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("casbin seed insert: {e}")))?;
    Ok(())
}

pub(crate) fn pad_rule_vec(rule: &[String]) -> [String; 6] {
    let mut cols = std::array::from_fn(|_| String::new());
    for (i, value) in rule.iter().enumerate().take(6) {
        cols[i] = value.clone();
    }
    cols
}

fn pad_rule_slice(rule: &[&str]) -> [String; 6] {
    let mut cols = std::array::from_fn(|_| String::new());
    for (i, value) in rule.iter().enumerate().take(6) {
        cols[i] = (*value).to_string();
    }
    cols
}

pub(crate) fn trim_rule(cols: [String; 6]) -> Vec<String> {
    let mut rule = cols.to_vec();
    while rule.last().is_some_and(|s| s.is_empty()) {
        rule.pop();
    }
    rule
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn pool() -> AnyPool {
        sqlx::any::install_default_drivers();
        AnyPool::connect("sqlite:file:memdb2?mode=memory&cache=shared")
            .await
            .expect("sqlite")
    }

    #[tokio::test]
    async fn seed_runs_once() {
        let pool = pool().await;
        crate::entity::migrate(&pool, SqlBackend::Sqlite).await.unwrap();
        seed_if_empty(&pool, SqlBackend::Sqlite).await.unwrap();
        seed_if_empty(&pool, SqlBackend::Sqlite).await.unwrap();
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM casbin_rule")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, DEFAULT_POLICIES.len() as i64);
    }
}
