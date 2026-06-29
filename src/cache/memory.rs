//! 进程内 LRU 内存缓存。

use super::cache::Cache;
use super::config::MemoryConfig;
use crate::error::AppError;
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::RwLock;

struct Entry {
    value: Vec<u8>,
    expires_at: Option<Instant>,
}

struct Inner {
    map: HashMap<String, Entry>,
    lru: VecDeque<String>,
    max_entries: usize,
    default_ttl: Option<Duration>,
}

#[derive(Clone)]
pub struct MemoryCache {
    inner: Arc<RwLock<Inner>>,
}

impl MemoryCache {
    pub fn new(config: &MemoryConfig) -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                map: HashMap::new(),
                lru: VecDeque::new(),
                max_entries: config.max_entries,
                default_ttl: config.default_ttl,
            })),
        }
    }

    fn is_expired(entry: &Entry) -> bool {
        entry.expires_at.is_some_and(|at| Instant::now() >= at)
    }

    fn touch(inner: &mut Inner, key: &str) {
        if let Some(pos) = inner.lru.iter().position(|k| k == key) {
            inner.lru.remove(pos);
        }
        inner.lru.push_back(key.to_string());
    }

    fn remove_key(inner: &mut Inner, key: &str) {
        inner.map.remove(key);
        if let Some(pos) = inner.lru.iter().position(|k| k == key) {
            inner.lru.remove(pos);
        }
    }

    fn evict_if_needed(inner: &mut Inner) {
        inner.lru.retain(|k| inner.map.contains_key(k));
        while inner.map.len() > inner.max_entries {
            if let Some(old) = inner.lru.pop_front() {
                inner.map.remove(&old);
            } else {
                break;
            }
        }
    }
}

impl Cache for MemoryCache {
    fn backend_name(&self) -> &'static str {
        "memory"
    }

    async fn ping(&self) -> Result<(), AppError> {
        Ok(())
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, AppError> {
        let mut inner = self.inner.write().await;
        let Some(entry) = inner.map.get(key) else {
            return Ok(None);
        };
        if Self::is_expired(entry) {
            Self::remove_key(&mut inner, key);
            return Ok(None);
        }
        let value = entry.value.clone();
        Self::touch(&mut inner, key);
        Ok(Some(value))
    }

    async fn set(
        &self,
        key: &str,
        value: &[u8],
        ttl: Option<u64>,
    ) -> Result<(), AppError> {
        let mut inner = self.inner.write().await;
        let ttl = ttl.map(Duration::from_secs).or(inner.default_ttl);
        inner.map.insert(
            key.to_string(),
            Entry {
                value: value.to_vec(),
                expires_at: ttl.map(|d| Instant::now() + d),
            },
        );
        Self::touch(&mut inner, key);
        Self::evict_if_needed(&mut inner);
        Ok(())
    }

    async fn delete(&self, keys: &[&str]) -> Result<usize, AppError> {
        let mut inner = self.inner.write().await;
        let mut removed = 0usize;
        for key in keys {
            if inner.map.contains_key(*key) {
                Self::remove_key(&mut inner, *key);
                removed += 1;
            }
        }
        Ok(removed)
    }
}
