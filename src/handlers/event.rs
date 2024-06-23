use axum::{extract::{Path, State}, http::StatusCode, Json,};
use serde::{Deserialize, Serialize};
use sqlx::{query_builder::QueryBuilder, FromRow, PgPool};
use serde_json::{json, Value};
use chrono::{Datelike, Duration, NaiveDateTime};
use chronoutil::{shift_months, shift_years};
use crate::helpers::error::database_err_mapper;

#[derive(Debug, Deserialize, sqlx::Type, Eq, PartialEq)]
#[serde(rename_all = "lowercase")]
#[sqlx(type_name = "week_day", rename_all = "lowercase")]
pub enum Weekday
{
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}
impl From<chrono::Weekday> for Weekday
{
    fn from(weekday: chrono::Weekday) -> Self
    {
        match weekday {
            chrono::Weekday::Mon => Weekday::Monday,
            chrono::Weekday::Tue => Weekday::Tuesday,
            chrono::Weekday::Wed => Weekday::Wednesday,
            chrono::Weekday::Thu => Weekday::Thursday,
            chrono::Weekday::Fri => Weekday::Friday,
            chrono::Weekday::Sat => Weekday::Saturday,
            chrono::Weekday::Sun => Weekday::Sunday,
        }
    }
}



#[derive(Debug, sqlx::Type)]
#[sqlx(type_name = "recurrence_type", rename_all = "lowercase")]
enum RecurrenceType
{
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase", tag = "type")]
pub enum EventConfiguration
{
    Daily {
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
        step: Option<i16>,
        repetitions: Option<i16>,
        end_date: Option<NaiveDateTime>,
    },
    Weekly {
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
        step: Option<i16>,
        repetitions: Option<i32>,
        end_date: Option<NaiveDateTime>,
        days_of_week: Vec<Weekday>,
    },
    Monthly {
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
        step: Option<i16>,
        repetitions: Option<i16>,
        end_date: Option<NaiveDateTime>,
    },
    Yearly {
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
        step: Option<i16>,
        repetitions: Option<i16>,
        end_date: Option<NaiveDateTime>,
    },
    Individual {
        start_time: NaiveDateTime,
        end_time: NaiveDateTime,
    }
}

#[derive(Deserialize)]
pub struct CreateEventRequest
{
    pub name: String,
    pub description: String,
    pub private: Option<bool>,
    pub user_id: i64,
    pub super_event_id: Option<i64>,
    pub configuration: Option<EventConfiguration>,
}

async fn create_recurrence(
    transaction: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    recurrence_type: RecurrenceType,
    step: i16,
    repetitions: i16,
    end_date: NaiveDateTime,
) -> Result<i64, sqlx::Error>
{
    let recurrence_result = sqlx::query!(
        "INSERT INTO recurrences (type, step, repetitions, end_date)
        VALUES ($1, $2, $3, $4) RETURNING id",
        recurrence_type as RecurrenceType,
        step,
        repetitions,
        end_date
    )
    .fetch_one(&mut **transaction)
    .await?;

    Ok(recurrence_result.id)
}

pub async fn create_event(
    State(db_pool): State<PgPool>,
    Json(req): Json<CreateEventRequest>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let mut transaction = db_pool.begin().await.map_err(database_err_mapper)?;

    let event_result = sqlx::query!(
        "INSERT INTO events (name, description, private, super_event_id)
        VALUES ($1, $2, $3, $4) RETURNING id",
        req.name,
        req.description,
        req.private.unwrap_or(false),
        req.super_event_id
    )
    .fetch_one(&mut *transaction)
    .await
    .map_err(database_err_mapper)?;

    sqlx::query!(
        "INSERT INTO users_events (user_id, event_id, confirmation, owner)
        VALUES ($1, $2, true, true)",
        req.user_id,
        event_result.id
    )
    .execute(&mut *transaction)
    .await
    .map_err(database_err_mapper)?;

    let mut schedule_times: Vec<(NaiveDateTime, NaiveDateTime)> = Vec::new();
    let mut recurrence_id: Option<i64> = None;

    match req.configuration
    {
        Some(EventConfiguration::Daily { start_time, end_time, step, repetitions, end_date }) =>
        {
            let mut curr_start_time = start_time;
            let mut curr_end_time = end_time;

            let final_step = step.unwrap_or(1);
            let final_end_date    = end_date.unwrap_or(start_time + Duration::days(final_step as i64 * repetitions.unwrap_or(1) as i64));
            let final_repetitions = ((final_end_date - end_time).num_days() / final_step as i64) as i16;

            while curr_end_time <= final_end_date
            {
                schedule_times.push((curr_start_time, curr_end_time));

                curr_start_time += Duration::days(final_step as i64);
                curr_end_time   += Duration::days(final_step as i64);
            }

            recurrence_id = Some(
                create_recurrence(&mut transaction, RecurrenceType::Daily, final_step, final_repetitions, final_end_date)
                .await
                .map_err(database_err_mapper)?);
        },
        Some(EventConfiguration::Weekly { start_time, end_time, step, repetitions, end_date, days_of_week }) =>
        {
            let mut curr_start_time = start_time;
            let mut curr_end_time   = start_time;

            let final_step = step.unwrap_or(1);
            let final_end_date    = end_date.unwrap_or(start_time + Duration::weeks(final_step as i64 * repetitions.unwrap_or(1) as i64));
            let final_repetitions = ((final_end_date - end_time).num_weeks() / final_step as i64) as i16;

            while curr_end_time <= final_end_date
            {
                if days_of_week.contains(&curr_end_time.weekday().into())
                {
                    schedule_times.push((curr_start_time, curr_end_time));
                }
                curr_start_time += Duration::days(1);
                curr_end_time   += Duration::days(1);
            }

            recurrence_id = Some(
                create_recurrence(&mut transaction, RecurrenceType::Daily, final_step, final_repetitions, final_end_date)
                .await
                .map_err(database_err_mapper)?);

            let mut insert_recurrence_week_days_query: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
                "INSERT INTO recurrences_week_days (recurrence_id, week_day)");

            insert_recurrence_week_days_query.push_values(days_of_week.iter(), |mut b, day_of_week|
            {
                b.push_bind(recurrence_id.unwrap())
                .push_bind(day_of_week);
            });

            insert_recurrence_week_days_query.build()
                .execute(&mut *transaction)
                .await
                .map_err(database_err_mapper)?;
        },
        Some(EventConfiguration::Monthly { start_time, end_time, step, repetitions, end_date }) =>
        {
            let mut curr_start_time = start_time;
            let mut curr_end_time   = end_time;

            let final_step = step.unwrap_or(1);
            let final_end_date    = end_date.unwrap_or(shift_months(start_time, final_step as i32 * repetitions.unwrap_or(1) as i32));

            let total_months      = (final_end_date.year() - end_time.year()) * 12 + (final_end_date.month() - end_time.month()) as i32;
            let final_repetitions = (total_months / final_step as i32) as i16;

            while curr_end_time <= final_end_date
            {
                schedule_times.push((curr_start_time, curr_end_time));

                curr_start_time = shift_months(curr_start_time, final_step as i32);
                curr_end_time   = shift_months(curr_end_time, final_step as i32);
            }

            recurrence_id = Some(
                create_recurrence(&mut transaction, RecurrenceType::Monthly, final_step, final_repetitions, final_end_date)
                .await
                .map_err(database_err_mapper)?);
        },
        Some(EventConfiguration::Yearly { start_time, end_time, step, repetitions, end_date }) =>
        {
            let mut curr_start_time = start_time;
            let mut curr_end_time   = end_time;

            let final_step = step.unwrap_or(1);
            let final_end_date    = end_date.unwrap_or(shift_years(start_time, final_step as i32 * repetitions.unwrap_or(1) as i32));

            let total_years       = final_end_date.year() - end_time.year();
            let final_repetitions = (total_years / final_step as i32) as i16;

            while curr_end_time <= final_end_date
            {
                schedule_times.push((curr_start_time, curr_end_time));

                curr_start_time = shift_years(curr_start_time, final_step as i32);
                curr_end_time   = shift_years(curr_end_time, final_step as i32);
            }

            recurrence_id = Some(
                create_recurrence(&mut transaction, RecurrenceType::Yearly, final_step, final_repetitions, final_end_date)
                .await
                .map_err(database_err_mapper)?);
        },
        Some(EventConfiguration::Individual { start_time, end_time }) =>
        {
            schedule_times.push((start_time, end_time));
        },
        None => {}
    }

