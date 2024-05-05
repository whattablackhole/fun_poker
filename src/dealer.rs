use pokereval_cactus::card::Card as PCard;
use pokereval_cactus::evaluator;
use std::collections::{BTreeMap, HashMap};

use crate::{
    card::CardDeck,
    protos::{
        card::CardPair,
        client_state::{GameStatus, Street, StreetStatus},
        player::{ActionType, Player, PlayerAction, PlayerPayload},
    },
};

pub struct Dealer {
    lobby_id: i32,
    game_state: GameState,
    deck_state: DeckState,
    player_state: PlayerState,
}

struct PlayerState {
    players: Vec<Player>,
    bank_map: HashMap<i32, i32>,
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

    pub fn update_player_bank(&mut self, value: i32, id: &i32) {
        let bank = self.bank_map.get_mut(id).unwrap();

        *bank += value;
    }
}

// it happens that to get deck we need duplicate field: deckstate.deck.deck
// TODO: refactor
pub struct DeckState {
    deck: CardDeck,
}

impl DeckState {
    pub fn reset(&mut self) {
        self.deck = CardDeck::new_random();
    }

    pub fn new(deck: CardDeck) -> DeckState {
        DeckState { deck }
    }
}
struct GameState {
    status: GameStatus,
    last_action: PlayerAction,
    next_player_index: i32,
    next_button_player_index: i32,
    street: Street,
    bank: i32,
    big_blind: i32,
    raise_amount: i32,
    raiser_index: Option<i32>,
}

impl GameState {
    pub fn reset(&mut self) {
        *self = GameState::new();
    }
    pub fn new() -> GameState {
        GameState {
            last_action: PlayerAction {
                action_type: ActionType::Empty.into(),
                bet: 0,
            },
            status: GameStatus::None,
            next_player_index: 0,
            // TODO: add calculations
            next_button_player_index: 0,
            street: Street {
                street_status: StreetStatus::Preflop.into(),
                cards: Vec::new(),
            },
            bank: 0,
            // define big blind from game settings
            big_blind: 10,
            raise_amount: 0,
            raiser_index: None,
        }
    }
}

#[derive(Debug)]
pub struct ClientGameState {
    pub next_player_id: i32,
    pub lobby_id: i32,
    pub street: Street,
    pub game_status: GameStatus,
    pub latest_winners: Vec<Player>,
    pub players: Vec<Player>,
}
impl Iterator for StreetStatus {
    type Item = StreetStatus;

    fn next(&mut self) -> Option<Self::Item> {
        use StreetStatus::*;
        match *self {
            Self::Preflop => {
                *self = Flop;
                Some(Preflop)
            }
            Self::Flop => {
                *self = Turn;
                Some(Flop)
            }
            Self::Turn => {
                *self = River;
                Some(Turn)
            }
            Self::River => {
                *self = Preflop;
                Some(River)
            }
        }
    }
}

impl Dealer {
    // TODO: implement game settings for bank size etc..
    pub fn new(lobby_id: i32, players: Vec<Player>) -> Dealer {
        Dealer {
            deck_state: DeckState {
                deck: CardDeck::new_random(),
            },
            game_state: GameState::new(),
            lobby_id: lobby_id,
            player_state: PlayerState::from(players, 1000),
        }
    }

    fn deal_cards(&mut self) {
        if self.game_state.street.street_status == StreetStatus::Preflop as i32 {
            for player in self.player_state.players.iter_mut() {
                let c1 = self.deck_state.deck.cards.pop_front().unwrap();
                let c2 = self.deck_state.deck.cards.pop_front().unwrap();
                player.cards = Some(CardPair {
                    card1: Some(c1),
                    card2: Some(c2),
                })
            }
        } else {
            panic!("Can't deal cards on other street!");
        }
    }

    pub fn start_new_table_loop(&mut self) -> ClientGameState {
        // self = self is not needed?
        self.deal_cards();
        self.game_state.status = GameStatus::Active;
        // TODO: implement new for Street... and others structs
        ClientGameState {
            players: self.player_state.players.clone(),
            game_status: self.game_state.status,
            next_player_id: self
                .player_state
                .players
                .get(self.game_state.next_player_index as usize)
                .unwrap()
                .user_id,
            street: Street {
                street_status: self.game_state.street.street_status,
                cards: self.game_state.street.cards.clone(),
            },
            lobby_id: self.lobby_id,
            latest_winners: Vec::new(),
        }
    }

