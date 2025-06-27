use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    http::StatusCode,
};
use serde::Deserialize;
use std::collections::HashMap;
use std::borrow::Cow;
use once_cell::sync::Lazy;

static ICONS: Lazy<HashMap<&'static str, &'static str>> = Lazy::new(|| {
    let mut icons = HashMap::new();
    icons.insert("sun", r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-sun-icon lucide-sun"><circle cx="12" cy="12" r="4"/><path d="M12 2v2"/><path d="M12 20v2"/><path d="m4.93 4.93 1.41 1.41"/><path d="m17.66 17.66 1.41 1.41"/><path d="M2 12h2"/><path d="M20 12h2"/><path d="m6.34 17.66-1.41 1.41"/><path d="m19.07 4.93-1.41 1.41"/></svg>"#);
    icons.insert("moon", r#"<svg xmlns="http://www.w3.org/2000/svg" width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round" class="lucide lucide-moon"><path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"/></svg>"#);
    icons
});

#[derive(Debug, Deserialize)]
pub struct IconQuery {
    classes: Option<String>,
}
pub async fn get_icon(
    Path(icon_name): Path<String>,
    Query(params): Query<IconQuery>,
) -> impl IntoResponse {
    let svg_content = match get_svg_content(&icon_name) {
        Some(content) => content,
        None => return (StatusCode::NOT_FOUND, Html("Icon not found".to_string())).into_response(),
    };

    let styled_svg = add_classes_to_svg(&svg_content, &params.classes.unwrap_or_default());
    
    Html(styled_svg.into_owned()).into_response()
}

fn get_svg_content(icon_name: &str) -> Option<&'static str> {
    ICONS.get(icon_name).copied()
}

fn add_classes_to_svg<'a>(svg_content: &'a str, classes: &str) -> Cow<'a, str> {
    if classes.is_empty() {
        return Cow::Borrowed(svg_content);
    }
    
    // find opening svg tag and inject classes
    if let Some(class_start) = svg_content.find("class=\"") {
        // append classes to existing classes
        let class_end = svg_content[class_start + 7..].find('"').unwrap() + class_start + 7;
        let mut result = svg_content.to_string();
        result.insert_str(class_end, &format!(" {}", classes));
        Cow::Owned(result)
    } else if let Some(svg_end) = svg_content.find(">") {
        // add classes to svg if no existing classes
        let mut result = svg_content.to_string();
        result.insert_str(svg_end, &format!(" class=\"{}\"", classes));
        Cow::Owned(result)
    } else {
        // fallback if we cant parse svg
        Cow::Owned(svg_content.to_string())
    }
} 