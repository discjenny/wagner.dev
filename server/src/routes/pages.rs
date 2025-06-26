use axum::{
    extract::Extension,
    http::StatusCode,
    response::IntoResponse,
};
use askama::Template;
use askama_web::WebTemplate;

use crate::database::DbPool;
use crate::middleware::UserContext;
use crate::token;

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate {
    pub theme: String,
}

#[derive(Template, WebTemplate)]
#[template(path = "error.html")]
pub struct ErrorTemplate {
    pub theme: String,
}

pub async fn index(
    Extension(user_context): Extension<UserContext>,
    Extension(db_pool): Extension<DbPool>,
) -> IndexTemplate {
    let theme = get_user_theme(&user_context, &db_pool).await
        .unwrap_or_else(|_| "dark".to_string());
    
    IndexTemplate { theme }
}

pub async fn not_found(
    Extension(user_context): Extension<UserContext>,
    Extension(db_pool): Extension<DbPool>,
) -> impl IntoResponse {
    let theme = get_user_theme(&user_context, &db_pool).await
        .unwrap_or_else(|_| "dark".to_string());
    
    (StatusCode::NOT_FOUND, ErrorTemplate { theme })
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

    // Default theme for new/anonymous users
    Ok("dark".to_string())
} 