use axum::{
    extract::Extension,
    middleware as axum_mw,
    routing::{get, post},
    Router
};
use tower_http::services::ServeDir;
use tower_cookies::CookieManagerLayer;

mod database;
mod routes;
mod middleware;
mod token;
use middleware as mw;

#[tokio::main]
async fn main() {
    let db_pool = database::init_db().await.expect("database connection failed");
    database::run_migrations(&db_pool).await.expect("database migrations failed");

    let app = Router::new()
        .route("/", get(routes::pages::index))
        .route("/api/theme", get(routes::themes::get_theme))
        .route("/api/theme", post(routes::themes::set_theme))
        .nest_service("/static", ServeDir::new("static"))
        .fallback(routes::pages::not_found)
        .layer(Extension(db_pool))
        .layer(axum_mw::from_fn(mw::jwt_cookie_middleware))
        .layer(CookieManagerLayer::new())
        .layer(axum_mw::from_fn(mw::logger));

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8000").await.unwrap();
    println!("server started");
    axum::serve(listener, app).await.unwrap();
}