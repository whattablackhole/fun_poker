use std::{
    collections::{HashMap, HashSet},
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
};

use rand::Rng;

use crate::{
    game::{Game, GameSettings},
    protos::{player::Player, user::User},
    responses::{generate_game_started_responses, GameChannelMessage, SocketSourceMessage},
    socket_pool::{ConnectionClosedEvent, SocketPool},
    thread_pool::ThreadPool,
};

pub struct GameOrchestrator {
    game_pool: Mutex<HashMap<i32, GameClient>>,
    user_map: Mutex<HashMap<i32, HashSet<i32>>>,
}
pub struct GameClient {
    game: Arc<RwLock<Game>>,
    sender: Arc<RwLock<Sender<GameChannelMessage>>>,
    receiver: Arc<Mutex<Receiver<GameChannelMessage>>>,
}
pub struct JoinGameMessage {
    pub player: Player,
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
        let user_map = self.user_map.lock().unwrap();

        match user_map.get(&event.user_id) {
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
                                .sender
                                .read()
                                .unwrap()
                                .send(GameChannelMessage::SocketSource(
                                    SocketSourceMessage::ConnectionClosed(event.clone()),
                                ))
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
        let game_mutex = RwLock::new(game);
        let game_arc = Arc::new(game_mutex);

        let (sender, receiver) = channel();

        pool.insert(
            lobby_id,
            GameClient {
                game: game_arc,
                sender: Arc::new(RwLock::new(sender)),
                receiver: Arc::new(Mutex::new(receiver)),
            },
        );

        true
    }

    pub fn join_game(&self, lobby_id: i32, user: User, socket_pool: &Arc<SocketPool>) {
        let id = user.id;
        let player = Player::from_user(user);

        let pool = match self.game_pool.try_lock() {
            Ok(v) => v,
            Err(e) => {
                panic!("error in join game while locking game_pool {:?}", e);
            }
        };

        let game_m = pool.get(&lobby_id).unwrap();

        let mut lock = game_m.game.try_write();

        if let Ok(ref mut mutex) = lock {
            mutex.add_player(player, socket_pool);
        } else {
            let g = game_m.sender.read().unwrap();
            g.send(GameChannelMessage::HttpRequestSource(JoinGameMessage {
                player,
            }))
            .unwrap();
        }
        self.user_map
            .lock()
            .unwrap()
            .entry(id)
            .or_insert_with(HashSet::new)
            .insert(lobby_id);
    }

    pub fn spawn_bot(&self, lobby_id: i32, socket_pool: &Arc<SocketPool>) {
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

        let game_m = pool.get(&lobby_id).unwrap();

        let mut lock = game_m.game.try_write();

        if let Ok(ref mut mutex) = lock {
            mutex.add_player(bot_player, socket_pool);
        } else {
            let g = game_m.sender.read().unwrap();
            g.send(GameChannelMessage::HttpRequestSource(JoinGameMessage {
                player: bot_player,
            }))
            .unwrap();
        }
    }

    pub fn should_start_game(&self, lobby_id: i32) -> bool {
        let pool = self.game_pool.lock().unwrap();

        let game_m = pool.get(&lobby_id).unwrap();

        let game = game_m.game.read().unwrap();
        // TODO: add game setting: auto_start?
        game.is_ready_to_start()
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

        let receiver_clone = Arc::clone(&game_client.receiver);
        let sender_clone = Arc::clone(&game_client.sender);

        thread_pool.execute(move || {
            let ref mut game = game_clone.write().unwrap();
            game.run(socket_pool, pool, receiver_clone, sender_clone);
        });
    }
}
