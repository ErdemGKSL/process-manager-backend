use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use serde_json::{json, Value};
use crate::library::model::{User, Process, ToJson};
use crate::State;

pub async fn trigger(Extension(state): Extension<State>, Extension(auth_user): Extension<User>, Path(id): Path<i32>) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;

    if !auth_user.admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let process = sqlx::query_as!(Process, "DELETE FROM \"Process\" WHERE id = $1 RETURNING *", id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "ok": true,
        "data": process.to_json()
    })))
}