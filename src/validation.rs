use std::collections::HashSet;

use crate::{error::AppError, models::NewPokemon};

fn bad(message: &str) -> AppError {
    AppError::BadRequest(message.to_string())
}

fn has_duplicates<'a>(mut items: impl Iterator<Item = &'a String>) -> bool {
    let mut seen = HashSet::new();
    items.any(|item| !seen.insert(item))
}

pub fn validate_new_pokemon(mut input: NewPokemon) -> Result<NewPokemon, AppError> {
    input.name = input.name.trim().to_lowercase();
    if input.name.is_empty() || input.name.len() > 100 {
        return Err(bad("name debe tener entre 1 y 100 caracteres"));
    }

    if input.height < 0 || input.weight < 0 {
        return Err(bad("height y weight no pueden ser negativos"));
    }

    if matches!(input.base_experience, Some(value) if value < 0) {
        return Err(bad("base_experience no puede ser negativo"));
    }

    input.types = input
        .types
        .iter()
        .map(|t| t.trim().to_lowercase())
        .collect();
    if input.types.is_empty() || input.types.len() > 2 {
        return Err(bad("un pokemon debe tener 1 o 2 tipos"));
    }
    if has_duplicates(input.types.iter()) {
        return Err(bad("los tipos no pueden repetirse"));
    }

    for ability in &mut input.abilities {
        ability.name = ability.name.trim().to_lowercase();
        if ability.name.is_empty() {
            return Err(bad("el nombre de una habilidad no puede estar vacío"));
        }
    }
    if has_duplicates(input.abilities.iter().map(|a| &a.name)) {
        return Err(bad("las habilidades no pueden repetirse"));
    }

    for stat in &mut input.stats {
        stat.name = stat.name.trim().to_lowercase();
        if stat.base_value < 0 {
            return Err(bad("base_value de un stat no puede ser negativo"));
        }
    }
    if has_duplicates(input.stats.iter().map(|s| &s.name)) {
        return Err(bad("los stats no pueden repetirse"));
    }

    Ok(input)
}
