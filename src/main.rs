use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::{Html, IntoResponse, Redirect},
    routing::{get, post},
    Json, Router,
};
use dotenv::dotenv;
use nanoid::nanoid;
use serde::{Deserialize, Serialize};
use sqlx::sqlite::{SqlitePool, SqliteRow};
use sqlx::Row;
use std::env;
use validator::Validate;

// Custom error type
#[derive(Debug)]
enum AppError {
    SqlxError(sqlx::Error),
    ValidationError(String),
    IdExists(String),
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::SqlxError(err)
    }
}

impl From<validator::ValidationErrors> for AppError {
    fn from(err: validator::ValidationErrors) -> Self {
        AppError::ValidationError(err.to_string())
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> axum::response::Response {
        let (status, error_message) = match self {
            AppError::SqlxError(sqlx::Error::RowNotFound) => {
                (StatusCode::NOT_FOUND, "Not Found".to_string())
            }
            AppError::SqlxError(err) => {
                eprintln!("Database error: {:?}", err);
                (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong".to_string())
            }
            AppError::ValidationError(err) => (StatusCode::BAD_REQUEST, err),
            AppError::IdExists(id) => (
                StatusCode::CONFLICT,
                format!("ID '{}' already exists", id),
            ),
        };

        (status, error_message).into_response()
    }
}

#[derive(Clone)]
struct AppState {
    pool: SqlitePool,
    host: String,
    port: u16,
}

#[derive(Deserialize, Validate)]
struct ShortenRequest {
    #[validate(url)]
    url: String,
    custom_id: Option<String>,
}

#[derive(Serialize)]
struct ShortenResponse {
    url: String,
}

async fn homepage() -> Html<&'static str> {
    Html(include_str!("../templates/index.html"))
}

async fn shorten(
    State(state): State<AppState>,
    Json(payload): Json<ShortenRequest>,
) -> Result<impl IntoResponse, AppError> {
    payload.validate()?;

    // Check if the URL already exists
    let existing_url: Result<SqliteRow, sqlx::Error> = sqlx::query("SELECT id FROM urls WHERE original_url = ?")
        .bind(&payload.url)
        .fetch_one(&state.pool)
        .await;

    if let Ok(row) = existing_url {
        let id: String = row.get("id");
        let short_url = format!("http://{}:{}/{}", state.host, state.port, id);
        return Ok((StatusCode::OK, Json(ShortenResponse { url: short_url })));
    }

    let id = if let Some(custom_id) = payload.custom_id {
        // Check if the custom ID already exists
        let result: Result<SqliteRow, sqlx::Error> = sqlx::query("SELECT id FROM urls WHERE id = ?")
            .bind(&custom_id)
            .fetch_one(&state.pool)
            .await;

        if result.is_ok() {
            return Err(AppError::IdExists(custom_id));
        }
        custom_id
    } else {
        nanoid!(10)
    };

    sqlx::query("INSERT INTO urls (id, original_url) VALUES (?, ?)")
        .bind(&id)
        .bind(&payload.url)
        .execute(&state.pool)
        .await?;

    let short_url = format!("http://{}:{}/{}", state.host, state.port, id);

    Ok((StatusCode::CREATED, Json(ShortenResponse { url: short_url })))
}

async fn redirect(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<impl IntoResponse, AppError> {
    let row: SqliteRow = sqlx::query("SELECT original_url FROM urls WHERE id = ?")
        .bind(id)
        .fetch_one(&state.pool)
        .await?;

    let original_url: String = row.get("original_url");
    Ok(Redirect::to(&original_url))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv().ok();

    let db_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = SqlitePool::connect(&db_url).await?;

    let host = env::var("HOST").unwrap_or_else(|_| "127.0.0.1".to_string());
    let port = env::var("PORT")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3000);

    let app_state = AppState {
        pool,
        host: host.clone(),
        port,
    };

    let app = Router::new()
        .route("/", get(homepage))
        .route("/shorten", post(shorten))
        .route("/:id", get(redirect))
        .with_state(app_state);

    let addr = format!("{}:{}", host, port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    println!("listening on {}", listener.local_addr()?);
    axum::serve(listener, app).await?;

    Ok(())
}