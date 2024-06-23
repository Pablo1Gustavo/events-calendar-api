use axum::{extract::{Path, State}, http::StatusCode, Json,};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use serde_json::{json, Value};
use crate::helpers::error::database_err_mapper;

#[derive(Debug, FromRow, Serialize)]
pub struct User
{
    pub id: i64,
    pub external_id: Option<String>,
    pub name: String,
}

pub async fn list_users(
    State(db_pool): State<PgPool>,
) -> Result<(StatusCode, Json<Vec<User>>), (StatusCode, Json<Value>)>
{
    let users = sqlx::query_as::<_, User>
        ("SELECT * FROM users")
        .fetch_all(&db_pool)
        .await
        .map_err(database_err_mapper)?;

    Ok((StatusCode::OK, Json(users)))
}

#[derive(Deserialize)]
pub struct CreateOrUpdateUserRequest
{
    name       : String,
    external_id: String,
}

pub async fn create_user(
    State(db_pool): State<PgPool>,
    Json(req): Json<CreateOrUpdateUserRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!(
        "INSERT INTO users (name, external_id) VALUES ($1, $2) RETURNING id",
        req.name,
        req.external_id
    )
    .fetch_one(&db_pool)
    .await
    .map_err(database_err_mapper)?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "User created successfully.",
            "id": result.id
        })),
    ))
}

pub async fn update_user(
    State(db_pool): State<PgPool>,
    Path(id): Path<i64>,
    Json(req): Json<CreateOrUpdateUserRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!(
        "UPDATE users SET name = $1, external_id = $2 WHERE id = $3",
        req.name,
        req.external_id,
        id
    )
    .execute(&db_pool)
    .await
    .map_err(database_err_mapper)?;

    match result.rows_affected()
    {
        0 => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"message": "User not found."})),
        )),
        _ => Ok((
            StatusCode::OK,
            Json(json!({"message": "User updated successfully.",})),
        )),
    }
}


pub async fn delete_user(
    State(db_pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!("DELETE FROM users WHERE id = $1", id)
        .execute(&db_pool)
        .await
        .map_err(database_err_mapper)?;

    match result.rows_affected()
    {
        0 => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"message": "User not found."}))
        )),
        _ => Ok((StatusCode::OK,
            Json(json!({"message": "User deleted successfully."}))
        ))
    }
}