use std::{
    collections::HashMap,
    sync::{
        mpsc::{Receiver, Sender},
        Arc, Mutex, RwLock,
    },
};

use prost::Message;

use crate::{
    card::CardDeck,
    dealer::Dealer,
    player::PlayerPayloadError,
    protos::{
        client_state::ClientState,
        game_state::{GameStatus, ShowdownOutcome, Street, StreetStatus},
        player::{Action, ActionType, Player, PlayerPayload, PlayerStatus},
    },
    responses::{
        generate_client_state_responses, EncodableMessage, GameChannelMessage, SocketSourceMessage,
    },
    socket_pool::{ConnectionClosedEvent, ReadMessageError, SocketPool},
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

    fn get_player(&mut self, user_id: &i32) -> Option<&mut Player> {
        self.player_state
            .players
            .iter_mut()
            .find(|p| p.user_id == *user_id)
    }

    pub fn is_ready_to_start(&self) -> bool {
        // TODO: improve checking in case of player game status is not ready
        self.game_state.status != GameStatus::Active && self.player_state.players.len() > 1
    }

    pub fn hande_connection_update(
        &mut self,
        event: &ConnectionClosedEvent,
        socket_pool: &Arc<SocketPool>,
    ) -> bool {
        let player = self.get_player(&event.user_id).unwrap();

        player.set_status(PlayerStatus::Disconnected);

        let states = self
            .dealer
            .get_client_states(&self.game_state, &self.player_state);

        // TODO: add hash sum for clientstate to check if client received current state or not
        socket_pool.update_clients(generate_client_state_responses(states));

        true
    }

    pub fn add_player(&mut self, mut player: Player, socket_pool: &Arc<SocketPool>) {
        let user_id = player.user_id;

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
                    let mut action = Action::default();
                    action.set_action_type(ActionType::Fold);
                    p.action = Some(action);
                } else {
                    p.status = PlayerStatus::WaitingForPlayers.into();
                }
            }
            None => {
                if self.game_state.status == GameStatus::Active {
                    player.status = PlayerStatus::Ready.into();
                    let mut action = Action::default();
                    action.set_action_type(ActionType::Fold);
                    player.action = Some(action);
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
        let game_states = self
            .dealer
            .start_new_game(
                &mut self.game_state,
                &mut self.player_state,
                &mut self.deck_state,
            )
            .unwrap();

        socket_pool.update_clients(generate_client_state_responses(game_states));

        'outer_loop: loop {
            // TODO: refactor
            let user_id = self
                .dealer
                .get_next_player_id(&mut self.game_state, &mut self.player_state);
            let player = self
                .dealer
                .get_next_player(&mut self.game_state, &mut self.player_state);
            let is_bot = player.is_bot;

            if is_bot {
                let lobby_id = self.lobby_id;
                let id = player.user_id;
                let clone_tx: Arc<RwLock<Sender<GameChannelMessage>>> = Arc::clone(&tx);
                let client_state =
                    self.dealer
                        .get_client_state(&id, &mut self.game_state, &mut self.player_state);
                let llama_bot_api_url = "http://127.0.0.1:5000/pocker_move";

                thread_pool.execute(move || {
                    let response = ureq::post(llama_bot_api_url)
                        .set("Content-Type", "application/json")
                        .send_bytes(&ClientState::encode_message(&client_state));

                    match response {
                        Ok(response) => {
                            if response.status() == 200 {
                                println!("Response: {:?}", response);

                                let mut buf: Vec<u8> = Vec::new();
                                response.into_reader().read_to_end(&mut buf).unwrap();

                                let mut reader = std::io::Cursor::new(buf);

                                let mut action = Action::default();
                                action.merge(&mut reader).unwrap();

                                println!("Bot message sent successfully: {:?}", action);
                                let mut payload = PlayerPayload::default();
                                payload.action = Some(action);
                                payload.lobby_id = lobby_id;
                                payload.player_id = id;
                                clone_tx
                                    .read()
                                    .unwrap()
                                    .send(GameChannelMessage::InnerSource(payload))
                                    .unwrap();
                            } else {
                                eprintln!("Failed to send bot message: {:?}", response.status());
                            }
                        }
                        Err(e) => {
                            eprintln!("Error sending bot message: {:?}", e);
                        }
                    }
                });
            } else {
                let clone_s_pool = Arc::clone(&socket_pool);
                let clone_tx: Arc<RwLock<Sender<GameChannelMessage>>> = Arc::clone(&tx);

                thread_pool.execute(move || {
                    let result: Result<PlayerPayload, ReadMessageError> =
                        clone_s_pool.read_client_message::<PlayerPayload>(user_id);
                    clone_tx
                        .read()
                        .unwrap()
                        .send(GameChannelMessage::SocketSource(
                            SocketSourceMessage::PlayerPayload(result),
                        ))
                        .unwrap();
                });
            }

            loop {
                let message = rx.lock().unwrap().recv().unwrap();
                match message {
                    GameChannelMessage::SocketSource(r) => match r {
                        SocketSourceMessage::PlayerPayload(p) => match p {
                            Ok(m) => {
                                self.update_game_state(&socket_pool, Ok(m));
                                continue 'outer_loop;
                            }
                            Err(e) => {
                                let error: PlayerPayloadError = match e {
                                    ReadMessageError::Disconnected => {
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
                                self.update_game_state(&socket_pool, Err(error));
                                continue 'outer_loop;
                            }
                        },
                        SocketSourceMessage::ConnectionClosed(e) => {
                            self.hande_connection_update(&e, &socket_pool);
                        }
                    },
                    GameChannelMessage::HttpRequestSource(r) => {
                        self.add_player(r.player, &socket_pool)
                    }
                    GameChannelMessage::InnerSource(m) => {
                        self.update_game_state(&socket_pool, Ok(m));
                        continue 'outer_loop;
                    }
                };
            }
        }
    }
}
