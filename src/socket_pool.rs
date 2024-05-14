use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};

use prost::Message;
use tungstenite::{Error as TError, Message as TMessage, WebSocket};

use crate::{dealer::Dealer, protos::client_state::ClientState, ThreadPool};

pub struct PlayerChannelClient {
    pub player_id: i32,
    pub socket: WebSocket<TcpStream>,
}
pub struct LobbySocketPool {
    pool: Arc<Mutex<HashMap<i32, Vec<PlayerChannelClient>>>>,
    thread_pool: ThreadPool,
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

pub enum ReadMessageError {
    Iddle,
    Disconnected,
}

impl LobbySocketPool {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(Mutex::new(HashMap::new())),
            thread_pool: ThreadPool::new(1),
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
    ) -> Result<T, ReadMessageError> {
        let arc_pool = Arc::clone(&self.pool);
        let (tx, rx) = mpsc::channel();

        self.thread_pool.execute(move || {
            let mut g = arc_pool.lock().unwrap();
            let player_channels = g.get_mut(&lobby_id).unwrap();
            let client = player_channels
                .iter_mut()
                .find(|c| c.player_id == player_id)
                .unwrap();
            let result: Result<TMessage, TError> = client.socket.read();
            tx.send(result).unwrap();
        });

        let result = rx.recv_timeout(Duration::from_secs(30));

        let message = match result {
            Err(e) => return Err(ReadMessageError::Iddle),
            Ok(r) => match r {
                Ok(m) => match m {
                    TMessage::Binary(_) => m,
                    TMessage::Text(_) => todo!(),
                    TMessage::Ping(_) => todo!(),
                    TMessage::Pong(_) => todo!(),
                    TMessage::Close(_) => return Err(ReadMessageError::Disconnected),
                    TMessage::Frame(_) => todo!(),
                },
                Err(e) => match e {
                    TError::ConnectionClosed => return Err(ReadMessageError::Disconnected),
                    TError::AlreadyClosed => return Err(ReadMessageError::Disconnected),
                    TError::Io(_) => todo!(),
                    TError::Tls(_) => todo!(),
                    TError::Capacity(_) => todo!(),
                    TError::Protocol(_) => todo!(),
                    TError::WriteBufferFull(_) => todo!(),
                    TError::Utf8 => todo!(),
                    TError::AttackAttempt => todo!(),
                    TError::Url(_) => todo!(),
                    TError::Http(_) => todo!(),
                    TError::HttpFormat(_) => todo!(),
                },
            },
        };

        let bytes = match message {
            TMessage::Binary(bytes) => bytes,
            _ => panic!("Expected binary message"),
        };

        let mut reader = std::io::Cursor::new(bytes);

        let request = T::decode(&mut reader).unwrap();
        Ok(request)
    }

    pub fn close_connection(&self, player_id: i32, lobby_id: i32) {
        let mut guard = self.pool.lock().unwrap();
        let player_channels = guard.get_mut(&lobby_id).unwrap();
        let index = player_channels
            .iter()
            .position(|c| c.player_id == player_id)
            .unwrap();
        let mut client = player_channels.remove(index);
        client.socket.close(None).unwrap();
    }

    pub fn update_clients(&self, states: Vec<ClientState>, lobby_id: i32) {
        let mut guard = self.pool.lock().unwrap();
        if let Some(clients) = guard.get_mut(&lobby_id) {
            for client in clients {
                let state = states
                    .iter()
                    .find(|s| s.player_id == client.player_id)
                    .expect("Failed to find the client state in the pool");
                let mut buf = Vec::new();
                state.encode(&mut buf).unwrap();
                client.socket.send(TMessage::binary(buf)).unwrap();
            }
        }
    }
}
