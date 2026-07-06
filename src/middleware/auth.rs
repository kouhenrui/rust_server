//! JWT authentication + Casbin authorization middleware.

use axum::body::Body;
use axum::extract::{FromRequestParts, Request, State};
use axum::http::header::AUTHORIZATION;
use axum::http::request::Parts;
use axum::middleware::Next;
use axum::response::Response;
use std::sync::Arc;

use crate::auth::{bearer_token, Claims};
use crate::error::AppError;
use crate::state::AppState;
use async_trait::async_trait;

/// Routes that require a valid JWT (Casbin alone is not enough).
const JWT_REQUIRED: &[&str] = &["/me"];

/// Verified JWT claims stored in request extensions.
#[derive(Clone, Debug)]
pub struct AuthClaims(pub Claims);

#[async_trait]
impl<S> FromRequestParts<S> for AuthClaims
where
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        parts
            .extensions
            .get::<Claims>()
            .cloned()
            .map(AuthClaims)
            .ok_or_else(|| AppError::Unauthorized("missing or invalid authorization".into()))
    }
}

/// Resolve JWT subject, enforce Casbin policy, attach [`Claims`] when present.
pub async fn authorize_middleware(
    State(state): State<Arc<AppState>>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, AppError> {
    let path = request.uri().path().to_string();
    let method = request.method().as_str().to_string();

    let subject = if let Some(header) = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|v| v.to_str().ok())
    {
        let token = bearer_token(header)
            .ok_or_else(|| AppError::Unauthorized("expected Bearer token".into()))?;
        let claims = state.jwt.verify(token)?;
        request.extensions_mut().insert(claims.clone());
        claims.sub
    } else {
        let api_path = path
            .strip_prefix(state.config.api_prefix.as_str())
            .unwrap_or(&path);
        if JWT_REQUIRED.iter().any(|p| *p == api_path) {
            return Err(AppError::Unauthorized(
                "missing authorization header".into(),
            ));
        }
        "anonymous".to_string()
    };

    let allowed = state.casbin.enforce(&subject, &path, &method).await?;
    if !allowed {
        return Err(AppError::Forbidden(format!(
            "access denied for {subject} on {method} {path}"
        )));
    }

    Ok(next.run(request).await)
}

/// Legacy name: protected routes use [`authorize_middleware`] on the merged router.
pub use authorize_middleware as auth_middleware;
