use axum::{
    extract::{Path, State},
    Json,
};
use sqlx::PgPool;

use crate::{error::AppError, models::Pokemon};

pub async fn list_pokemon(State(pool): State<PgPool>) -> Result<Json<Vec<Pokemon>>, AppError> {
    let rows = sqlx::query_as::<_, Pokemon>(
        "SELECT id, name, height, weight, base_experience, sprite_url
         FROM pokemon ORDER BY id",
    )
    .fetch_all(&pool)
    .await?;

    Ok(Json(rows))
}

pub async fn get_pokemon_by_id(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<Pokemon>, AppError> {
    sqlx::query_as::<_, Pokemon>(
        "SELECT id, name, height, weight, base_experience, sprite_url
         FROM pokemon WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .map(Json)
    .ok_or_else(|| AppError::NotFound(format!("pokemon {id} no encontrado")))
}

pub async fn get_pokemon_by_name(
    State(pool): State<PgPool>,
    Path(name): Path<String>,
) -> Result<Json<Pokemon>, AppError> {
    sqlx::query_as::<_, Pokemon>(
        "SELECT id, name, height, weight, base_experience, sprite_url
         FROM pokemon WHERE LOWER(name) = LOWER($1)",
    )
    .bind(&name)
    .fetch_optional(&pool)
    .await?
    .map(Json)
    .ok_or_else(|| AppError::NotFound(format!("pokemon '{name}' no encontrado")))
}