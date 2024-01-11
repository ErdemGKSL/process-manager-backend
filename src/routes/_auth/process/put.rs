use axum::{Extension, Json};
use axum::http::StatusCode;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::library::model::User;
use crate::State;

pub async fn trigger(Extension(state): Extension<State>, Extension(auth_user): Extension<User>, Json(body): Json<RequestBody>) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;
    if !auth_user.admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let result = sqlx::query!("INSERT INTO \"Process\" (name, dir, cmd) VALUES ($1, $2, $3) RETURNING id", body.name, body.dir, body.cmd)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(Json(json!({
        "ok": true,
        "data": {
            "id": result.id,
            "name": body.name,
            "process_id": null,
            "dir": body.dir,
            "cmd": body.cmd
        }
    })))
}

#[derive(Deserialize)]
pub struct RequestBody {
    name: String,
    dir: String,
    cmd: String,
}