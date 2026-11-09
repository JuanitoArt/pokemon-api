use std::time::Duration;

use serde::Deserialize;
use sqlx::{postgres::PgPoolOptions, PgPool};

#[derive(Deserialize)]
struct Named {
    name: String,
}

#[derive(Deserialize)]
struct Sprites {
    front_default: Option<String>,
}

#[derive(Deserialize)]
struct ApiType {
    slot: i32,
    #[serde(rename = "type")]
    kind: Named,
}

#[derive(Deserialize)]
struct ApiAbility {
    is_hidden: bool,
    ability: Named,
}

#[derive(Deserialize)]
struct ApiStat {
    base_stat: i32,
    stat: Named,
}

#[derive(Deserialize)]
struct ApiPokemon {
    id: i32,
    name: String,
    height: i32,
    weight: i32,
    base_experience: Option<i32>,
    sprites: Sprites,
    types: Vec<ApiType>,
    abilities: Vec<ApiAbility>,
    stats: Vec<ApiStat>,
}

async fn fetch(client: &reqwest::Client, id: i32) -> Result<ApiPokemon, reqwest::Error> {
    client
        .get(format!("https://pokeapi.co/api/v2/pokemon/{id}"))
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
}

async fn fetch_with_retry(client: &reqwest::Client, id: i32) -> Result<ApiPokemon, reqwest::Error> {
    let mut attempt = 1;
    loop {
        match fetch(client, id).await {
            Ok(pokemon) => return Ok(pokemon),
            Err(err) if attempt >= 3 => return Err(err),
            Err(_) => {
                attempt += 1;
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
}

async fn save(pool: &PgPool, p: &ApiPokemon) -> Result<(), sqlx::Error> {
    let mut tx = pool.begin().await?;

    sqlx::query(
        "INSERT INTO pokemon (id, name, height, weight, base_experience, sprite_url)
         VALUES ($1, $2, $3, $4, $5, $6)
         ON CONFLICT (id) DO UPDATE SET
             name = EXCLUDED.name,
             height = EXCLUDED.height,
             weight = EXCLUDED.weight,
             base_experience = EXCLUDED.base_experience,
             sprite_url = EXCLUDED.sprite_url,
             updated_at = now()",
    )
    .bind(p.id)
    .bind(&p.name)
    .bind(p.height)
    .bind(p.weight)
    .bind(p.base_experience)
    .bind(&p.sprites.front_default)
    .execute(&mut *tx)
    .await?;

    sqlx::query("DELETE FROM pokemon_types WHERE pokemon_id = $1")
        .bind(p.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM pokemon_abilities WHERE pokemon_id = $1")
        .bind(p.id)
        .execute(&mut *tx)
        .await?;
    sqlx::query("DELETE FROM pokemon_stats WHERE pokemon_id = $1")
        .bind(p.id)
        .execute(&mut *tx)
        .await?;

    for t in &p.types {
        let type_id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO types (name) VALUES ($1)
             ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
             RETURNING id",
        )
        .bind(&t.kind.name)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO pokemon_types (pokemon_id, type_id, slot) VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
        )
        .bind(p.id)
        .bind(type_id)
        .bind(t.slot)
        .execute(&mut *tx)
        .await?;
    }

    for a in &p.abilities {
        let ability_id = sqlx::query_scalar::<_, i32>(
            "INSERT INTO abilities (name) VALUES ($1)
             ON CONFLICT (name) DO UPDATE SET name = EXCLUDED.name
             RETURNING id",
        )
        .bind(&a.ability.name)
        .fetch_one(&mut *tx)
        .await?;

        sqlx::query(
            "INSERT INTO pokemon_abilities (pokemon_id, ability_id, is_hidden) VALUES ($1, $2, $3)
             ON CONFLICT DO NOTHING",
        )
        .bind(p.id)
        .bind(ability_id)
        .bind(a.is_hidden)
        .execute(&mut *tx)
        .await?;
    }

    for s in &p.stats {
        sqlx::query(
            "INSERT INTO pokemon_stats (pokemon_id, stat_type_id, base_value)
             SELECT $1, id, $3 FROM stat_types WHERE name = $2
             ON CONFLICT DO NOTHING",
        )
        .bind(p.id)
        .bind(&s.stat.name)
        .bind(s.base_stat)
        .execute(&mut *tx)
        .await?;
    }

    tx.commit().await
}

#[tokio::main]
async fn main() {
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL").expect("DATABASE_URL no está definida");
    let total: i32 = std::env::args()
        .nth(1)
        .and_then(|arg| arg.parse().ok())
        .unwrap_or(151);

    let pool = PgPoolOptions::new()
        .max_connections(2)
        .connect(&database_url)
        .await
        .expect("no se pudo conectar a PostgreSQL");

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(20))
        .build()
        .unwrap();

    let mut saved = 0;
    let mut failed = Vec::new();

    for id in 1..=total {
        match fetch_with_retry(&client, id).await {
            Ok(pokemon) => match save(&pool, &pokemon).await {
                Ok(()) => saved += 1,
                Err(err) => {
                    eprintln!("no se pudo guardar el pokemon {id}: {err}");
                    failed.push(id);
                }
            },
            Err(err) => {
                eprintln!("no se pudo descargar el pokemon {id}: {err}");
                failed.push(id);
            }
        }

        if id % 50 == 0 {
            println!("{id}/{total}");
        }
    }

    sqlx::query("SELECT setval(pg_get_serial_sequence('pokemon', 'id'), (SELECT MAX(id) FROM pokemon))")
        .execute(&pool)
        .await
        .expect("no se pudo ajustar la secuencia");

    println!("guardados: {saved}, fallidos: {}", failed.len());
    if !failed.is_empty() {
        println!("ids fallidos: {failed:?}");
    }
}