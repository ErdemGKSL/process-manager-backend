mod put;
mod p_id;
mod get;

use axum::Router;
use axum::routing::{get, put};

pub fn get_router() -> Router {
    Router::new()
        .route("/", put(put::trigger))
        .route("/", get(get::trigger))
        .nest("/:process_id", p_id::get_router())
}
