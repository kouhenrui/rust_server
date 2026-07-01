//! SQL-backed user storage for login.

use crate::auth::{hash_password, verify_password};
use crate::error::{AppError, AppResult};
use sqlx::AnyPool;

/// Run backend-specific `users` table migration.
pub async fn migrate(pool: &AnyPool, backend: &str) -> AppResult<()> {
    let sql = match backend {
        "sqlite" => {
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now'))
            )
            "#
        }
        "postgres" => {
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id BIGSERIAL PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
            )
            "#
        }
        "mysql" => {
            r#"
            CREATE TABLE IF NOT EXISTS users (
                id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP
            )
            "#
        }
        other => {
            return Err(AppError::Internal(format!(
                "user auth not supported for db backend '{other}'"
            )));
        }
    };
    sqlx::query(sql)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("users migrate: {e}")))?;
    Ok(())
}

/// Insert or replace a user (for tests and bootstrap).
pub async fn upsert_user_for_backend(
    pool: &AnyPool,
    backend: &str,
    username: &str,
    plain_password: &str,
) -> AppResult<()> {
    let password_hash = hash_password(plain_password)?;
    match backend {
        "mysql" => {
            sqlx::query(
                r#"
                INSERT INTO users (username, password_hash)
                VALUES (?, ?)
                ON DUPLICATE KEY UPDATE password_hash = VALUES(password_hash)
                "#,
            )
            .bind(username)
            .bind(&password_hash)
            .execute(pool)
            .await
        }
        _ => {
            sqlx::query(
                r#"
                INSERT INTO users (username, password_hash)
                VALUES (?, ?)
                ON CONFLICT(username) DO UPDATE SET password_hash = excluded.password_hash
                "#,
            )
            .bind(username)
            .bind(&password_hash)
            .execute(pool)
            .await
        }
    }
    .map_err(|e| AppError::Internal(format!("upsert user: {e}")))?;
    Ok(())
}

/// Verify credentials; returns the username (`sub` for JWT) on success.
pub async fn authenticate(
    pool: &AnyPool,
    username: &str,
    plain_password: &str,
) -> AppResult<String> {
    let row: Option<(String,)> = sqlx::query_as("SELECT password_hash FROM users WHERE username = ?")
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("user lookup: {e}")))?;

    let (hash,) = row.ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;
    if !verify_password(plain_password, &hash)? {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }
    Ok(username.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    async fn sqlite_pool() -> AnyPool {
        sqlx::any::install_default_drivers();
        AnyPool::connect("sqlite:file:memdb1?mode=memory&cache=shared")
            .await
            .expect("sqlite memory")
    }

    #[tokio::test]
    async fn authenticate_roundtrip() {
        let pool = sqlite_pool().await;
        migrate(&pool, "sqlite").await.unwrap();
        upsert_user_for_backend(&pool, "sqlite", "alice", "secret").await.unwrap();
        let sub = authenticate(&pool, "alice", "secret").await.unwrap();
        assert_eq!(sub, "alice");
        assert!(authenticate(&pool, "alice", "wrong").await.is_err());
    }
}
