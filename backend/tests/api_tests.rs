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

    let exam_data: serde_json::Value = exam_resp.json().await.unwrap();
    let questions = exam_data["questions"]
        .as_array()
        .expect("Questions not found");
    let exam_token = exam_data["exam_token"]
        .as_str()
        .expect("Exam token not found");
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
        .json(&serde_json::json!({
            "answers": answers,
            "exam_token": exam_token
        }))
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

#[tokio::test]
async fn test_community_flow() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect to DB for direct manipulation
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    // 1. Register User A
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

    // Login
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

    // 2. Try to Post (Unverified) -> Should Fail
    let post_resp = client
        .post(&format!("{}/api/posts", address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "title": "My First Post",
            "content": "Hello World!"
        }))
        .send()
        .await
        .expect("Post request failed");

    // Should be 401 Unauthorized (AuthError)
    assert_eq!(post_resp.status().as_u16(), 401);

    // 3. Verify User A directly in DB
    sqlx::query!(
        "UPDATE users SET is_verified = TRUE WHERE username = $1",
        username
    )
    .execute(&pool)
    .await
    .expect("Failed to verify user");

    // 4. Try to Post Again (Verified) -> Should Success
    let post_resp = client
        .post(&format!("{}/api/posts", address))
        .header("Authorization", format!("Bearer {}", token))
        .json(&serde_json::json!({
            "title": "My First Post",
            "content": "Hello World!"
        }))
        .send()
        .await
        .expect("Post request failed");

    assert_eq!(post_resp.status().as_u16(), 201);

    let post_json: serde_json::Value = post_resp.json().await.unwrap();
    let post_id = post_json["id"].as_i64().expect("Post ID not found");

    // 5. List Posts
    let list_resp = client
        .get(&format!("{}/api/posts", address))
        .send()
        .await
        .expect("List request failed");

    assert_eq!(list_resp.status().as_u16(), 200);

    let posts: Vec<serde_json::Value> = list_resp.json().await.unwrap();
    // Check if our post is in the list
    let found = posts.iter().any(|p| p["id"].as_i64() == Some(post_id));
    assert!(found, "Created post should appear in the list");

    // 6. Get Post Details
    let detail_resp = client
        .get(&format!("{}/api/posts/{}", address, post_id))
        .send()
        .await
        .expect("Detail request failed");

    assert_eq!(detail_resp.status().as_u16(), 200);

    // 7. Delete Post
    let del_resp = client
        .delete(&format!("{}/api/posts/{}", address, post_id))
        .header("Authorization", format!("Bearer {}", token))
        .send()
        .await
        .expect("Delete request failed");

    assert_eq!(del_resp.status().as_u16(), 204);

    // 8. Verify Soft Delete (List should not contain it)
    let list_resp_2 = client
        .get(&format!("{}/api/posts", address))
        .send()
        .await
        .expect("List request failed");

    let posts_2: Vec<serde_json::Value> = list_resp_2.json().await.unwrap();
    let found_2 = posts_2.iter().any(|p| p["id"].as_i64() == Some(post_id));
    assert!(!found_2, "Deleted post should NOT appear in the list");

    // 9. Verify Detail (Should be 404)
    let detail_resp_2 = client
        .get(&format!("{}/api/posts/{}", address, post_id))
        .send()
        .await
        .expect("Detail request failed");

    assert_eq!(detail_resp_2.status().as_u16(), 404);
}