    fn next_game_loop(&mut self) -> ClientGameState {
        self.to_default_state();
        let state = self.start_new_table_loop();
        state
    }

    fn to_default_state(&mut self) {
        self.deck_state.reset();
        self.player_state
            .players
            .iter_mut()
            .for_each(|p| p.bet_in_current_seed = 0);
        self.game_state.reset();
    }

    fn get_client_game_state(&mut self) -> ClientGameState {
        let state = ClientGameState {
            players: self.player_state.players.clone(),
            game_status: self.game_state.status,
            next_player_id: self
                .player_state
                .players
                .get(self.game_state.next_player_index as usize)
                .unwrap()
                .user_id,
            street: Street {
                street_status: self.game_state.street.street_status,
                cards: self.game_state.street.cards.clone(),
            },
            lobby_id: self.lobby_id,
            latest_winners: Vec::new(),
        };
        state
    }

    fn calculate_winner(&mut self) -> Vec<Player> {
        let mut win_result: Vec<Player> = Vec::new();
        let mut winners_map: BTreeMap<i32, Vec<&mut Player>> = BTreeMap::new();

        for player in self.player_state.players.iter_mut() {
            // TODO: will be great to write own 2+2 evaluator;
            let eval = evaluator::Evaluator::new();
            let street: Vec<i32> = self
                .game_state
                .street
                .cards
                .iter()
                .map(|card| PCard::new(card.to_string()))
                .collect();

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
                street,
            );

            if winners_map.contains_key(&result) {
                let winners = winners_map.get_mut(&result).unwrap();
                winners.push(player);
            } else {
                winners_map.insert(result, vec![player]);
            }
        }
        for winners in winners_map.values_mut() {
            let sum_of_winners_bets: i32 = winners.iter().map(|w| w.bet_in_current_seed).sum();
            let winners_amount = winners.len() as i32;
            for w in winners {
                let win_points = w.bet_in_current_seed
                    + ((self.game_state.bank - sum_of_winners_bets) / winners_amount);
                w.bank += win_points;

                let remainder = if self.game_state.bank != 0 {
                    win_points % self.game_state.bank
                } else {
                    0
                };
                if (remainder > 0) {
                    self.game_state.bank -= remainder;
                } else {
                    self.game_state.bank = 0;
                }

                win_result.push((*w).clone());
            }
            if self.game_state.bank == 0 {
                break;
            }
        }
        win_result
    }

    pub fn can_determine_winner(&self) -> Option<bool> {
        let mut remaining_players = self.player_state.players.iter().filter(|p| {
            p.action.is_some() && p.action.as_ref().unwrap().action_type != ActionType::Fold.into()
        });

        match (remaining_players.next(), remaining_players.next()) {
            (Some(_), None) => Some(true),
            _ => None,
        }
    }

    fn update_next_player_index(&mut self, reset: bool) {
        if reset {
            self.game_state.next_player_index = 0;
        } else {
            self.icrement_player_index();
        }
    }

    // TODO: refactor due to kinda bad utilitarity
    fn icrement_player_index(&mut self) {
        self.game_state.next_player_index =
            (self.game_state.next_player_index + 1) % self.player_state.players.len() as i32;
    }

    fn deal_new_street(&mut self, street_status: i32) {
        self.game_state.street.street_status = street_status;
        if street_status == StreetStatus::Flop as i32 {
            // maybe not needed, TODO: refactor
            self.game_state.street.cards.clear();

            self.game_state
                .street
                .cards
                .push(self.deck_state.deck.cards.pop_front().unwrap());
            self.game_state
                .street
                .cards
                .push(self.deck_state.deck.cards.pop_front().unwrap());
            self.game_state
                .street
                .cards
                .push(self.deck_state.deck.cards.pop_front().unwrap());
        } else {
            self.game_state
                .street
                .cards
                .push(self.deck_state.deck.cards.pop_front().unwrap());
        }
    }

    fn is_last_turn(&mut self) -> bool {
        let mut is_last_player_turn = self.game_state.next_player_index == 0;

        // TODO: change check to boolean flag
        if self.has_active_raiser() {
            is_last_player_turn = false;
        }

        is_last_player_turn
    }
    fn has_active_raiser(&self) -> bool {
        self.game_state.raiser_index.is_some()
    }

    pub fn update_game_state(&mut self, payload: PlayerPayload) -> ClientGameState {
        if payload.lobby_id != self.lobby_id {
            panic!("Wrong lobby id in payload");
        }

        assert!(self
            .player_state
            .players
            .iter()
            .any(|p| { p.user_id == payload.player_id }));

        // TODO: Reset player action on new game loop

        match ActionType::try_from(payload.action.as_ref().unwrap().action_type).unwrap() {
            ActionType::Fold => {
                self.process_fold_action(payload.player_id);
                let result = self.can_determine_winner();
                if result.is_some() && result.unwrap() == true {
                    // TODO: set_winner
                    let winners = self.calculate_winner();
                    // in future: add checks when players are elimanated;
                    let mut state = self.next_game_loop();
                    state.latest_winners = winners;
                    return state;
                }
            }
            ActionType::Call => {
                self.process_call_action(payload.player_id, payload.action.unwrap().bet);
            }
            ActionType::Raise => {
                self.process_raise_action(payload.player_id, payload.action.unwrap().bet);
            }
            ActionType::Empty => println!("Empty"),
        }

        self.update_next_player_index(false);

        if self.has_active_raiser()
            && self.game_state.next_player_index == self.game_state.raiser_index.unwrap()
        {
            self.game_state.raiser_index = None;
            self.update_next_player_index(true);
        }

        if self.is_last_turn() {
            let curr_street = self.game_state.street.street_status;

            if curr_street == StreetStatus::River as i32 {
                let winners: Vec<Player> = self.calculate_winner();
                let mut state = self.next_game_loop();
                // TODO: set_winner
                state.latest_winners = winners;
                return state;
            }
            self.update_next_player_index(true);
            self.deal_new_street(curr_street + 1);
        }

        self.get_client_game_state()
    }

    fn process_fold_action(&mut self, player_id: i32) {
        let player = self
            .player_state
            .players
            .iter_mut()
            .find(|p| p.user_id == player_id)
            .expect("user not found");

        if player.action.as_ref().unwrap().action_type == ActionType::Fold.into() {
            println!("Player has folded already and cannot fold again");
            return;
        }
        player.action = Some(PlayerAction {
            action_type: ActionType::Fold.into(),
            bet: 0,
        });
        self.game_state.last_action = PlayerAction {
            action_type: ActionType::Fold.into(),
            bet: 0,
        };
    }

    fn process_call_action(&mut self, player_id: i32, bet_amount: i32) {
        let player = self
            .player_state
            .players
            .iter_mut()
            .find(|p| p.user_id == player_id)
            .expect("user not found");

        if player.action.as_ref().unwrap().action_type == ActionType::Fold.into() {
            println!("Player has folded already and cannot call");
            return;
        }

        if player.bank < bet_amount {
            println!("Player does not have enough points!");
            return;
        }
        // winners contains non-updated field bet_in_current_seed
        player.bet_in_current_seed += bet_amount;
        player.bank -= bet_amount;
        self.game_state.last_action = PlayerAction {
            action_type: ActionType::Call.into(),
            bet: bet_amount,
        };
        player.action = Some(PlayerAction {
            action_type: ActionType::Call.into(),
            bet: bet_amount,
        });
        self.game_state.bank += bet_amount;
    }

    fn process_raise_action(&mut self, player_id: i32, bet_amount: i32) {
        if let Some(index) = self
            .player_state
            .players
            .iter()
            .position(|p| p.user_id == player_id)
        {
            self.game_state.raiser_index = Some(index as i32);

            let player: &mut Player = &mut self.player_state.players[index];
            if player.action.as_ref().unwrap().action_type == ActionType::Fold.into() {
                println!("Player has folded already and cannot raise");
                return;
            }

            if player.bank < bet_amount {
                println!("Player does not have enough points!");
                return;
            }

            let valid_min_raise = if self.game_state.raise_amount == 0 {
                self.game_state.big_blind * 2
            } else {
                self.game_state.last_action.bet + self.game_state.raise_amount
            };

            if bet_amount < valid_min_raise {
                println!("Player raise amount is not valid");
                return;
            }

            player.bet_in_current_seed += bet_amount;
            player.bank -= bet_amount;
            self.game_state.bank += bet_amount;
            self.game_state.raise_amount = if self.game_state.raise_amount == 0 {
                bet_amount - self.game_state.big_blind
            } else {
                bet_amount - self.game_state.raise_amount
            };
            self.game_state.last_action = PlayerAction {
                action_type: ActionType::Raise.into(),
                bet: bet_amount,
            };
            player.action = Some(PlayerAction {
                action_type: ActionType::Raise.into(),
                bet: bet_amount,
            });
        } else {
            println!("the player with the given ID is not found");
        }
    }
}
