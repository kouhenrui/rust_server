//! HTTP client wrapper for remote image sources.

use crate::error::{AppError, AppResult};
use std::time::Duration;

/// Thin wrapper around `reqwest::Client` with project defaults.
#[derive(Clone)]
pub struct HttpClient {
    inner: reqwest::Client,
}

impl HttpClient {
    /// Build a client with the given request timeout.
    pub fn build(timeout: Duration) -> AppResult<Self> {
        let inner = reqwest::Client::builder()
            .timeout(timeout)
            .user_agent("thumbor/0.1")
            .build()
            .map_err(|e| AppError::Internal(format!("http client: {e}")))?;
        Ok(Self { inner })
    }

    /// `GET` a URL and return the body bytes, enforcing `max_bytes`.
    pub async fn fetch(&self, url: &str, max_bytes: usize) -> AppResult<Vec<u8>> {
        let resp = self.inner.get(url).send().await?;
        if !resp.status().is_success() {
            return Err(AppError::Upstream(format!("status {}", resp.status())));
        }
        if let Some(len) = resp.content_length() {
            if len as usize > max_bytes {
                return Err(AppError::SourceTooLarge { max: max_bytes });
            }
        }
        let bytes = resp.bytes().await?;
        if bytes.len() > max_bytes {
            return Err(AppError::SourceTooLarge { max: max_bytes });
        }
        Ok(bytes.to_vec())
    }
}
