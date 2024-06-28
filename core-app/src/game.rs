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
    protos::{
        client_state::ClientState,
        game_state::{Action, ActionType, GameStatus, ShowdownOutcome, Street, StreetStatus},
        player::{Player, PlayerStatus},
        requests::PlayerActionRequest,
        responses::{GameOverMessage, ResponseMessageType},
    },
    responses::{
        create_message_response, generate_client_state_responses, EncodableMessage,
        GameChannelMessage, PlayerActionRequestError, SocketSourceMessage, TMessageResponse,
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
    pub fn new(blind_size: i32) -> GameState {
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
            game_state: GameState::new(settings.blind_size),
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

    fn process_elimated_players(&mut self, socket_pool: &Arc<SocketPool>) {
        let players: &mut Vec<Player> = &mut self.player_state.players;

        let (retained, removed): (Vec<Player>, Vec<Player>) = players
            .drain(..)
            .partition(|p| p.status() != PlayerStatus::Eliminated);
        self.player_state.players = retained;

        let messages: Vec<TMessageResponse> = removed
            .iter()
            .filter_map(|p| {
                // TODO: Add messages in message queue for disconnected players
                if p.status() == PlayerStatus::Disconnected {
                    None
                } else {
                    return Some(create_message_response(
                        GameOverMessage {
                            user_id: p.user_id,
                            reason: String::from("Not enough funds to continue"),
                        },
                        ResponseMessageType::GameOver,
                        p.user_id,
                    ));
                }
            })
            .collect();

        socket_pool.update_clients(messages);
    }

    fn process_disconnected_players(&mut self) {
        let players: &mut Vec<Player> = &mut self.player_state.players;

        let (retained, removed): (Vec<Player>, Vec<Player>) = players
            .drain(..)
            .partition(|p| p.status() != PlayerStatus::Disconnected);

        // TODO: provide disconnect event for other players?
        self.player_state.players = retained;
    }


    fn prepare_to_game_stop(&mut self) {
        // TODO: handle settings struct;
        let blind_size = self.game_state.big_blind;

        self.deck_state = DeckState::new(CardDeck::new_random());
        self.game_state = GameState::new(blind_size);
        self.player_state.players.iter_mut().for_each(|p|{
            p.action = None;
            p.bet_in_current_seed = 0;
            p.cards = None;
            if p.status() == PlayerStatus::Ready {
                p.status = PlayerStatus::WaitingForPlayers.into();
            }
        })
    }

    fn update_game_state(
        &mut self,
        socket_pool: &Arc<SocketPool>,
        action: Result<PlayerActionRequest, PlayerActionRequestError>,
    ) -> GameStatus {
        let mut updated_state = self.dealer.update_game_state(
            action,
            &mut self.game_state,
            &mut self.player_state,
            &mut self.deck_state,
        );

        socket_pool.update_clients(generate_client_state_responses(updated_state.client_states));

        'a: loop {
            if updated_state.is_ready_for_next_hand {
                // WARN: locally tested: sometimes client is responding with pong right before disconnecting 
                // that leads to additional game cycle for disconnected player
                self.verify_connections(socket_pool);
                self.process_elimated_players(&socket_pool);
                self.process_disconnected_players();
                
                let players_count = self.player_state.players.len();

                if players_count == 0 {
                    self.prepare_to_game_stop();
                    self.game_state.status = GameStatus::None;
                    return GameStatus::None;
                }
                let active_players: Vec<&Player> = self.player_state.players.iter().filter(|p| {
                    // can be sitouted players and we wanna pause game in such case
                    p.status() == PlayerStatus::Ready
                }).collect();

                // TODO: handle all possible cases
                if active_players.len() < 2 {
                    self.prepare_to_game_stop();
                    self.game_state.status = GameStatus::WaitingForPlayers;
                    let states = self
                        .dealer
                        .get_client_states(&mut self.game_state, &mut self.player_state);
                    socket_pool.update_clients(generate_client_state_responses(states));
                    return GameStatus::WaitingForPlayers;
                }

                updated_state = self.dealer.setup_next_cycle(
                    &mut self.game_state,
                    &mut self.player_state,
                    &mut self.deck_state,
                );

                socket_pool
                    .update_clients(generate_client_state_responses(updated_state.client_states));
                continue 'a;
            } else if updated_state.should_complete_game_cycle_automatically {
                updated_state = self.dealer.complete_game_cycle_automatically(
                    &mut self.game_state,
                    &mut self.player_state,
                    &mut self.deck_state,
                );
                socket_pool
                    .update_clients(generate_client_state_responses(updated_state.client_states));
                continue 'a;
            }
            break;
        }

        return GameStatus::Active;
    }

    fn verify_connections(&mut self, socket_pool: &Arc<SocketPool>) {
        for p in  self.player_state.players.iter_mut() {
            // TODO: optimize by batching
            let connected = socket_pool.check_connection_health(p.user_id);

            if !connected {
                p.status = PlayerStatus::Disconnected.into();
            }
        }
    }

    pub fn run(
        &mut self,
        socket_pool: Arc<SocketPool>,
        thread_pool: Arc<ThreadPool>,
        rx: Arc<Mutex<Receiver<GameChannelMessage>>>,
        tx: Arc<RwLock<Sender<GameChannelMessage>>>,
    ) -> Result<(), &str> {
        self.verify_connections(&socket_pool);
        self.process_disconnected_players();

        let players_count = self.player_state.players.len();

        if players_count < 2 {
            return Err("Not enough players to start a new game");
        }
        // TODO: think about merging it with start_next_cylce function
        let game_states: Vec<ClientState> = self
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
                                let mut payload = PlayerActionRequest::default();
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
                    let result: Result<PlayerActionRequest, ReadMessageError> =
                        clone_s_pool.read_client_message::<PlayerActionRequest>(user_id);
                    clone_tx
                        .read()
                        .unwrap()
                        .send(GameChannelMessage::SocketSource(
                            SocketSourceMessage::PlayerActionRequest(result),
                        ))
                        .unwrap();
                });
            }

            loop {
                let message = rx.lock().unwrap().recv().unwrap();
                match message {
                    GameChannelMessage::SocketSource(r) => match r {
                        SocketSourceMessage::PlayerActionRequest(p) => match p {
                            Ok(m) => {
                                let game_status = self.update_game_state(&socket_pool, Ok(m));
                                if game_status == GameStatus::WaitingForPlayers || game_status == GameStatus::None  {
                                    break 'outer_loop;
                                } else {
                                    continue 'outer_loop;
                                }
                            }
                            Err(e) => {
                                let error: PlayerActionRequestError = match e {
                                    ReadMessageError::Disconnected => {
                                        PlayerActionRequestError::Disconnected {
                                            id: self.dealer.get_next_player_id(
                                                &mut self.game_state,
                                                &mut self.player_state,
                                            ),
                                            lobby_id: self.lobby_id,
                                        }
                                    }
                                    ReadMessageError::Iddle => PlayerActionRequestError::Iddle {
                                        id: self.dealer.get_next_player_id(
                                            &mut self.game_state,
                                            &mut self.player_state,
                                        ),
                                        lobby_id: self.lobby_id,
                                    },
                                };
                                let game_status = self.update_game_state(&socket_pool, Err(error));
                                if game_status == GameStatus::WaitingForPlayers || game_status == GameStatus::None  {
                                    break 'outer_loop;
                                } else {
                                    continue 'outer_loop;
                                }
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
                        let game_status = self.update_game_state(&socket_pool, Ok(m));
                        if game_status == GameStatus::WaitingForPlayers || game_status == GameStatus::None  {
                            break 'outer_loop;
                        } else {
                            continue 'outer_loop;
                        }
                    }
                };
            }
        }
        return Ok(());
    }
}
