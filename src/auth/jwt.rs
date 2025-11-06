use jsonwebtoken::{encode, decode, Header, Validation, Algorithm, EncodingKey, DecodingKey};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum AuthError {
    InvalidToken,
    ExpiredToken,
    MissingToken,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i32, // user id
    pub exp: usize,
}

pub struct JWTConfig {
    pub secret: String,
}

impl JWTConfig {
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_string(),
        }
    }
}

pub fn create_token(user_id: i32, config: &JWTConfig) -> String {
    let expiration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize + 24 * 60 * 60; // 24 hours

    let claims = Claims {
        sub: user_id,
        exp: expiration,
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.secret.as_ref()),
    )
    .unwrap()
}

pub fn validate_token(token: &str, config: &JWTConfig) -> Result<Claims, AuthError> {
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.secret.as_ref()),
        &validation,
    ).map_err(|e| match e.kind() {
        jsonwebtoken::errors::ErrorKind::ExpiredSignature => AuthError::ExpiredToken,
        _ => AuthError::InvalidToken,
    })?;

    Ok(token_data.claims)
}