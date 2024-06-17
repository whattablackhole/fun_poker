use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex, RwLock,
    },
};

use crate::{
    card::CardDeck,
    dealer::Dealer,
    player::PlayerPayloadError,
    protos::{
        game_state::{GameStatus, ShowdownOutcome, Street, StreetStatus},
        player::{Action, Player, PlayerPayload, PlayerStatus},
        user::User,
    },
    responses::{generate_client_state_responses, GameChannelMessage},
    socket_pool::{ReadMessageError, SocketPool},
    thread_pool::ThreadPool,
};

pub struct DeckState {
    pub deck: CardDeck,
}

impl DeckState {
    pub fn new_random(&mut self) {
        self.deck = CardDeck::new_random();
    }

    pub fn new(deck: CardDeck) -> DeckState {
        DeckState { deck }
    }
}
#[derive(Debug)]
pub struct KeyPositions {
    pub small_blind_index: Option<usize>,
    pub big_blind_index: Option<usize>,
    pub curr_player_index: Option<usize>,
    pub button_index: Option<usize>,
}

impl KeyPositions {
    fn new() -> Self {
        KeyPositions {
            small_blind_index: None,
            big_blind_index: None,
            curr_player_index: None,
            button_index: None,
        }
    }
}
pub struct GameState {
    pub status: GameStatus,
    pub street: Street,
    pub game_bank: i32,
    pub big_blind: i32,
    pub raise_amount: i32,
    pub raiser_index: Option<usize>,
    pub positions: KeyPositions,
    pub biggest_bet_on_curr_street: i32,
    pub action_history: Vec<Action>,
    pub showdown_outcome: Option<ShowdownOutcome>,
}

impl GameState {
    pub fn new(_players_amount: i32, blind_size: i32) -> GameState {
        GameState {
            status: GameStatus::WaitingForPlayers,
            street: Street {
                street_status: StreetStatus::Preflop.into(),
                cards: Vec::new(),
            },
            big_blind: blind_size,
            game_bank: 0,
            raise_amount: 0,
            biggest_bet_on_curr_street: blind_size,
            raiser_index: None,
            positions: KeyPositions::new(),
            action_history: Vec::new(),
            showdown_outcome: None,
        }
    }
}
pub struct PlayerState {
    pub players: Vec<Player>,
    pub bank_map: HashMap<i32, i32>,
}
impl PlayerState {
    pub fn new() -> PlayerState {
        PlayerState {
            bank_map: HashMap::new(),
            players: Vec::new(),
        }
    }

    pub fn from(mut players: Vec<Player>, bank_size: i32) -> PlayerState {
        let mut bank_map = HashMap::new();
        for player in players.iter_mut() {
            player.bank = bank_size;
            bank_map.insert(player.user_id, bank_size);
        }
        PlayerState { bank_map, players }
    }
}

pub struct GameSettings {
    pub blind_size: i32,
}

pub struct Game {
    dealer: Dealer,
    game_state: GameState,
    deck_state: DeckState,
    player_state: PlayerState,
    lobby_id: i32,
}

impl Game {
    pub fn new(lobby_id: i32, settings: GameSettings) -> Self {
        Game {
            dealer: Dealer::new(lobby_id),
            deck_state: DeckState::new(CardDeck::new_random()),
            game_state: GameState::new(0, settings.blind_size),
            player_state: PlayerState::new(),
            lobby_id,
        }
    }
    pub fn get_game_status(&self) -> GameStatus {
        self.game_state.status
    }

    pub fn is_ready_to_start(&self) -> bool {
        // TODO: improve checking in case of player game status is not ready
        self.game_state.status != GameStatus::Active && self.player_state.players.len() > 1
    }

    pub fn add_player(&mut self, user: User, socket_pool: &Arc<SocketPool>) {
        let user_id = user.id;

        // TODO: refactor ....
        match self
            .player_state
            .players
            .iter_mut()
            .find(|p| p.user_id == user_id)
        {
            Some(p) => {
                if self.game_state.status == GameStatus::Active {
                    p.status = PlayerStatus::Ready.into();
                } else {
                    p.status = PlayerStatus::WaitingForPlayers.into();
                }
            }
            None => {
                let mut player = Player::from_user(user);

                if self.game_state.status == GameStatus::Active {
                    player.status = PlayerStatus::Ready.into();
                }
                self.player_state.players.push(player);
            }
        }
        let states = self
            .dealer
            .get_client_states(&self.game_state, &self.player_state);

        // TODO: add hash sum for clientstate to check if client received current state or not
        socket_pool.update_clients(generate_client_state_responses(states));
    }