#[tokio::test]
async fn test_community_pagination() {
    use std::time::Duration;

    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Connect DB
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test DB");

    // 1. Register & Verify User
    let username = format!("u_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let password = "password123";

    client
        .post(&format!("{}/api/auth/register", address))
        .json(&serde_json::json!({"username": username, "password": password}))
        .send()
        .await
        .expect("Register failed");

    sqlx::query!(
        "UPDATE users SET is_verified = TRUE WHERE username = $1",
        username
    )
    .execute(&pool)
    .await
    .expect("Verify failed");

    let login_resp = client
        .post(&format!("{}/api/auth/login", address))
        .json(&serde_json::json!({"username": username, "password": password}))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    let token = login_resp["token"].as_str().unwrap();

    // 2. Create 3 posts with small delays
    for i in 1..=3 {
        client
            .post(&format!("{}/api/posts", address))
            .header("Authorization", format!("Bearer {}", token))
            .json(&serde_json::json!({"title": format!("Post {}", i), "content": "Content"}))
            .send()
            .await
            .expect("Post failed");

        // Ensure timestamp difference
        tokio::time::sleep(Duration::from_millis(100)).await;
    }

    // 3. Fetch Page 1 (Limit 2)
    // Expected order: Post 3, Post 2
    let page1_resp = client
        .get(&format!("{}/api/posts?limit=2", address))
        .send()
        .await
        .expect("List failed");

    let page1: Vec<serde_json::Value> = page1_resp.json().await.unwrap();
    assert_eq!(page1.len(), 2);
    assert_eq!(page1[0]["title"], "Post 3");
    assert_eq!(page1[1]["title"], "Post 2");

    // 4. Fetch Page 2 (Cursor = Post 2's created_at)
    let cursor = page1[1]["created_at"].as_str().unwrap();

    let page2_resp = client
        .get(&format!("{}/api/posts", address))
        .query(&[("limit", "2"), ("cursor", cursor)])
        .send()
        .await
        .expect("List page 2 failed");

    let page2: Vec<serde_json::Value> = page2_resp.json().await.unwrap();
    assert!(page2.len() >= 1, "Page 2 should contain at least one post");
    // Since we sort by created_at DESC, and Post 1 is the oldest of our three,
    // it should be the first one after Post 2's cursor (if no other posts were made exactly at that time).
    assert_eq!(page2[0]["title"], "Post 1");
}

#[tokio::test]
async fn test_interaction_flow() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap();

    // 1. Setup Users A and B (Both verified)
    let user_a = format!("ua_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let user_b = format!("ub_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let password = "password123";

    for u in &[&user_a, &user_b] {
        client
            .post(&format!("{}/api/auth/register", address))
            .json(&serde_json::json!({"username": u, "password": password}))
            .send()
            .await
            .unwrap();
        sqlx::query!("UPDATE users SET is_verified = TRUE WHERE username = $1", u)
            .execute(&pool)
            .await
            .unwrap();
    }

    // Login A
    let login_a = client
        .post(&format!("{}/api/auth/login", address))
        .json(&serde_json::json!({"username": user_a, "password": password}))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let token_a = login_a["token"].as_str().unwrap();

    // Login B
    let login_b = client
        .post(&format!("{}/api/auth/login", address))
        .json(&serde_json::json!({"username": user_b, "password": password}))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let token_b = login_b["token"].as_str().unwrap();

    // 2. User A Creates Post
    let post_resp = client.post(&format!("{}/api/posts", address))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&serde_json::json!({"title": "Interactions Test", "content": "Let's like and comment!"}))
        .send().await.unwrap();
    let post_id = post_resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap();

    // 3. User B Likes Post
    let like_resp = client
        .post(&format!("{}/api/posts/{}/like", address, post_id))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .unwrap();
    assert_eq!(like_resp.status().as_u16(), 200);
    assert_eq!(
        like_resp.json::<serde_json::Value>().await.unwrap()["liked"],
        true
    );

    // Verify Like Count
    let p_detail = client
        .get(&format!("{}/api/posts/{}", address, post_id))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(p_detail["likes_count"], 1);
    assert_eq!(p_detail["is_liked"], true);

    // 4. User B Unlikes Post
    client
        .post(&format!("{}/api/posts/{}/like", address, post_id))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .unwrap();
    let p_detail_2 = client
        .get(&format!("{}/api/posts/{}", address, post_id))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(p_detail_2["likes_count"], 0);

    // 5. User B Comments (Root)
    let c1_resp = client
        .post(&format!("{}/api/posts/{}/comments", address, post_id))
        .header("Authorization", format!("Bearer {}", token_b))
        .json(&serde_json::json!({"content": "This is root comment"}))
        .send()
        .await
        .unwrap();
    let c1_id = c1_resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap();

    // 6. User A Replies to B (Level 2)
    let c2_resp = client
        .post(&format!("{}/api/posts/{}/comments", address, post_id))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&serde_json::json!({"content": "This is a reply", "parent_id": c1_id}))
        .send()
        .await
        .unwrap();
    let c2_id = c2_resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap();

    // 7. Verify Comments and Counts
    let p_detail_3 = client
        .get(&format!("{}/api/posts/{}", address, post_id))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    assert_eq!(p_detail_3["comments_count"], 2);

    let comments_resp = client
        .get(&format!("{}/api/posts/{}/comments", address, post_id))
        .send()
        .await
        .unwrap();
    let comments: Vec<serde_json::Value> = comments_resp.json().await.unwrap();
    assert_eq!(comments.len(), 2);

    // Check root_id of Level 2 comment
    let reply = comments
        .iter()
        .find(|c| c["id"].as_i64() == Some(c2_id))
        .unwrap();
    assert_eq!(reply["root_id"].as_i64(), Some(c1_id));
    assert_eq!(reply["parent_id"].as_i64(), Some(c1_id));
}

