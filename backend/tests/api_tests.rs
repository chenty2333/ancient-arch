// tests/api_tests.rs

use backend::{config::Config, routes, state::AppState};
use sqlx::postgres::PgPoolOptions;
use std::collections::HashMap;

/// Helper function to spawn the app on a random port for testing.
/// Returns the base URL (e.g., "http://127.0.0.1:12345").
async fn spawn_app() -> String {
    // Note: For Postgres, you must have a running database.
    // We'll read from DATABASE_URL environment variable.
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // 1. Create a pool
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres for testing. Make sure DATABASE_URL is set.");

    // 2. Run migrations
    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    // 3. Create test configuration and state
    let config = Config {
        database_url: database_url.clone(),
        jwt_secret: "test_secret_for_integration_tests".to_string(),
        jwt_expiration: 600, // 10 minutes for tests
        rust_log: "error".to_string(),
        admin_username: None,
        admin_password: None,
    };

    let state = AppState { pool, config };

    // 4. Create the router with the app state
    let app = routes::create_router(state);

    // 5. Bind to port 0 to get a random available port
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("Failed to bind random port");

    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    // 6. Spawn the server in the background
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
    // Truncate UUID to ensure username length < 20
    let unique_name = format!("u_{}", &uuid::Uuid::new_v4().to_string()[..8]);

    // Act
    let response = client
        .post(&format!("{}/api/auth/register", address))
        .json(&serde_json::json!({
            "username": unique_name,
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

#[tokio::test]
async fn test_qualification_flow() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to DB for Seeding
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    // 0. Seed questions
    for i in 0..20 {
        sqlx::query!(
            r#"
            INSERT INTO questions (type, content, options, answer, analysis)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            "single",
            format!("Question {}", i),
            serde_json::json!(["A", "B", "C", "D"]),
            "A",
            "Analysis"
        )
        .execute(&pool)
        .await
        .unwrap();
    }

    // 1. Register
    let username = format!("u_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let password = "password123";

    client
        .post(&format!("{}/api/auth/register", address))
        .json(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .expect("Register failed");

    // 2. Login to get token and check initial status
    let login_resp = client
        .post(&format!("{}/api/auth/login", address))
        .json(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .expect("Login failed")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse login json");

    let token = login_resp["token"].as_str().expect("Token not found");
    assert_eq!(
        login_resp["is_verified"], false,
        "User should initially be unverified"
    );

    // 3. Fetch Exam
    let exam_resp = client
        .get(&format!("{}/api/auth/qualification", address))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Fetch exam failed");

    assert_eq!(exam_resp.status().as_u16(), 200);

    let questions: Vec<serde_json::Value> = exam_resp.json().await.unwrap();
    assert!(questions.len() > 0);

    // 4. Submit Answers (All 'A', which is correct per our seed)
    let mut answers = HashMap::new();
    for q in questions {
        let id = q["id"].as_i64().unwrap();
        answers.insert(id, "A".to_string());
    }

    let submit_resp = client
        .post(&format!("{}/api/auth/qualification/submit", address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({ "answers": answers }))
        .send()
        .await
        .expect("Submit failed");

    assert_eq!(submit_resp.status().as_u16(), 200);
    let result: serde_json::Value = submit_resp.json().await.unwrap();
    assert_eq!(result["passed"], true);

    // 5. Login again to verify status updated
    let login_resp_2 = client
        .post(&format!("{}/api/auth/login", address))
        .json(&serde_json::json!({
            "username": username,
            "password": password
        }))
        .send()
        .await
        .expect("Login failed")
        .json::<serde_json::Value>()
        .await
        .expect("Failed to parse login json");

    assert_eq!(
        login_resp_2["is_verified"], true,
        "User should be verified after passing exam"
    );
}
