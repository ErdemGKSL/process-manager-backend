use axum::Router;
use axum::routing::{delete, get, patch, post};

mod patch;
mod get;
mod delete;
mod logs;
mod post;

pub fn get_router() -> Router {
    Router::new()
        .route("/", patch(patch::trigger))
        .route("/", get(get::trigger))
        .route("/", delete(delete::trigger))
        .route("/", post(post::trigger))
        .nest("/logs", logs::get_router())
}