use crate::library::model::{Process, User};
use crate::State;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde_json::{json, Value};

pub async fn trigger(
    Extension(state): Extension<State>,
    Extension(auth_user): Extension<User>,
    Path(id): Path<i32>,
) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;
    let _process = sqlx::query_as!(Process, "SELECT * FROM \"Process\" WHERE id = $1", id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !auth_user.admin {
        return Ok(Json(json!({
            "ok": false,
            "message": "You are not allowed to interact with this process."
        })));
    }

    unimplemented!()
}

#[derive(Deserialize)]
pub struct RequestBody {
    action: RequestBodyType,
}
