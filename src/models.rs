use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Serialize, FromRow)]
pub struct Pokemon {
    pub id: i32,
    pub name: String,
    pub height: i32,
    pub weight: i32,
    pub base_experience: Option<i32>,
    pub sprite_url: Option<String>,
}

#[derive(Serialize, FromRow)]
pub struct TypeItem {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize, FromRow)]
pub struct TypeRef {
    pub id: i32,
    pub name: String,
    pub slot: i32,
}

#[derive(Serialize, FromRow)]
pub struct AbilityRef {
    pub id: i32,
    pub name: String,
    pub is_hidden: bool,
}

#[derive(Serialize, FromRow)]
pub struct StatRef {
    pub name: String,
    pub base_value: i32,
}

#[derive(Serialize)]
pub struct PokemonDetail {
    #[serde(flatten)]
    pub pokemon: Pokemon,
    pub types: Vec<TypeRef>,
    pub abilities: Vec<AbilityRef>,
    pub stats: Vec<StatRef>,
}

#[derive(Deserialize)]
pub struct NewAbility {
    pub name: String,
    #[serde(default)]
    pub is_hidden: bool,
}

#[derive(Deserialize)]
pub struct NewStat {
    pub name: String,
    pub base_value: i32,
}

#[derive(Deserialize)]
pub struct NewPokemon {
    pub name: String,
    pub height: i32,
    pub weight: i32,
    pub base_experience: Option<i32>,
    pub sprite_url: Option<String>,
    pub types: Vec<String>,
    #[serde(default)]
    pub abilities: Vec<NewAbility>,
    #[serde(default)]
    pub stats: Vec<NewStat>,
}

#[derive(Deserialize)]
pub struct ListParams {
    pub name: Option<String>,
    #[serde(rename = "type")]
    pub type_name: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

#[derive(Serialize)]
pub struct Page<T> {
    pub data: Vec<T>,
    pub total: i64,
    pub limit: i64,
    pub offset: i64,
}
