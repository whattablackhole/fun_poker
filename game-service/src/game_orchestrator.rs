use std::{
    collections::{HashMap, HashSet},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
};

use rand::Rng;

use crate::{
    channel_messages::{GameChannelRequestMessage, GameChannelResponseMessage, JoinGameMessage},
    game::{Game, GameSettings, JoinGameError},
    protos::{player::Player, user::User},
    responses::generate_game_started_responses,
    socket_pool::{ConnectionClosedEvent, SocketPool},
    thread_pool::ThreadPool,
};

pub struct GameOrchestrator {
    game_pool: Mutex<HashMap<i32, GameClient>>,
    user_map: Mutex<HashMap<i32, HashSet<i32>>>,
}
pub struct GameClient {
    game: Arc<RwLock<Game>>,
    request_sender: Arc<RwLock<Sender<GameChannelRequestMessage>>>,
    request_receiver: Arc<Mutex<Receiver<GameChannelRequestMessage>>>,
    response_sender: Arc<RwLock<Sender<GameChannelResponseMessage>>>,
    response_receiver: Arc<Mutex<Receiver<GameChannelResponseMessage>>>,
}

impl GameOrchestrator {
    pub fn new() -> Self {
        return GameOrchestrator {
            game_pool: Mutex::new(HashMap::new()),
            user_map: Mutex::new(HashMap::new()),
        };
    }
    // TODO: add More ConnectionEvents:
    // e.g. ConnectionRestored
    pub fn update_player_connection_status(
        &self,
        event: ConnectionClosedEvent,
        socket_pool: &Arc<SocketPool>,
    ) {
        let mut user_map = self.user_map.lock().unwrap();
        let user_servers = user_map.remove(&event.user_id);
        
        match user_servers {
            None => return,
            Some(game_ids) => {
                game_ids.iter().for_each(move |id| {
                    let mut game_pool = self.game_pool.lock().unwrap();
                    let game_client = game_pool.get_mut(id).unwrap();

                    match game_client.game.try_write() {
                        Ok(mut game) => {
                            game.hande_connection_update(&event, socket_pool);
                        }
                        Err(_) => {
                            game_client
                                .request_sender
                                .read()
                                .unwrap()
                                .send(GameChannelRequestMessage::ConnectionClosed(event.clone()))
                                .unwrap();
                        }
                    };
                });
            }
        }
    }

    pub fn is_game_exists(&self, lobby_id: i32) -> bool {
        let pool = self.game_pool.lock().unwrap();

        pool.contains_key(&lobby_id)
    }

    pub fn create_game(&self, lobby_id: i32, settings: GameSettings) -> bool {
        let mut pool = self.game_pool.lock().unwrap();

        let game = Game::new(lobby_id, settings);
        let game_clientutex = RwLock::new(game);
        let game_arc = Arc::new(game_clientutex);

        let (request_sender, request_receiver) = channel();
        let (response_sender, response_receiver) = channel();

        pool.insert(
            lobby_id,
            GameClient {
                game: game_arc,
                request_sender: Arc::new(RwLock::new(request_sender)),
                request_receiver: Arc::new(Mutex::new(request_receiver)),
                response_sender: Arc::new(RwLock::new(response_sender)),
                response_receiver: Arc::new(Mutex::new(response_receiver)),
            },
        );

        true
    }

    pub fn join_game(
        &self,
        lobby_id: i32,
        user: User,
        socket_pool: &Arc<SocketPool>,
    ) -> Result<(), JoinGameError> {
        let id = user.id;
        let player = Player::from_user(user);

        let pool = match self.game_pool.try_lock() {
            Ok(v) => v,
            Err(e) => {
                panic!("error in join game while locking game_pool {:?}", e);
            }
        };

        let game_client = pool.get(&lobby_id).unwrap();

        let mut lock = game_client.game.try_write();

        if let Ok(ref mut mutex) = lock {
            return mutex.add_player(player, socket_pool);
        } else {
            let g: std::sync::RwLockReadGuard<Sender<GameChannelRequestMessage>> =
                game_client.request_sender.read().unwrap();
            g.send(GameChannelRequestMessage::JoinGame(JoinGameMessage {
                player,
            }))
            .unwrap();

            let result = game_client
                .response_receiver
                .lock()
                .unwrap()
                .recv()
                .unwrap();
            match result {
                GameChannelResponseMessage::JoinGame(r) => {
                    if r.is_ok() {
                        self.user_map
                            .lock()
                            .unwrap()
                            .entry(id)
                            .or_insert_with(HashSet::new)
                            .insert(lobby_id);
                    }
                    return r;
                }
            }
        }
    }

    pub fn spawn_bot(
        &self,
        lobby_id: i32,
        socket_pool: &Arc<SocketPool>,
    ) -> Result<(), JoinGameError> {
        let mut bot_player = Player::default();
        bot_player.user_name = String::from("Chat gpt");
        bot_player.bank = 10000;
        bot_player.is_bot = true;
        let mut rng = rand::thread_rng();
        bot_player.user_id = -rng.gen_range(1..i32::MAX);

        let pool = match self.game_pool.try_lock() {
            Ok(v) => v,
            Err(e) => {
                panic!("error in join game while locking game_pool {:?}", e);
            }
        };

        let game_client = pool.get(&lobby_id).unwrap();

        let mut lock = game_client.game.try_write();

        if let Ok(ref mut mutex) = lock {
            return mutex.add_player(bot_player, socket_pool);
        } else {
            let g: std::sync::RwLockReadGuard<Sender<GameChannelRequestMessage>> =
                game_client.request_sender.read().unwrap();
            g.send(GameChannelRequestMessage::JoinGame(JoinGameMessage {
                player: bot_player,
            }))
            .unwrap();

            let result = game_client
                .response_receiver
                .lock()
                .unwrap()
                .recv()
                .unwrap();
            match result {
                GameChannelResponseMessage::JoinGame(r) => {
                    return r;
                }
            }
        }
    }

    pub fn should_start_game(&self, lobby_id: i32) -> bool {
        let pool = self.game_pool.lock().unwrap();

        let game_client = pool.get(&lobby_id).unwrap();

        match game_client.game.try_read() {
            Ok(g) => return g.is_ready_to_start(),
            Err(_) => return false,
        };
    }

    pub fn start_game(
        &self,
        lobby_id: i32,
        thread_pool: Arc<ThreadPool>,
        socket_pool: Arc<SocketPool>,
    ) {
        let pool = self.game_pool.lock().unwrap();
        let game_client = pool.get(&lobby_id).unwrap();

        let game_clone = Arc::clone(&game_client.game);
        let pool = Arc::clone(&thread_pool);

        let game_started_responses = generate_game_started_responses(lobby_id, &Vec::new(), 10);
        socket_pool.update_clients(game_started_responses);

        let request_receiver_clone = Arc::clone(&game_client.request_receiver);
        let request_sender_clone = Arc::clone(&game_client.request_sender);
        let response_sender_clone = Arc::clone(&game_client.response_sender);

        thread_pool.execute(move || {
            let ref mut game = game_clone.write().unwrap();

            match game.run(
                socket_pool,
                pool,
                request_receiver_clone,
                request_sender_clone,
                response_sender_clone,
            ) {
                Ok(_) => {}
                Err(er) => println!("game shutdown abruptly: {}", er),
            };
        });
    }
}
