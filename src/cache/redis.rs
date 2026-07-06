//! Redis 缓存后端。

use super::client::Cache;
use super::config::RedisConfig;
use crate::error::AppError;
use redis::aio::MultiplexedConnection;
use redis::{AsyncCommands, RedisError};

#[derive(Clone)]
pub struct RedisCache {
    conn: MultiplexedConnection,
}

impl RedisCache {
    pub async fn connect(config: &RedisConfig) -> Result<Self, AppError> {
        let info = config.into_connection_info()?;
        crate::info!(redis = %config.redacted_display(), "connecting to redis");
        let client = redis::Client::open(info)
            .map_err(|e| AppError::Internal(format!("failed to open redis connection: {e}")))?;
        let conn = client
            .get_multiplexed_async_connection()
            .await
            .map_err(|e| AppError::Internal(format!("failed to get redis connection: {e}")))?;
        Ok(Self { conn })
    }

    fn map_err(ctx: &str, e: RedisError) -> AppError {
        AppError::Internal(format!("{ctx}: {e}"))
    }
}

impl Cache for RedisCache {
    fn backend_name(&self) -> &'static str {
        "redis"
    }

    async fn ping(&self) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        conn.ping::<String>()
            .await
            .map(|_| ())
            .map_err(|e| Self::map_err("redis ping", e))
    }

    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>, AppError> {
        let mut conn = self.conn.clone();
        conn.get(key)
            .await
            .map_err(|e| Self::map_err("redis get", e))
    }

    async fn set(&self, key: &str, value: &[u8], ttl: Option<u64>) -> Result<(), AppError> {
        let mut conn = self.conn.clone();
        if let Some(secs) = ttl {
            conn.set_ex::<_, _, ()>(key, value, secs)
                .await
                .map_err(|e| Self::map_err("redis set_ex", e))?;
        } else {
            conn.set::<_, _, ()>(key, value)
                .await
                .map_err(|e| Self::map_err("redis set", e))?;
        }
        Ok(())
    }

    async fn delete(&self, keys: &[&str]) -> Result<usize, AppError> {
        let mut conn = self.conn.clone();
        conn.del(keys)
            .await
            .map_err(|e| Self::map_err("redis del", e))
    }
}
