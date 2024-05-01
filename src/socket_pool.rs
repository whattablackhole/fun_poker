use std::{collections::HashMap, net::TcpStream, sync::Mutex};

use tungstenite::{Message, WebSocket};

pub struct LobbySocketPool {
    pool: Mutex<HashMap<String, Vec<WebSocket<TcpStream>>>>,
}

impl LobbySocketPool {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(HashMap::new()),
        }
    }

    pub fn add(&self, lobby_id: String, v: WebSocket<TcpStream>) {
        let mut pool = self.pool.lock().unwrap();
        if let Some(entry) = pool.get_mut(&lobby_id) {
            entry.push(v);
        } else {
            let mut new_list = Vec::new();
            new_list.push(v);
            pool.insert(lobby_id, new_list);
        }
    }

    pub fn send_message_to_all(&self, message: String) {
        let mes = Message::text(message);
        let mut pool = self.pool.lock().unwrap();
        for (_, sockets) in pool.iter_mut() {
            for socket in sockets.into_iter() {
                if let Err(e) = socket.send(mes.clone()) {
                    eprintln!("Failed to send message: {}", e);
                }
            }
        }
    }
}
