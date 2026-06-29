//! Cache 模块配置：后端选择与各实现专有参数。

use crate::error::AppError;
use redis::{ConnectionAddr, ConnectionInfo, IntoConnectionInfo, RedisConnectionInfo};
use std::time::Duration;

/// 缓存后端通用认证参数（用户名 / 密码）。
#[derive(Debug, Clone, Default)]
pub struct CacheAuth {
    pub username: Option<String>,
    pub password: Option<String>,
}

impl CacheAuth {
    /// 从指定环境变量读取认证信息；空字符串视为未设置。
    pub fn from_env(username_var: &str, password_var: &str) -> Self {
        let mut auth = Self::default();
        if let Ok(v) = std::env::var(username_var) {
            if !v.is_empty() {
                auth.username = Some(v);
            }
        }
        if let Ok(v) = std::env::var(password_var) {
            if !v.is_empty() {
                auth.password = Some(v);
            }
        }
        auth
    }

    /// 日志脱敏展示中的 `user:***@` 或 `:***@` 前缀（不含 scheme）。
    pub fn redacted_userinfo(&self) -> String {
        let user = self.username.as_deref().unwrap_or("");
        if self.password.is_some() {
            if user.is_empty() {
                ":***@".to_string()
            } else {
                format!("{user}:***@")
            }
        } else if !user.is_empty() {
            format!("{user}@")
        } else {
            String::new()
        }
    }
}

/// 热插拔后端选择，由 `THUMBOR_CACHE_BACKEND` 驱动。
#[derive(Debug, Clone)]
pub enum CacheBackendConfig {
    /// 不启用缓存（读写均为 no-op）。
    Disabled,
    Redis(RedisConfig),
    Memory(MemoryConfig),
}

impl Default for CacheBackendConfig {
    fn default() -> Self {
        Self::Disabled
    }
}

impl CacheBackendConfig {
    /// 从 `THUMBOR_CACHE_BACKEND` 及对应子配置环境变量加载。
    ///
    /// | `THUMBOR_CACHE_BACKEND` | 行为 |
    /// |-------------------------|------|
    /// | `redis`                 | 连接 Redis |
    /// | `memory`                | 进程内内存缓存 |
    /// | `disabled` / `none` / `off` / 未设置 | 禁用 |
    pub fn from_env() -> Self {
        let backend = std::env::var("THUMBOR_CACHE_BACKEND")
            .unwrap_or_else(|_| "disabled".into())
            .to_ascii_lowercase();

        match backend.as_str() {
            "redis" => Self::Redis(RedisConfig::from_env()),
            "memory" => Self::Memory(MemoryConfig::from_env()),
            "disabled" | "none" | "off" => Self::Disabled,
            other => {
                crate::warn!(
                    backend = %other,
                    "unknown THUMBOR_CACHE_BACKEND, cache disabled"
                );
                Self::Disabled
            }
        }
    }
}

/// Redis 连接参数。可用完整 URL，或分立 host / port / 账号 / 密码。
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// 完整连接 URL；设置后忽略下方分立字段。
    pub url: Option<String>,
    pub host: String,
    pub port: u16,
    pub db: i64,
    pub auth: CacheAuth,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: None,
            host: "127.0.0.1".into(),
            port: 6379,
            db: 0,
            auth: CacheAuth::default(),
        }
    }
}

impl RedisConfig {
    pub fn from_env() -> Self {
        if let Ok(url) = std::env::var("THUMBOR_REDIS_URL") {
            if !url.is_empty() {
                return Self {
                    url: Some(url),
                    ..Self::default()
                };
            }
        }

        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("THUMBOR_REDIS_HOST") {
            cfg.host = v;
        }
        if let Ok(v) = std::env::var("THUMBOR_REDIS_PORT") {
            match v.parse() {
                Ok(p) => cfg.port = p,
                Err(e) => crate::warn!(error = %e, "invalid THUMBOR_REDIS_PORT"),
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_REDIS_DB") {
            match v.parse() {
                Ok(db) => cfg.db = db,
                Err(e) => crate::warn!(error = %e, "invalid THUMBOR_REDIS_DB"),
            }
        }
        cfg.auth = CacheAuth::from_env("THUMBOR_REDIS_USERNAME", "THUMBOR_REDIS_PASSWORD");
        cfg
    }

    pub fn into_connection_info(&self) -> Result<ConnectionInfo, AppError> {
        if let Some(url) = &self.url {
            return url
                .as_str()
                .into_connection_info()
                .map_err(|e| AppError::Internal(format!("invalid redis url: {e}")));
        }
        Ok(ConnectionInfo {
            addr: ConnectionAddr::Tcp(self.host.clone(), self.port),
            redis: RedisConnectionInfo {
                db: self.db,
                username: self.auth.username.clone(),
                password: self.auth.password.clone(),
                ..RedisConnectionInfo::default()
            },
        })
    }

    pub fn redacted_display(&self) -> String {
        if let Some(url) = &self.url {
            return redact_url_credentials(url);
        }
        format!(
            "redis://{}{}:{}/{}",
            self.auth.redacted_userinfo(),
            self.host,
            self.port,
            self.db
        )
    }
}

/// 进程内内存缓存参数。
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    /// 最大条目数；超出时按 LRU 淘汰最久未访问项。
    pub max_entries: usize,
    /// 默认 TTL；`None` 表示条目永不过期（直到 LRU 淘汰）。
    pub default_ttl: Option<Duration>,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            max_entries: 1024,
            default_ttl: Some(Duration::from_secs(3600)),
        }
    }
}

impl MemoryConfig {
    pub fn from_env() -> Self {
        let mut cfg = Self::default();
        if let Ok(v) = std::env::var("THUMBOR_CACHE_MEMORY_MAX_ENTRIES") {
            match v.parse() {
                Ok(n) if n > 0 => cfg.max_entries = n,
                Ok(_) => crate::warn!("THUMBOR_CACHE_MEMORY_MAX_ENTRIES must be > 0"),
                Err(e) => crate::warn!(error = %e, "invalid THUMBOR_CACHE_MEMORY_MAX_ENTRIES"),
            }
        }
        if let Ok(v) = std::env::var("THUMBOR_CACHE_MEMORY_TTL_SECS") {
            match v.parse::<u64>() {
                Ok(0) => cfg.default_ttl = None,
                Ok(secs) => cfg.default_ttl = Some(Duration::from_secs(secs)),
                Err(e) => crate::warn!(error = %e, "invalid THUMBOR_CACHE_MEMORY_TTL_SECS"),
            }
        }
        cfg
    }
}

fn redact_url_credentials(url: &str) -> String {
    if let Some(scheme_end) = url.find("://") {
        let (scheme, rest) = url.split_at(scheme_end + 3);
        if let Some(at) = rest.find('@') {
            let (auth, host_part) = rest.split_at(at + 1);
            let user = auth
                .strip_suffix('@')
                .and_then(|a| a.split(':').next())
                .unwrap_or("");
            let redacted = if auth.contains(':') {
                if user.is_empty() {
                    ":***@".to_string()
                } else {
                    format!("{user}:***@")
                }
            } else {
                format!("{auth}@")
            };
            return format!("{scheme}{redacted}{host_part}");
        }
    }
    url.to_string()
}
