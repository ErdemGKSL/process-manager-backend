use axum::http::StatusCode;
use axum::{Extension, Json};
use serde_json::{json, Value};
use crate::library::model::{Process, User, ToJson};
use crate::State;

pub async fn trigger(Extension(state): Extension<State>, Extension(auth_user): Extension<User>) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;

    let processes = {
        if auth_user.admin {
            sqlx::query_as!(Process, "SELECT * FROM \"Process\"")
                .fetch_all(db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        } else {
            sqlx::query_as!(Process, "SELECT * FROM \"Process\" WHERE id IN (SELECT process_id FROM \"ProcessOwner\" WHERE user_id = $1)", auth_user.id)
                .fetch_all(db)
                .await
                .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        }
    };

    Ok(Json(json!({
        "ok": true,
        "data": processes.iter().map(|process| process.to_json()).collect::<Vec<_>>()
    })))
}