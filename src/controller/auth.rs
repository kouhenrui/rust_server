//! Authentication controllers: login and protected profile.

use crate::auth::{authenticate, hash_password};
use crate::entity::{AccountRepository, SqlBackend};
use crate::error::{AppError, AppResult};
use crate::middleware::AuthClaims;
use crate::state::AppState;
use axum::extract::State;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginData {
    pub token: String,
    pub expires_at: u64,
}

#[derive(Debug, Serialize)]
pub struct MeData {
    pub username: String,
}

/// `POST /login` — JSON `{ username, password }` → JWT token.
pub async fn login(State(state): State<Arc<AppState>>, Json(body): Json<LoginRequest>) -> Response {
    do_login(&state, &body.username, &body.password)
        .await
        .map(|data| crate::ok!(data))
        .unwrap_or_else(|err| err.into_response())
}

/// `GET /me` — requires valid Bearer JWT (see `auth_middleware`).
pub async fn me(AuthClaims(claims): AuthClaims) -> Response {
    crate::ok!(MeData {
        username: claims.sub,
    })
}

async fn do_login(state: &AppState, username: &str, password: &str) -> AppResult<LoginData> {
    if username.trim().is_empty() || password.is_empty() {
        return Err(AppError::BadRequest(
            "username and password are required".into(),
        ));
    }
    let pool = state
        .db
        .sql_pool()
        .ok_or_else(|| AppError::Internal("login requires a SQL database".into()))?;

    let subject = authenticate(pool, username, password).await?;
    let token = state.jwt.sign(&subject)?;
    let expires_at = login_expires_at(&state.jwt, &token)?;
    Ok(LoginData { token, expires_at })
}

fn login_expires_at(jwt: &crate::auth::JwtAuth, token: &str) -> AppResult<u64> {
    let claims = jwt.verify(token)?;
    Ok(claims.exp as u64)
}

/// Bootstrap a user when env vars are set (optional, idempotent).
pub async fn bootstrap_admin(state: &AppState) -> AppResult<()> {
    let username = match std::env::var("THUMBOR_BOOTSTRAP_USERNAME") {
        Ok(v) if !v.is_empty() => v,
        _ => return Ok(()),
    };
    let password = std::env::var("THUMBOR_BOOTSTRAP_PASSWORD").map_err(|_| {
        AppError::Internal("THUMBOR_BOOTSTRAP_PASSWORD required with USERNAME".into())
    })?;
    if password.is_empty() {
        return Err(AppError::Internal(
            "THUMBOR_BOOTSTRAP_PASSWORD must not be empty".into(),
        ));
    }
    let (backend, pool) = SqlBackend::require_from_db(&state.db)?;
    let password_hash = hash_password(&password)?;
    AccountRepository::upsert(pool, backend, &username, &password_hash).await?;
    state.casbin.add_role_for_user(&username, "admin").await?;
    crate::info!(username = %username, "bootstrap user ensured");
    Ok(())
}
