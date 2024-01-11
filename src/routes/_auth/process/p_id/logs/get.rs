use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use serde_json::{json, Value};
use crate::library::cache::LOGS;
use crate::library::model::{Process, ProcessOwner, User};
use crate::State;

pub async fn trigger(Extension(state): Extension<State>, Extension(auth_user): Extension<User>, Path(id): Path<i32>) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;
    let process = sqlx::query_as!(Process, "SELECT * FROM \"Process\" WHERE id = $1", id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let process_owners: Vec<_> = sqlx::query_as!(ProcessOwner, "SELECT * FROM \"ProcessOwner\" WHERE process_id = $1", id)
        .fetch_all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !auth_user.admin && !process_owners.iter().any(|owner| owner.user_id == auth_user.id) {
        return Err(StatusCode::FORBIDDEN);
    }

    let process_id= process.id;

    let logs = LOGS.lock().await;
    let logs = logs.get(&(process_id as _)).ok_or(StatusCode::NO_CONTENT)?;
    let logs = logs.iter().map(|log| log.to_string()).collect::<Vec<_>>().join("\n");

    Ok(Json(json!({
        "ok": true,
        "data": logs
    })))
}