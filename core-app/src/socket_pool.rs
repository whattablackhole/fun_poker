use std::{
    collections::HashMap,
    io::ErrorKind,
    net::TcpStream,
    sync::{Arc, Mutex, MutexGuard, RwLock},
    thread::{self},
    time::{Duration, SystemTime},
};

use tungstenite::{protocol::CloseFrame, Error as TError, Message as TMessage, WebSocket};

use crate::{
    protos::responses::ResponseMessage,
    responses::{EncodableMessage, TMessageResponse},
};

#[derive(Clone)]
pub struct ConnectionClosedEvent {
    pub user_id: i32,
}

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

    pub fn read_client_message<T: prost::Message + Default + 'static>(
        &self,
        client_id: i32,
    ) -> Result<T, ReadMessageError> {
        let socket = self.get_channel(&client_id).unwrap();
        let result = self.read_sync(socket);

        let message = match result {
            Err(e) => match e {
                TError::ConnectionClosed => {
                    self.remove_connection(client_id);
                    return Err(ReadMessageError::Disconnected);
                }
                TError::AlreadyClosed => {
                    self.remove_connection(client_id);
                    return Err(ReadMessageError::Disconnected);
                }
                TError::Io(_) => return Err(ReadMessageError::Iddle),
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
            Ok(r) => r,
        };

        let bytes = match message {
            TMessage::Binary(bytes) => bytes,
            TMessage::Close(frame) => {
                let connection = self.remove_connection(client_id);
                self.close_connection(connection, frame);
                return Err(ReadMessageError::Disconnected);
            }
            _ => panic!("Expected binary message"),
        };

        let mut reader = std::io::Cursor::new(bytes);

        let request = T::decode(&mut reader).unwrap();
        Ok(request)
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
            println!("{}", &response.receiver_id);
            
            let socket = client_channels.get(&response.receiver_id).unwrap();

            let encoded: Vec<u8> = response_message.encode_message();
            socket
                .lock()
                .unwrap()
                .send(TMessage::Binary(encoded))
                .unwrap();
        }
    }

    pub fn check_connections(
        &self,
    ) -> Arc<Mutex<Vec<Box<dyn Fn(ConnectionClosedEvent) + Send + Sync>>>> {
        let pool_clone = Arc::clone(&self.pool);

        let jobs: Arc<Mutex<Vec<Box<dyn Fn(ConnectionClosedEvent) + Send + Sync>>>> =
            Arc::new(Mutex::new(Vec::new()));

        let jobs_clone = Arc::clone(&jobs);

        thread::spawn(move || loop {
            thread::sleep(Duration::from_millis(5000));

            let closed_connections: Vec<i32> = {
                let pool = pool_clone.read().unwrap();
                pool.iter()
                    .filter_map(|(user_id, socket)| {
                        let mut socket_guard = socket.lock().unwrap();
                        if !SocketPool::ping(&mut socket_guard) {
                            Some(*user_id)
                        } else {
                            None
                        }
                    })
                    .collect::<Vec<_>>()
            };

            for connection_id in closed_connections {
                pool_clone.write().unwrap().remove(&connection_id);

                let jobs = jobs_clone.lock().unwrap();

                for job in jobs.iter() {
                    job(ConnectionClosedEvent {
                        user_id: connection_id,
                    });
                }
            }
        });
        Arc::clone(&jobs)
    }

    fn ping(socket_guard: &mut MutexGuard<WebSocket<TcpStream>>) -> bool {
        if let Err(_) = socket_guard.write(TMessage::Ping(Vec::new())) {
            return false;
        }
        if let Err(_) = socket_guard.flush() {
            return false;
        }
        true
    }
    fn remove_connection(&self, client_id: i32) -> Arc<Mutex<WebSocket<TcpStream>>> {
        let mut client_channels = self.pool.write().unwrap();
        client_channels.remove(&client_id).unwrap()
    }

    fn close_connection(
        &self,
        connection: Arc<Mutex<WebSocket<TcpStream>>>,
        frame: Option<CloseFrame<'static>>,
    ) {
        let mut guard = connection.lock().unwrap();
        guard.close(frame).unwrap();
        guard.flush().unwrap();
    }

    fn get_channel(&self, client_id: &i32) -> Option<Arc<Mutex<WebSocket<TcpStream>>>> {
        let client_channels = self.pool.read().unwrap();

        client_channels.get(client_id).cloned()
    }

    fn read_sync(&self, socket: Arc<Mutex<WebSocket<TcpStream>>>) -> Result<TMessage, TError> {
        let start_time = SystemTime::now();
        loop {
            let message: Result<TMessage, TError> = socket.lock().unwrap().read();
            match message {
                Ok(msg) => {
                    if !msg.is_pong() {
                        return Ok(msg);
                    }
                }
                Err(tungstenite::Error::Io(ref err)) if err.kind() == ErrorKind::WouldBlock => {
                    if start_time.elapsed().unwrap() >= Duration::from_secs(300) {
                        return Err(tungstenite::Error::Io(std::io::Error::new(
                            ErrorKind::TimedOut,
                            "Read operation timed out",
                        )));
                    }
                    // think about optimizing sleeping time to balance between cpu usage and responsiveness
                    std::thread::sleep(Duration::from_millis(200));
                }
                Err(err) => {
                    return Err(err);
                }
            };
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
    //     }
}
