use axum::http::StatusCode;
use axum::{Extension, Json};
use axum::extract::Path;
use serde::Deserialize;
use serde_json::{json, Value};
use crate::library::model::User;
use crate::State;

pub async fn trigger(
    Extension(state): Extension<State>,
    Extension(auth_user): Extension<User>,
    Path(process_id): Path<i32>,
    Json(body): Json<RequestBody>
) -> Result<Json<Value>, StatusCode> {
    if !auth_user.admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let db = &state.db;
    
    let user_id = body.user_id;
    
    let _ = sqlx::query!("SELECT * FROM \"User\" WHERE id = $1", user_id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let _ = sqlx::query!("SELECT * FROM \"Process\" WHERE id = $1", process_id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;
    
    let _ = sqlx::query!("INSERT INTO \"ProcessOwner\" (process_id, user_id) VALUES ($1, $2)", process_id, user_id)
        .execute(db)
        .await
        .map_err(|_| StatusCode::CONFLICT)?;
    
    Ok(Json(json!({
        "ok": true,
        "data": {
            "process_id": process_id,
            "user_id": user_id
        }
    })))
}

#[derive(Deserialize)]
pub struct RequestBody {
    pub user_id: i32
}