//! 关系型数据库方言（Casbin / 用户表仅支持 SQL 后端）。

use crate::error::{AppError, AppResult};

/// 实体表所支持的关系型后端。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SqlBackend {
    Postgres,
    Mysql,
    Sqlite,
}

impl SqlBackend {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Mysql => "mysql",
            Self::Sqlite => "sqlite",
        }
    }

    pub fn from_db_backend(name: &str) -> Option<Self> {
        match name {
            "postgres" => Some(Self::Postgres),
            "mysql" => Some(Self::Mysql),
            "sqlite" => Some(Self::Sqlite),
            _ => None,
        }
    }

    /// 从 [`crate::db::Db`] 解析；MongoDB 等非 SQL 后端返回明确错误。
    pub fn require_from_db(db: &crate::db::Db) -> AppResult<(Self, &sqlx::AnyPool)> {
        let pool = db.sql_pool().ok_or_else(|| {
            AppError::Internal(
                "entity tables require a relational database (postgres, mysql, or sqlite); \
                 mongodb is not supported"
                    .into(),
            )
        })?;
        let backend = Self::from_db_backend(db.backend_name()).ok_or_else(|| {
            AppError::Internal(format!(
                "entity tables require postgres, mysql, or sqlite; got '{}'",
                db.backend_name()
            ))
        })?;
        Ok((backend, pool))
    }

    pub fn accounts_upsert_sql(self) -> &'static str {
        match self {
            Self::Mysql => {
                r#"
                INSERT INTO accounts (username, password_hash, status)
                VALUES (?, ?, ?)
                ON DUPLICATE KEY UPDATE
                    password_hash = VALUES(password_hash),
                    status = VALUES(status),
                    deleted_at = NULL,
                    updated_at = CURRENT_TIMESTAMP
                "#
            }
            Self::Postgres => {
                r#"
                INSERT INTO accounts (username, password_hash, status)
                VALUES (?, ?, ?)
                ON CONFLICT(username) DO UPDATE SET
                    password_hash = excluded.password_hash,
                    status = excluded.status,
                    deleted_at = NULL,
                    updated_at = NOW()
                "#
            }
            Self::Sqlite => {
                r#"
                INSERT INTO accounts (username, password_hash, status)
                VALUES (?, ?, ?)
                ON CONFLICT(username) DO UPDATE SET
                    password_hash = excluded.password_hash,
                    status = excluded.status,
                    deleted_at = NULL,
                    updated_at = datetime('now')
                "#
            }
        }
    }

    pub fn casbin_insert_sql(self) -> &'static str {
        match self {
            Self::Sqlite => {
                r#"
                INSERT OR IGNORE INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#
            }
            Self::Postgres => {
                r#"
                INSERT INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                ON CONFLICT DO NOTHING
                "#
            }
            Self::Mysql => {
                r#"
                INSERT IGNORE INTO casbin_rule (ptype, v0, v1, v2, v3, v4, v5)
                VALUES (?, ?, ?, ?, ?, ?, ?)
                "#
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_relational_backends_are_supported() {
        assert_eq!(
            SqlBackend::from_db_backend("postgres"),
            Some(SqlBackend::Postgres)
        );
        assert_eq!(
            SqlBackend::from_db_backend("mysql"),
            Some(SqlBackend::Mysql)
        );
        assert_eq!(
            SqlBackend::from_db_backend("sqlite"),
            Some(SqlBackend::Sqlite)
        );
        assert!(SqlBackend::from_db_backend("mongodb").is_none());
    }
}
