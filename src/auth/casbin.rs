//! Casbin RBAC enforcer (RESTful model with keyMatch2).
//!
//! 策略持久化在关系型数据库的 `casbin_rule` 表（PostgreSQL / MySQL / SQLite），
//! 不支持 MongoDB。

use crate::config::Config;
use crate::error::{AppError, AppResult};
use casbin::{CoreApi, DefaultModel, Enforcer, MgmtApi};
use sqlx::AnyPool;
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

use super::casbin_adapter::SqlxAnyAdapter;
use super::casbin_db::{seed_if_empty};
use crate::entity::SqlBackend;

const DEFAULT_MODEL: &str = include_str!("../../config/casbin_model.conf");

/// Thread-safe Casbin enforcer wrapper with SQL-backed policies.
#[derive(Clone)]
pub struct CasbinAuth {
    enforcer: Arc<RwLock<Enforcer>>,
}

impl CasbinAuth {
    /// Load model from file and policies from the `casbin_rule` SQL table.
    pub async fn new(config: &Config, pool: &AnyPool, backend: SqlBackend) -> AppResult<Self> {
        seed_if_empty(pool, backend).await?;

        let model = load_model(&config.casbin_model).await?;
        let adapter = SqlxAnyAdapter::new(pool.clone(), backend);
        let enforcer = Enforcer::new(model, adapter)
            .await
            .map_err(|e| AppError::Internal(format!("casbin init: {e}")))?;
        Ok(Self {
            enforcer: Arc::new(RwLock::new(enforcer)),
        })
    }

    /// Check whether `subject` may perform `act` on `obj` (HTTP path).
    pub async fn enforce(&self, subject: &str, obj: &str, act: &str) -> AppResult<bool> {
        let enforcer = self.enforcer.read().await;
        enforcer
            .enforce((subject, obj, act))
            .map_err(|e| AppError::Internal(format!("casbin enforce: {e}")))
    }

    /// Assign `role` to `user` (e.g. `admin`, `user`); persisted in `casbin_rule`.
    pub async fn add_role_for_user(&self, user: &str, role: &str) -> AppResult<()> {
        let mut enforcer = self.enforcer.write().await;
        enforcer
            .add_grouping_policy(vec![user.to_string(), role.to_string()])
            .await
            .map_err(|e| AppError::Internal(format!("casbin add role: {e}")))?;
        Ok(())
    }
}

async fn load_model(path: &Path) -> AppResult<DefaultModel> {
    if path.is_file() {
        DefaultModel::from_file(path)
            .await
            .map_err(|e| AppError::Internal(format!("casbin model {}: {e}", path.display())))
    } else {
        DefaultModel::from_str(DEFAULT_MODEL)
            .await
            .map_err(|e| AppError::Internal(format!("casbin default model: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    async fn auth() -> CasbinAuth {
        sqlx::any::install_default_drivers();
        let pool = AnyPool::connect("sqlite:file:memdb3?mode=memory&cache=shared")
            .await
            .unwrap();
        crate::entity::migrate(&pool, SqlBackend::Sqlite)
            .await
            .unwrap();
        CasbinAuth::new(&Config::default(), &pool, SqlBackend::Sqlite)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn anonymous_can_health_but_not_me() {
        let auth = auth().await;
        assert!(auth.enforce("anonymous", "/health", "GET").await.unwrap());
        assert!(!auth.enforce("anonymous", "/me", "GET").await.unwrap());
    }

    #[tokio::test]
    async fn user_role_can_me() {
        let auth = auth().await;
        assert!(auth.enforce("testuser", "/me", "GET").await.unwrap());
    }

    #[tokio::test]
    async fn add_role_persists() {
        let auth = auth().await;
        auth.add_role_for_user("alice", "user").await.unwrap();
        assert!(auth.enforce("alice", "/me", "GET").await.unwrap());
    }
}
