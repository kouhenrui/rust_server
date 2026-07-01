//! Health check controller.

use crate::state::AppState;
use axum::extract::State;
use axum::response::Response;
use std::sync::Arc;

/// 主动健康检查：进程存活 + 缓存/数据库 ping 状态。
pub async fn health(State(state): State<Arc<AppState>>) -> Response {
    crate::ok!(state.check_health().await)
}