    if !schedule_times.is_empty()
    {
        let mut insert_schedules_query: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
            "INSERT INTO schedules (event_id, recurrence_id, start_time, end_time)");
    
        insert_schedules_query.push_values(schedule_times.iter(), |mut b, (start_time, end_time)|
        {
            b.push_bind(event_result.id)
            .push_bind(recurrence_id)
            .push_bind(start_time)
            .push_bind(end_time);
        });
        
        insert_schedules_query.build()
            .execute(&mut *transaction)
            .await
            .map_err(database_err_mapper)?;
    }

    transaction.commit().await.map_err(database_err_mapper)?;
    
    Ok((
        StatusCode::CREATED,
        Json(json!({
            "message": "Event created successfully.",
            "id"     : event_result.id
        })),
    ))
}

#[derive(Debug, FromRow, Serialize)]
pub struct Schedule
{
    id           : i64,
    recurrence_id: Option<i64>,
    event_id     : i64,
    start_time   : NaiveDateTime,
    end_time     : NaiveDateTime,
}


pub async fn list_event_schedules(
    Path(event_id): Path<i64>,
    State(db_pool): State<PgPool>,
) -> Result<(StatusCode, Json<Vec<Schedule>>), (StatusCode, Json<Value>)>
{
    let schedules = sqlx::query_as::<_, Schedule>
        ("SELECT * FROM schedules WHERE event_id = $1
        ORDER BY start_time")
        .bind(event_id)
        .fetch_all(&db_pool)
        .await
        .map_err(database_err_mapper)?;

    Ok((StatusCode::OK, Json(schedules)))
}

pub async fn list_user_schedules(
    Path(user_id): Path<i64>,
    State(db_pool): State<PgPool>,
) -> Result<(StatusCode, Json<Vec<Schedule>>), (StatusCode, Json<Value>)>
{
    let schedules = sqlx::query_as::<_, Schedule>
        ("SELECT id, recurrence_id, event_id, start_time, end_time FROM schedules
        NATURAL JOIN users_events
        WHERE user_id = $1
        ORDER BY start_time")
        .bind(user_id)
        .fetch_all(&db_pool)
        .await
        .map_err(database_err_mapper)?;

    Ok((StatusCode::OK, Json(schedules)))
}

pub async fn delete_schedule(
    Path(id): Path<i64>,
    State(db_pool): State<PgPool>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!("DELETE FROM schedules WHERE id = $1", id)
        .execute(&db_pool)
        .await
        .map_err(database_err_mapper)?;

    match result.rows_affected()
    {
        0 => Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"message": "Schedule not found."})),
        )),
        _ => Ok((
            StatusCode::OK,
            Json(json!({"message": "Schedule deleted successfully."})),
        )),
    }
}

pub async fn delete_recurrence(
    Path(id): Path<i64>,
    State(db_pool): State<PgPool>,
) -> Result<(StatusCode, Json<Value>), (StatusCode, Json<Value>)>
{
    let result = sqlx::query!("DELETE FROM recurrences WHERE id = $1", id)
        .execute(&db_pool)
        .await
        .map_err(database_err_mapper)?;

    match result.rows_affected()
    {
        0 => Ok((
            StatusCode::NOT_FOUND,
            Json(json!({"message": "Recurrence not found."})),
        )),
        _ => Ok((
            StatusCode::OK,
            Json(json!({"message": "Schedules with recurrence deleted successfully."})),
        )),
    }
}