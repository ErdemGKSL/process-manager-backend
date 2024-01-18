mod routes;
pub mod library;

use std::sync::Arc;
use axum::Extension;
use ctrlc_async::set_handler;
use sqlx::PgPool;
use sqlx::postgres::PgPoolOptions;
use tower_cookies::CookieManagerLayer;
use tower_http::cors::CorsLayer;
use crate::routes::_auth::process::p_id::post::{kill_with_group_id, start_process};

#[tokio::main]
async fn main() {
    dotenv::dotenv().expect("Failed to read .env file");
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set in .env file");
    let port = std::env::var("PORT").expect("PORT must be set in .env file");

    let db = PgPoolOptions::new()
        .max_connections(20)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres");

    // update script: if process_id is not null, set it to null while returning the process_id
    kill_childs(&db).await;

    {
        let db = db.clone();
        set_handler(move || {
            let db = db.clone();
            tokio::spawn(async move {
                kill_childs(&db).await;
                std::process::exit(0);
            });
        }).expect("Failed to set Ctrl-C handler");
    }

    {
        let db = db.clone();
        std::panic::set_hook(Box::new(move |info| {
            let db = db.clone();
            println!("{}", info);
            tokio::spawn(async move {
                kill_childs(&db).await;
                std::process::exit(1);
            });
        }));
    }

    let app = routes::get_app()
        .layer(Extension(Arc::new(StateData {
            db,
        })))
        .layer(CorsLayer::very_permissive())
        .layer(CookieManagerLayer::new());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:".to_owned() + &port)
        .await
        .expect("Failed to bind to port");

    println!("Listening on port {}", port);
    axum::serve(listener, app).await.expect("Failed to start server")
}

async fn kill_childs(db: &PgPool) -> Vec<i32> {
    let ids = sqlx::query!("SELECT id FROM \"Process\" WHERE process_id IS NOT NULL")
        .fetch_all(db)
        .await
        .expect("Failed to get process ids")
        .into_iter()
        .map(|row| row.id)
        .collect::<Vec<_>>();

    let _ = sqlx::query!("UPDATE \"Process\" SET process_id = NULL WHERE process_id IS NOT NULL")
        .execute(db)
        .await;

    let mut childs = library::cache::CHILDS.lock().await;
    for id in ids.iter() {
        if let Some(child) = childs.remove(&(*id as _)) {
            kill_with_group_id(child.group_id, 0);
        }
    }
    return ids;
}
pub type State = Arc<StateData>;
pub struct StateData {
    pub db: PgPool,
    // target_user: Option<User>
}