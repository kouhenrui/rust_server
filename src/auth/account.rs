//! 账户登录业务（密码校验、JWT subject）。

use crate::auth::verify_password;
use crate::entity::AccountRepository;
use crate::error::{AppError, AppResult};
use sqlx::AnyPool;

/// 校验凭据；成功返回 username（JWT `sub`）。
pub async fn authenticate(
    pool: &AnyPool,
    username: &str,
    plain_password: &str,
) -> AppResult<String> {
    let auth = AccountRepository::find_auth_by_username(pool, username)
        .await?
        .ok_or_else(|| AppError::Unauthorized("invalid credentials".into()))?;

    if !auth.is_active() {
        return Err(AppError::Unauthorized("account is disabled".into()));
    }
    if !verify_password(plain_password, &auth.password_hash)? {
        return Err(AppError::Unauthorized("invalid credentials".into()));
    }

    AccountRepository::touch_last_login(pool, auth.id).await?;
    Ok(username.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::auth::hash_password;
    use crate::entity::{test_util, AccountRepository, SqlBackend};

    #[tokio::test]
    async fn authenticate_roundtrip() {
        let pool = test_util::migrated_pool("memdb_auth_account").await;
        let hash = hash_password("secret").unwrap();
        AccountRepository::upsert(&pool, SqlBackend::Sqlite, "alice", &hash)
            .await
            .unwrap();
        let sub = authenticate(&pool, "alice", "secret").await.unwrap();
        assert_eq!(sub, "alice");
        assert!(authenticate(&pool, "alice", "wrong").await.is_err());
    }
}