#[tokio::test]
async fn test_contribution_flow() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap();

    // 1. Setup User (Verified) and Admin
    let user_name = format!("u_c_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let admin_name = format!("admin_{}", &uuid::Uuid::new_v4().to_string()[..8]);
    let password = "password123";

    // Register User
    client
        .post(&format!("{}/api/auth/register", address))
        .json(&serde_json::json!({"username": user_name, "password": password}))
        .send()
        .await
        .unwrap();
    sqlx::query!(
        "UPDATE users SET is_verified = TRUE WHERE username = $1",
        user_name
    )
    .execute(&pool)
    .await
    .unwrap();
    let login_user = client
        .post(&format!("{}/api/auth/login", address))
        .json(&serde_json::json!({"username": user_name, "password": password}))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let user_token = login_user["token"].as_str().unwrap();

    // Setup Admin (via direct DB because we need role='admin')
    let hashed_pw = backend::utils::hash::hash_password(password).unwrap();
    sqlx::query!(
        "INSERT INTO users (username, password, role) VALUES ($1, $2, 'admin')",
        admin_name,
        hashed_pw
    )
    .execute(&pool)
    .await
    .unwrap();
    let login_admin = client
        .post(&format!("{}/api/auth/login", address))
        .json(&serde_json::json!({"username": admin_name, "password": password}))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();
    let admin_token = login_admin["token"].as_str().unwrap();

    // 2. Submit valid architecture
    let arch_payload = serde_json::json!({
        "type": "architecture",
        "data": {
            "category": "Palace",
            "name": "Forbidden City Contribution",
            "dynasty": "Ming",
            "location": "Beijing",
            "description": "Crowdsourced desc",
            "cover_img": "http://img.com",
            "carousel_imgs": ["http://1.com"]
        }
    });

    let resp = client
        .post(&format!("{}/api/contributions", address))
        .header("Authorization", format!("Bearer {}", user_token))
        .json(&arch_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status().as_u16(), 201);
    let contrib_id = resp.json::<serde_json::Value>().await.unwrap()["id"]
        .as_i64()
        .unwrap();

    // 3. Try to submit again same day -> Should Fail (409 Conflict)
    let resp_fail = client
        .post(&format!("{}/api/contributions", address))
        .header("Authorization", format!("Bearer {}", user_token))
        .json(&arch_payload)
        .send()
        .await
        .unwrap();
    assert_eq!(resp_fail.status().as_u16(), 409);

    // 4. Admin reviews and approves
    let review_resp = client
        .put(&format!(
            "{}/api/admin/contributions/{}/review",
            address, contrib_id
        ))
        .header("Authorization", format!("Bearer {}", admin_token))
        .json(&serde_json::json!({
            "status": "approved",
            "admin_comment": "Excellent work!"
        }))
        .send()
        .await
        .unwrap();
    assert_eq!(review_resp.status().as_u16(), 200);

    // 5. Verify it's in the real architectures table
    let arch_check = client
        .get(&format!("{}/api/architectures", address))
        .send()
        .await
        .unwrap()
        .json::<Vec<serde_json::Value>>()
        .await
        .unwrap();
    let found = arch_check
        .iter()
        .any(|a| a["name"] == "Forbidden City Contribution");
    assert!(
        found,
        "The approved architecture should be in the main list"
    );
}
