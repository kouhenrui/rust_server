//! HTTP route registration. Handlers live in [`crate::controller`].

use crate::controller;
use crate::middleware::{authorize_middleware, logging_middleware};
use crate::state::AppState;
use axum::middleware::from_fn;
use axum::middleware::from_fn_with_state;
use axum::routing::{get, post};
use axum::Router;
use std::sync::Arc;

/// Build the application router. Exposed for tests and the binary.
pub fn router(state: AppState) -> Router {
    let state = Arc::new(state);

    Router::new()
        .nest(state.config.api_prefix.as_str(), api_v1_routes())
        .layer(from_fn_with_state(state.clone(), authorize_middleware))
        .layer(from_fn(logging_middleware))
        .with_state(state)
}

/// API 路由组：按业务模块 merge，共享同一 state。
fn api_v1_routes() -> Router<Arc<AppState>> {
    Router::new()
        .merge(health_routes())
        .merge(auth_routes())
        .merge(img_routes())
}

fn health_routes() -> Router<Arc<AppState>> {
    Router::new().route("/health", get(controller::health::health))
}

fn auth_routes() -> Router<Arc<AppState>> {
    Router::new()
        .route("/login", post(controller::auth::login))
        .route("/me", get(controller::auth::me))
}

fn img_routes() -> Router<Arc<AppState>> {
    Router::new().route(
        "/img",
        get(controller::img::img_get).post(controller::img::img_post),
    )
}
