use axum::{
    extract::{Extension, Form},
    http::StatusCode,
    response::{IntoResponse, Html},
    Json,
};
use tower_cookies::{Cookie, Cookies};
use serde::{Deserialize, Serialize};
use time::Duration;

use crate::database::DbPool;
use crate::middleware::UserContext;
use crate::token;

#[derive(Debug, Deserialize)]
pub struct ThemeForm {
    pub theme: String,
}

#[derive(Debug, Serialize)]
pub struct ThemeResponse {
    pub theme: String,
    pub success: bool,
}

/// Get current user's theme preference
pub async fn get_theme(
    Extension(user_context): Extension<UserContext>,
    Extension(db_pool): Extension<DbPool>,
) -> impl IntoResponse {
    let theme = match get_user_theme(&user_context, &db_pool).await {
        Ok(theme) => theme,
        Err(_) => "dark".to_string(),
    };

    Json(ThemeResponse {
        theme,
        success: true,
    })
}

/// Set user's theme preference
pub async fn set_theme(
    cookies: Cookies,
    Extension(user_context): Extension<UserContext>,
    Extension(db_pool): Extension<DbPool>,
    Form(form): Form<ThemeForm>,
) -> impl IntoResponse {
    if !matches!(form.theme.as_str(), "light" | "dark") {
        return Json(ThemeResponse {
            theme: "dark".to_string(),
            success: false,
        }).into_response();
    }

    let claims = match ensure_user_token(&user_context, &cookies).await {
        Ok(claims) => claims,
        Err(_) => {
            return Json(ThemeResponse {
                theme: "dark".to_string(),
                success: false,
            }).into_response();
        }
    };

    match save_user_theme(&claims, &form.theme, &db_pool).await {
        Ok(_) => {
            Json(ThemeResponse {
                theme: form.theme,
                success: true,
            }).into_response()
        }
        Err(_) => {
            Json(ThemeResponse {
                theme: "dark".to_string(),
                success: false,
            }).into_response()
        }
    }
}

/// Get user's theme from database or return default
async fn get_user_theme(
    user_context: &UserContext,
    db_pool: &DbPool,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(claims) = user_context.get_claims() {
        let preference_key = token::get_preference_key(claims);
        
        let conn = db_pool.get().await?;
        let rows = conn
            .query("SELECT theme FROM user_preferences WHERE preference_key = $1", &[&preference_key])
            .await?;

        if let Some(row) = rows.first() {
            let theme: String = row.get("theme");
            return Ok(theme);
        }
    }

    Ok("dark".to_string())
}

/// Save user's theme preference to database
async fn save_user_theme(
    claims: &token::Claims,
    theme: &str,
    db_pool: &DbPool,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let preference_key = token::get_preference_key(claims);
    
    let conn = db_pool.get().await?;
    
    conn.execute(
        "INSERT INTO user_preferences (preference_key, theme) 
         VALUES ($1, $2) 
         ON CONFLICT (preference_key) 
         DO UPDATE SET theme = $2, updated_at = NOW()",
        &[&preference_key, &theme],
    ).await?;

    Ok(())
}

/// Ensure user has a valid token, creating one if needed
async fn ensure_user_token(
    user_context: &UserContext,
    cookies: &Cookies,
) -> Result<token::Claims, Box<dyn std::error::Error + Send + Sync>> {
    match user_context {
        UserContext::Authenticated(claims) => {
            if token::should_refresh_token(claims) {
                let new_token = if let Some(user_id) = claims.user_id {
                    token::generate_user_token(user_id)?
                } else {
                    token::generate_anonymous_token()?
                };
                
                set_auth_cookie(cookies, &new_token);
                Ok(token::verify_token(&new_token)?)
            } else {
                Ok(claims.clone())
            }
        }
        _ => {
            let new_token = token::generate_anonymous_token()?;
            set_auth_cookie(cookies, &new_token);
            Ok(token::verify_token(&new_token)?)
        }
    }
}

/// Set secure auth cookie
fn set_auth_cookie(cookies: &Cookies, token: &str) {
    let token_owned = token.to_string();
    let cookie = Cookie::build(("auth_token", token_owned))
        .http_only(true)
        .secure(true)
        .same_site(tower_cookies::cookie::SameSite::Strict)
        .max_age(Duration::days(365))
        .path("/")
        .build();
    
    cookies.add(cookie);
} 