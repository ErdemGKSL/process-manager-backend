use crate::library::cache::LOGS;
use crate::library::model::{Process, User};
use crate::State;
use axum::extract::Path as APath;
use axum::http::StatusCode;
use axum::{Extension, Json};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::chrono;
use std::path::Path;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process;

pub async fn trigger(
    Extension(state): Extension<State>,
    Extension(auth_user): Extension<User>,
    APath(id): APath<i32>,
    Json(body): Json<RequestBody>,
) -> Result<Json<Value>, StatusCode> {
    let db = &state.db;

    if !auth_user.admin {
        return Err(StatusCode::FORBIDDEN);
    }

    let mut process = sqlx::query_as!(Process, "SELECT * FROM \"Process\" WHERE id = $1", id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    if !Path::new(&process.dir).join(".git").exists() {
        return Ok(Json(json!({
            "ok": true,
            "data": false
        })));
    }

    match body.action {
        RequestAction::Status => {
            let _ = execute_git_command(&mut process, "status".to_owned()).await?;

            Ok(Json(json!({
                "ok": true
            })))
        }
        RequestAction::Pull => {
            let _ = execute_git_command(&mut process, "pull".to_owned()).await?;

            Ok(Json(json!({
                "ok": true
            })))
        }
    }
}

pub async fn execute_git_command(
    process: &mut Process,
    command: String,
) -> Result<u32, StatusCode> {
    let name = std::env::var("GIT_PATH").unwrap_or("git".to_string());
    let args = command.split(' ').collect::<Vec<_>>();

    let mut command = process::Command::new(&name);

    let mut child = command
        .args(args.clone())
        .current_dir(&process.dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::null())
        .spawn()
        .map_err(|_| StatusCode::FAILED_DEPENDENCY)?;

    let stdout = child.stdout.take().unwrap();
    let stderr = child.stderr.take().unwrap();

    let stdout = BufReader::new(stdout);
    let stderr = BufReader::new(stderr);

    let process_id = child.id().ok_or(StatusCode::FAILED_DEPENDENCY)?;
    let database_id = process.id;

    let mut logs = LOGS.lock().await;
    let log = logs.get_mut(&(database_id as _));
    let timestamp_string = format!("{}", chrono::Utc::now().format("%d/%m/%Y %H:%M:%S"));
    if let Some(log) = log {
        log.push(format!("[{timestamp_string}] $: {name} {}", args.join(" ")));
    } else {
        logs.insert(
            database_id as _,
            vec![format!("[{timestamp_string}] $: {name} {}", args.join(" "))],
        );
    }

    {
        let process = process.clone();
        tokio::spawn(async move {
            let mut lines = stdout.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut logs = LOGS.lock().await;
                let log = logs.get_mut(&(database_id as _));
                if let Some(log) = log {
                    let timestamp_string =
                        format!("{}", chrono::Utc::now().format("%d/%m/%Y %H:%M:%S"));
                    log.push(
                        format!("[{timestamp_string}] [Git]: {line}")
                            .chars()
                            .take(200)
                            .collect::<String>(),
                    );

                    if log.len() > 200 {
                        log.remove(0);
                    }
                }
            }
            println!(
                "Process stopped with id {process_id} and name {}",
                process.name
            );
        });
    }

    tokio::spawn(async move {
        let mut lines = stderr.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let mut logs = LOGS.lock().await;
            let log = logs.get_mut(&(database_id as _));
            if let Some(log) = log {
                let timestamp_string =
                    format!("{}", chrono::Utc::now().format("%d/%m/%Y %H:%M:%S"));
                log.push(
                    format!("[{timestamp_string}] [Git-Error]: {line}")
                        .chars()
                        .take(300)
                        .collect::<String>(),
                );

                if log.len() > 200 {
                    log.remove(0);
                }
            }
        }
    });

    Ok(process_id)
}

#[derive(Deserialize)]
pub enum RequestAction {
    Status,
    Pull,
}

#[derive(Deserialize)]
pub struct RequestBody {
    pub action: RequestAction,
}
