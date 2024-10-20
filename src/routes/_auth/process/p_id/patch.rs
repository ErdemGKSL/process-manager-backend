use crate::library::model::{Process, User};
use crate::State;
use axum::extract::Path;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde_json::{json, Value};
use serde::Deserialize;
use sqlx::types::chrono::NaiveDateTime;

pub async fn trigger(
    Extension(state): Extension<State>,
    Extension(auth_user): Extension<User>,
    Path(id): Path<i32>,
    Json(body): Json<RequestBody>
) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;
    let mut _process = sqlx::query_as!(Process, "SELECT * FROM \"Process\" WHERE id = $1", id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !auth_user.admin {
        return Ok(Json(json!({
            "ok": false,
            "message": "You are not allowed to interact with this process."
        })));
    }

    if let Some(name) = body.name {
        let _ = sqlx::query!("UPDATE \"Process\" SET name = $1 WHERE id = $2", name, id)
            .execute(db)
            .await;
        _process.name = name;
    }

    if let Some(dir) = body.dir {
        let _ = sqlx::query!("UPDATE \"Process\" SET dir = $1 WHERE id = $2", dir, id)
            .execute(db)
            .await;
        _process.dir = dir;
    }

    if let Some(cmd) = body.cmd {
        let _ = sqlx::query!("UPDATE \"Process\" SET cmd = $1 WHERE id = $2", cmd, id)
            .execute(db)
            .await;
        _process.cmd = cmd;
    }

    if let Some(until) = body.until {
        if let Until::Date(until) = until {
            let _ = sqlx::query!("UPDATE \"Process\" SET until = $1 WHERE id = $2", until, id)
                .execute(db)
                .await;
            _process.until = Some(until);
        } else {
            let _ = sqlx::query!("UPDATE \"Process\" SET until = NULL WHERE id = $1", id)
                .execute(db)
                .await;
            _process.until = None;
        }
    }

    Ok(Json(json!({
        "ok": true,
        "data": _process
    })))
}

#[derive(Deserialize)]
pub struct RequestBody {
    name: Option<String>,
    dir: Option<String>,
    cmd: Option<String>,
    until: Option<Until>,
}

#[derive(Deserialize)]
enum Until {
    Date(NaiveDateTime),
    Infinite
}
