use axum::http::{header, HeaderMap, StatusCode};
use axum::{Extension, Json};
use serde_json::{json, Value};
use tower_cookies::Cookies;
use crate::library::model::{User, ToJson};
use crate::State;

pub async fn trigger(Extension(state): Extension<State>, cookies: Cookies, headers: HeaderMap) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;

    let auth_cookie = cookies.get("token").map(|cookie| cookie.value().to_owned());
    let auth_header = headers
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok()).map(|header| header.to_owned());

    // let mut user = sqlx::query_as!(User, "SELECT id, password_hash, token, username, admin FROM \"User\" WHERE token = $1", auth_cookie)
    //     .fetch_one(db)
    //     .await
    //     .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let mut user: Option<User> = None;

    if auth_cookie.is_some() {
        user = sqlx::query_as!(User, "SELECT * FROM \"User\" WHERE token = $1", auth_cookie)
            .fetch_optional(db)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    if let Some(auth_header) = auth_header {
        if user.is_none() {
            user = sqlx::query_as!(User, "SELECT * FROM \"User\" WHERE token = $1", auth_header)
                .fetch_optional(db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        }
    }

    let mut user = user.ok_or(StatusCode::UNAUTHORIZED)?;

    if user.id == 0 { user.admin = true; }

    Ok(Json(json!({
        "ok": true,
        "data": user.to_json()
    })))
}