    fn update_game_state(
        &mut self,
        socket_pool: &Arc<SocketPool>,
        result: Result<PlayerPayload, PlayerPayloadError>,
    ) {
        let updated_state = self.dealer.update_game_state(
            result,
            &mut self.game_state,
            &mut self.player_state,
            &mut self.deck_state,
        );

        socket_pool.update_clients(generate_client_state_responses(updated_state.client_states));

        let mut is_ready = updated_state.is_ready_for_next_hand;

        if updated_state.should_complete_game_cycle_automatically {
            let updated_state = self.dealer.complete_game_cycle_automatically(
                &mut self.game_state,
                &mut self.player_state,
                &mut self.deck_state,
            );
            socket_pool
                .update_clients(generate_client_state_responses(updated_state.client_states));
            is_ready = updated_state.is_ready_for_next_hand;
        }

        while is_ready {
            let updated_state = self.dealer.setup_next_cycle(
                &mut self.game_state,
                &mut self.player_state,
                &mut self.deck_state,
            );

            socket_pool
                .update_clients(generate_client_state_responses(updated_state.client_states));

            if updated_state.should_complete_game_cycle_automatically {
                let updated_state = self.dealer.complete_game_cycle_automatically(
                    &mut self.game_state,
                    &mut self.player_state,
                    &mut self.deck_state,
                );
                is_ready = updated_state.is_ready_for_next_hand;
                socket_pool
                    .update_clients(generate_client_state_responses(updated_state.client_states));
            } else {
                is_ready = updated_state.is_ready_for_next_hand;
            }
        }
    }

    pub fn run(
        &mut self,
        socket_pool: Arc<SocketPool>,
        thread_pool: Arc<ThreadPool>,
        rx: Arc<Mutex<Receiver<GameChannelMessage>>>,
        tx: Arc<RwLock<Sender<GameChannelMessage>>>,
    ) {
        // TODO: think about how can we manage game status and state:
        // e.g. stopping and continue loop mechanisms - pause game
        // using channels, boolean variables etc...
        let game_states = self
            .dealer
            .start_new_game(
                &mut self.game_state,
                &mut self.player_state,
                &mut self.deck_state,
            )
            .unwrap();

        socket_pool.update_clients(generate_client_state_responses(game_states));

        loop {
            //TODO: handle the cases where a client is not responding, or has closed the connection;
            //TODO: use seperate messages for separated responses to decrease memory load and bandwidth
            let user_id = self
                .dealer
                .get_next_player_id(&mut self.game_state, &mut self.player_state);
            let clone_s_pool = Arc::clone(&socket_pool);
            let clone_tx: Arc<RwLock<Sender<GameChannelMessage>>> = Arc::clone(&tx);

            thread_pool.execute(move || {
                let result: Result<PlayerPayload, ReadMessageError> =
                    clone_s_pool.read_client_message::<PlayerPayload>(user_id);

                clone_tx
                    .read()
                    .unwrap()
                    .send(GameChannelMessage::DynMessageResult(result))
                    .unwrap();
            });

            let message = rx.lock().unwrap().recv().unwrap();
            
            match message {
                GameChannelMessage::DynMessageResult(r) => match r {
                    Ok(m) => self.update_game_state(&socket_pool, Ok(m)),
                    Err(e) => {
                        let error: PlayerPayloadError = match e {
                            ReadMessageError::Disconnected => {
                                socket_pool.close_connection(self.dealer.get_next_player_id(
                                    &mut self.game_state,
                                    &mut self.player_state,
                                ));
                                PlayerPayloadError::Disconnected {
                                    id: self.dealer.get_next_player_id(
                                        &mut self.game_state,
                                        &mut self.player_state,
                                    ),
                                    lobby_id: self.lobby_id,
                                }
                            }
                            ReadMessageError::Iddle => PlayerPayloadError::Iddle {
                                id: self.dealer.get_next_player_id(
                                    &mut self.game_state,
                                    &mut self.player_state,
                                ),
                                lobby_id: self.lobby_id,
                            },
                        };
                        self.update_game_state(&socket_pool, Err(error))
                    }
                },
                GameChannelMessage::HttpRequestSource(r) => self.add_player(r.user, &socket_pool),
            };
        }
    }
}
