use axum::{extract::{Path, State}, http::StatusCode, Json,};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use serde_json::{json, Value};
use crate::helpers::error::database_err_mapper;

#[derive(Debug, FromRow, Serialize)]
pub struct Tag 
{
    pub id: i64,
    pub name: String,
    pub color: String,
}

pub async fn list_tags(
    State(db_pool): State<PgPool>,
) -> Result<(StatusCode, Json<Vec<Tag>>), (StatusCode, Json<Value>)>
{
    let tags = sqlx::query_as::<_, Tag>("SELECT * FROM tags")
        .fetch_all(&db_pool)
        .await
        .map_err(database_err_mapper)?;

    Ok((StatusCode::OK, Json(tags)))
}

#[derive(Deserialize)]
pub struct CreateOrUpdateTagRequest {
    name: String,
    color: String,
}

pub async fn create_tag(
    State(db_pool): State<PgPool>,
    Json(req): Json<CreateOrUpdateTagRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!(
        "INSERT INTO tags (name, color) VALUES ($1, $2) RETURNING id",
        req.name,
        req.color
    )
    .fetch_one(&db_pool)
    .await
    .map_err(database_err_mapper)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Tag created successfully.",
            "id": result.id
        })),
    ))
}

pub async fn update_tag(
    State(db_pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(req): Json<CreateOrUpdateTagRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!(
        "UPDATE tags SET name = $1, color = $2 WHERE id = $3",
        req.name,
        req.color,
        id
    )
    .execute(&db_pool)
    .await
    .map_err(database_err_mapper)?;

    match result.rows_affected()
    {
        0 => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"message": "Tag not found."})),
        )),
        _ => Ok((
            StatusCode::OK,
            Json(json!({"message": "Tag updated successfully."})),
        )),
    }
}

pub async fn delete_tag(
    State(db_pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)> 
{
    let result = sqlx::query!("DELETE FROM tags WHERE id = $1", id)
        .execute(&db_pool)
        .await
        .map_err(database_err_mapper)?;

    match result.rows_affected()
    {
        0 => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"message": "Tag not found."}))
        )),
        _ => Ok((
            StatusCode::OK,
            Json(json!({"message": "Tag deleted successfully.",}))
        ))
    }
}
