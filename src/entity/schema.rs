//! 实体表 DDL（PostgreSQL / MySQL / SQLite）。

use super::SqlBackend;

pub use super::models::tables;

pub fn accounts(backend: SqlBackend) -> &'static str {
    match backend {
        SqlBackend::Sqlite => {
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                email TEXT,
                phone TEXT,
                nickname TEXT,
                status TEXT NOT NULL DEFAULT 'active',
                last_login_at TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                deleted_at TEXT
            )
            "#
        }
        SqlBackend::Postgres => {
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id BIGSERIAL PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                email VARCHAR(255),
                phone VARCHAR(32),
                nickname VARCHAR(255),
                status VARCHAR(32) NOT NULL DEFAULT 'active',
                last_login_at TIMESTAMPTZ,
                created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
                deleted_at TIMESTAMPTZ
            )
            "#
        }
        SqlBackend::Mysql => {
            r#"
            CREATE TABLE IF NOT EXISTS accounts (
                id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                username VARCHAR(255) NOT NULL UNIQUE,
                password_hash TEXT NOT NULL,
                email VARCHAR(255),
                phone VARCHAR(32),
                nickname VARCHAR(255),
                status VARCHAR(32) NOT NULL DEFAULT 'active',
                last_login_at TIMESTAMP NULL,
                created_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
                updated_at TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP,
                deleted_at TIMESTAMP NULL
            )
            "#
        }
    }
}

pub fn casbin_rule(backend: SqlBackend) -> &'static str {
    match backend {
        SqlBackend::Sqlite => {
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
        SqlBackend::Postgres => {
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
        SqlBackend::Mysql => {
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
    }
}
