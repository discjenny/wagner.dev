use axum::{
    http::{StatusCode},
    response::{Html, IntoResponse, Response},
};
use std::fs;

pub async fn index() -> Html<String> {
    let html_content = fs::read_to_string("pages/index.html")
        .unwrap_or_else(|_| "<h1>error loading index</h1>".to_string());
    Html(html_content)
}

pub async fn not_found() -> Response {
    let html_content = fs::read_to_string("pages/error.html")
        .unwrap_or_else(|_| "<h1>404 - Page Not Found</h1>".to_string());
    
    (StatusCode::NOT_FOUND, Html(html_content)).into_response()
} 