use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
    Json,
};
use axum_extra::extract::cookie::CookieJar;
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use serde_json::json;
use time::Duration;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub exp: usize,
    pub iat: usize,
}

pub async fn logout(jar: CookieJar) -> (CookieJar, Json<serde_json::Value>) {
    let cookie = axum_extra::extract::cookie::Cookie::build(("token", ""))
        .path("/")
        .http_only(true)
        .max_age(Duration::seconds(0))
        .build();

    (jar.add(cookie), Json(json!({ "success": true })))
}

pub async fn me() -> Json<serde_json::Value> {
    Json(json!({ "authenticated": true }))
}

pub async fn require_auth(
    jar: CookieJar,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    let path = request.uri().path();

    if path == "/api/auth/signin" || path == "/api/auth/signup" || path == "/api/auth/status" {
        return Ok(next.run(request).await);
    }

    let token = jar
        .get("token")
        .map(|c| c.value().to_string())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let secret = std::env::var("JWT_SECRET").unwrap_or_else(|_| "dev-secret-change-me".to_string());

    decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|_| StatusCode::UNAUTHORIZED)?;

    Ok(next.run(request).await)
}
