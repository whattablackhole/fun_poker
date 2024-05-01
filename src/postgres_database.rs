use postgres::types::{FromSql, Type};
use postgres::{Client, NoTls};
use prost::bytes::Buf;
use std::io::BufRead;
use std::sync::Mutex;

use crate::lobby::GameName;
use crate::lobby::GameType;
use crate::lobby::Lobby;
use crate::lobby::LobbyList;

pub struct PostgresDatabase {
    client: Mutex<Client>,
}

impl FromSql<'_> for GameName {
    fn from_sql<'a>(
        _: &'a Type,
        buf: &[u8],
    ) -> Result<Self, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let reader = buf.reader();

        let string_value = reader.lines().next().unwrap().unwrap();
        let value = GameName::from_str_name(&string_value).unwrap();
        Ok(value)
    }
    fn accepts(sql_type: &Type) -> bool {
        sql_type.name() == "game_name_enum"
    }
}

impl FromSql<'_> for GameType {
    fn from_sql<'a>(
        _: &'a Type,
        buf: &[u8],
    ) -> Result<Self, Box<(dyn std::error::Error + Send + Sync + 'static)>> {
        let reader = buf.reader();

        let string_value = reader.lines().next().unwrap().unwrap();
        let value: GameType = GameType::from_str_name(&string_value).unwrap();
        Ok(value)
    }
    fn accepts(sql_type: &Type) -> bool {
        sql_type.name() == "game_type_enum"
    }
}

impl PostgresDatabase {
    pub fn new() -> Result<PostgresDatabase, postgres::Error> {
        let client = Client::connect("postgres://postgres:1337@localhost/FunPokerDB", NoTls)?;

        Ok(PostgresDatabase {
            client: Mutex::new(client),
        })
    }

    pub fn get_lobbies(&self) -> LobbyList {
        let rows = self
            .client
            .lock()
            .unwrap()
            .query("SELECT * from lobbies", &[])
            .unwrap();

        let mut lobbies: Vec<Lobby> = Vec::new();

        for row in rows {
            let lobby_id: i32 = row.get("id");
            let name: String = row.get("name");
            let author_id: i32 = row.get("author_id");
            let game_type: GameType = row.get("game_type");
            let game_name: GameName = row.get("game_name");
            let players_registered: i32 = row.get("players_registered");

            lobbies.push(Lobby {
                id: lobby_id,
                name,
                author_id,
                game_type: game_type.into(),
                game_name: game_name.into(),
                players_registered,
            });
        }

        LobbyList { list: lobbies }
    }

    pub fn init(&self) -> Result<(), postgres::Error> {
        let mut client_lock = self.client.lock().unwrap();

        client_lock.batch_execute(
            "
        CREATE TABLE IF NOT EXISTS users (
            id              SERIAL PRIMARY KEY,
            name            VARCHAR NOT NULL,
            country         VARCHAR NOT NULL,
            email           VARCHAR(100) CHECK (email ~* '^.+@.+$')           
            )
    ",
        )?;

        client_lock.batch_execute(
            "
      CREATE TYPE game_name_enum AS ENUM ('Holdem');  
      CREATE TYPE game_type_enum AS ENUM ('Tournament', 'Cash');      
      CREATE TABLE IF NOT EXISTS lobbies  (
        id              SERIAL PRIMARY KEY,
        name           VARCHAR NOT NULL,
        author_id       INTEGER NOT NULL REFERENCES users,
        players_registered INTEGER NOT NULL,
        game_name game_name_enum NOT NULL,
        game_type game_type_enum NOT NULL
        )  
",
        )?;

        client_lock.batch_execute(
            "
    CREATE TABLE IF NOT EXISTS players_lobbies (
        player_lobby_id SERIAL PRIMARY KEY,
        player_id INTEGER NOT NULL REFERENCES users(id),
        lobby_id INTEGER NOT NULL REFERENCES lobbies(id)
        )
",
        )?;

        Ok(())
    }
}
