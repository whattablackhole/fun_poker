use std::{collections::HashMap, net::TcpStream, sync::Mutex};

use prost::Message;
use tungstenite::{Message as TMessage, WebSocket};

use crate::{
    dealer::{ClientGameState, Dealer},
    protos::{
        client_state::{ClientState, Street},
        player::Player,
    },
};

pub struct PlayerChannelClient {
    pub player_id: i32,
    pub socket: WebSocket<TcpStream>,
}
pub struct LobbySocketPool {
    pool: Mutex<HashMap<i32, Vec<PlayerChannelClient>>>,
}

pub struct DealerPool {
    // TODO:
    // In future we have to think about how to implement mapping dealers to tables and tables to lobbies
    // In current implementation all action happens on a single table on a single lobby
    // TODO: improve method readability, scalability
    pool: Mutex<HashMap<String, Vec<Dealer>>>,
}

impl DealerPool {
    pub fn new() -> Self {
        DealerPool {
            pool: Mutex::new(HashMap::new()),
        }
    }

    pub fn add(&self, lobby_id: String, v: Dealer) {
        let mut pool = self.pool.lock().unwrap();
        if let Some(entry) = pool.get_mut(&lobby_id) {
            entry.push(v);
        } else {
            let mut new_list = Vec::new();
            new_list.push(v);
            pool.insert(lobby_id, new_list);
        }
    }
}

impl LobbySocketPool {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(HashMap::new()),
        }
    }

    pub fn add(&self, lobby_id: i32, v: PlayerChannelClient) {
        let mut pool = self.pool.lock().unwrap();
        if let Some(entry) = pool.get_mut(&lobby_id) {
            entry.push(v);
        } else {
            let mut new_list = Vec::new();
            new_list.push(v);
            pool.insert(lobby_id, new_list);
        }
    }
    pub fn get_active_player_ids_by_lobby_id(&self, lobby_id: i32) -> Vec<i32> {
        let mut pool = self.pool.lock().unwrap();
        let result = pool.get_mut(&lobby_id).unwrap();

        let mut ids = Vec::new();

        for channel in result {
            ids.push(channel.player_id);
        }
        ids
    }
    pub fn send_message_to_all(&self, message: String) {
        let mes = TMessage::text(message);
        let mut pool = self.pool.lock().unwrap();
        for (_, clients) in pool.iter_mut() {
            for client in clients.into_iter() {
                if let Err(e) = client.socket.send(mes.clone()) {
                    eprintln!("Failed to send message: {}", e);
                }
            }
        }
    }
    pub fn read_client_message<T: prost::Message + Default>(
        &self,
        player_id: i32,
        lobby_id: i32,
    ) -> T {
        let mut pool = self.pool.lock().unwrap();
        let result = pool.get_mut(&lobby_id).unwrap();
        let client = result
            .iter_mut()
            .find(|c| c.player_id == player_id)
            .unwrap();

        let msg = client.socket.read().unwrap();

        let bytes = match msg {
            TMessage::Binary(bytes) => bytes,
            _ => panic!("Expected binary message"),
        };

        let mut reader = std::io::Cursor::new(bytes);

        let request = T::decode(&mut reader).unwrap();
        request
    }
    pub fn update_clients(&self, state: ClientGameState) {
        let mut guard = self.pool.lock().unwrap();
        if let Some(clients) = guard.get_mut(&state.lobby_id) {
            for client in clients {
                let player = state
                    .players
                    .iter()
                    .find(|p| p.user_id == client.player_id)
                    .expect("Failed to find PlayOutload for client's player_id");
                let pl: Vec<Player> = state
                    .players
                    .clone()
                    .iter_mut()
                    .map(|p| {
                        let mut cloned_player = p.clone();
                        cloned_player.cards = None;
                        cloned_player
                    })
                    .collect();
                let client_state = ClientState {
                    players: pl,
                    player_id: client.player_id,
                    // send cards in another msg
                    cards: player.cards.clone(),
                    next_player_id: state.next_player_id,
                    lobby_id: state.lobby_id,
                    street: Some(Street {
                        street_status: state.street.street_status,
                        cards: state.street.cards.clone(),
                    }),
                    game_status: state.game_status as i32,
                    latest_winners: state
                        .latest_winners
                        .clone()
                        .iter_mut()
                        .map(|p| {
                            // there are many cases, where we might need to show enemies cards
                            // TODO: evaluate these scenarios and handle in a proper way
                            let mut cloned_player = p.clone();
                            cloned_player.cards = None;
                            cloned_player
                        })
                        .collect(),
                };

                let mut buf = Vec::new();
                client_state.encode(&mut buf).unwrap();
                client.socket.send(TMessage::binary(buf)).unwrap();
            }
        } else {
            // TODO: sync UpdatedGameState with ClientState and GameState structs
            // it has to be simplified in a single struct
        }
    }
}
