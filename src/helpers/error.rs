use axum::{http::StatusCode, Json,};
use serde_json::{json, Value};

pub fn database_err_mapper(e: sqlx::Error) -> (StatusCode, Json<Value>)
{
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"message": e.to_string()})),
    )
}