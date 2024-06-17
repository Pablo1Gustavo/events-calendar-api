use axum::{
    routing::get,
    Router,
    response::Html
};

#[tokio::main]
async fn main()
{
    let app = Router::new()
        .route("/hello", get(|| async { Html("<h1> Hello world! </h1>")}));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
