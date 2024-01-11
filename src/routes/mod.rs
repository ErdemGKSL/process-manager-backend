mod _auth;
mod auth;
mod get;

use axum::Router;
use axum::routing::get;

pub fn get_app() -> Router {
    Router::new()
        .nest("/", _auth::get_router())
        .nest("/auth", auth::get_router())
        .route("/", get(get::trigger))
}