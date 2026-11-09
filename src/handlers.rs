use axum::{
    extract::{rejection::JsonRejection, Path, State},
    http::StatusCode,
    Json,
};
use sqlx::PgPool;

use crate::{
    error::AppError,
    models::{AbilityRef, NewPokemon, Pokemon, PokemonDetail, StatRef, TypeItem, TypeRef},
    validation::validate_new_pokemon,
};

async fn load_detail(pool: &PgPool, pokemon: Pokemon) -> Result<PokemonDetail, AppError> {
    let types = sqlx::query_as::<_, TypeRef>(
        "SELECT t.id, t.name, pt.slot
         FROM pokemon_types pt JOIN types t ON t.id = pt.type_id
         WHERE pt.pokemon_id = $1 ORDER BY pt.slot",
    )
    .bind(pokemon.id)
    .fetch_all(pool)
    .await?;

    let abilities = sqlx::query_as::<_, AbilityRef>(
        "SELECT a.id, a.name, pa.is_hidden
         FROM pokemon_abilities pa JOIN abilities a ON a.id = pa.ability_id
         WHERE pa.pokemon_id = $1 ORDER BY pa.is_hidden, a.id",
    )
    .bind(pokemon.id)
    .fetch_all(pool)
    .await?;

    let stats = sqlx::query_as::<_, StatRef>(
        "SELECT st.name, ps.base_value
         FROM pokemon_stats ps JOIN stat_types st ON st.id = ps.stat_type_id
         WHERE ps.pokemon_id = $1 ORDER BY st.id",
    )
    .bind(pokemon.id)
    .fetch_all(pool)
    .await?;

    Ok(PokemonDetail { pokemon, types, abilities, stats })
}

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
) -> Result<Json<PokemonDetail>, AppError> {
    let pokemon = sqlx::query_as::<_, Pokemon>(
        "SELECT id, name, height, weight, base_experience, sprite_url
         FROM pokemon WHERE id = $1",
    )
    .bind(id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("pokemon {id} no encontrado")))?;

    Ok(Json(load_detail(&pool, pokemon).await?))
}

pub async fn get_pokemon_by_name(
    State(pool): State<PgPool>,
    Path(name): Path<String>,
) -> Result<Json<PokemonDetail>, AppError> {
    let pokemon = sqlx::query_as::<_, Pokemon>(
        "SELECT id, name, height, weight, base_experience, sprite_url
         FROM pokemon WHERE LOWER(name) = LOWER($1)",
    )
    .bind(&name)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::NotFound(format!("pokemon '{name}' no encontrado")))?;

    Ok(Json(load_detail(&pool, pokemon).await?))
}

pub async fn list_types(State(pool): State<PgPool>) -> Result<Json<Vec<TypeItem>>, AppError> {
    let rows = sqlx::query_as::<_, TypeItem>("SELECT id, name FROM types ORDER BY id")
        .fetch_all(&pool)
        .await?;

    Ok(Json(rows))
}

pub async fn list_pokemon_by_type(
    State(pool): State<PgPool>,
    Path(id): Path<i32>,
) -> Result<Json<Vec<Pokemon>>, AppError> {
    let exists = sqlx::query_scalar::<_, bool>("SELECT EXISTS (SELECT 1 FROM types WHERE id = $1)")
        .bind(id)
        .fetch_one(&pool)
        .await?;

    if !exists {
        return Err(AppError::NotFound(format!("tipo {id} no encontrado")));
    }

    let rows = sqlx::query_as::<_, Pokemon>(
        "SELECT p.id, p.name, p.height, p.weight, p.base_experience, p.sprite_url
         FROM pokemon p JOIN pokemon_types pt ON pt.pokemon_id = p.id
         WHERE pt.type_id = $1 ORDER BY p.id",
    )
    .bind(id)
    .fetch_all(&pool)
    .await?;

    Ok(Json(rows))
}

pub async fn create_pokemon(
    State(pool): State<PgPool>,
    payload: Result<Json<NewPokemon>, JsonRejection>,
) -> Result<(StatusCode, Json<PokemonDetail>), AppError> {
    let Json(body) = payload.map_err(|e| AppError::BadRequest(e.body_text()))?;
    let input = validate_new_pokemon(body)?;

    let exists = sqlx::query_scalar::<_, bool>(
        "SELECT EXISTS (SELECT 1 FROM pokemon WHERE LOWER(name) = $1)",
    )
    .bind(&input.name)
    .fetch_one(&pool)
    .await?;

    if exists {
        return Err(AppError::Conflict(format!(
            "ya existe un pokemon llamado '{}'",
            input.name
        )));
    }

    let mut tx = pool.begin().await?;

    let pokemon = sqlx::query_as::<_, Pokemon>(
        "INSERT INTO pokemon (name, height, weight, base_experience, sprite_url)
         VALUES ($1, $2, $3, $4, $5)
         RETURNING id, name, height, weight, base_experience, sprite_url",
    )
    .bind(&input.name)
    .bind(input.height)
    .bind(input.weight)
    .bind(input.base_experience)
    .bind(&input.sprite_url)
    .fetch_one(&mut *tx)
    .await?;

    for (index, type_name) in input.types.iter().enumerate() {
        let type_id = sqlx::query_scalar::<_, i32>("SELECT id FROM types WHERE name = $1")
            .bind(type_name)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::BadRequest(format!("el tipo '{type_name}' no existe")))?;

        sqlx::query("INSERT INTO pokemon_types (pokemon_id, type_id, slot) VALUES ($1, $2, $3)")
            .bind(pokemon.id)
            .bind(type_id)
            .bind(index as i32 + 1)
            .execute(&mut *tx)
            .await?;
    }

    for ability in &input.abilities {
        let ability_id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO abilities (name) VALUES ($1)
             ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
             RETURNING id",
        )
        .bind(&ability.name)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO pokemon_abilities (pokemon_id, ability_id, is_hidden) VALUES ($1, $2, $3)",
        )
        .bind(pokemon.id)
        .bind(ability_id)
        .bind(ability.is_hidden)
        .execute(&mut *tx)
        .await?;
    }

    for stat in &input.stats {
        let stat_id = sqlx::query_scalar::<_, i32>("SELECT id FROM stat_types WHERE name = $1")
            .bind(&stat.name)
            .fetch_optional(&mut *tx)
            .await?
            .ok_or_else(|| AppError::BadRequest(format!("el stat '{}' no existe", stat.name)))?;

        sqlx::query(
            "INSERT INTO pokemon_stats (pokemon_id, stat_type_id, base_value) VALUES ($1, $2, $3)",
        )
        .bind(pokemon.id)
        .bind(stat_id)
        .bind(stat.base_value)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await?;

    let detail = load_detail(&pool, pokemon).await?;
    Ok((StatusCode::CREATED, Json(detail)))
}