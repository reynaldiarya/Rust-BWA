mod errors;
mod handlers;
mod models;

use axum::{
    Router,
    routing::{delete, get, post, put},
};

use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use tower_http::trace::{DefaultMakeSpan, TraceLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenvy::dotenv().ok();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "rust_crud=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;
    tracing::info!("Connected to the database");

    let app = Router::new()
        .route("/items", get(handlers::get_item_list))
        .route("/items/{id}", get(handlers::get_item_by_id))
        .route("/items", post(handlers::create_item))
        .route("/items/{id}", put(handlers::update_item_by_id))
        .route("/items/{id}", delete(handlers::delete_item_by_id))
        .with_state(pool)
        .layer(
            TraceLayer::new_for_http().make_span_with(DefaultMakeSpan::new().include_headers(true)),
        );

    let port_str = std::env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let port = port_str.parse::<u16>().expect("PORT must be a number");

    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    Ok(())
}
