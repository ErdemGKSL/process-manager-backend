use std::process::Stdio;
use tokio::process;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use axum::{Extension, Json};
use axum::extract::Path;
use axum::http::StatusCode;
use rand::random;
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::PgPool;
use sqlx::types::chrono;
use crate::library::cache::{CHILDS, LOGS};
use crate::library::model::{Process, ProcessOwner, User};
use crate::State;

pub async fn trigger(Extension(state): Extension<State>, Extension(auth_user): Extension<User>, Path(id): Path<i32>, Json(body): Json<RequestBody>) -> Result<Json<Value>, StatusCode> {
    let db  = &state.db;
    let mut process = sqlx::query_as!(Process, "SELECT * FROM \"Process\" WHERE id = $1", id)
        .fetch_one(db)
        .await
        .map_err(|_| StatusCode::NOT_FOUND)?;

    let process_owners: Vec<_> = sqlx::query_as!(ProcessOwner, "SELECT * FROM \"ProcessOwner\" WHERE process_id = $1", id)
        .fetch_all(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if !auth_user.admin && !process_owners.iter().any(|owner| owner.user_id == auth_user.id) {
        return Err(StatusCode::FORBIDDEN);
    }

    match body.action {
        RequestBodyType::Start => {
            if process.process_id.is_some() {
                return Err(StatusCode::CONFLICT);
            }

            start_process(&mut process, db).await?;

            Ok(Json(json!({
                "ok": true
            })))
        },
        RequestBodyType::Stop => {
            if process.process_id.is_none() {
                return Err(StatusCode::CONFLICT);
            }

            // stop_process(&process, db).await?;

            let mut childs = CHILDS.lock().await;

            let mut child = childs.remove(&(process.process_id.unwrap() as _)).ok_or(StatusCode::NOT_FOUND).map_err(|e| {
                println!("Error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            tokio::spawn(async move {
                let id = child.id();
                println!("Killing process with id {:?}", id);
                if let Some(mut stdin) = child.stdin.take() {
                    let _ = stdin.write_all(b"").await;
                }
            });

            Ok(Json(json!({
                "ok": true
            })))
        },
        RequestBodyType::Restart => {
            if process.process_id.is_none() {
                return Err(StatusCode::CONFLICT);
            }

            let mut childs = CHILDS.lock().await;

            let mut child = childs.remove(&(process.process_id.unwrap() as _)).ok_or(StatusCode::NOT_FOUND).map_err(|e| {
                println!("Error: {:?}", e);
                StatusCode::INTERNAL_SERVER_ERROR
            })?;

            tokio::spawn(async move {
                let id = child.id();
                println!("Killing process with id {:?}", id);
                let _ = child.kill().await;
                let _ = child.start_kill();
            });

            start_process(&mut process, db).await?;

            Ok(Json(json!({
                "ok": true
            })))
        }
    }
}

pub async fn start_process(process: &mut Process, db: &PgPool) -> Result<u32, StatusCode> {

    let name = process.cmd.split(' ').next().unwrap();
    let args = process.cmd.split(' ').skip(1).collect::<Vec<_>>();

    let mut command = process::Command::new(name);

    let id: u32 = random();

    let mut child =
        command
            .args(args)
            .current_dir(&process.dir)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .gid(id)
            .kill_on_drop(true)
            .spawn()
            .map_err(|e| {
                println!("Error: {e:?}");
                StatusCode::FAILED_DEPENDENCY
            })?;

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
        log.push(
            format!("[{timestamp_string}] Process started")
        );
    } else {
        logs.insert(database_id as _, vec![
            format!("[{timestamp_string}] Process started")
        ]);
    }

    {
        let process = process.clone();
        let db = db.clone();
        tokio::spawn(async move {
            let mut lines = stdout.lines();
            while let Ok(Some(line)) = lines.next_line().await {
                let mut logs = LOGS.lock().await;
                let log = logs.get_mut(&(database_id as _));
                if let Some(log) = log {
                    let timestamp_string = format!("{}", chrono::Utc::now().format("%d/%m/%Y %H:%M:%S"));
                    log.push(
                        format!("[{timestamp_string}] [Runtime]: {line}")
                            .chars()
                            .take(200)
                            .collect::<String>()
                    );

                    if log.len() > 200 {
                        log.remove(0);
                    }
                }
            }
            let _ = stop_process(&process, &db).await;
            println!("Process stopped with id {process_id} and name {}", process.name);
        });
    }

    tokio::spawn(async move {
        let mut lines = stderr.lines();
        while let Ok(Some(line)) = lines.next_line().await {
            let mut logs = LOGS.lock().await;
            let log = logs.get_mut(&(database_id as _));
            if let Some(log) = log {
                let timestamp_string = format!("{}", chrono::Utc::now().format("%d/%m/%Y %H:%M:%S"));
                log.push(
                    format!("[{timestamp_string}] [Error]: {line}")
                        .chars()
                        .take(300)
                        .collect::<String>()
                );

                if log.len() > 200 {
                    log.remove(0);
                }
            }
        }
    });

    sqlx::query!("UPDATE \"Process\" SET process_id = $1 WHERE id = $2", process_id as i32, database_id as _)
        .execute(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    process.process_id = Some(process_id as i32);

    let mut childs = CHILDS.lock().await;
    childs.insert(process_id as _, child);

    Ok(process_id)
}

pub async fn stop_process(process: &Process, db: &PgPool) -> Result<(), StatusCode> {
    let mut logs = LOGS.lock().await;
    let log = logs.get_mut(&(process.id as _));
    let timestamp_string = format!("{}", chrono::Utc::now().format("%d/%m/%Y %H:%M:%S"));
    if let Some(log) = log {
        log.push(
            format!("[{timestamp_string}] Process stopped")
        );

        if log.len() > 200 {
            log.remove(0);
        }
    } else {
        logs.insert(process.id as _, vec![
            format!("[{timestamp_string}] Process stopped")
        ]);
    }

    sqlx::query!("UPDATE \"Process\" SET process_id = NULL WHERE id = $1", process.id)
        .execute(db)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

#[derive(Deserialize)]
enum RequestBodyType {
    Start,
    Stop,
    Restart
}

#[derive(Deserialize)]
pub struct RequestBody {
    action: RequestBodyType
}