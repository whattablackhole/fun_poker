use postgres::types::{to_sql_checked, FromSql, IsNull, ToSql, Type};
use postgres::{Client, NoTls};
use prost::bytes::{Buf, BytesMut};
use std::io::BufRead;
use std::sync::Mutex;

use crate::protos::lobby::{GameName, GameType, Lobby, LobbyList};
use crate::protos::user::User;

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

impl ToSql for GameType {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            GameType::Tournament => "Tournament".to_sql(ty, out)?,
            GameType::Cash => "Cash".to_sql(ty, out)?,

            _ => return Ok(IsNull::Yes),
        };

        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "game_type_enum"
    }

    to_sql_checked!();
}

impl ToSql for GameName {
    fn to_sql(
        &self,
        ty: &Type,
        out: &mut BytesMut,
    ) -> Result<IsNull, Box<dyn std::error::Error + Sync + Send>> {
        match self {
            GameName::Holdem => "Holdem".to_sql(ty, out)?,
            _ => return Ok(IsNull::Yes),
        };

        Ok(IsNull::No)
    }

    fn accepts(ty: &Type) -> bool {
        ty.name() == "game_name_enum"
    }

    to_sql_checked!();
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

    pub fn get_users_by_lobby_id(&self, lobby_id: i32) -> Vec<User> {
        let mut guard = self.client.lock().unwrap();

        let query = "SELECT * FROM users WHERE id IN (SELECT player_id FROM players_lobbies WHERE lobby_id = $1)";

        let rows = guard.query(query, &[&lobby_id]).unwrap();

        let mut users = Vec::new();

        for row in rows {
            let id: i32 = row.get("id");
            let name: String = row.get("name");
            let country: String = row.get("country");
            let email: String = row.get("email");

            users.push(User {
                id,
                name,
                country,
                email,
            });
        }
        users
    }

    pub fn create_lobby(&self, lobby: Lobby) -> i32 {
        let mut guard = self.client.lock().unwrap();

        let query = "INSERT INTO lobbies(name, author_id, players_registered, game_name, game_type) VALUES($1, $2, $3, $4, $5) RETURNING id";
        
        let row = guard
            .query_one(
                query,
                &[
                    &lobby.name,
                    &lobby.author_id,
                    &lobby.players_registered,
                    &lobby.game_name(),
                    &lobby.game_type(),
                ],
            )
            .unwrap();

        let lobby_id: i32 = row.get(0);

        lobby_id
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
                id: Some(lobby_id),
                name,
                author_id,
                game_type: game_type.into(),
                game_name: game_name.into(),
                players_registered,
            });
        }

        LobbyList { list: lobbies }
    }

    pub fn get_user_by_id(&self, user_id: i32) -> User {
        let mut client_lock = self.client.lock().unwrap();

        let query = "SELECT * FROM users WHERE id = $1";

        let row = client_lock.query_one(query, &[&user_id]).unwrap();

        let id: i32 = row.get("id");
        let name: String = row.get("name");
        let country: String = row.get("country");
        let email: String = row.get("email");

        User {
            id,
            name,
            country,
            email,
        }
    }

    pub fn add_user_to_lobby(&self, lobby_id: i32, user_id: i32) {
        let mut client_lock = self.client.lock().unwrap();
        let query = "INSERT INTO players_lobbies (player_id, lobby_id) VALUES ($1, $2)";
        client_lock.execute(query, &[&user_id, &lobby_id]).unwrap();
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
