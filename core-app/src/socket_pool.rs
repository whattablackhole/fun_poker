use std::{
    collections::HashMap,
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
    time::Duration,
};

use tungstenite::{Error as TError, Message as TMessage, WebSocket};

use crate::{
    protos::responses::ResponseMessage,
    responses::{EncodableMessage, TMessageResponse},
    thread_pool::ThreadPool,
};

pub struct PlayerChannelClient {
    pub client_id: i32,
    pub socket: WebSocket<TcpStream>,
}
pub struct SocketPool {
    pool: Arc<Mutex<HashMap<i32, WebSocket<TcpStream>>>>,
    thread_pool: ThreadPool,
}

pub enum ReadMessageError {
    Iddle,
    Disconnected,
}

impl SocketPool {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(Mutex::new(HashMap::new())),
            thread_pool: ThreadPool::new(1),
        }
    }

    pub fn add(&self, v: PlayerChannelClient) {
        let mut pool = self.pool.lock().unwrap();
        pool.insert(v.client_id, v.socket);

        println!("LENGTH: {}", pool.len())
    }

    // pub fn get_active_client_ids_by_lobby_id(&self, lobby_id: i32) -> Vec<i32> {
    //     let mut pool = self.pool.lock().unwrap();
    //     let result = pool.get_mut(&lobby_id).unwrap();

    //     let mut ids = Vec::new();

    //     for channel in result {
    //         ids.push(channel.client_id);
    //     }
    //     ids
    // }
    // pub fn send_message_to_all(&self, message: String) {
    //     let mes = TMessage::text(message);
    //     let mut pool = self.pool.lock().unwrap();
    //     for (_, clients) in pool.iter_mut() {
    //         for client in clients.into_iter() {
    //             if let Err(e) = client.socket.send(mes.clone()) {
    //                 eprintln!("Failed to send message: {}", e);
    //             }
    //         }
    //     }
    // }

    pub fn read_client_message<T: prost::Message + Default>(
        &self,
        client_id: i32,
    ) -> Result<T, ReadMessageError> {
        let arc_pool = Arc::clone(&self.pool);
        let (tx, rx) = mpsc::channel();

        self.thread_pool.execute(move || {
            let mut client_channels: std::sync::MutexGuard<HashMap<i32, WebSocket<TcpStream>>> =
                arc_pool.lock().unwrap();
            let socket = client_channels.get_mut(&client_id).unwrap();

            let result: Result<TMessage, TError> = socket.read();
            tx.send(result).unwrap();
        });

        let result = rx.recv_timeout(Duration::from_secs(30));

        let message = match result {
            Err(_e) => return Err(ReadMessageError::Iddle),
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

    pub fn close_connection(&self, client_id: i32) {
        let mut client_channels = self.pool.lock().unwrap();
        let mut socket: WebSocket<TcpStream> = client_channels.remove(&client_id).unwrap();
        socket.close(None).unwrap();
    }

    pub fn update_clients(&self, responses: Vec<TMessageResponse>) {
        let mut client_channels = self.pool.lock().unwrap();

        for response in responses {
            let response_message = ResponseMessage {
                payload: response.message.encode_message(),
                payload_type: response.message_type.into(),
            };

            let socket = client_channels.get_mut(&response.receiver_id).unwrap();

            let encoded: Vec<u8> = response_message.encode_message();
            socket.send(TMessage::Binary(encoded)).unwrap();
        }
    }
}
