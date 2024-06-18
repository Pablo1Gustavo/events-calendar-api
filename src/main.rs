use std::time::Duration;
use axum::{
    routing::get,
    Router
};
use sqlx::postgres::PgPoolOptions;
use tokio::net::TcpListener;

#[tokio::main]
async fn main()
{
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
        .with_state(db_pool);

    let listener = TcpListener::bind(format!("0.0.0.0:{}", app_port)).await.unwrap();

    println!("App listening on {}", listener.local_addr().unwrap());

    axum::serve(listener, app).await.unwrap();
}

async fn health_check() -> &'static str
{
    "I'm alive!"
}