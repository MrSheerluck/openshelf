mod auth;
mod books;
mod db;
mod storage;
mod users;

use axum::{http::Method, middleware, routing::get, Json, Router};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::{AllowOrigin, CorsLayer};

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    db: String,
}

pub struct AppState {
    pub db: Mutex<Connection>,
    pub jwt_secret: String,
    pub storage: Option<storage::Storage>,
}

async fn health_check(state: axum::extract::State<Arc<AppState>>) -> Json<HealthResponse> {
    let db_status = match state.db.lock().await.execute_batch("SELECT 1") {
        Ok(_) => "connected".to_string(),
        Err(e) => format!("error: {}", e),
    };

    Json(HealthResponse {
        status: "ok".to_string(),
        db: db_status,
    })
}

#[tokio::main]
async fn main() {
    let jwt_secret =
        std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());

    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./app.db".to_string());
    let conn = Connection::open(&db_path).expect("failed to open database");

    db::run_migrations(&conn);

    let s3_storage = storage::Storage::from_env();

    let state = Arc::new(AppState {
        db: Mutex::new(conn),
        jwt_secret,
        storage: s3_storage,
    });

    let public = Router::new()
        .route("/auth/status", get(users::status))
        .route("/auth/signup", axum::routing::post(users::signup))
        .route("/auth/signin", axum::routing::post(users::signin));

    let protected = Router::new()
        .route("/auth/me", get(auth::me))
        .route("/auth/logout", axum::routing::post(auth::logout))
        .route("/books", get(books::list_books).post(books::upload_book))
        .route("/books/{id}", get(books::get_book).delete(books::delete_book))
        .route("/books/{id}/file", get(books::serve_book_file))
        .route_layer(middleware::from_fn(auth::require_auth));

    let api_routes = Router::new().merge(public).merge(protected);

    let frontend_url =
        std::env::var("FRONTEND_URL").unwrap_or_else(|_| "http://localhost:3000".to_string());

    let cors = CorsLayer::new()
        .allow_origin(AllowOrigin::exact(frontend_url.parse().unwrap()))
        .allow_credentials(true)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers([axum::http::header::CONTENT_TYPE]);

    let app = Router::new()
        .route("/health", get(health_check))
        .nest("/api", api_routes)
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Backend listening on 0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}
