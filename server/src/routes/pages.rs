use axum::{
    http::StatusCode,
    response::IntoResponse,
};
use askama::Template;
use askama_web::WebTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "index.html")]
pub struct IndexTemplate;

#[derive(Template, WebTemplate)]
#[template(path = "error.html")]
pub struct ErrorTemplate;

pub async fn index() -> IndexTemplate {
    IndexTemplate
}

pub async fn not_found() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, ErrorTemplate)
} 