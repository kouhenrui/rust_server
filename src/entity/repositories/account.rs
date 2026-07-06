//! `accounts` 表仓储。

use crate::entity::models::account::{status, AccountAuth, TABLE};
use crate::entity::repositories::sql_err;
use crate::entity::SqlBackend;
use crate::error::AppResult;
use sqlx::AnyPool;

const SELECT_AUTH_BY_USERNAME: &str = "SELECT id, password_hash, status FROM accounts \
     WHERE username = ? AND deleted_at IS NULL";

const TOUCH_LAST_LOGIN: &str = "UPDATE accounts SET last_login_at = CURRENT_TIMESTAMP WHERE id = ?";

/// `accounts` 表访问入口。
pub struct AccountRepository;

impl AccountRepository {
    pub async fn find_auth_by_username(
        pool: &AnyPool,
        username: &str,
    ) -> AppResult<Option<AccountAuth>> {
        sqlx::query_as(SELECT_AUTH_BY_USERNAME)
            .bind(username)
            .fetch_optional(pool)
            .await
            .map_err(|e| sql_err(TABLE, "find_auth_by_username", e))
    }

    pub async fn upsert(
        pool: &AnyPool,
        backend: SqlBackend,
        username: &str,
        password_hash: &str,
    ) -> AppResult<()> {
        sqlx::query(backend.accounts_upsert_sql())
            .bind(username)
            .bind(password_hash)
            .bind(status::ACTIVE)
            .execute(pool)
            .await
            .map_err(|e| sql_err(TABLE, "upsert", e))?;
        Ok(())
    }

    pub async fn touch_last_login(pool: &AnyPool, id: i64) -> AppResult<()> {
        sqlx::query(TOUCH_LAST_LOGIN)
            .bind(id)
            .execute(pool)
            .await
            .map_err(|e| sql_err(TABLE, "touch_last_login", e))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::hash_password;
    use crate::entity::test_util;

    #[tokio::test]
    async fn upsert_and_find_auth_roundtrip() {
        let pool = test_util::migrated_pool("memdb_account_repo").await;
        let hash = hash_password("secret").unwrap();
        AccountRepository::upsert(&pool, SqlBackend::Sqlite, "alice", &hash)
            .await
            .unwrap();
        let auth = AccountRepository::find_auth_by_username(&pool, "alice")
            .await
            .unwrap()
            .expect("row");
        assert!(auth.is_active());
        assert!(crate::auth::verify_password("secret", &auth.password_hash).unwrap());
    }
}
