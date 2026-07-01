//! JWT issue and verification (HS256).

use crate::config::Config;
use crate::error::{AppError, AppResult};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT payload stored in the token.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Claims {
    /// Subject (typically user id).
    pub sub: String,
    /// Expiration (Unix seconds).
    pub exp: usize,
    /// Issued-at (Unix seconds).
    pub iat: usize,
}

/// JWT helper bound to runtime config.
#[derive(Debug, Clone)]
pub struct JwtAuth {
    secret: String,
    expire_secs: u64,
}

impl JwtAuth {
    pub fn new(config: &Config) -> Self {
        Self {
            secret: config.jwt_secret.clone(),
            expire_secs: config.jwt_expire_secs,
        }
    }

    /// Issue a signed token for `subject`.
    pub fn sign(&self, subject: &str) -> AppResult<String> {
        if subject.is_empty() {
            return Err(AppError::BadRequest("jwt subject must not be empty".into()));
        }
        let now = unix_now_secs();
        let claims = Claims {
            sub: subject.to_string(),
            iat: now,
            exp: now + self.expire_secs as usize,
        };
        encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(self.secret.as_bytes()),
        )
        .map_err(|e| AppError::Internal(format!("jwt sign: {e}")))
    }

    /// Verify a token and return decoded claims.
    pub fn verify(&self, token: &str) -> AppResult<Claims> {
        let data = decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.secret.as_bytes()),
            &Validation::default(),
        )
        .map_err(|e| AppError::InvalidToken(e.to_string()))?;
        Ok(data.claims)
    }
}

/// Parse `Authorization: Bearer <token>`; returns the token slice.
pub fn bearer_token(auth_header: &str) -> Option<&str> {
    auth_header
        .strip_prefix("Bearer ")
        .or_else(|| auth_header.strip_prefix("bearer "))
        .map(str::trim)
        .filter(|s| !s.is_empty())
}

fn unix_now_secs() -> usize {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::Config;

    #[test]
    fn sign_and_verify_roundtrip() {
        let jwt = JwtAuth::new(&Config::default());
        let token = jwt.sign("user-42").unwrap();
        let claims = jwt.verify(&token).unwrap();
        assert_eq!(claims.sub, "user-42");
    }

    #[test]
    fn rejects_invalid_token() {
        let jwt = JwtAuth::new(&Config::default());
        assert!(jwt.verify("not.a.jwt").is_err());
    }

    #[test]
    fn parses_bearer_header() {
        assert_eq!(bearer_token("Bearer abc.def"), Some("abc.def"));
        assert_eq!(bearer_token("Basic xyz"), None);
    }
}
