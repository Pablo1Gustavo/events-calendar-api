use std::time::Duration;
use axum::{
    routing::{get, put, post, delete},
    Router
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

mod handlers;
mod helpers;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let app_port = std::env::var("APP_PORT").unwrap_or("3000".to_owned());
    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL not found in env file");

    let db_pool = PgPoolOptions::new()
        .max_connections(8)
        .acquire_timeout(Duration::from_secs(5))
        .connect(&database_url)
        .await
        .expect("can't connect to database");

    let app = Router::new()
        .route("/health-check", get(health_check))
        .route("/users",
            get(handlers::user::list_users)
            .post(handlers::user::create_user)
        )
        .route("/users/:id",
            put(handlers::user::update_user)
            .delete(handlers::user::delete_user)
        )
        .route("/tags",
            get(handlers::tag::list_tags)
            .post(handlers::tag::create_tag)
        )
        .route("/tags/:id",
            put(handlers::tag::update_tag)
            .delete(handlers::tag::delete_tag)
        )
        .route("/events",
            post(handlers::event::create_event)
        )
        .route("/events/:event_id/user/:user_id",
            post(handlers::event::add_user_to_event)
            .delete(handlers::event::delete_user_from_event)
        )
        .route("/events/:id/commnents",
            post(handlers::event::add_comment_to_event)
        )
        .route("/comments/:id",
            put(handlers::event::update_comment)
            .delete(handlers::event::delete_comment)
        ) 
        .route("/contacts", 
            post(handlers::contact::create_contact)
        )
        .route("/contacts/:id", 
            delete(handlers::contact::delete_contact)
        )
        .route("/contacts/:contact_id/event/:event_id",
            post(handlers::user::add_reminder)
            .delete(handlers::user::remove_reminder)
        )
        .route("/users/:user_id/reminders",
            get(handlers::user::list_reminders_from_user)
        )
        .route("/users/:user_id/contacts", 
            get(handlers::contact::list_contacts)
        )
        .route("/schedules/:id",
            delete(handlers::event::delete_schedule)
        )
        .route("/schedules/event/:id",
            get(handlers::event::list_event_schedules)
        )
        .route("/schedules/user/:id",
            get(handlers::event::list_user_schedules)
        )
        .route("/recurrences/:id",
            delete(handlers::event::delete_recurrence)
        )
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(db_pool);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", app_port)).await.unwrap();

    println!("App listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str {
    "I'm alive!"
}