use crate::protos::{player::{Player, PlayerStatus}, user::User};

impl Player {
    // pass Settings with tournament settings
    // maybe store user directly...
    pub fn from_users(users: Vec<User>) -> Vec<Player> {
        users
            .iter()
            .map(|u| {
                Player {
                    action: None,
                    bank: 0,
                    country: u.country.clone(),
                    user_id: u.id,
                    // can be nickname
                    user_name: u.name.clone(),
                    cards: None,
                    bet_in_current_seed: 0,
                    status: PlayerStatus::SitOut.into()
                }
            })
            .collect()
    }

    pub fn from_user(u: User) -> Player {
        Player {
            action: None,
            bank: 10000,
            country: u.country,
            user_id: u.id,
            user_name: u.name,
            cards: None,
            bet_in_current_seed: 0,
            status: PlayerStatus::SitOut.into()
        }
    }
}

#[derive(Debug)]
pub enum PlayerPayloadError {
    Disconnected { id: i32, lobby_id: i32 },
    Iddle { id: i32, lobby_id: i32 },
}
