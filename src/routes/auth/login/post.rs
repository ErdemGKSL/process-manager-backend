use axum::{Extension, Json};
use axum::http::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sha3::{Digest, Sha3_256};
use tower_cookies::Cookies;
use crate::library::model::User;
use crate::State;

pub async fn trigger(
    Extension(state): Extension<State>,
    _cookies: Cookies,
    Json(body): Json<RequestBody>,
) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;
    let user = sqlx::query_as!(User, "SELECT * FROM \"User\" WHERE username = $1", body.username)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::UNAUTHORIZED)?;

    let password_hash = Sha3_256::digest(body.password.as_bytes());
    let password_hash = format!("{:x}", password_hash);

    if password_hash != user.password_hash {
        Err(StatusCode::UNAUTHORIZED)
    } else {
        // cookies.add(
        //     Cookie::build(("token", user.token.clone()))
        //         .max_age(Duration::days(365))
        //         .domain("erdemg.dev")
        //         .path("/")
        //         .build()
        // );

        Ok(Json(json!({
            "ok": true,
            "token": &user.token,
            "message": "Successfully logged in as ".to_owned() + &user.username
        })))
    }
}
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestBody {
    username: String,
    password: String,
}