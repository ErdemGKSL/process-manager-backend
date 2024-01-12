mod delete;

use axum::Router;
use axum::routing::delete;

pub fn get_router() -> Router {
    Router::new()
        .route("/", delete(delete::trigger))
}