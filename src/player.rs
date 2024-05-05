use crate::protos::{
    player::{ActionType, Player, PlayerAction},
    user::User,
};

impl PlayerAction {
    pub fn new() -> PlayerAction {
        PlayerAction {
            action_type: ActionType::Empty.into(),
            bet: 0,
        }
    }
}

impl Player {
    // pass Settings with tournament settings
    // maybe store user directly...
    pub fn from_users(users: Vec<User>) -> Vec<Player> {
        users
            .iter()
            .map(|u| {
                Player {
                    action: Some(PlayerAction::new()),
                    bank: 0,
                    country: u.country.clone(),
                    user_id: u.id,
                    // can be nickname
                    user_name: u.name.clone(),
                    cards: Option::None,
                    bet_in_current_seed: 0,
                }
            })
            .collect()
    }
}
