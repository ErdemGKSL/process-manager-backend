use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use serde_json::{json, Value};
use crate::library::model::{Process, ProcessOwner, User, ToJson};
use crate::State;

pub async fn trigger(Extension(state): Extension<State>, Extension(auth_user): Extension<User>, Path(id): Path<i32>) -> Result<Json<Value>, StatusCode> {
    if !auth_user.admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let db = &state.db;

    let _ = sqlx::query_as!(Process, "SELECT * FROM \"Process\" WHERE id = $1", id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let process_owners: Vec<_> = sqlx::query_as!(ProcessOwner, "SELECT * FROM \"ProcessOwner\" WHERE process_id = $1", id)
        .fetch_all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "ok": true,
        "data": process_owners.iter().map(|owner| owner.to_json()).collect::<Vec<_>>()
    })))
}