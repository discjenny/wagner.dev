use axum::{
    middleware as axum_mw,
    routing::{get},
    Router
};
use tower_http::services::ServeDir;

mod routes;
mod middleware;
use middleware as mw;

#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/", get(routes::pages::index))
        .nest_service("/static", ServeDir::new("static"))
        .fallback(routes::pages::not_found)
        .layer(axum_mw::from_fn(mw::logger));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("server started");
    axum::serve(listener, app).await.unwrap();
}