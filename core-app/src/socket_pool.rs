use std::{
    collections::HashMap,
    io::ErrorKind,
    net::TcpStream,
    sync::{Arc, Mutex, RwLock},
    time::{Duration, SystemTime},
};

use tungstenite::{Error as TError, Message as TMessage, WebSocket};

use crate::{
    protos::responses::ResponseMessage,
    responses::{EncodableMessage, TMessageResponse},
};

pub struct PlayerChannelClient {
    pub client_id: i32,
    pub socket: WebSocket<TcpStream>,
}
pub struct SocketPool {
    pool: Arc<RwLock<HashMap<i32, Arc<Mutex<WebSocket<TcpStream>>>>>>,
}

#[derive(Debug)]
pub enum ReadMessageError {
    Iddle,
    Disconnected,
}

impl SocketPool {
    pub fn new() -> Self {
        Self {
            pool: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn add(&self, v: PlayerChannelClient) {
        let mut pool = self
            .pool
            .try_write()
            .inspect_err(|e| {
                println!("{:?}", e);
            })
            .unwrap();

        pool.insert(v.client_id, Arc::new(Mutex::new(v.socket)));

        println!("LENGTH: {}", pool.len())
    }

    fn get_channel(&self, client_id: &i32) -> Option<Arc<Mutex<WebSocket<TcpStream>>>> {
        let client_channels = self.pool.read().unwrap();

        client_channels.get(client_id).cloned()
    }

    fn read_sync(&self, socket: Arc<Mutex<WebSocket<TcpStream>>>) -> Result<TMessage, TError> {
        let start_time = SystemTime::now();
        loop {
            let message = socket.lock().unwrap().read();
            match message {
                Ok(msg) => {
                    return Ok(msg);
                }
                Err(tungstenite::Error::Io(ref err)) if err.kind() == ErrorKind::WouldBlock => {
                    if start_time.elapsed().unwrap() >= Duration::from_secs(300) {
                        return Err(tungstenite::Error::Io(std::io::Error::new(
                            ErrorKind::TimedOut,
                            "Read operation timed out",
                        )));
                    }
                    std::thread::sleep(Duration::from_millis(1000));
                }
                Err(err) => {
                    return Err(err);
                }
            };
        }
    }

    pub fn read_client_message<T: prost::Message + Default + 'static>(
        &self,
        client_id: i32,
    ) -> Result<T, ReadMessageError> {
        let socket = self.get_channel(&client_id).unwrap();

        let result = self.read_sync(socket);

        let message = match result {
            Err(e) => match e {
                TError::ConnectionClosed => return Err(ReadMessageError::Disconnected),
                TError::AlreadyClosed => return Err(ReadMessageError::Disconnected),
                TError::Io(_) => return Err(ReadMessageError::Iddle),
                TError::Tls(_) => todo!(),
                TError::Capacity(_) => todo!(),
                TError::Protocol(_) => return Err(ReadMessageError::Disconnected),
                TError::WriteBufferFull(_) => todo!(),
                TError::Utf8 => todo!(),
                TError::AttackAttempt => todo!(),
                TError::Url(_) => todo!(),
                TError::Http(_) => todo!(),
                TError::HttpFormat(_) => todo!(),
            },
            Ok(r) => r,
        };
        let bytes = match message {
            TMessage::Binary(bytes) => bytes,
            TMessage::Close(_) => return Err(ReadMessageError::Disconnected),
            _ => panic!("Expected binary message"),
        };

        let mut reader = std::io::Cursor::new(bytes);

        let request = T::decode(&mut reader).unwrap();
        Ok(request)
    }

    pub fn close_connection(&self, client_id: i32) {
        // let mut client_channels = self.pool.write().unwrap();
        // let socket = client_channels.remove(&client_id).unwrap();
        // let mut guard = socket.lock().unwrap();
        // if guard.can_write() {
        //     guard.close(None).unwrap();
        // }
    }

    pub fn update_clients(&self, responses: Vec<TMessageResponse>) {
        let client_channels = self
            .pool
            .try_read()
            .inspect_err(|e| {
                println!("{:?}", e);
            })
            .unwrap();
        for response in responses {
            let response_message = ResponseMessage {
                payload: response.message.encode_message(),
                payload_type: response.message_type.into(),
            };

            let socket = client_channels.get(&response.receiver_id).unwrap();

            let encoded: Vec<u8> = response_message.encode_message();
            socket
                .lock()
                .unwrap()
                .send(TMessage::Binary(encoded))
                .unwrap();
        }
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
    // fn get_channel(&self, client_id: i32) -> Option<MutexGuard<WebSocket<TcpStream>>> {
    //     // Acquire read lock
    //     let client_channels = self.pool.read().unwrap();

    //     // Retrieve the socket using client_id
    //     if let Some(socket_mutex) = client_channels.get(&client_id) {
    //         // Acquire the lock and return the guard
    //         Some(socket_mutex.lock().unwrap())
    //     } else {
    //         None
    //     }
    // }
}
