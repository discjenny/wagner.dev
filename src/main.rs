use axum::{
    middleware as axum_mw,
    routing::{get},
    Router
};

mod routes;
mod middleware;
use middleware as mw;

#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/", get(routes::pages::index))
        .route("/favicon.ico", get(routes::pages::favicon))
        .route("/favicon.svg", get(routes::pages::favicon))
        .fallback(routes::pages::not_found)
        .layer(axum_mw::from_fn(mw::logger));

    let listener = tokio::net::TcpListener::bind("localhost:8000").await.unwrap();
    println!("server started");
    axum::serve(listener, app).await.unwrap();
}