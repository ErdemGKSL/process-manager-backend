use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use serde_json::{json, Value};
use crate::library::model::{User, ProcessOwner, ToJson};
use crate::State;

pub async fn trigger(Extension(state): Extension<State>, Extension(auth_user): Extension<User>, Path((process_id, user_id)): Path<(i32, i32)>) -> Result<Json<Value>, StatusCode> {
    if !auth_user.admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let db = &state.db;

    let deleted = sqlx::query_as!(ProcessOwner, "DELETE FROM \"ProcessOwner\" WHERE process_id = $1 AND user_id = $2 RETURNING process_id, user_id", process_id, user_id)
        .fetch_one(db)
        .await
        .map(|owner| owner.to_json())
        .map_err(|_| StatusCode::NOT_FOUND)?;

    Ok(Json(json!({
        "ok": true,
        "data": deleted
    })))
}