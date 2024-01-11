mod process;

use axum::{Extension, middleware, Router};
use axum::extract::Request;
use axum::middleware::Next;
use axum::response::Response;
use axum::http::{header, StatusCode};
use tower_cookies::Cookies;
use crate::library::model::User;
use crate::State;

pub fn get_router() -> Router {
    Router::new()
        .nest("/process", process::get_router())
        .layer(middleware::from_fn(auth))
}

async fn auth(Extension(state): Extension<State>, cookies: Cookies, mut req: Request, next: Next) -> Result<Response, StatusCode> {
    let db = &state.db;

    let auth_cookie = cookies.get("token").map(|cookie| cookie.value().to_owned());
    let auth_header = req.headers()
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

    req.extensions_mut().insert(user);

    Ok(next.run(req).await)
}