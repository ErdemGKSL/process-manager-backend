pub mod put;
pub mod p_id;
pub mod get;

use axum::Router;
use axum::routing::{get, put};

pub fn get_router() -> Router {
    Router::new()
        .route("/", put(put::trigger))
        .route("/", get(get::trigger))
        .nest("/:process_id", p_id::get_router())
}
