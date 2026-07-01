//! Casbin `casbin_rule` table migration and default policy seeding.

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

/// Create the Casbin policy table for the active SQL backend.
pub async fn migrate(pool: &AnyPool, backend: &str) -> AppResult<()> {
    let sql = match backend {
        "sqlite" => {
            r#"
            CREATE TABLE IF NOT EXISTS casbin_rule (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ptype VARCHAR(32) NOT NULL,
                v0 VARCHAR(255) NOT NULL DEFAULT '',
                v1 VARCHAR(255) NOT NULL DEFAULT '',
                v2 VARCHAR(255) NOT NULL DEFAULT '',
                v3 VARCHAR(255) NOT NULL DEFAULT '',
                v4 VARCHAR(255) NOT NULL DEFAULT '',
                v5 VARCHAR(255) NOT NULL DEFAULT '',
                UNIQUE(ptype, v0, v1, v2, v3, v4, v5)
            )
            "#
        }
        "postgres" => {
            r#"
            CREATE TABLE IF NOT EXISTS casbin_rule (
                id BIGSERIAL PRIMARY KEY,
                ptype VARCHAR(32) NOT NULL,
                v0 VARCHAR(255) NOT NULL DEFAULT '',
                v1 VARCHAR(255) NOT NULL DEFAULT '',
                v2 VARCHAR(255) NOT NULL DEFAULT '',
                v3 VARCHAR(255) NOT NULL DEFAULT '',
                v4 VARCHAR(255) NOT NULL DEFAULT '',
                v5 VARCHAR(255) NOT NULL DEFAULT '',
                CONSTRAINT casbin_rule_unique UNIQUE(ptype, v0, v1, v2, v3, v4, v5)
            )
            "#
        }
        "mysql" => {
            r#"
            CREATE TABLE IF NOT EXISTS casbin_rule (
                id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                ptype VARCHAR(32) NOT NULL,
                v0 VARCHAR(255) NOT NULL DEFAULT '',
                v1 VARCHAR(255) NOT NULL DEFAULT '',
                v2 VARCHAR(255) NOT NULL DEFAULT '',
                v3 VARCHAR(255) NOT NULL DEFAULT '',
                v4 VARCHAR(255) NOT NULL DEFAULT '',
                v5 VARCHAR(255) NOT NULL DEFAULT '',
                UNIQUE KEY casbin_rule_unique (ptype, v0, v1, v2, v3, v4, v5)
            )
            "#
        }
        other => {
            return Err(AppError::Internal(format!(
                "casbin policy storage not supported for db backend '{other}'"
            )));
        }
    };
    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("casbin migrate: {e}")))?;
    Ok(())
}

/// Insert built-in policies when the table has no rows.
pub async fn seed_if_empty(pool: &AnyPool, backend: &str) -> AppResult<()> {
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
    backend: &str,
    ptype: &str,
    rule: &[&str],
) -> AppResult<()> {
    let cols = pad_rule_slice(rule);
    let sql = match backend {
        "sqlite" => {
            r#"
            INSERT OR IGNORE INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        }
        "postgres" => {
            r#"
            INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            ON CONFLICT DO NOTHING
            "#
        }
        "mysql" => {
            r#"
            INSERT IGNORE INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#
        }
        other => {
            return Err(AppError::Internal(format!(
                "casbin insert not supported for '{other}'"
            )));
        }
    };
    sqlx::query(sql)
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
        migrate(&pool, "sqlite").await.unwrap();
        seed_if_empty(&pool, "sqlite").await.unwrap();
        seed_if_empty(&pool, "sqlite").await.unwrap();
        let (count,): (i64,) = sqlx::query_as("SELECT COUNT(*) FROM casbin_rule")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, DEFAULT_POLICIES.len() as i64);
    }
}
