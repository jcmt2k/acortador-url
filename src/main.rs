use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use dotenv::dotenv;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqliteRow};
use sqlx::Row;
use std::env;

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
}

#[derive(Deserialize)]
struct ShortenRequest {
    url: String,
}

#[derive(Serialize)]
struct ShortenResponse {
    url: String,
}

async fn shorten(
    State(state): State<AppState>,
    Json(payload): Json<ShortenRequest>,
) -> impl IntoResponse {
    let id = nanoid!(10);

    sqlx::query("INSERT INTO urls (id, original_url) VALUES (?, ?)")
        .bind(&id)
        .bind(&payload.url)
        .execute(&state.pool)
        .await
        .unwrap();

    let short_url = format!("http://localhost:3000/{}", id);

    (StatusCode::CREATED, Json(ShortenResponse { url: short_url }))
}

async fn redirect(State(state): State<AppState>, Path(id): Path<String>) -> impl IntoResponse {
    let result: Result<SqliteRow, sqlx::Error> = sqlx::query("SELECT original_url FROM urls WHERE id = ?")
        .bind(id)
        .fetch_one(&state.pool)
        .await;

    match result {
        Ok(row) => {
            let original_url: String = row.get("original_url");
            Redirect::to(&original_url).into_response()
        }
        Err(_) => (StatusCode::NOT_FOUND, "Not Found").into_response(),
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&db_url).await.unwrap();

    let app = Router::new()
        .route("/shorten", post(shorten))
        .route("/:id", get(redirect))
        .with_state(AppState { pool });

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}