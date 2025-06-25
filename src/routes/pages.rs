use axum::{
    http::{StatusCode, header},
    response::{Html, IntoResponse, Response},
};
use std::fs;

pub async fn index() -> Html<String> {
    let html_content = fs::read_to_string("static/index.html")
        .unwrap_or_else(|_| "<h1>error loading index</h1>".to_string());
    Html(html_content)
}

pub async fn favicon() -> Response {
    match fs::read_to_string("static/icon.svg") {
        Ok(svg_content) => {
            (
                StatusCode::OK,
                [(header::CONTENT_TYPE, "image/svg+xml")],
                svg_content,
            ).into_response()
        }
        Err(_) => {
            (StatusCode::NOT_FOUND, "Favicon not found").into_response()
        }
    }
}

pub async fn not_found() -> Response {
    let html_content = fs::read_to_string("static/error.html")
        .unwrap_or_else(|_| "<h1>404 - Page Not Found</h1>".to_string());
    
    (StatusCode::NOT_FOUND, Html(html_content)).into_response()
} 