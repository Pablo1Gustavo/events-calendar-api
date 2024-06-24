use axum::{extract::{Path, State}, http::StatusCode, Json};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use serde_json::{json, Value};
use sqlx::Type;

#[derive(Debug, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "contact_type", rename_all = "lowercase")]
pub enum ContactType
{
    Email,
    Phone,
}

#[derive(Debug, FromRow, Serialize)]
pub struct UserContact
{
    pub id          : i64,
    pub user_id     : i64,
    pub contact     : String,
    pub contact_type: String,
}

#[derive(Deserialize)]
pub struct CreateContactRequest
{
    pub user_id: i64,
    pub contact: String,
    pub r#type : ContactType,
}

pub async fn create_contact(
    State(db_pool): State<PgPool>,
    Json(req): Json<CreateContactRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!(
        "INSERT INTO users_contacts (user_id, contact, type) VALUES ($1, $2, $3) RETURNING id",
        req.user_id,
        req.contact,
        req.r#type as ContactType
    )
    .fetch_one(&db_pool)
    .await
    .map_err(|e| (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({
            "status": "error",
            "message": e.to_string()
        })),
    ))?;

    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Contact created successfully.",
            "id": result.id
        })),
    ))
}


pub async fn delete_contact(
    State(db_pool): State<PgPool>,
    Path(id): Path<i64>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!("DELETE FROM users_contacts WHERE id = $1", id)
        .execute(&db_pool)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": e.to_string()
            })),
        ))?;

    match result.rows_affected()
    {
        0 => Err((
            StatusCode::NOT_FOUND,
            Json(json!({"message": "Contact not found."})),
        )),
        _ => Ok((
            StatusCode::OK,
            Json(json!({"message": "Contact deleted successfully.",})),
        )),
    }
}

pub async fn list_contacts(
    State(db_pool): State<PgPool>,
    Path(user_id): Path<i64>,
) -> Result<(StatusCode, Json<Vec<UserContact>>), (StatusCode, Json<Value>)>
{
    let contacts = sqlx::query_as::<_, UserContact>
        ("SELECT * FROM users_contacts WHERE user_id = $1")
        .bind(user_id)
        .fetch_all(&db_pool)
        .await
        .map_err(|e| (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({
                "status": "error",
                "message": e.to_string()
            })),
        ))?;

    Ok((StatusCode::OK, Json(contacts)))
}
