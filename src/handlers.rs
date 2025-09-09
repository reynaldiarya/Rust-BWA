use crate::{
    errors::{AppError, AppResponse},
    models::{CreateItemPayload, Item, UpdateItemPayload},
};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
};
use validator::Validate;

use sqlx::PgPool;
use uuid::Uuid;
type DbPool = PgPool;

pub async fn create_item(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateItemPayload>,
) -> AppResponse<impl IntoResponse> {
    payload.validate()?;

    let item = sqlx::query_as!(
        Item,
        r#"
        INSERT INTO items (name, description) VALUES ($1, $2) RETURNING *;
        "#,
        payload.name,
        payload.description
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::CREATED, Json(item)))
}

pub async fn get_item_list(State(pool): State<DbPool>) -> AppResponse<impl IntoResponse> {
    let items = sqlx::query_as!(
        Item,
        r#"
        SELECT * FROM items ORDER BY created_at DESC;
        "#
    )
    .fetch_all(&pool)
    .await?;

    Ok((StatusCode::OK, Json(items)))
}

pub async fn get_item_by_id(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> AppResponse<impl IntoResponse> {
    let item = sqlx::query_as!(
        Item,
        r#"
        SELECT * FROM items WHERE id = $1;
        "#,
        id
    )
    .fetch_optional(&pool)
    .await?;

    Ok((StatusCode::OK, Json(item)))
}

pub async fn update_item_by_id(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
    Json(payload): Json<UpdateItemPayload>,
) -> AppResponse<impl IntoResponse> {
    payload.validate()?;

    let existing_item = sqlx::query_as!(
        Item,
        r#"
        SELECT * FROM items WHERE id = $1;
        "#,
        id
    )
    .fetch_optional(&pool)
    .await?;

    let existing_item = match existing_item {
        Some(item) => item,
        None => {
            return Err(AppError::NotFound);
        }
    };

    let updated_name = payload.name.unwrap_or(existing_item.name);
    let updated_description = payload
        .description
        .unwrap_or(existing_item.description.unwrap_or_default());

    let updated_item = sqlx::query_as!(
        Item,
        r#"
        UPDATE items SET name = $1, description = $2 WHERE id = $3 RETURNING *;
        "#,
        updated_name,
        updated_description,
        id
    )
    .fetch_one(&pool)
    .await?;

    Ok((StatusCode::OK, Json(updated_item)))
}

pub async fn delete_item_by_id(
    State(pool): State<DbPool>,
    Path(id): Path<Uuid>,
) -> AppResponse<impl IntoResponse> {
    let result = sqlx::query!(
        r#"
        DELETE FROM items WHERE id = $1;
        "#,
        id
    )
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(AppError::NotFound);
    }

    Ok((StatusCode::NO_CONTENT, ()))
}
