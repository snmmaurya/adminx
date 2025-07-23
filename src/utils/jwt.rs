use jsonwebtoken::{encode, EncodingKey, Header};
use serde::{Serialize, Deserialize};
use std::env;
use anyhow::{Result, Context};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,         // Subject (e.g., user ID)
    pub exp: usize,          // Expiration (as timestamp)
    pub email: String,       // Optional additional fields
    pub role: String,        // e.g., "admin"
}

pub fn create_jwt_token(user_id: &str, email: &str, role: &str) -> Result<String> {
    let secret = env::var("JWT_SECRET").context("JWT_SECRET not set in environment")?;

    let expiration = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::days(7))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_owned(),
        exp: expiration,
        email: email.to_owned(),
        role: role.to_owned(),
    };

    let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(secret.as_ref()))
        .context("Failed to encode JWT")?;

    Ok(token)
}
