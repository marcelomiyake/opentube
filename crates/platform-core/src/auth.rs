use chrono::{Duration, Utc};
use jsonwebtoken::{Algorithm, DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub email: String,
    pub exp: usize,
}

#[derive(Debug, Error)]
pub enum TokenError {
    #[error("invalid token")]
    Invalid,
}

pub fn issue_token(
    user_id: Uuid,
    email: &str,
    secret: &[u8],
    lifetime_minutes: i64,
) -> Result<String, TokenError> {
    let claims = Claims {
        sub: user_id,
        email: email.to_owned(),
        exp: (Utc::now() + Duration::minutes(lifetime_minutes)).timestamp() as usize,
    };
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret),
    )
    .map_err(|_| TokenError::Invalid)
}

pub fn verify_token(token: &str, secret: &[u8]) -> Result<Claims, TokenError> {
    let validation = Validation::new(Algorithm::HS256);
    decode::<Claims>(token, &DecodingKey::from_secret(secret), &validation)
        .map(|data| data.claims)
        .map_err(|_| TokenError::Invalid)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issued_token_round_trips_for_same_secret() {
        let id = Uuid::new_v4();
        let token = issue_token(id, "creator@opentube.local", b"unit-test-secret", 30).unwrap();
        let claims = verify_token(&token, b"unit-test-secret").unwrap();
        assert_eq!(claims.sub, id);
        assert_eq!(claims.email, "creator@opentube.local");
    }

    #[test]
    fn token_rejects_a_different_secret() {
        let token = issue_token(Uuid::new_v4(), "creator@opentube.local", b"first", 30).unwrap();
        assert!(verify_token(&token, b"second").is_err());
    }
}
