use hello_word::handlers::{create_item, get_item_by_id, get_item_list, update_item_by_id};
use hello_word::models::{CreateItemPayload, UpdateItemPayload};

use axum::{Json, extract::State};

use serial_test::serial;
use sqlx::postgres::PgPoolOptions;

async fn setup_test_db() -> sqlx::PgPool {
    let database_url = std::env::var("TEST_DATABASE_URL")
        .unwrap_or_else(|_| "postgres://root:password@localhost:5432/rust_bwa".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("Failed to connect to test database");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("Failed to run migrations");

    pool
}

async fn cleanup_test_data(pool: &sqlx::PgPool) {
    sqlx::query!("DELETE FROM items")
        .execute(pool)
        .await
        .expect("Failed to cleanup test data");
}

#[tokio::test]
#[serial]
async fn test_create_item() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let payload = CreateItemPayload {
        name: "Test Item".to_string(),
        description: Some("Test description".to_string()),
    };

    let result = create_item(State(pool.clone()), Json(payload)).await;

    assert!(result.is_ok(), "Create item should succeed");

    let _response = result.unwrap();

    let items = sqlx::query!("SELECT * FROM items")
        .fetch_all(&pool)
        .await
        .expect("Failed to fetch items");

    assert_eq!(items.len(), 1);
    assert_eq!(items[0].name, "Test Item");
    assert_eq!(items[0].description, Some("Test description".to_string()));

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_get_item_list() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    sqlx::query!(
        "INSERT INTO items (name, description) VALUES ($1, $2), ($3, $4)",
        "Item 1",
        "Description 1",
        "Item 2",
        "Description 2"
    )
    .execute(&pool)
    .await
    .expect("Failed to insert test data");

    let result = get_item_list(State(pool.clone())).await;
    assert!(result.is_ok(), "Get item list should succeed");
    let _response = result.unwrap();

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_get_item_by_id() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let item = sqlx::query!(
        "INSERT INTO items (name, description) VALUES ($1, $2) RETURNING *",
        "Test Item",
        "Test description"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert test data");
    let item_id = item.id;

    let result = get_item_by_id(State(pool.clone()), axum::extract::Path(item_id)).await;
    assert!(result.is_ok(), "Get item by id should succeed");
    let _response = result.unwrap();

    cleanup_test_data(&pool).await;
}

#[tokio::test]
#[serial]
async fn test_update_item_by_id() {
    let pool = setup_test_db().await;
    cleanup_test_data(&pool).await;

    let result = sqlx::query!(
        "INSERT INTO items (name, description) VALUES ($1, $2) RETURNING id",
        "Original Name",
        "Original Description"
    )
    .fetch_one(&pool)
    .await
    .expect("Failed to insert test item");

    let item_id = result.id;

    let update_payload = UpdateItemPayload {
        name: Some("Updated Name".to_string()),
        description: Some("Updated Description".to_string()),
    };

    let result = update_item_by_id(
        State(pool.clone()),
        axum::extract::Path(item_id),
        Json(update_payload),
    )
    .await;

    assert!(result.is_ok(), "Update item should succeed");

    let _response = result.unwrap();
    cleanup_test_data(&pool).await;
}
