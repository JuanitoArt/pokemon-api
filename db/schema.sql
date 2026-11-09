CREATE TABLE types (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE
);

CREATE TABLE abilities (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE
);

CREATE TABLE stat_types (
    id SERIAL PRIMARY KEY,
    name VARCHAR(50) NOT NULL UNIQUE
);

CREATE TABLE pokemon (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    height INTEGER NOT NULL CHECK (height >= 0),
    weight INTEGER NOT NULL CHECK (weight >= 0),
    base_experience INTEGER,
    sprite_url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE UNIQUE INDEX pokemon_name_lower_idx ON pokemon (LOWER(name));

CREATE TABLE pokemon_types (
    pokemon_id INTEGER NOT NULL REFERENCES pokemon(id) ON DELETE CASCADE,
    type_id INTEGER NOT NULL REFERENCES types(id) ON DELETE CASCADE,
    slot INTEGER NOT NULL,
    PRIMARY KEY (pokemon_id, type_id),
    UNIQUE (pokemon_id, slot)
);

CREATE TABLE pokemon_abilities (
    pokemon_id INTEGER NOT NULL REFERENCES pokemon(id) ON DELETE CASCADE,
    ability_id INTEGER NOT NULL REFERENCES abilities(id) ON DELETE CASCADE,
    is_hidden BOOLEAN NOT NULL DEFAULT false,
    PRIMARY KEY (pokemon_id, ability_id)
);

CREATE TABLE pokemon_stats (
    pokemon_id INTEGER NOT NULL REFERENCES pokemon(id) ON DELETE CASCADE,
    stat_type_id INTEGER NOT NULL REFERENCES stat_types(id) ON DELETE CASCADE,
    base_value INTEGER NOT NULL CHECK (base_value >= 0),
    PRIMARY KEY (pokemon_id, stat_type_id)
);

CREATE INDEX pokemon_types_type_idx ON pokemon_types (type_id);

INSERT INTO stat_types (name) VALUES
    ('hp'), ('attack'), ('defense'),
    ('special-attack'), ('special-defense'), ('speed');