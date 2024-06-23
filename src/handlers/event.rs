use axum::{extract::{Path, State}, http::StatusCode, Json,};
use serde::{Deserialize, Serialize};
use sqlx::{query_builder::QueryBuilder, FromRow, PgPool};
use serde_json::{json, Value};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime};
use chronoutil::{shift_months, shift_years};

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

fn database_err_mapper(e: sqlx::Error) -> (StatusCode, Json<Value>)
{
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(json!({"message": e.to_string()})),
    )
}

async fn database_err_mapper_with_transaction(
    e: sqlx::Error,
    transaction: sqlx::Transaction<'_, sqlx::Postgres>,
) -> (StatusCode, Json<Value>)
{
    if let Err(e) = transaction.rollback().await
    {
        return database_err_mapper(e);
    };
    database_err_mapper(e)
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

            let recurrence_result = sqlx::query!(
                "INSERT INTO recurrences (type, step, repetitions, end_date)
                VALUES ($1, $2, $3, $4) RETURNING id",
                RecurrenceType::Daily as RecurrenceType,
                final_step,
                final_repetitions,
                final_end_date
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_err_mapper)?;

            recurrence_id = Some(recurrence_result.id);
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

            let recurrence_result = sqlx::query!(
                "INSERT INTO recurrences (type, step, repetitions, end_date)
                VALUES ($1, $2, $3, $4) RETURNING id",
                RecurrenceType::Weekly as RecurrenceType,
                final_step,
                final_repetitions,
                final_end_date
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_err_mapper)?;

            let mut insert_recurrence_week_days_query: QueryBuilder<'_, sqlx::Postgres> = QueryBuilder::new(
                "INSERT INTO recurrences_week_days (recurrence_id, week_day)");

            insert_recurrence_week_days_query.push_values(days_of_week.iter(), |mut b, day_of_week|
            {
                b.push_bind(recurrence_result.id)
                .push_bind(day_of_week);
            });

            insert_recurrence_week_days_query.build()
                .execute(&mut *transaction)
                .await
                .map_err(database_err_mapper)?;

            recurrence_id = Some(recurrence_result.id);
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

            let recurrence_result = sqlx::query!(
                "INSERT INTO recurrences (type, step, repetitions, end_date)
                VALUES ($1, $2, $3, $4) RETURNING id",
                RecurrenceType::Monthly as RecurrenceType,
                final_step,
                final_repetitions,
                final_end_date
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_err_mapper)?;

            recurrence_id = Some(recurrence_result.id);
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

            let recurrence_result = sqlx::query!(
                "INSERT INTO recurrences (type, step, repetitions, end_date)
                VALUES ($1, $2, $3, $4) RETURNING id",
                RecurrenceType::Yearly as RecurrenceType,
                final_step,
                final_repetitions,
                final_end_date
            )
            .fetch_one(&mut *transaction)
            .await
            .map_err(database_err_mapper)?;

            recurrence_id = Some(recurrence_result.id);
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