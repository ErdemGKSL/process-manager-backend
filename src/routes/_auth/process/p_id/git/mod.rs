mod get;
mod post;

use axum::Router;
use axum::routing::{get, post};

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(get::trigger))
        .route("/", post(post::trigger))
}