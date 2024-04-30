use std::sync::Mutex;
use postgres::{Client, NoTls};

use crate::lobby::Lobby;
use crate::lobby::LobbyList;

pub struct PostgresDatabase {
    client: Mutex<Client>,
}


impl PostgresDatabase {
    pub fn new() -> Result<PostgresDatabase, postgres::Error> {
        let client = Client::connect("postgres://postgres:1337@localhost/FunPokerDB", NoTls)?;

        Ok(PostgresDatabase { client: Mutex::new(client) })
    }
    
    pub fn get_lobbies(&self) -> LobbyList {
        let rows = self.client.lock().unwrap().query("SELECT * from lobbies",&[]).unwrap();
        
        let mut lobbies: Vec<Lobby> = Vec::new();

        for row in rows {
            let lobby_id: i32 = row.get(0);
            let name: String = row.get(1);
            let author_id: i32 = row.get(2);
            lobbies.push(Lobby{id: lobby_id, name, author_id});
        }

        LobbyList{list: lobbies}
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
    CREATE TABLE IF NOT EXISTS lobbies  (
        id              SERIAL PRIMARY KEY,
        name           VARCHAR NOT NULL,
        author_id       INTEGER NOT NULL REFERENCES users
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
