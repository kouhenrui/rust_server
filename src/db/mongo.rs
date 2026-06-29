//! MongoDB 客户端连接。

use super::db::DbProvider;
use crate::error::AppError;
use mongodb::{bson::doc, Client};

#[derive(Clone)]
pub struct MongoDb {
    client: Client,
}

impl MongoDb {
    pub async fn connect(url: &str) -> Result<Self, AppError> {
        crate::info!(url = %redact_url(url), "connecting to mongodb");
        let client = Client::with_uri_str(url)
            .await
            .map_err(|e| AppError::Internal(format!("mongodb connect: {e}")))?;
        Ok(Self { client })
    }

    /// 底层 MongoDB 客户端，供业务层直接操作 collection。
    pub fn client(&self) -> &Client {
        &self.client
    }
}

impl DbProvider for MongoDb {
    fn backend_name(&self) -> &'static str {
        "mongodb"
    }

    async fn ping(&self) -> Result<(), AppError> {
        self.client
            .database("admin")
            .run_command(doc! { "ping": 1 })
            .await
            .map(|_| ())
            .map_err(|e| AppError::Internal(format!("mongodb ping: {e}")))
    }
}

fn redact_url(url: &str) -> String {
    if let Some(at) = url.find('@') {
        if let Some(scheme_end) = url.find("://") {
            let scheme = &url[..scheme_end + 3];
            let rest = &url[at + 1..];
            return format!("{scheme}***@{rest}");
        }
    }
    url.to_string()
}
