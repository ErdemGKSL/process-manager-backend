use axum::{Extension, Json};
use axum::extract::Path as APath;
use axum::http::StatusCode;
use serde_json::{json, Value};
use std::path::Path;
use crate::library::model::{Process, User};
use crate::State;

pub async fn trigger(Extension(state): Extension<State>, Extension(auth_user): Extension<User>, APath(id): APath<i32>) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;

    if !auth_user.admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let process = sqlx::query_as!(Process, "SELECT * FROM \"Process\" WHERE id = $1", id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !Path::new(&process.dir).join(".git").exists() {
        Ok(Json(json!({
            "ok": true,
            "data": false
        })))
    } else {
        Ok(Json(json!({
            "ok": true,
            "data": true
        })))
    }
}