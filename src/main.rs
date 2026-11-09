mod error;
mod handlers;
mod models;
mod validation;

use axum::{extract::State, http::StatusCode, routing::get, Router};
use sqlx::{postgres::PgPoolOptions, PgPool};

async fn health(State(pool): State<PgPool>) -> Result<&'static str, StatusCode> {
    sqlx::query("SELECT 1")
        .execute(&pool)
        .await
        .map(|_| "ok")
        .map_err(|_| StatusCode::SERVICE_UNAVAILABLE)
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL no está definida");
    let port = std::env::var("PORT").unwrap_or_else(|_| "3000".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await
        .expect("no se pudo conectar a PostgreSQL");

    let app = Router::new()
        .route("/health", get(health))
        .route("/pokemon", get(handlers::list_pokemon).post(handlers::create_pokemon))
        .route(
            "/pokemon/{id}",
            get(handlers::get_pokemon_by_id)
                .put(handlers::update_pokemon)
                .delete(handlers::delete_pokemon),
        )
        .route("/pokemon/name/{name}", get(handlers::get_pokemon_by_name))
        .route("/types", get(handlers::list_types))
        .route("/types/{id}/pokemon", get(handlers::list_pokemon_by_type))
        .with_state(pool);

    let listener = tokio::net::TcpListener::bind(format!("0.0.0.0:{port}"))
        .await
        .unwrap();

    axum::serve(listener, app).await.unwrap();
}