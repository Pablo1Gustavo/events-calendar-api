use axum::{extract::{Path, State}, http::StatusCode, Json,};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool, Type};
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

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "reminder_type", rename_all = "lowercase")]
pub enum ReminderType
{
    Email,
    SMS,
    WhatsApp,
    Telegram,
    Notification,
}

#[derive(Debug, FromRow, Serialize)]
pub struct Reminder
{
    pub id            : i64,
    pub event_id      : i64,
    pub user_contact_id: i64,
    pub r#type        : ReminderType,
    pub minutes_before: i32,
}

pub async fn list_reminders_from_user(
    State(db_pool): State<PgPool>,
    Path(user_id): Path<i64>,
) -> Result<(StatusCode, Json<Vec<Reminder>>), (StatusCode, Json<Value>)>
{
    let reminders = sqlx::query_as::<_, Reminder>
        ("SELECT * FROM reminders WHERE user_contact_id = $1")
        .bind(user_id)
        .fetch_all(&db_pool)
        .await
        .map_err(database_err_mapper)?;

    Ok((StatusCode::OK, Json(reminders)))
}

#[derive(Deserialize)]
pub struct AddReminderRequest
{
    r#type        : ReminderType,
    minutes_before: i32
}

pub async fn add_reminder(
    State(db_pool): State<PgPool>,
    Path((user_contact_id, event_id)): Path<(i64, i64)>,
    Json(req): Json<AddReminderRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!(
        "INSERT INTO reminders (event_id, user_contact_id, type, minutes_before)
        VALUES ($1, $2, $3, $4)",
        event_id,
        user_contact_id,
        req.r#type as ReminderType,
        req.minutes_before
    )
    .execute(&db_pool)
    .await
    .map_err(database_err_mapper)?;

    match result.rows_affected()
    {
        0 => Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"message": "Event or user contact not found."})),
        )),
        _ => Ok((
            StatusCode::OK,
            Json(json!({"message": "Reminder added to event successfully."})),
        )),
    }
}

pub async fn remove_reminder(
    State(db_pool): State<PgPool>,
    Path((user_contact_id, event_id)): Path<(i64, i64)>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!(
        "DELETE FROM reminders WHERE event_id = $1 AND user_contact_id = $2",
        event_id,
        user_contact_id
    )
    .execute(&db_pool)
    .await
    .map_err(database_err_mapper)?;

    match result.rows_affected()
    {
        0 => Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"message": "Reminder not found."})),
        )),
        _ => Ok((
            StatusCode::OK,
            Json(json!({"message": "Reminder removed successfully."})),
        )),
    }
}