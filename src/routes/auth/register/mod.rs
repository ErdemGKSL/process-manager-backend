mod post;

use axum::Router;
use axum::routing::post;

pub fn get_router() -> Router {
    Router::new()
        .route("/", post(post::trigger))
}
