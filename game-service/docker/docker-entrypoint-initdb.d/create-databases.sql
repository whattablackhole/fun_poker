CREATE DATABASE fun_poker_game_service_db;

CREATE ROLE rudolf WITH LOGIN SUPERUSER PASSWORD '1337';

\c fun_poker_game_service_db;

CREATE TYPE game_name_enum AS ENUM ('Holdem');
CREATE TYPE game_type_enum AS ENUM ('Tournament', 'Cash');

CREATE TABLE IF NOT EXISTS lobbies (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    author_id INTEGER NOT NULL,
    game_name game_name_enum NOT NULL,
    game_type game_type_enum NOT NULL
);

CREATE TABLE IF NOT EXISTS players_lobbies (
    player_id INTEGER NOT NULL,
    lobby_id INTEGER NOT NULL REFERENCES lobbies(id),
    PRIMARY KEY (player_id, lobby_id)
);