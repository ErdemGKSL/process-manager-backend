use serde_json::{json, Value};
use sqlx::types::chrono::NaiveDateTime;
use serde::Serialize;

#[derive(Clone)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password_hash: String,
    pub token: String,
    pub admin: bool,
}

#[derive(Clone, Serialize)]
pub struct Process {
    pub id: i32,
    pub name: String,
    pub process_id: Option<i32>,
    pub dir: String,
    pub cmd: String,
    pub until: Option<NaiveDateTime>,
}

pub struct ProcessOwner {
    pub process_id: i32,
    pub user_id: i32,
}

pub trait ToJson {
    fn to_json(&self) -> Value;
}

impl ToJson for Process {
    fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "name": self.name,
            "process_id": self.process_id,
            "dir": self.dir,
            "cmd": self.cmd,
            "until": self.until.map(|u| u.and_utc().timestamp_millis()),
        })
    }
}

impl ToJson for User {
    fn to_json(&self) -> Value {
        json!({
            "id": self.id,
            "username": self.username,
            "admin": self.admin,
        })
    }
}

impl ToJson for ProcessOwner {
    fn to_json(&self) -> Value {
        json!({
            "process_id": self.process_id,
            "user_id": self.user_id,
        })
    }
}
