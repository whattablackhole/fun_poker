use std::{collections::HashMap, sync::Mutex};

use crate::dealer::Dealer;

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