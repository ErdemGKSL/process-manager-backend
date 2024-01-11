use axum::Json;
use serde_json::{json, Value};

pub async fn trigger() -> Json<Value> {
    Json(json!({
        "ok": true,
        "message": "Handshake!",
    }))
}