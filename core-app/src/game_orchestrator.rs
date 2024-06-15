use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use crate::{
    game::{Game, GameSettings}, protos::user::User, responses::generate_game_started_responses, socket_pool::SocketPool, thread_pool::ThreadPool
};

pub struct GameOrchestrator {
    game_pool: Mutex<HashMap<i32, Arc<Mutex<Game>>>>,
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

        pool.insert(lobby_id, Arc::new(Mutex::new(game)));

        true
    }

    pub fn join_game(&self, lobby_id: i32, user: User, socket_pool: &Arc<SocketPool>) {
        let pool = self.game_pool.lock().unwrap();
        
        let game_m = pool.get(&lobby_id).unwrap();

        let mut game =  game_m.lock().unwrap();

        game.add_player(user, socket_pool);

       
    }

    pub fn can_start_game(&self, lobby_id: i32) -> bool {
        let pool = self.game_pool.lock().unwrap();
        
        let game_m = pool.get(&lobby_id).unwrap();

        let game =  game_m.lock().unwrap();
        // TODO: add game setting: auto_start?
        game.is_ready_to_start()
    }

    pub fn start_game(
        &self,
        lobby_id: i32,
        thread_pool: &Arc<ThreadPool>,
        socket_pool: Arc<SocketPool>,
    ) {
        let pool = self.game_pool.lock().unwrap();

        let game = pool.get(&lobby_id).unwrap();

        let game_clone = Arc::clone(game);

        let game_started_responses = generate_game_started_responses(lobby_id, &Vec::new(), 10);

        socket_pool.update_clients(game_started_responses);

        thread_pool.execute(move || {
            // thread::sleep(Duration::from_secs(10));

            let mut game = game_clone.lock().unwrap();
            game.run(socket_pool);
        });
    }
}
