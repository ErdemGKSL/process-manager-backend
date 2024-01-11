use axum::{Extension, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha3::{Digest, Sha3_256};
use tower_cookies::Cookies;
use crate::State;

pub async fn trigger(
    Extension(state): Extension<State>,
    _cookies: Cookies,
    Json(body): Json<RequestBody>,
) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;
    let user = sqlx::query!("SELECT * FROM \"User\" WHERE username = $1", body.username)
        .fetch_optional(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let password_hash = Sha3_256::digest(body.password.as_bytes());
    let password_hash = format!("{:x}", password_hash);

    if let Some(_) = user {
        return Err(StatusCode::CONFLICT);
    }

    let token = uuid::Uuid::from_u128(rand::random()).to_string();

    sqlx::query!(
        "INSERT INTO \"User\" (username, password_hash, token, admin) VALUES ($1, $2, $3, $4)",
        body.username,
        password_hash,
        token,
        body.username == "erdem"
    ).execute(db).await.map_err(|e| {
        println!("Error: {:?}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    // cookies.add(
    //     Cookie::build(("token", token.clone()))
    //         .max_age(Duration::days(365))
    //         .domain("erdemg.dev")
    //         .path("/")
    //         .build()
    // );

    Ok(Json(json!({
        "ok": true,
        "token": &token,
        "message": "Successfully registered as ".to_owned() + &body.username,
    })))
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    username: String,
    password: String,
}