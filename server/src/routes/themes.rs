use axum::{
    extract::{Extension, Form},
    response::{IntoResponse},
    Json,
};
use tower_cookies::{Cookie, Cookies};
use serde::{Deserialize, Serialize};
use time::Duration;
use std::borrow::Cow;

use crate::database::DbPool;
use crate::middleware::UserContext;
use crate::token;
use crate::DEFAULT_THEME;

#[derive(Debug, Deserialize)]
pub struct ThemeForm {
    pub theme: String,
}

#[derive(Debug, Serialize)]
pub struct ThemeResponse {
    pub theme: String,
    pub success: bool,
}

pub async fn get_theme(
    Extension(user_context): Extension<UserContext>,
    Extension(db_pool): Extension<DbPool>,
) -> impl IntoResponse {
    let theme = get_user_theme(&user_context, &db_pool).await
        .unwrap_or(Cow::Borrowed(DEFAULT_THEME));

    Json(ThemeResponse {
        theme: theme.into_owned(),
        success: true,
    })
}

pub async fn set_theme(
    cookies: Cookies,
    Extension(user_context): Extension<UserContext>,
    Extension(db_pool): Extension<DbPool>,
    Form(form): Form<ThemeForm>,
) -> impl IntoResponse {
    if !matches!(form.theme.as_str(), "light" | "dark") {
        return Json(ThemeResponse {
            theme: DEFAULT_THEME.to_string(),
            success: false,
        }).into_response();
    }

    let claims = match ensure_user_token(&user_context, &cookies).await {
        Ok(claims) => claims,
        Err(_) => {
            return Json(ThemeResponse {
                theme: DEFAULT_THEME.to_string(),
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
                theme: DEFAULT_THEME.to_string(),
                success: false,
            }).into_response()
        }
    }
}

// get user theme from database or return default
async fn get_user_theme(
    user_context: &UserContext,
    db_pool: &DbPool,
) -> Result<Cow<'static, str>, Box<dyn std::error::Error + Send + Sync>> {
    if let Some(claims) = user_context.get_claims() {
        let preference_key = token::get_preference_key(claims);
        
        let conn = db_pool.get().await?;
        let rows = conn
            .query("SELECT theme FROM user_preferences WHERE preference_key = $1", &[&preference_key])
            .await?;

        if let Some(row) = rows.first() {
            let theme: String = row.get("theme");
            return Ok(Cow::Owned(theme));
        }
    }

    Ok(Cow::Borrowed(DEFAULT_THEME))
}

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

// ensure user has a valid token, creating one if needed
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

fn set_auth_cookie(cookies: &Cookies, token: &str) {
    let cookie = Cookie::build(("auth_token", token.to_string()))
        .http_only(true)
        .secure(true)
        .same_site(tower_cookies::cookie::SameSite::Strict)
        .max_age(Duration::days(365))
        .path("/")
        .build();
    
    cookies.add(cookie);
} 