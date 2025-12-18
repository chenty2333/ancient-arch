// tests/api_tests.rs

use backend::routes;
use sqlx::sqlite::SqlitePoolOptions;
use std::net::TcpListener;

/// Helper function to spawn the app on a random port for testing.
/// Returns the base URL (e.g., "http://127.0.0.1:12345").
async fn spawn_app() -> String {
    // 1. Create an in-memory database for isolation
    let pool = SqlitePoolOptions::new()
        .connect("sqlite::memory:")
        .await
        .expect("Failed to connect to memory db");

    // 2. Run migrations to ensure the schema exists
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    // 3. Create the router with the test pool
    let app = routes::create_router(pool);

    // 4. Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");
    
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // 5. Spawn the server in the background
    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    address
}

#[tokio::test]
async fn health_check_404() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .get(&format!("{}/random_path_that_does_not_exist", address))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 404);
}

#[tokio::test]
async fn register_works() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act
    let response = client
        .post(&format!("{}/api/auth/register", address))
        .json(&serde_json::json!({
            "username": "testuser",
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 201);
}

#[tokio::test]
async fn register_fails_validation() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();

    // Act: Send a username that is too short
    let response = client
        .post(&format!("{}/api/auth/register", address))
        .json(&serde_json::json!({
            "username": "yo", 
            "password": "password123"
        }))
        .send()
        .await
        .expect("Failed to execute request");

    // Assert
    assert_eq!(response.status().as_u16(), 400);
}
