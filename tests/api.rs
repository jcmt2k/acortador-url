use acortador_url::app;
use axum::{
    body::Body,
    http::{Request, StatusCode},
};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePool;
use tower::ServiceExt; // for `oneshot`

async fn setup_test_db() -> SqlitePool {
    let pool = SqlitePool::connect(":memory:").await.unwrap();
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    pool
}

#[tokio::test]
async fn test_shorten() {
    let pool = setup_test_db().await;
    let app = app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/shorten")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "url": "https://www.rust-lang.org/" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert!(body["url"].as_str().unwrap().starts_with("http://"));
}

#[tokio::test]
async fn test_shorten_invalid_url() {
    let pool = setup_test_db().await;
    let app = app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/shorten")
                .header("Content-Type", "application/json")
                .body(Body::from(json!({ "url": "not-a-url" }).to_string()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::BAD_REQUEST);
}

#[tokio::test]
async fn test_shorten_custom_id() {
    let pool = setup_test_db().await;
    let app = app(pool);

    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/shorten")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "url": "https://www.google.com", "custom_id": "my-google" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();

    assert_eq!(body["url"].as_str().unwrap(), "http://127.0.0.1:3000/my-google");
}

#[tokio::test]
async fn test_shorten_custom_id_exists() {
    let pool = setup_test_db().await;
    let app = app(pool);

    // Shorten a URL with a custom ID first
    app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/shorten")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "url": "https://www.google.com", "custom_id": "my-google" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    // Try to shorten another URL with the same custom ID
    let response = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/shorten")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "url": "https://www.rust-lang.org/", "custom_id": "my-google" })
                        .to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CONFLICT);
}

#[tokio::test]
async fn test_redirect() {
    let pool = setup_test_db().await;
    let app = app(pool);

    // Shorten a URL first
    let response = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/shorten")
                .header("Content-Type", "application/json")
                .body(Body::from(
                    json!({ "url": "https://www.rust-lang.org/" }).to_string(),
                ))
                .unwrap(),
        )
        .await
        .unwrap();

    let body = response.into_body().collect().await.unwrap().to_bytes();
    let body: Value = serde_json::from_slice(&body).unwrap();
    let short_url = body["url"].as_str().unwrap();

    // Redirect to the original URL
    let response = app
        .oneshot(
            Request::builder()
                .method("GET")
                .uri(short_url)
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    assert_eq!(response.headers().get("location").unwrap(), "https://www.rust-lang.org/");
}