pub mod card;
pub mod dealer;
pub mod dealer_pool;
pub mod game;
pub mod game_orchestrator;
pub mod lobby;
pub mod player;
pub mod postgres_database;
pub mod responses;
pub mod socket_pool;
pub mod thread_pool;

pub mod protos {
    pub mod client_state {
        include!("protos_rs/client_state.rs");
    }

    pub mod lobby {
        include!("protos_rs/lobby.rs");
    }

    pub mod game_state {
        include!("protos_rs/game_state.rs");
    }

    pub mod empty {
        include!("protos_rs/empty.rs");
    }

    pub mod player {
        include!("protos_rs/player.rs");
    }

    pub mod user {
        include!("protos_rs/user.rs");
    }

    pub mod card {
        include!("protos_rs/card.rs");
    }

    pub mod requests {
        include!("protos_rs/requests.rs");
    }

    pub mod responses {
        include!("protos_rs/responses.rs");
    }
}
