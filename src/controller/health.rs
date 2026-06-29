//! Health check controller.

use crate::response::{api_success, HealthData};
use axum::response::Response;

/// 主动健康检查端点，返回统一信封 `{ code, message, data }`。
pub async fn health() -> Response {
    api_success(HealthData { status: "ok" })
}
