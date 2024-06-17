use std::{
    collections::HashMap,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex, RwLock,
    },
};

use crate::{
    game::{Game, GameSettings},
    protos::user::User,
    responses::{generate_game_started_responses, GameChannelMessage},
    socket_pool::SocketPool,
    thread_pool::ThreadPool,
};

pub struct GameOrchestrator {
    game_pool: Mutex<HashMap<i32, GameClient>>,
}
pub struct GameClient {
    game: Arc<RwLock<Game>>,
    sender: Arc<RwLock<Sender<GameChannelMessage>>>,
    receiver: Arc<Mutex<Receiver<GameChannelMessage>>>,
}
pub struct JoinGameMessage {
    pub user: User,
}

impl GameOrchestrator {
    pub fn new() -> Self {
        return GameOrchestrator {
            game_pool: Mutex::new(HashMap::new()),
        };
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
        let pool = match self.game_pool.try_lock() {
            Ok(v) => v,
            Err(e) => {
                panic!("error in join game while locking game_pool {:?}", e);
            }
        };

        let game_m = pool.get(&lobby_id).unwrap();

        let mut lock = game_m.game.try_write();

        if let Ok(ref mut mutex) = lock {
            mutex.add_player(user, socket_pool);
        } else {
            let g = game_m.sender.read().unwrap();
            g.send(GameChannelMessage::HttpRequestSource(JoinGameMessage {
                user,
            }))
            .unwrap();
        }
    }

    pub fn can_start_game(&self, lobby_id: i32) -> bool {
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
