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
        let mut pool = self.pool.try_write().unwrap();

        pool.insert(v.client_id, Arc::new(Mutex::new(v.socket)));

        println!("LENGTH: {}", pool.len())
    }

    pub fn read_client_message<T: prost::Message + Default + 'static>(
        &self,
        client_id: i32,
    ) -> Result<T, ReadMessageError> {
        let socket = match self.get_channel(&client_id) {
            Some(s) => s,
            None => {
                println!(
                    "User disconnected and removed from pool before health check event: {}",
                    &client_id
                );
                return Err(ReadMessageError::Disconnected);
            }
        };

        let result = self.read_non_blocking(socket);

        let message = match result {
            Err(e) => match e {
                TError::ConnectionClosed => {
                    self.remove_connection(&client_id);
                    return Err(ReadMessageError::Disconnected);
                }
                TError::AlreadyClosed => {
                    self.remove_connection(&client_id);
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
                let connection = self.remove_connection(&client_id);
                self.close_connection(connection, frame);
                return Err(ReadMessageError::Disconnected);
            }
            _ => panic!("Expected binary message"),
        };

        let mut reader = std::io::Cursor::new(bytes);

        let request = T::decode(&mut reader).unwrap();
        Ok(request)
    }

    // TODO: return Array of results
    pub fn update_clients(&self, responses: Vec<TMessageResponse>) -> Vec<i32> {
        let mut unsuccessful_clients = Vec::new();

        let client_channels = self.pool.try_read().unwrap();

        for response in responses {
            let response_message = ResponseMessage {
                payload: response.message.encode_message(),
                payload_type: response.message_type.into(),
            };

            match client_channels.get(&response.receiver_id) {
                Some(s) => {
                    match s.lock() {
                        Ok(mut guard) => {
                            match guard.send(TMessage::Binary(response_message.encode_message())) {
                                Ok(_) => {}
                                Err(e) => {
                                    println!("Error occurred during sending message to client {}: {}", response.receiver_id, e);
                                    unsuccessful_clients.push(response.receiver_id);
                                }
                            }
                        }
                        Err(e) => {
                            println!("Failed to lock mutex for client {}: {}", response.receiver_id, e);
                            unsuccessful_clients.push(response.receiver_id);
                        }
                    }
                },
                None => {
                    println!(
                        "no such user connection with provided id: {}",
                        &response.receiver_id
                    );
                    unsuccessful_clients.push(response.receiver_id);

                }
            };
        }

        drop(client_channels);

        for c in unsuccessful_clients.iter() {
            self.remove_connection(c);
        }

        unsuccessful_clients
    }

    // TODO: return Result instead

    pub fn check_connection_health(&self, connection_id: i32) -> bool {
        let pool = self.pool.read().unwrap();
        let connected = match pool.get(&connection_id) {
            Some(socket) => {
                let mut socket_guard = socket.lock().unwrap();
                let mut connected = false;

                SocketPool::ping(&mut socket_guard);
                let time = SystemTime::now();

                loop {
                    match socket_guard.read() {
                        Ok(r) => match r {
                            TMessage::Pong(_) => {
                                connected = true;
                                break;
                            }
                            TMessage::Close(_) => {
                                connected = false;
                                break;
                            }
                            _ => {
                                if time.elapsed().unwrap() > Duration::from_millis(3000) {
                                    break;
                                }
                                thread::sleep(Duration::from_millis(200));
                            }
                        },
                        Err(_) => {
                            if time.elapsed().unwrap() > Duration::from_millis(3000) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(200));
                        }
                    }
                }
                connected
            }
            None => {
                println!("connection with id {} is already removed", &connection_id);
                false
            }
        };

        if !connected {
            drop(pool);
            self.pool.write().unwrap().remove(&connection_id);
        }

        connected
    }

    pub fn spawn_health_checker(
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
                        // TODO: rethink
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
    fn remove_connection(&self, client_id: &i32) -> Arc<Mutex<WebSocket<TcpStream>>> {
        let mut client_channels = self.pool.write().unwrap();
        client_channels.remove(client_id).unwrap()
    }

    fn close_connection(
        &self,
        connection: Arc<Mutex<WebSocket<TcpStream>>>,
        frame: Option<CloseFrame<'static>>,
    ) {
        let mut guard = connection.lock().unwrap();

        match guard.close(frame) {
            Ok(_) => match guard.flush() {
                Ok(_) => {},
                Err(e) => match e {
                    tungstenite::Error::ConnectionClosed => {
                        println!("Connection closed abruptly by client!")
                    },
                    _ => {
                        println!("Unprocessed error while flushing connection close: {}", e);
                    } ,
                },
            },
            Err(e) => match e {
                tungstenite::Error::ConnectionClosed => {
                    println!("Connection closed abruptly by client!")
                },
                _ => {
                    println!("Unprocessed error while closing connection: {}", e);
                }
            },
        }
    }

    fn get_channel(&self, client_id: &i32) -> Option<Arc<Mutex<WebSocket<TcpStream>>>> {
        let client_channels = self.pool.read().unwrap();

        client_channels.get(client_id).cloned()
    }

    fn read_non_blocking(&self, socket: Arc<Mutex<WebSocket<TcpStream>>>) -> Result<TMessage, TError> {
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
}
