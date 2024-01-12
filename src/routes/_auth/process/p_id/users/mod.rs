mod put;
mod get;
mod u_id;

use axum::Router;
use axum::routing::{get, put};

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(get::trigger))
        .route("/", put(put::trigger))
        .nest("/:user_id", u_id::get_router())
}