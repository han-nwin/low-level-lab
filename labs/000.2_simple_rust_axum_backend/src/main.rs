mod routes;
mod state;

use axum::{Router, routing::get};
use axum::{http::StatusCode, serve::ListenerExt};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
use state::AppState;
use std::str::FromStr;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let connection_options = SqliteConnectOptions::from_str(&database_url)
        .expect("DATABASE_URL must be a valid SQLite URL")
        .create_if_missing(true)
        .foreign_keys(true);

    let db = SqlitePoolOptions::new()
        .connect_with(connection_options)
        .await
        .unwrap();

    // Run migrations on startup
    sqlx::migrate!().run(&db).await.unwrap();

    // Add db into appstate to share across routes
    let state = AppState { db };

    let app = Router::new()
        .route("/health", get(|| async { StatusCode::OK }))
        .merge(routes::router())
        .merge(SwaggerUi::new("/docs").url("/api-docs/openapi.json", routes::ApiDoc::openapi()))
        .with_state(state);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:4000")
        .await
        .unwrap()
        .tap_io(|stream| match stream.peer_addr() {
            Ok(addr) => println!("New connection from: {addr}"),
            Err(err) => eprintln!("Failed to get peer address: {err}"),
        });

    println!("> TeamFlow API listening on http://localhost:4000");

    axum::serve(listener, app).await.unwrap();
}
