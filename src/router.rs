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
        .route("/health", get(controller::health::health))
        .route("/login", post(controller::auth::login))
        .route(
            "/img",
            get(controller::img::img_get).post(controller::img::img_post),
        )
        .route("/me", get(controller::auth::me))
        .layer(from_fn_with_state(state.clone(), authorize_middleware))
        .layer(from_fn(logging_middleware))
        .with_state(state)
}
