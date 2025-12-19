// tests/profile_tests.rs

use backend::{config::Config, routes, state::AppState};
use sqlx::postgres::PgPoolOptions;

async fn spawn_app() -> String {
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .expect("Failed to connect to Postgres for testing.");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to migrate database");

    let config = Config {
        database_url: database_url.clone(),
        jwt_secret: "profile_test_secret".to_string(),
        jwt_expiration: 600,
        rust_log: "error".to_string(),
        admin_username: None,
        admin_password: None,
    };

    let state = AppState { pool, config };
    let app = routes::create_router(state);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let port = listener.local_addr().unwrap().port();
    let address = format!("http://127.0.0.1:{}", port);

    tokio::spawn(async move {
        axum::serve(listener, app).await.unwrap();
    });

    address
}

#[tokio::test]
async fn test_profile_complex_flow() {
    // Arrange
    let address = spawn_app().await;
    let client = reqwest::Client::new();
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPoolOptions::new()
        .max_connections(1)
        .connect(&database_url)
        .await
        .unwrap();

    // 1. Setup User A and User B
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
        // Manual verification
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

    // 2. User A creates 2 posts
    for i in 1..=2 {
        client
            .post(&format!("{}/api/posts", address))
            .header("Authorization", format!("Bearer {}", token_a))
            .json(&serde_json::json!({"title": format!("A Post {}", i), "content": "Content"}))
            .send()
            .await
            .unwrap();
    }

    // 3. User B likes A's first post and favorites A's second post
    let posts_a: Vec<serde_json::Value> = client
        .get(&format!("{}/api/posts", address))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    let post_a1_id = posts_a.iter().find(|p| p["title"] == "A Post 1").unwrap()["id"]
        .as_i64()
        .unwrap();
    let post_a2_id = posts_a.iter().find(|p| p["title"] == "A Post 2").unwrap()["id"]
        .as_i64()
        .unwrap();

    // B likes A1
    client
        .post(&format!("{}/api/posts/{}/like", address, post_a1_id))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .unwrap();

    // B favorites A2
    client
        .post(&format!("{}/api/posts/{}/favorite", address, post_a2_id))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .unwrap();

    // 4. Test /api/profile/me for User A
    let me_a = client
        .get(&format!("{}/api/profile/me", address))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .unwrap()
        .json::<serde_json::Value>()
        .await
        .unwrap();

    assert_eq!(me_a["username"], user_a);
    assert_eq!(me_a["posts_count"], 2);
    assert_eq!(me_a["total_likes_received"], 1);

    // 5. Test /api/profile/favorites for User B
    let favs_b = client
        .get(&format!("{}/api/profile/favorites", address))
        .header("Authorization", format!("Bearer {}", token_b))
        .send()
        .await
        .unwrap()
        .json::<Vec<serde_json::Value>>()
        .await
        .unwrap();

    assert_eq!(favs_b.len(), 1);
    assert_eq!(favs_b[0]["title"], "A Post 2");
    assert_eq!(favs_b[0]["author_username"], user_a);

    // 6. Test /api/profile/posts for User A (Check interaction status)
    // A likes A1 (Self-like)
    client
        .post(&format!("{}/api/posts/{}/like", address, post_a1_id))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .unwrap();

    let my_posts_a = client
        .get(&format!("{}/api/profile/posts", address))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .unwrap()
        .json::<Vec<serde_json::Value>>()
        .await
        .unwrap();

    let a1_status = my_posts_a
        .iter()
        .find(|p| p["id"].as_i64() == Some(post_a1_id))
        .unwrap();
    assert_eq!(a1_status["is_liked"], true);
    assert_eq!(a1_status["likes_count"], 2); // B + A self-like

    // 7. Test /api/profile/contributions
    let contrib_payload = serde_json::json!({
        "type": "question",
        "data": {
            "question_type": "single",
            "content": "What is dougong?",
            "options": ["A", "B", "C", "D"],
            "answer": "A",
            "analysis": "..."
        }
    });
    client
        .post(&format!("{}/api/contributions", address))
        .header("Authorization", format!("Bearer {}", token_a))
        .json(&contrib_payload)
        .send()
        .await
        .unwrap();

    let my_contribs_a = client
        .get(&format!("{}/api/profile/contributions", address))
        .header("Authorization", format!("Bearer {}", token_a))
        .send()
        .await
        .unwrap()
        .json::<Vec<serde_json::Value>>()
        .await
        .unwrap();

    assert_eq!(my_contribs_a.len(), 1);
    assert_eq!(my_contribs_a[0]["type"], "question");
}
