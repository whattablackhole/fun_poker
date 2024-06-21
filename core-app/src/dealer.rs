use pokereval_cactus::card::Card as PCard;
use pokereval_cactus::evaluator;
use rand::Rng;
use std::collections::BTreeMap;

use crate::{
    game::{DeckState, GameState, KeyPositions, PlayerState},
    protos::{
        card::CardPair,
        client_state::ClientState,
        game_state::{
            Action, ActionType, GameStatus, PlayerCards, ShowdownOutcome, Street, StreetStatus,
            Winner,
        },
        google::protobuf::{BoolValue, Int32Value},
        player::{Player, PlayerStatus},
        requests::PlayerActionRequest,
    },
    responses::PlayerActionRequestError,
};
pub struct Dealer {
    lobby_id: i32,
}

#[derive(Debug)]
pub struct UpdatedState {
    pub client_states: Vec<ClientState>,
    pub is_ready_for_next_hand: bool,
    pub should_complete_game_cycle_automatically: bool,
}

impl StreetStatus {
    pub fn len() -> i32 {
        4
    }
}

impl Dealer {
    // STATIC PUBLIC --------------------------------------------------------

    pub fn new(lobby_id: i32) -> Dealer {
        Dealer { lobby_id }
    }

    // PUBLIC --------------------------------------------------

    pub fn start_new_game(
        &self,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
        deck_state: &mut DeckState,
    ) -> Result<Vec<ClientState>, &str> {
        let button_index = rand::thread_rng().gen_range(0..player_state.players.len());

        let positions =
            self.calculate_key_positions(button_index, player_state.players.len() as i32);

        game_state.positions = positions;
        game_state.status = GameStatus::Active;

        // TODO: let player decide whether he ready or not
        player_state
            .players
            .iter_mut()
            .for_each(|p| p.status = PlayerStatus::Ready.into());

        self.deal_cards(deck_state, player_state, game_state);

        self.setup_blinds(
            player_state,
            game_state.positions.small_blind_index.unwrap(),
            game_state.positions.big_blind_index.unwrap(),
            game_state.big_blind,
            game_state,
        );

        let state = self.create_client_states(game_state, player_state);
        Ok(state)
    }

    pub fn get_client_state(
        &self,
        player_id: &i32,
        game_state: &GameState,
        player_state: &PlayerState,
    ) -> ClientState {
        let player = player_state
            .players
            .iter()
            .find(|p| p.user_id == *player_id)
            .unwrap();
        self.create_client_state(player, game_state, player_state)
    }

    pub fn get_client_states(
        &self,
        game_state: &GameState,
        player_state: &PlayerState,
    ) -> Vec<ClientState> {
        self.create_client_states(game_state, player_state)
    }

    pub fn complete_game_cycle_automatically(
        &self,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
        deck_state: &mut DeckState,
    ) -> UpdatedState {
        game_state.street.street_status = StreetStatus::River.into();
        while game_state.street.cards.len() != 5 {
            game_state
                .street
                .cards
                .push(deck_state.deck.cards.pop_front().unwrap());
        }
        let showdown_outcome = self.calculate_winner(true, game_state, player_state);

        game_state.showdown_outcome = Some(showdown_outcome);

        self.mark_eliminated_players(player_state);
        let states = self.create_client_states(game_state, player_state);

        UpdatedState {
            client_states: states,
            should_complete_game_cycle_automatically: false,
            is_ready_for_next_hand: true,
        }
    }

    pub fn get_next_player_id(
        &self,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
    ) -> i32 {
        self.get_player_id_by_index(
            game_state.positions.curr_player_index.unwrap(),
            player_state,
        )
    }

