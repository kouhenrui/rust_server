//! SQL-backed account storage for login.

use crate::auth::{hash_password, verify_password};
use crate::entity::{AccountAuth, SqlBackend};
use crate::error::{AppError, AppResult};
use sqlx::AnyPool;

/// Insert or replace an account (for tests and bootstrap).
pub async fn upsert_account_for_backend(
    pool: &AnyPool,
    backend: SqlBackend,
    username: &str,
    plain_password: &str,
) -> AppResult<()> {
    let password_hash = hash_password(plain_password)?;
    match backend {
        SqlBackend::Mysql => {
            sqlx::query(
                r#"
                INSERT INTO accounts (username, password_hash, status)
                VALUES (?, ?, 'active')
                ON DUPLICATE KEY UPDATE
                    password_hash = VALUES(password_hash),
                    status = 'active',
                    deleted_at = NULL,
                    updated_at = CURRENT_TIMESTAMP
                "#,
            )
            .bind(username)
            .bind(&password_hash)
            .execute(pool)
            .await
        }
        SqlBackend::Postgres => {
            sqlx::query(
                r#"
                INSERT INTO accounts (username, password_hash, status)
                VALUES (?, ?, 'active')
                ON CONFLICT(username) DO UPDATE SET
                    password_hash = excluded.password_hash,
                    status = 'active',
                    deleted_at = NULL,
                    updated_at = NOW()
                "#,
            )
            .bind(username)
            .bind(&password_hash)
            .execute(pool)
            .await
        }
        SqlBackend::Sqlite => {
            sqlx::query(
                r#"
                INSERT INTO accounts (username, password_hash, status)
                VALUES (?, ?, 'active')
                ON CONFLICT(username) DO UPDATE SET
                    password_hash = excluded.password_hash,
                    status = 'active',
                    deleted_at = NULL,
                    updated_at = datetime('now')
                "#,
            )
            .bind(username)
            .bind(&password_hash)
            .execute(pool)
            .await
        }
    }
    .map_err(|e| AppError::Internal(format!("upsert account: {e}")))?;
    Ok(())
}

/// Verify credentials; returns the username (`sub` for JWT) on success.
pub async fn authenticate(
    pool: &AnyPool,
    username: &str,
    plain_password: &str,
) -> AppResult<String> {
    let auth: Option<AccountAuth> = sqlx::query_as(AccountAuth::select_by_username())
        .bind(username)
        .fetch_optional(pool)
        .await
        .map_err(|e| AppError::Internal(format!("account lookup: {e}")))?;

    let auth = auth.ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;
    if !auth.is_active() {
        return Err(AppError::Unauthorized("account is disabled".into()));
    }
    if !verify_password(plain_password, &auth.password_hash)? {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    touch_last_login(pool, auth.id).await?;
    Ok(username.to_string())
}

async fn touch_last_login(pool: &AnyPool, id: i64) -> AppResult<()> {
    sqlx::query("UPDATE accounts SET last_login_at = CURRENT_TIMESTAMP WHERE id = ?")
        .bind(id)
        .execute(pool)
        .await
        .map_err(|e| AppError::Internal(format!("update last_login_at: {e}")))?;
    Ok(())
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
        crate::entity::migrate(&pool, SqlBackend::Sqlite)
            .await
            .unwrap();
        upsert_account_for_backend(&pool, SqlBackend::Sqlite, "alice", "secret")
            .await
            .unwrap();
        let sub = authenticate(&pool, "alice", "secret").await.unwrap();
        assert_eq!(sub, "alice");
        assert!(authenticate(&pool, "alice", "wrong").await.is_err());
    }
}
