//! 缓存统一抽象；[`super::redis::RedisCache`]、[`super::memory::MemoryCache`] 为具体实现。

use super::config::CacheBackendConfig;
use super::memory::MemoryCache;
use super::redis::RedisCache;
use crate::error::AppError;

/// 缓存后端统一抽象。
#[allow(async_fn_in_trait)]
pub trait Cache: Send + Sync {
    fn backend_name(&self) -> &'static str;

    fn is_enabled(&self) -> bool {
        self.backend_name() != "disabled"
    }

    async fn ping(&self) -> Result<(), AppError>;

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, AppError>;

    async fn set(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<u64>,
    ) -> Result<(), AppError>;

    async fn delete(&self, keys: &[&str]) -> Result<usize, AppError>;
}

#[derive(Debug, Clone, Copy, Default)]
struct NoopCache;

const NOOP: NoopCache = NoopCache;

impl Cache for NoopCache {
    fn backend_name(&self) -> &'static str {
        "disabled"
    }

    async fn ping(&self) -> Result<(), AppError> {
        Ok(())
    }

    async fn get(&self, _key: &str) -> Result<Option<Vec<u8>>, AppError> {
        Ok(None)
    }

    async fn set(
        &self,
        _key: &str,
        _value: &[u8],
        _ttl: Option<u64>,
    ) -> Result<(), AppError> {
        Ok(())
    }

    async fn delete(&self, _keys: &[&str]) -> Result<usize, AppError> {
        Ok(0)
    }
}

/// 热插拔缓存入口（`mod.rs` 中 re-export 为 [`super::Cache`]）。
#[derive(Clone)]
pub enum CacheClient {
    Disabled,
    Redis(RedisCache),
    Memory(MemoryCache),
}

/// 同步方法委托
macro_rules! delegate_sync {
    ($self:expr, $method:ident ()) => {
        match $self {
            CacheClient::Disabled => Cache::$method(&NOOP),
            CacheClient::Redis(b) => Cache::$method(b),
            CacheClient::Memory(b) => Cache::$method(b),
        }
    };
}

/// 异步方法委托
macro_rules! delegate_async {
    ($self:expr, $method:ident ( $($arg:expr),* $(,)? )) => {
        match $self {
            CacheClient::Disabled => Cache::$method(&NOOP, $($arg),*).await,
            CacheClient::Redis(b) => Cache::$method(b, $($arg),*).await,
            CacheClient::Memory(b) => Cache::$method(b, $($arg),*).await,
        }
    };
}

impl CacheClient {
    pub fn disabled() -> Self {
        Self::Disabled
    }

    pub async fn connect(config: &CacheBackendConfig) -> Result<Self, AppError> {
        match config {
            CacheBackendConfig::Disabled => Ok(Self::disabled()),
            CacheBackendConfig::Redis(cfg) => {
                Ok(Self::Redis(RedisCache::connect(cfg).await?))
            }
            CacheBackendConfig::Memory(cfg) => Ok(Self::Memory(MemoryCache::new(cfg))),
        }
    }

    pub fn backend_name(&self) -> &'static str {
        delegate_sync!(self, backend_name())
    }

    pub fn is_enabled(&self) -> bool {
        delegate_sync!(self, is_enabled())
    }

    pub async fn ping(&self) -> Result<(), AppError> {
        delegate_async!(self, ping())
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, AppError> {
        delegate_async!(self, get(key))
    }

    pub async fn set(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<u64>,
    ) -> Result<(), AppError> {
        delegate_async!(self, set(key, value, ttl))
    }

    pub async fn delete(&self, keys: &[&str]) -> Result<usize, AppError> {
        delegate_async!(self, delete(keys))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cache::config::MemoryConfig;

    #[tokio::test]
    async fn disabled_cache_is_no_op() {
        let cache = CacheClient::disabled();
        assert!(!cache.is_enabled());
        cache.set("k", b"v", Some(60)).await.unwrap();
        assert!(cache.get("k").await.unwrap().is_none());
    }

    #[tokio::test]
    async fn memory_backend_roundtrip() {
        let cache = CacheClient::Memory(MemoryCache::new(&MemoryConfig {
            max_entries: 8,
            default_ttl: None,
        }));
        cache.set("img", b"png-bytes", None).await.unwrap();
        assert_eq!(cache.get("img").await.unwrap(), Some(b"png-bytes".to_vec()));
        assert_eq!(cache.delete(&["img"]).await.unwrap(), 1);
    }
}