    pub fn get_next_player<'a>(
        &self,
        game_state: &mut GameState,
        player_state: &'a mut PlayerState,
    ) -> &'a Player {
        &player_state.players[game_state.positions.curr_player_index.unwrap()]
    }

    pub fn handle_disconnect(
        &self,
        player_id: i32,
        lobby_id: i32,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
    ) -> PlayerActionRequest {
        let player = player_state
            .players
            .iter_mut()
            .find(|p| p.user_id == player_id)
            .unwrap();

        player.status = PlayerStatus::Disconnected.into();

        let action = Action {
            action_type: ActionType::Fold.into(),
            bet: 0,
            player_id,
            street_status: None,
        };
        PlayerActionRequest {
            action: Some(action),
            lobby_id,
            player_id,
        }
    }

    pub fn handle_idle(
        &self,
        player_id: i32,
        lobby_id: i32,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
    ) -> PlayerActionRequest {
        let player = player_state
            .players
            .iter_mut()
            .find(|p| p.user_id == player_id)
            .unwrap();

        player.status = PlayerStatus::SitOut.into();

        let action = Action {
            action_type: ActionType::Fold.into(),
            bet: 0,
            player_id,
            street_status: None,
        };
        PlayerActionRequest {
            action: Some(action),
            lobby_id,
            player_id,
        }
    }

    pub fn update_game_state(
        &self,
        payload: Result<PlayerActionRequest, PlayerActionRequestError>,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
        deck_state: &mut DeckState,
    ) -> UpdatedState {
        let payload = match payload {
            Ok(p) => p.clone(),
            Err(e) => match e {
                PlayerActionRequestError::Disconnected { id, lobby_id } => {
                    self.handle_disconnect(id, lobby_id, game_state, player_state)
                }
                PlayerActionRequestError::Iddle { id, lobby_id } => {
                    self.handle_idle(id, lobby_id, game_state, player_state)
                }
            },
        };

        if payload.lobby_id != self.lobby_id {
            panic!("Wrong lobby id in payload");
        }

        assert!(player_state
            .players
            .iter()
            .any(|p| { p.user_id == payload.player_id }));

        let action_type =
            ActionType::try_from(payload.action.as_ref().unwrap().action_type).unwrap();
        match action_type {
            ActionType::Fold => {
                self.process_fold_action(payload.player_id, game_state, player_state);
                let result = self.can_determine_winner(player_state);
                if result.is_some() && result.unwrap() == true {
                    let showdown_outcome = self.calculate_winner(false, game_state, player_state);
                    game_state.showdown_outcome = Some(showdown_outcome);
                    self.mark_eliminated_players(player_state);
                    let states = self.create_client_states(game_state, player_state);
                    return UpdatedState {
                        client_states: states,
                        is_ready_for_next_hand: true,
                        should_complete_game_cycle_automatically: false,
                    };
                }
            }
            ActionType::Call => {
                self.process_call_action(
                    payload.player_id,
                    payload.action.as_ref().unwrap().bet,
                    game_state,
                    player_state,
                );
            }
            ActionType::Raise => {
                self.process_raise_action(
                    payload.player_id,
                    payload.action.as_ref().unwrap().bet,
                    game_state,
                    player_state,
                );
            }
            ActionType::Check => {
                self.process_check_action(
                    payload.player_id,
                    payload.action.as_ref().unwrap().bet,
                    game_state,
                    player_state,
                );
            }
            _ => println!("Illegal move!"),
        }
        // TODO: in case player has made invalid action, we have to wait until his timer ends and proceed it as fold

        let last_active = self
            .calculate_last_active_player_index(&player_state, game_state)
            .unwrap();
        if last_active == game_state.positions.curr_player_index.unwrap() {
            game_state.raiser_index = None;
            let curr_street = game_state.street.street_status;
            if curr_street == StreetStatus::River as i32 {
                let showdown_outcome = self.calculate_winner(false, game_state, player_state);
                self.mark_eliminated_players(player_state);
                // TODO: set showdown in seperate fn
                game_state.showdown_outcome = Some(showdown_outcome);
                let states = self.create_client_states(game_state, player_state);
                return UpdatedState {
                    client_states: states,
                    is_ready_for_next_hand: true,
                    should_complete_game_cycle_automatically: false,
                };
            }
            if self.should_complete_game_cycle_automatically(player_state) == true {
                self.mark_eliminated_players(player_state);
                game_state.showdown_outcome = None;
                let states = self.create_client_states(game_state, player_state);
                return UpdatedState {
                    client_states: states,
                    is_ready_for_next_hand: false,
                    should_complete_game_cycle_automatically: true,
                };
            }
            self.next_street(game_state, deck_state, player_state);
        } else {
            self.next_player(&player_state, game_state);
        }
        game_state.showdown_outcome = None;
        let states = self.create_client_states(game_state, player_state);
        return UpdatedState {
            client_states: states,
            is_ready_for_next_hand: false,
            should_complete_game_cycle_automatically: false,
        };
    }

    // PRIVATE ----------------------------------------------------------------------------------------

    fn calculate_key_positions(
        &self,
        init_button_index: usize,
        players_amount: i32,
    ) -> KeyPositions {
        let is_heads_up = self.is_heads_up(players_amount);

        let small_blind_index = if is_heads_up {
            init_button_index
        } else {
            self.get_loop_incremented_index(init_button_index, players_amount)
        };

        let big_blind_index = self.get_loop_incremented_index(small_blind_index, players_amount);

        let curr_player_index = if is_heads_up {
            self.get_loop_incremented_index(init_button_index, players_amount)
        } else {
            self.get_loop_incremented_index(big_blind_index, players_amount)
        };

        KeyPositions {
            small_blind_index: Some(small_blind_index),
            big_blind_index: Some(big_blind_index),
            curr_player_index: Some(curr_player_index),
            button_index: Some(init_button_index),
        }
    }

    fn is_heads_up(&self, players_amount: i32) -> bool {
        players_amount == 2
    }

    fn filter_player_cards(&self, mut player: Player) -> Player {
        player.cards = None;
        player
    }

    fn is_all_in(&self, player: &Player, bet_amount: i32) -> bool {
        player.bank == bet_amount
    }

    fn calculate_valid_call_amount(
        &self,
        p: &Player,
        game_state: &GameState,
        player_state: &PlayerState,
    ) -> i32 {
        if game_state.raiser_index.is_some() {
            let min_call_amount = player_state.players[game_state.raiser_index.unwrap() as usize]
                .bet_in_current_seed
                - p.bet_in_current_seed;
            if min_call_amount > p.bank {
                // all-in is valid call even it less than min call amount
                p.bank
            } else {
                min_call_amount
            }
        } else {
            // we can't use big blind value here as players bank amount can be less than big blind
            (game_state.biggest_bet_on_curr_street - p.bet_in_current_seed).max(0)
        }
    }

    fn calculate_min_raise(&self, _p: &Player, game_state: &GameState) -> i32 {
        // do we need to return player's bank if he cannot afford min raise?
        // 1. his bank < min raise && his bank < curr_biggest_bet = it's not a raise but allin  call
        // 2. his bank < min raise && his bank > curr_biggest_bet = valid raise all in
        // let min_raise = game_state.biggest_bet_on_curr_street + game_state.raise_amount;
        // if min_raise > p.bank { p.bank } else { min_raise }
        if game_state.raiser_index.is_some() {
            return game_state.biggest_bet_on_curr_street + game_state.raise_amount;
        } else {
            return if game_state.street.street_status() == StreetStatus::Preflop {
                game_state.big_blind * 2 // FIX: or the biggest bank on the table except self
            } else {
                game_state.big_blind
            };
        }
    }

    fn get_player_id_by_index(&self, index: usize, player_state: &PlayerState) -> i32 {
        player_state.players.get(index as usize).unwrap().user_id
    }

    fn get_filtered_players(
        &self,
        _game_state: &GameState,
        player_state: &PlayerState,
    ) -> Vec<Player> {
        let players: Vec<Player> = player_state
            .players
            .iter()
            .map(|player| self.filter_player_cards(player.clone()))
            .collect();
        players
    }

    fn create_client_state(
        &self,
        p: &Player,
        game_state: &GameState,
        player_state: &PlayerState,
    ) -> ClientState {
        let filtered_players = self.get_filtered_players(game_state, player_state);

        // TODO: think about using optional fields in game_state instead
        if game_state.status == GameStatus::WaitingForPlayers {
            return ClientState {
                player_id: p.user_id,
                cards: None,
                amount_to_call: None,
                min_amount_to_raise: None,
                action_history: Vec::new(),
                can_raise: None,
                curr_big_blind_id: None,
                curr_button_id: None,
                curr_player_id: None,
                curr_small_blind_id: None,
                game_status: game_state.status.clone().into(),
                lobby_id: self.lobby_id,
                players: filtered_players,
                showdown_outcome: None,
                street: None,
            };
        }

        ClientState {
            player_id: p.user_id,
            cards: p.cards.clone(),
            amount_to_call: Some(Int32Value {
                value: self.calculate_valid_call_amount(p, game_state, player_state),
            }),
            min_amount_to_raise: Some(Int32Value {
                value: self.calculate_min_raise(p, game_state),
            }),
            can_raise: Some(BoolValue {
                value: self.can_raise(p, game_state, player_state),
            }),
            players: filtered_players.clone(),
            game_status: game_state.status.into(),
            curr_player_id: Some(Int32Value {
                value: self.get_player_id_by_index(
                    game_state.positions.curr_player_index.unwrap(),
                    player_state,
                ),
            }),
            curr_button_id: Some(Int32Value {
                value: self.get_player_id_by_index(
                    game_state.positions.button_index.unwrap(),
                    player_state,
                ),
            }),
            curr_big_blind_id: Some(Int32Value {
                value: self.get_player_id_by_index(
                    game_state.positions.big_blind_index.unwrap(),
                    player_state,
                ),
            }),
            curr_small_blind_id: Some(Int32Value {
                value: self.get_player_id_by_index(
                    game_state.positions.small_blind_index.unwrap(),
                    player_state,
                ),
            }),
            street: Some(game_state.street.clone()),
            lobby_id: self.lobby_id,
            showdown_outcome: game_state.showdown_outcome.clone(),
            action_history: game_state.action_history.clone(),
        }
    }

    // TODO: Send delta updates in future
    fn create_client_states(
        &self,
        game_state: &GameState,
        player_state: &PlayerState,
    ) -> Vec<ClientState> {
        let states = player_state
            .players
            .iter()
            .filter(|p| p.status() != PlayerStatus::Disconnected && !p.is_bot)
            .map(|p| self.create_client_state(p, game_state, player_state))
            .collect();

        states
    }

    fn calculate_winner(
        &self,
        is_manual_street: bool,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
    ) -> ShowdownOutcome {
        let mut winners: Vec<Winner> = Vec::new();
        let mut winners_map: BTreeMap<i32, Vec<&mut Player>> = BTreeMap::new();

        let players: Vec<&mut Player> = player_state
            .players
            .iter_mut()
            .filter(|p| {
                // is some for debug purposes
                // remove when not needed
                p.action.is_some() && p.action.as_ref().unwrap().action_type() != ActionType::Fold
            })
            .collect();
        let mut players_cards = Vec::new();
        let street: Vec<i32> = game_state
            .street
            .cards
            .iter()
            .map(|card| PCard::new(card.to_string()))
            .collect();
        if players.len() == 1 {
            winners_map.insert(1, players);
        } else {
            for player in players {
                // TODO: will be great to write own 2+2 evaluator;
                let eval = evaluator::Evaluator::new();
                players_cards.push(PlayerCards {
                    player_id: player.user_id,
                    cards: player.cards.clone(),
                });

                let result = eval.evaluate(
                    vec![
                        PCard::new(
                            player
                                .cards
                                .as_ref()
                                .unwrap()
                                .card1
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        ),
                        PCard::new(
                            player
                                .cards
                                .as_ref()
                                .unwrap()
                                .card2
                                .as_ref()
                                .unwrap()
                                .to_string(),
                        ),
                    ],
                    street.clone(),
                );

                if winners_map.contains_key(&result) {
                    let w = winners_map.get_mut(&result).unwrap();
                    w.push(player);
                } else {
                    winners_map.insert(result, vec![player]);
                }
            }
        }
        for win_players in winners_map.values_mut() {
            let sum_of_winners_bets: i32 = win_players.iter().map(|w| w.bet_in_current_seed).sum();
            let winners_amount = win_players.len() as i32;
            let bank = game_state.game_bank;
            for w in win_players {
                let win_points =
                    w.bet_in_current_seed + ((bank - sum_of_winners_bets) / winners_amount);
                w.bank += win_points;

                let remainder = if game_state.game_bank != 0 {
                    win_points % game_state.game_bank
                } else {
                    0
                };
                if remainder > 0 {
                    game_state.game_bank -= remainder;
                } else {
                    game_state.game_bank = 0;
                }

                winners.push(Winner {
                    player_id: w.user_id,
                    win_amout: win_points,
                });
            }
            if game_state.game_bank == 0 {
                break;
            }
        }

        ShowdownOutcome {
            players_cards,
            street_history: Some(game_state.street.clone()),
            winners,
            process_flop_automatically: is_manual_street,
        }
    }

    fn can_determine_winner(&self, player_state: &PlayerState) -> Option<bool> {
        // TODO: refactor to return Player and skip winner evaluation
        let mut remaining_players = player_state.players.iter().filter(|p| {
            p.action.is_some() && p.action.as_ref().unwrap().action_type() != ActionType::Fold
        });

        match (remaining_players.next(), remaining_players.next()) {
            (Some(_), None) => Some(true),
            _ => None,
        }
    }

    fn mark_eliminated_players(&self, player_state: &mut PlayerState) {
        player_state.players.iter_mut().for_each(|p| {
            if p.bank == 0 && p.bet_in_current_seed == 0 {
                p.status = PlayerStatus::Eliminated.into();
            };
        });
    }

    fn should_complete_game_cycle_automatically(&self, player_state: &PlayerState) -> bool {
        let active_players_with_non_zero_bank_count = player_state
            .players
            .iter()
            .filter(|p| {
                (p.action.is_none() || p.action.as_ref().unwrap().action_type() != ActionType::Fold)
                    && p.bank > 0
            })
            .count();

        active_players_with_non_zero_bank_count < 2
    }

    fn process_fold_action(
        &self,
        player_id: i32,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
    ) {
        let player: &mut Player = player_state
            .players
            .iter_mut()
            .find(|p| p.user_id == player_id)
            .expect("user not found");

        if player.action.as_ref().is_some()
            && player.action.as_ref().unwrap().action_type() == ActionType::Fold
        {
            println!("Player has folded already and cannot fold again");
            return;
        }
        let action = Action {
            action_type: ActionType::Fold.into(),
            bet: 0,
            player_id: player.user_id,
            street_status: Some(game_state.street.street_status),
        };

        player.action = Some(action.clone());
        game_state.action_history.push(action.clone());
    }

    fn process_check_action(
        &self,
        player_id: i32,
        bet_amount: i32,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
    ) {
        let valid_call_amount = {
            let player = player_state
                .players
                .iter()
                .find(|p| p.user_id == player_id)
                .expect("user not found");

            self.calculate_valid_call_amount(player, game_state, player_state)
        };

        if bet_amount != 0 || valid_call_amount != 0 {
            println!("Bet amount is not valid for check");
            return;
        }

        let player = player_state
            .players
            .iter_mut()
            .find(|p| p.user_id == player_id)
            .expect("user not found");

        // winners contains non-updated field bet_in_current_seed
        // it may be fixed in winner calculation stage

        let action = Action {
            action_type: ActionType::Check.into(),
            bet: 0,
            player_id: player.user_id,
            street_status: game_state.street.street_status.into(),
        };

        player.action = Some(action.clone());
        game_state.action_history.push(action.clone());
    }

    fn process_call_action(
        &self,
        player_id: i32,
        bet_amount: i32,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
    ) {
        // TODO: Refactor repeated code using RefCell?
        let valid_call_amount = {
            let player = player_state
                .players
                .iter()
                .find(|p| p.user_id == player_id)
                .expect("user not found");

            self.calculate_valid_call_amount(player, game_state, player_state)
        };

        if bet_amount != valid_call_amount {
            println!("Bet amount is not valid for minimum call");
            return;
        }

        let player = player_state
            .players
            .iter_mut()
            .find(|p| p.user_id == player_id)
            .expect("user not found");

        if player.bank < bet_amount {
            println!("Player does not have enough points!");
            return;
        }

        // winners contains non-updated field bet_in_current_seed
        // it may be fixed in winner calculation stage
        player.bet_in_current_seed += bet_amount;
        player.bank -= bet_amount;

        let action = Action {
            action_type: ActionType::Call.into(),
            bet: bet_amount,
            player_id: player.user_id,
            street_status: game_state.street.street_status.into(),
        };
        player.action = Some(action.clone());
        game_state.action_history.push(action.clone());

        game_state.game_bank += bet_amount;
    }

    pub fn setup_next_cycle(
        &self,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
        deck_state: &mut DeckState,
    ) -> UpdatedState {
        self.setup_next_hand(player_state, deck_state, game_state);
        // TODO: refactor showdown automation cycle handling;
        game_state.showdown_outcome = None;
        let states = self.create_client_states(game_state, player_state);
        return UpdatedState {
            client_states: states,
            is_ready_for_next_hand: false,
            should_complete_game_cycle_automatically: self
                .should_complete_game_cycle_automatically(player_state),
        };
    }

    // If a player decides to go "all in" by betting all of their chips,
    // but their bet is smaller than the minimum raise allowed,
    // but bigger than the last biggest bet made by another player,
    // then the players who have already placed bets before them cannot raise again unless
    // there's another player who raises after the all-in bet.
    fn can_raise(
        &self,
        player: &Player,
        game_state: &GameState,
        player_state: &PlayerState,
    ) -> bool {
        if let Some(index) = game_state.raiser_index {
            let raiser_bank = player_state.players[index as usize].bank;
            return raiser_bank != 0 || raiser_bank == 0 && player.action.as_ref().is_none();
        }
        player.bank > 0
    }

    fn process_raise_action(
        &self,
        player_id: i32,
        bet_amount: i32,
        game_state: &mut GameState,
        player_state: &mut PlayerState,
    ) {
        if let Some(index) = player_state
            .players
            .iter()
            .position(|p| p.user_id == player_id)
        {
            let player: &Player = &player_state.players[index];

            let valid_min_raise = self.calculate_min_raise(player, game_state);

            let bet_is_all_in = self.is_all_in(player, bet_amount);

            if self.can_raise(player, game_state, player_state) == false {
                println!("Player not eligable to raise");
                return;
            }

            if bet_amount < valid_min_raise
                && (bet_is_all_in == false && bet_amount < game_state.biggest_bet_on_curr_street)
            {
                println!("Player raise amount is not valid");
                return;
            }

            if player.bank < bet_amount {
                println!("Player does not have enough points!");
                return;
            }

            let player = &mut player_state.players[index];

            game_state.raiser_index = Some(index);

            player.bet_in_current_seed += bet_amount;
            player.bank -= bet_amount;
            game_state.game_bank += bet_amount;

            if bet_amount > game_state.biggest_bet_on_curr_street {
                game_state.raise_amount = bet_amount - game_state.biggest_bet_on_curr_street;
                game_state.biggest_bet_on_curr_street = bet_amount;
            }

            let action = Action {
                action_type: ActionType::Raise.into(),
                bet: bet_amount,
                player_id: player.user_id,
                street_status: Some(game_state.street.street_status),
            };

            player.action = Some(action.clone());
            game_state.action_history.push(action);
        } else {
            println!("the player with the given ID is not found");
        }
    }

    // REFACTOR

    fn get_loop_incremented_index(&self, index: usize, range: i32) -> usize {
        return (index + 1) % range as usize;
    }

    fn setup_blinds(
        &self,
        player_state: &mut PlayerState,
        small_blind_index: usize,
        big_blind_index: usize,
        blind_size: i32,
        game_state: &mut GameState,
    ) {
        let player = &mut player_state.players[small_blind_index as usize];
        let small_blind_size = blind_size / 2;
        let small_blind_bet_amount = if small_blind_size > player.bank {
            player.bank
        } else {
            small_blind_size
        };
        player.bet_in_current_seed = small_blind_bet_amount;

        game_state.game_bank += small_blind_bet_amount;
        player.bank -= small_blind_bet_amount;
        let action = Action {
            action_type: ActionType::Blind.into(),
            bet: player.bet_in_current_seed,
            player_id: player.user_id,
            street_status: Some(game_state.street.street_status),
        };
        game_state.action_history.push(action.clone());
        player.action = Some(action);

        let player = &mut player_state.players[big_blind_index as usize];
        let big_blind_bet_amount = if blind_size > player.bank {
            player.bank
        } else {
            blind_size
        };
        player.bet_in_current_seed = big_blind_bet_amount;
        game_state.game_bank += big_blind_bet_amount;

        player.bank -= big_blind_bet_amount;
        let action = Action {
            action_type: ActionType::Blind.into(),
            bet: player.bet_in_current_seed,
            player_id: player.user_id,
            street_status: Some(game_state.street.street_status),
        };
        game_state.action_history.push(action.clone());
        player.action = Some(action);
    }

    fn deal_cards(
        &self,
        deck_state: &mut DeckState,
        player_state: &mut PlayerState,
        game_state: &mut GameState,
    ) {
        if game_state.street.street_status == StreetStatus::Preflop as i32 {
            for player in player_state.players.iter_mut() {
                let c1 = deck_state.deck.cards.pop_front().unwrap();
                let c2 = deck_state.deck.cards.pop_front().unwrap();
                player.cards = Some(CardPair {
                    card1: Some(c1),
                    card2: Some(c2),
                })
            }
        } else {
            panic!("Can't deal cards on other street!");
        }
    }

    fn calculate_curr_player_index_on_new_street(
        &self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> Option<usize> {
        let mut new_curr =
            (game_state.positions.button_index.unwrap() + 1) % player_state.players.len();

        for _ in 0..player_state.players.len() {
            if player_state.players[new_curr]
                .action
                .as_ref()
                .unwrap()
                .action_type()
                != ActionType::Fold
            {
                return Some(new_curr);
            } else {
                new_curr = (new_curr + 1) % player_state.players.len() as usize;
            }
        }

        None
    }

    fn get_default_last_player_index(
        &self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> usize {
        if game_state.street.street_status() == StreetStatus::Preflop {
            if self.is_heads_up(player_state.players.len() as i32) {
                game_state.positions.button_index.unwrap()
            } else {
                game_state.positions.big_blind_index.unwrap()
            }
        } else {
            game_state.positions.button_index.unwrap()
        }
    }

    fn calculate_last_active_player_index(
        &self,
        player_state: &PlayerState,
        game_state: &GameState,
    ) -> Option<usize> {
        let mut last_player_index = if let Some(raiser_index) = game_state.raiser_index {
            (raiser_index + player_state.players.len() - 1) % player_state.players.len()
        } else {
            self.get_default_last_player_index(player_state, game_state)
        } as usize;

        for _ in 0..player_state.players.len() {
            let action = player_state.players[last_player_index]
                .action
                .as_ref()
                .unwrap();
            if action.action_type != ActionType::Fold as i32 {
                return Some(last_player_index);
            }
            last_player_index =
                (last_player_index + player_state.players.len() - 1) % player_state.players.len();
        }

        None
    }

    fn next_street(
        &self,
        game_state: &mut GameState,
        deck_state: &mut DeckState,
        player_state: &mut PlayerState,
    ) {
        game_state.street.street_status =
            (game_state.street.street_status + 1) % StreetStatus::len();
        if game_state.street.street_status == StreetStatus::Preflop as i32 {
            self.setup_next_hand(player_state, deck_state, game_state);
            return;
        } else {
            game_state.positions.curr_player_index =
                self.calculate_curr_player_index_on_new_street(&player_state, game_state)
        }

        game_state.raise_amount = 0;
        game_state.raiser_index = None;
        game_state.biggest_bet_on_curr_street = 0;

        if game_state.street.street_status == StreetStatus::Flop as i32 {
            game_state.street.cards.clear();

            game_state
                .street
                .cards
                .push(deck_state.deck.cards.pop_front().unwrap());
            game_state
                .street
                .cards
                .push(deck_state.deck.cards.pop_front().unwrap());
            game_state
                .street
                .cards
                .push(deck_state.deck.cards.pop_front().unwrap());
        } else {
            game_state
                .street
                .cards
                .push(deck_state.deck.cards.pop_front().unwrap());
        }
    }
    // hands?
    fn setup_next_hand(
        &self,
        player_state: &mut PlayerState,
        deck_state: &mut DeckState,
        game_state: &mut GameState,
    ) {
        game_state.street = Street::default();
        deck_state.new_random();
        game_state.action_history = Vec::new();
        self.deal_cards(deck_state, player_state, game_state);
        game_state.biggest_bet_on_curr_street = game_state.big_blind;
        game_state.raise_amount = 0;
        game_state.raiser_index = None;
        player_state.players.iter_mut().for_each(|p| {
            p.bet_in_current_seed = 0;
            p.action = None;
        });
        game_state.game_bank = 0;
        let players_amount = player_state.players.len() as i32;
        let next_button_index = self
            .get_loop_incremented_index(game_state.positions.button_index.unwrap(), players_amount);

        game_state.positions = self.calculate_key_positions(next_button_index, players_amount);
        self.setup_blinds(
            player_state,
            game_state.positions.small_blind_index.unwrap(),
            game_state.positions.big_blind_index.unwrap(),
            game_state.big_blind,
            game_state,
        );
    }

    fn next_player(&self, player_state: &PlayerState, game_state: &mut GameState) {
        let mut curr_next = game_state.positions.curr_player_index.unwrap() as usize;
        let mut is_set = false;

        for _ in 0..player_state.players.len() {
            curr_next = (curr_next + 1) % player_state.players.len();
            if let Some(player) = player_state.players.get(curr_next) {
                if let Some(action) = &player.action {
                    if action.action_type() != ActionType::Fold {
                        game_state.positions.curr_player_index = Some(curr_next);
                        is_set = true;
                        break;
                    }
                }
            }
        }
        if !is_set {
            panic!("Could not find next active player");
        }
    }
}
