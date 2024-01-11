mod login;
mod register;
mod get;

use axum::Router;
use axum::routing::get;

pub fn get_router() -> Router {
    Router::new()
        .nest("/login", login::get_router())
        .nest("/register", register::get_router())
        .route("/", get(get::trigger))
}
