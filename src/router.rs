//! HTTP route registration. Handlers live in [`crate::controller`].

use crate::controller;
use crate::middleware::logging_middleware;
use crate::state::AppState;
use axum::middleware::from_fn;
use axum::routing::get;
use axum::Router;
use std::sync::Arc;

/// Build the application router. Exposed for tests and the binary.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(controller::health::health))
        .route(
            "/img",
            get(controller::img::img_get).post(controller::img::img_post),
        )
        .layer(from_fn(logging_middleware))
        .with_state(Arc::new(state))
}
