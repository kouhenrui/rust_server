//! 数据库模块配置。

use crate::error::AppError;
use crate::util::parse_or_warn;

/// 数据库连接参数：完整 URL 或 host / port / database + 账号密码。
#[derive(Debug, Clone, Default)]
pub struct DbAuth {
    /// 完整连接 URL；设置后忽略下方分立字段。
    pub url: Option<String>,
    pub host: String,
    pub port: u16,
    pub database: String,
    /// SQLite 文件路径（`url` 未设置时用于拼装 `sqlite://`）。
    pub path: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
}

impl DbAuth {
    pub fn from_env(kind: &str) -> Self {
        let url = std::env::var("THUMBOR_DB_URL")
            .ok()
            .filter(|s| !s.is_empty());

        let mut auth = Self {
            url,
            host: std::env::var("THUMBOR_DB_HOST").unwrap_or_else(|_| "127.0.0.1".into()),
            port: 0,
            database: std::env::var("THUMBOR_DB_NAME").unwrap_or_else(|_| "thumbor".into()),
            path: None,
            username: None,
            password: None,
        };

        if let Ok(v) = std::env::var("THUMBOR_DB_PORT") {
            if let Some(p) = parse_or_warn(&v, "invalid THUMBOR_DB_PORT") {
                auth.port = p;
            }
        }

        if let Ok(v) = std::env::var("THUMBOR_DB_USERNAME") {
            if !v.is_empty() {
                auth.username = Some(v);
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_DB_PASSWORD") {
            if !v.is_empty() {
                auth.password = Some(v);
            }
        }

        if kind == "sqlite" && auth.url.is_none() {
            let path = std::env::var("THUMBOR_DB_PATH").unwrap_or_else(|_| "thumbor.db".into());
            auth.path = Some(path);
            auth.url = Some(format!("sqlite://{}?mode=rwc", auth.path.as_ref().unwrap()));
        }

        auth
    }

    pub fn url_or_build(
        &self,
        scheme: &str,
        default_port: u16,
    ) -> Result<String, AppError> {
        if let Some(url) = &self.url {
            if !url.is_empty() {
                return Ok(url.clone());
            }
        }
        let userinfo = match (&self.username, &self.password) {
            (Some(u), Some(p)) => format!("{u}:{p}@"),
            (None, Some(p)) => format!(":{p}@"),
            (Some(u), None) => format!("{u}@"),
            (None, None) => String::new(),
        };
        Ok(format!(
            "{scheme}://{userinfo}{}:{}/{}",
            self.host,
            if self.port == 0 { default_port } else { self.port },
            self.database
        ))
    }

    pub fn postgres_url(&self) -> Result<String, AppError> {
        self.url_or_build("postgres", 5432)
    }

    pub fn mysql_url(&self) -> Result<String, AppError> {
        self.url_or_build("mysql", 3306)
    }

    pub fn sqlite_url(&self) -> Result<String, AppError> {
        self.url_or_build("sqlite", 0)
    }

    pub fn mongodb_url(&self) -> Result<String, AppError> {
        self.url_or_build("mongodb", 27017)
    }
}

/// 热插拔后端选择，由 `THUMBOR_DB_BACKEND` 驱动。
#[derive(Debug, Clone)]
pub enum DbBackendConfig {
    Postgres(DbAuth),
    Mysql(DbAuth),
    Sqlite(DbAuth),
    Mongodb(DbAuth),
}

impl DbBackendConfig {
    /// 从环境变量加载；未设置 `THUMBOR_DB_BACKEND` 时默认 `sqlite`。
    pub fn from_env() -> Result<Self, AppError> {
        let backend = std::env::var("THUMBOR_DB_BACKEND")
            .unwrap_or_else(|_| "sqlite".into())
            .to_ascii_lowercase();

        Ok(match backend.as_str() {
            "postgres" | "postgresql" => Self::Postgres(DbAuth::from_env("postgres")),
            "mysql" => Self::Mysql(DbAuth::from_env("mysql")),
            "sqlite" => Self::Sqlite(DbAuth::from_env("sqlite")),
            "mongodb" | "mongo" => Self::Mongodb(DbAuth::from_env("mongodb")),
            "disabled" | "none" | "off" => {
                return Err(AppError::Internal(
                    "THUMBOR_DB_BACKEND=disabled is not supported; database connection is required"
                        .into(),
                ));
            }
            other => {
                return Err(AppError::Internal(format!(
                    "unknown THUMBOR_DB_BACKEND: {other}"
                )));
            }
        })
    }
}
