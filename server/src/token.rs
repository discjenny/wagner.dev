use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use once_cell::sync::Lazy;
use std::env;
use uuid::Uuid;

static SECRET_KEY: Lazy<String> = Lazy::new(|| {
    env::var("JWT_SECRET").unwrap_or_else(|_| {
        eprintln!("WARNING: JWT_SECRET not set, using insecure default key for development");
        "dev_only_insecure_key_change_in_production".to_string()
    })
});

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub anonymous_id: Option<String>,
    pub user_id: Option<i32>,
    pub exp: usize,
    pub iat: usize,
}

#[derive(Debug)]
pub enum TokenError {
    InvalidToken,
    ExpiredToken,
    GenerationFailed,
}

impl std::fmt::Display for TokenError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TokenError::InvalidToken => write!(f, "Invalid token"),
            TokenError::ExpiredToken => write!(f, "Token expired"),
            TokenError::GenerationFailed => write!(f, "Token generation failed"),
        }
    }
}

impl std::error::Error for TokenError {}

impl From<jsonwebtoken::errors::Error> for TokenError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        match err.kind() {
            jsonwebtoken::errors::ErrorKind::ExpiredSignature => TokenError::ExpiredToken,
            _ => TokenError::InvalidToken,
        }
    }
}

pub fn generate_anonymous_token() -> Result<String, TokenError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + (365 * 24 * 60 * 60);
    
    let claims = Claims {
        anonymous_id: Some(Uuid::new_v4().to_string()),
        user_id: None,
        exp,
        iat: now,
    };
    
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY.as_bytes()),
    ).map_err(|_| TokenError::GenerationFailed)
}

pub fn generate_user_token(user_id: i32) -> Result<String, TokenError> {
    let now = chrono::Utc::now().timestamp() as usize;
    let exp = now + (30 * 24 * 60 * 60);
    
    let claims = Claims {
        anonymous_id: None,
        user_id: Some(user_id),
        exp,
        iat: now,
    };
    
    encode(
        &Header::new(Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(SECRET_KEY.as_bytes()),
    ).map_err(|_| TokenError::GenerationFailed)
}

/// Verify and decode any token (anonymous or authenticated)
pub fn verify_token(token: &str) -> Result<Claims, TokenError> {
    let validation = Validation::new(Algorithm::HS256);
    
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY.as_bytes()),
        &validation,
    )?;
    
    Ok(token_data.claims)
}

pub fn get_preference_key(claims: &Claims) -> String {
    if let Some(user_id) = claims.user_id {
        format!("user_{}", user_id)
    } else if let Some(anonymous_id) = &claims.anonymous_id {
        format!("anon_{}", anonymous_id)
    } else {
        format!("anon_{}", Uuid::new_v4())
    }
}

// pub fn is_authenticated(claims: &Claims) -> bool {
//     claims.user_id.is_some()
// }

// check if token is within 7 days of expiration
pub fn should_refresh_token(claims: &Claims) -> bool {
    let now = chrono::Utc::now().timestamp() as usize;
    let seven_days = 7 * 24 * 60 * 60;
    claims.exp.saturating_sub(now) < seven_days
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_anonymous_token_flow() {
        let token = generate_anonymous_token().unwrap();
        let claims = verify_token(&token).unwrap();
        
        assert!(claims.anonymous_id.is_some());
        assert!(claims.user_id.is_none());
        // assert!(!is_authenticated(&claims));
        
        let key = get_preference_key(&claims);
        assert!(key.starts_with("anon_"));
    }
    
    #[test]
    fn test_user_token_flow() {
        let user_id = 123;
        let token = generate_user_token(user_id).unwrap();
        let claims = verify_token(&token).unwrap();
        
        assert!(claims.anonymous_id.is_none());
        assert_eq!(claims.user_id, Some(user_id));
        // assert!(is_authenticated(&claims));
        
        let key = get_preference_key(&claims);
        assert_eq!(key, "user_123");
    }
} 