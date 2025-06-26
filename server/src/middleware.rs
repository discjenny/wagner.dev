use axum::{
    extract::Request,
    middleware::Next,
    response::Response,
};
use tower_cookies::Cookies;
use std::time::SystemTime;

use crate::token::{self, Claims};

pub async fn logger(req: Request, next: Next) -> Response {
    let start = SystemTime::now();
    let method = req.method().clone();
    let uri = req.uri().clone();
    
    let response = next.run(req).await;
    
    let duration = start.elapsed().unwrap_or_default();
    let status = response.status();
    
    println!(
        "{} {} {} - {:?}",
        method, uri, status.as_u16(), duration
    );
    
    response
}

pub async fn jwt_cookie_middleware(
    cookies: Cookies,
    mut req: Request,
    next: Next,
) -> Response {
    let token = cookies
        .get("auth_token")
        .map(|cookie| cookie.value().to_string());

    if let Some(token_str) = token {
        match token::verify_token(&token_str) {
            Ok(claims) => {
                req.extensions_mut().insert(UserContext::Authenticated(claims));
            }
            Err(token::TokenError::ExpiredToken) => {
                req.extensions_mut().insert(UserContext::Anonymous);
            }
            Err(_) => {
                req.extensions_mut().insert(UserContext::InvalidToken);
            }
        }
    } else {
        req.extensions_mut().insert(UserContext::Anonymous);
    }

    next.run(req).await
}

#[derive(Debug, Clone)]
pub enum UserContext {
    Authenticated(Claims),
    Anonymous,
    InvalidToken,
}

impl UserContext {
    pub fn get_claims(&self) -> Option<&Claims> {
        match self {
            UserContext::Authenticated(claims) => Some(claims),
            _ => None,
        }
    }
    
    pub fn is_authenticated(&self) -> bool {
        matches!(self, UserContext::Authenticated(_))
    }
    
    pub fn needs_new_token(&self) -> bool {
        matches!(self, UserContext::Anonymous | UserContext::InvalidToken)
    }
}
