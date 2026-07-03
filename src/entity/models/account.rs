//! 与 `accounts` 账户表对应的实体。

/// 表名。
pub const TABLE: &str = "accounts";

/// 账户状态。
#[allow(dead_code)]
pub mod status {
    pub const ACTIVE: &str = "active";
    pub const DISABLED: &str = "disabled";
    pub const LOCKED: &str = "locked";
}

/// 账户（登录主体，JWT `sub` = `username`）。
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct Account {
    pub id: i64,
    pub username: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub nickname: Option<String>,
    pub status: String,
    pub last_login_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
    /// 软删除时间；`None` 表示未删除。
    pub deleted_at: Option<String>,
}

/// 登录校验所需字段。
#[derive(Debug, sqlx::FromRow)]
pub struct AccountAuth {
    pub id: i64,
    pub password_hash: String,
    pub status: String,
}

impl Account {
    pub fn select_by_username() -> &'static str {
        "SELECT id, username, password_hash, email, phone, nickname, status, \
         last_login_at, created_at, updated_at, deleted_at FROM accounts \
         WHERE username = ? AND deleted_at IS NULL"
    }

    pub fn is_deleted(&self) -> bool {
        self.deleted_at.is_some()
    }

    pub fn is_active(&self) -> bool {
        self.status == status::ACTIVE && !self.is_deleted()
    }
}

impl AccountAuth {
    pub fn select_by_username() -> &'static str {
        "SELECT id, password_hash, status FROM accounts \
         WHERE username = ? AND deleted_at IS NULL"
    }

    pub fn is_active(&self) -> bool {
        self.status == status::ACTIVE
    }
}
