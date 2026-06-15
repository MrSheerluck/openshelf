use axum::{routing::get, Json, Router};
use rusqlite::Connection;
use serde::Serialize;
use std::sync::Arc;
use tokio::sync::Mutex;
use tower_http::cors::CorsLayer;

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    db: String,
}

struct AppState {
    db: Mutex<Connection>,
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
    let db_path = std::env::var("DATABASE_PATH").unwrap_or_else(|_| "./app.db".to_string());
    let conn = Connection::open(&db_path).expect("failed to open database");

    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS health (id INTEGER PRIMARY KEY, checked_at TEXT DEFAULT (datetime('now')))",
    )
    .expect("failed to create table");

    conn.execute("INSERT INTO health DEFAULT VALUES", [])
        .expect("failed to insert health check");

    let state = Arc::new(AppState {
        db: Mutex::new(conn),
    });

    let cors = CorsLayer::permissive();

    let app = Router::new()
        .route("/health", get(health_check))
        .layer(cors)
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3001").await.unwrap();
    println!("Backend listening on 0.0.0.0:3001");
    axum::serve(listener, app).await.unwrap();
}
