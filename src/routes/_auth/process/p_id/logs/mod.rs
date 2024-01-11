mod get;

use axum::Router;
use axum::routing::get;

pub fn get_router() -> Router {
    Router::new()
        .route("/", get(get::trigger))
}