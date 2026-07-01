//! Password hashing with bcrypt.

use crate::error::{AppError, AppResult};

/// Default bcrypt cost (2^12 rounds).
const DEFAULT_COST: u32 = 12;

/// Hash a plaintext password for storage.
pub fn hash_password(plain: &str) -> AppResult<String> {
    if plain.is_empty() {
        return Err(AppError::BadRequest("password must not be empty".into()));
    }
    bcrypt::hash(plain, DEFAULT_COST).map_err(|e| AppError::Internal(format!("bcrypt hash: {e}")))
}

/// Verify `plain` against a stored bcrypt hash.
pub fn verify_password(plain: &str, hash: &str) -> AppResult<bool> {
    bcrypt::verify(plain, hash).map_err(|e| AppError::Internal(format!("bcrypt verify: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hash_and_verify_roundtrip() {
        let hash = hash_password("p@ssw0rd").unwrap();
        assert!(verify_password("p@ssw0rd", &hash).unwrap());
        assert!(!verify_password("wrong", &hash).unwrap());
    }

    #[test]
    fn rejects_empty_password() {
        assert!(hash_password("").is_err());
    }
}
