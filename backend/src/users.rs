use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2, PasswordHash, PasswordVerifier,
};
use axum::{extract::State, http::StatusCode, Json};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use jsonwebtoken::{encode, EncodingKey, Header};
use rusqlite::Connection;
use serde::Deserialize;
use serde_json::json;
use std::sync::Arc;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use crate::auth::Claims;
use crate::AppState;

#[derive(Deserialize)]
pub struct AuthBody {
    pub password: String,
}

fn hash_password(password: &str) -> Result<String, StatusCode> {
    let salt = SaltString::generate(&mut OsRng);
    Argon2::default()
        .hash_password(password.as_bytes(), &salt)
        .map(|h| h.to_string())
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
}

fn verify_password(password: &str, hash: &str) -> Result<(), StatusCode> {
    let parsed = PasswordHash::new(hash).map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Argon2::default()
        .verify_password(password.as_bytes(), &parsed)
        .map_err(|_| StatusCode::UNAUTHORIZED)
}

pub fn user_count(conn: &Connection) -> i64 {
    conn.query_row("SELECT COUNT(*) FROM users", [], |row| row.get(0))
        .unwrap_or(0)
}

fn create_auth_cookie(secret: &str) -> Result<Cookie<'static>, StatusCode> {
    let now = OffsetDateTime::now_utc();
    let exp = now + Duration::days(30);

    let claims = Claims {
        sub: "admin".into(),
        exp: exp.unix_timestamp() as usize,
        iat: now.unix_timestamp() as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Cookie::build(("token", token))
        .path("/")
        .http_only(true)
        .secure(false)
        .same_site(axum_extra::extract::cookie::SameSite::Lax)
        .max_age(Duration::days(30))
        .build())
}

pub async fn status(State(state): State<Arc<AppState>>) -> Json<serde_json::Value> {
    let count = state.db.lock().await;
    let count = user_count(&count);
    Json(json!({ "has_users": count > 0 }))
}

pub async fn signup(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<AuthBody>,
) -> Result<(CookieJar, Json<serde_json::Value>), StatusCode> {
    let db = state.db.lock().await;
    if user_count(&db) > 0 {
        return Err(StatusCode::CONFLICT);
    }

    let hash = hash_password(&body.password)?;

    db.execute(
        "INSERT INTO users (id, username, password_hash) VALUES (?1, 'admin', ?2)",
        rusqlite::params![Uuid::new_v4().to_string(), hash],
    )
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    drop(db);

    let cookie = create_auth_cookie(&state.jwt_secret)?;
    Ok((jar.add(cookie), Json(json!({ "success": true }))))
}

pub async fn signin(
    State(state): State<Arc<AppState>>,
    jar: CookieJar,
    Json(body): Json<AuthBody>,
) -> Result<(CookieJar, Json<serde_json::Value>), StatusCode> {
    let db = state.db.lock().await;
    let hash: String = db
        .query_row("SELECT password_hash FROM users LIMIT 1", [], |row| {
            row.get(0)
        })
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    verify_password(&body.password, &hash)?;
    drop(db);

    let cookie = create_auth_cookie(&state.jwt_secret)?;
    Ok((jar.add(cookie), Json(json!({ "success": true }))))
}
