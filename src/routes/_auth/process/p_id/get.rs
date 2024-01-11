use axum::{Extension, Json};
use serde_json::{json, Value};
use axum::extract::Path;
use axum::http::StatusCode;
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

    Ok(Json(json!({
        "ok": true,
        "data": {
            "id": process.id,
            "name": process.name,
            "process_id": process.process_id,
            "dir": process.dir,
            "cmd": process.cmd,
            "owners": process_owners.iter().map(|owner| owner.user_id).collect::<Vec<_>>(),
        }
    })))
}