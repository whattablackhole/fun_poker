use pokereval_cactus::card::Card as PCard;
use pokereval_cactus::evaluator;
use rand::Rng;
use std::collections::{BTreeMap, HashMap};

use crate::{
    card::CardDeck,
    protos::{
        card::CardPair,
        client_state::{ClientState, GameStatus, Street, StreetStatus},
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

    fn set_up_blinds(&mut self, small_blind_index: i32, big_blind_index: i32, blind_size: i32) {
        // fix if blinds are bigger than players bank
        let player = &mut self.players[small_blind_index as usize];
        let small_blind_size = blind_size / 2;
        player.bet_in_current_seed += small_blind_size;
        player.bank -= small_blind_size;
        player.action = Some(PlayerAction {
            action_type: ActionType::Blind.into(),
            bet: small_blind_size,
        });
        let player = &mut self.players[big_blind_index as usize];
        player.bet_in_current_seed += blind_size;
        player.bank -= blind_size;
        player.action = Some(PlayerAction {
            action_type: ActionType::Blind.into(),
            bet: blind_size,
        })
    }

    fn update_player_bank(&mut self, value: i32, id: &i32) {
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
    pub fn new_random(&mut self) {
        self.deck = CardDeck::new_random();
    }

    pub fn new(deck: CardDeck) -> DeckState {
        DeckState { deck }
    }
}
struct GameState {
    status: GameStatus,
    // might not need
    last_bet_action: PlayerAction,
    street: Street,
    game_bank: i32,
    big_blind: i32,
    raise_amount: i32,
    raiser_index: Option<i32>,
    positions: KeyPositions,
    biggest_bet_on_curr_street: i32,
}

#[derive(Debug)]
struct KeyPositions {
    small_blind_index: i32,
    big_blind_index: i32,
    curr_player_index: i32,
    button_index: i32,
}

impl GameState {
    fn get_loop_incremented_index(index: i32, player_amounts: i32) -> i32 {
        return (index + 1) % player_amounts;
    }

    fn deal_cards(&mut self, deck_state: &mut DeckState, player_state: &mut PlayerState) {
        if self.street.street_status == StreetStatus::Preflop as i32 {
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

    fn calculate_key_positions(init_button_index: i32, players_amount: i32) -> KeyPositions {
        let is_heads_up = GameState::is_heads_up(players_amount);
        let small_blind_index = if is_heads_up {
            init_button_index
        } else {
            GameState::get_loop_incremented_index(init_button_index, players_amount)
        };

        let big_blind_index =
            GameState::get_loop_incremented_index(small_blind_index, players_amount);

        let curr_player_index = if is_heads_up {
            GameState::get_loop_incremented_index(init_button_index, players_amount)
        } else {
            GameState::get_loop_incremented_index(big_blind_index, players_amount)
        };
        // Return values as a struct
        KeyPositions {
            small_blind_index: small_blind_index,
            big_blind_index: big_blind_index,
            curr_player_index: curr_player_index,
            button_index: init_button_index,
        }
    }

    fn is_heads_up(player_amounts: i32) -> bool {
        player_amounts == 2
    }

    pub fn new(players_amount: i32, blind_size: i32) -> GameState {
        let button_index = rand::thread_rng().gen_range(0..players_amount) as i32;
        let positions = GameState::calculate_key_positions(button_index, players_amount);

        GameState {
            last_bet_action: PlayerAction {
                action_type: ActionType::Empty.into(),
                bet: 0,
            },
            status: GameStatus::Active,
            street: Street {
                street_status: StreetStatus::Preflop.into(),
                cards: Vec::new(),
            },
            big_blind: blind_size,
            game_bank: 0,
            raise_amount: 0,
            biggest_bet_on_curr_street: blind_size,
            raiser_index: None,
            positions,
        }
    }

    fn calculate_curr_player_index_on_new_street(
        &self,
        player_state: &PlayerState,
    ) -> Option<usize> {
        let mut new_curr =
            ((self.positions.button_index + 1) % player_state.players.len() as i32) as usize;

        for _ in 0..player_state.players.len() {
            if player_state.players[new_curr]
                .action
                .as_ref()
                .unwrap()
                .action_type
                != ActionType::Fold.into()
            {
                return Some(new_curr);
            } else {
                new_curr = (new_curr + 1) % player_state.players.len() as usize;
            }
        }

        None
    }

    fn get_default_last_player_index(&self, player_state: &PlayerState) -> i32 {
        if self.street.street_status() == StreetStatus::Preflop {
            if GameState::is_heads_up(player_state.players.len() as i32) {
                self.positions.button_index as i32
            } else {
                self.positions.big_blind_index as i32
            }
        } else {
            self.positions.button_index as i32
        }
    }

    fn calculate_last_active_player_index(&self, player_state: &PlayerState) -> Option<usize> {
        let mut last_player_index = if let Some(raiser_index) = self.raiser_index {
            (raiser_index + player_state.players.len() as i32 - 1)
                % player_state.players.len() as i32
        } else {
            self.get_default_last_player_index(player_state)
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

    fn next_street(&mut self, deck_state: &mut DeckState, player_state: &mut PlayerState) {
        self.street.street_status =
            GameState::get_loop_incremented_index(self.street.street_status, StreetStatus::len());

        if self.street.street_status == StreetStatus::Preflop as i32 {
            self.next_cycle(player_state, deck_state);
            return;
        } else {
            self.positions.curr_player_index = self
                .calculate_curr_player_index_on_new_street(&player_state)
                .unwrap() as i32;
        }

        self.last_bet_action = PlayerAction::new();
        self.raise_amount = 0;
        self.raiser_index = None;
        self.biggest_bet_on_curr_street = 0;

        if self.street.street_status == StreetStatus::Flop as i32 {
            self.street.cards.clear();

            self.street
                .cards
                .push(deck_state.deck.cards.pop_front().unwrap());
            self.street
                .cards
                .push(deck_state.deck.cards.pop_front().unwrap());
            self.street
                .cards
                .push(deck_state.deck.cards.pop_front().unwrap());
        } else {
            self.street
                .cards
                .push(deck_state.deck.cards.pop_front().unwrap());
        }
    }

    fn next_cycle(&mut self, player_state: &mut PlayerState, deck_state: &mut DeckState) {
        self.street = Street::default();
        deck_state.new_random();
        self.deal_cards(deck_state, player_state);
        self.last_bet_action = PlayerAction::new();
        self.biggest_bet_on_curr_street = self.big_blind;
        self.raise_amount = 0;
        self.raiser_index = None;
        player_state.players.iter_mut().for_each(|p| {
            p.bet_in_current_seed = 0;
            p.action = Some(PlayerAction::new())
        });
        self.game_bank = 0;
        let players_amount = player_state.players.len() as i32;
        let next_button_index = (self.positions.button_index + 1) % players_amount;
        self.positions = GameState::calculate_key_positions(next_button_index, players_amount);
        player_state.set_up_blinds(
            self.positions.small_blind_index,
            self.positions.big_blind_index,
            self.big_blind,
        );
    }

    fn next_player(&mut self, player_state: &PlayerState) {
        let mut curr_next = self.positions.curr_player_index as usize;
        let mut is_set = false;

        for _ in 0..player_state.players.len() {
            curr_next = (curr_next + 1) % player_state.players.len();
            if let Some(player) = player_state.players.get(curr_next) {
                if let Some(action) = &player.action {
                    if action.action_type != ActionType::Fold.into() {
                        self.positions.curr_player_index = curr_next as i32;
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

impl StreetStatus {
    pub fn len() -> i32 {
        4
    }
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
            game_state: GameState::new(players.len() as i32, 100),
            lobby_id: lobby_id,
            player_state: PlayerState::from(players, 100 * 100),
        }
    }

    fn calculate_min_call(&self, p: &Player) -> i32 {
        if self.game_state.raiser_index.is_some() {
            self.player_state.players[self.game_state.raiser_index.unwrap() as usize]
                .bet_in_current_seed
                - p.bet_in_current_seed
        } else {
            // we can't use big blind value here as players bank amount can be less than big blind
            (self.game_state.biggest_bet_on_curr_street - p.bet_in_current_seed).max(0)
        }
    }

    fn calculate_min_raise(&self, p: &Player) -> i32 {
        // do we need to return player's bank if he cannot afford min raise?
        // 1. his bank < min raise && his bank < curr_biggest_bet = it's not a raise but allin  call
        // 2. his bank < min raise && his bank > curr_biggest_bet = valid raise all in
        // let min_raise = self.game_state.biggest_bet_on_curr_street + self.game_state.raise_amount;
        // if min_raise > p.bank { p.bank } else { min_raise }
        if self.game_state.raiser_index.is_some() {
            return self.game_state.biggest_bet_on_curr_street + self.game_state.raise_amount;
        } else {
            return if self.game_state.street.street_status() == StreetStatus::Preflop {
                self.game_state.big_blind * 2
            } else {
                self.game_state.big_blind
            };
        }
    }

    pub fn get_next_player_id(&self) -> i32 {
        self.get_player_id_by_index(self.game_state.positions.curr_player_index)
    }

    fn get_player_id_by_index(&self, index: i32) -> i32 {
        self.player_state
            .players
            .get(index as usize)
            .unwrap()
            .user_id
    }

    fn filter_player_cards(mut player: Player) -> Player {
        player.cards = None;
        player
    }

    fn get_filtered_players(&self) -> Vec<Player> {
        let players: Vec<Player> = self
            .player_state
            .players
            .iter()
            .map(|player| Dealer::filter_player_cards(player.clone()))
            .collect();
        players
    }

    pub fn start_new_game(&mut self) -> Result<Vec<ClientState>, &str> {
        // TODO: rethink what entity has to manage cards
        self.game_state
            .deal_cards(&mut self.deck_state, &mut self.player_state);

        self.player_state.set_up_blinds(
            self.game_state.positions.small_blind_index,
            self.game_state.positions.big_blind_index,
            self.game_state.big_blind,
        );

        let state = self.create_client_state(Vec::new());
        Ok(state)
    }

    fn create_client_state(&mut self, winners: Vec<Player>) -> Vec<ClientState> {
        let filtered_players = self.get_filtered_players();

        let states = self
            .player_state
            .players
            .iter()
            .map(|p| {
                let client = ClientState {
                    player_id: p.user_id,
                    cards: p.cards.clone(),
                    min_amount_to_call: self.calculate_min_call(p),
                    min_amount_to_raise: self.calculate_min_raise(p),
                    can_raise: self.can_raise(p),
                    players: filtered_players.clone(),
                    game_status: self.game_state.status.into(),
                    curr_player_id: self
                        .get_player_id_by_index(self.game_state.positions.curr_player_index),
                    curr_button_id: self
                        .get_player_id_by_index(self.game_state.positions.button_index),
                    curr_big_blind_id: self
                        .get_player_id_by_index(self.game_state.positions.big_blind_index),
                    curr_small_blind_id: self
                        .get_player_id_by_index(self.game_state.positions.small_blind_index),
                    street: Some(self.game_state.street.clone()),
                    lobby_id: self.lobby_id,
                    latest_winners: winners.clone(),
                };

                client
            })
            .collect();

        states
    }

    fn calculate_winner(&mut self) -> Vec<Player> {
        let mut win_result: Vec<Player> = Vec::new();
        let mut winners_map: BTreeMap<i32, Vec<&mut Player>> = BTreeMap::new();

        let mut players = self.player_state.players.iter_mut().filter(|p| {
            p.action.as_ref().unwrap().action_type() != ActionType::Fold
                && p.action.as_ref().unwrap().action_type() != ActionType::Empty
        });

        if players.by_ref().count() == 1 {
            winners_map.insert(1, players.collect());
        } else {
            for player in players {
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
        }
        for winners in winners_map.values_mut() {
            let sum_of_winners_bets: i32 = winners.iter().map(|w| w.bet_in_current_seed).sum();
            let winners_amount = winners.len() as i32;
            for w in winners {
                let win_points = w.bet_in_current_seed
                    + ((self.game_state.game_bank - sum_of_winners_bets) / winners_amount);
                w.bank += win_points;

                let remainder = if self.game_state.game_bank != 0 {
                    win_points % self.game_state.game_bank
                } else {
                    0
                };
                if remainder > 0 {
                    self.game_state.game_bank -= remainder;
                } else {
                    self.game_state.game_bank = 0;
                }

                win_result.push((*w).clone());
            }
            if self.game_state.game_bank == 0 {
                break;
            }
        }
        win_result
    }

    fn can_determine_winner(&self) -> Option<bool> {
        // TODO: refactor to return Player and skip winner evaluation
        let mut remaining_players = self.player_state.players.iter().filter(|p| {
            p.action.is_some()
                && p.action.as_ref().unwrap().action_type() != ActionType::Fold
                && p.action.as_ref().unwrap().action_type() != ActionType::Empty
        });

        match (remaining_players.next(), remaining_players.next()) {
            (Some(_), None) => Some(true),
            _ => None,
        }
    }

    pub fn update_game_state(&mut self, payload: PlayerPayload) -> Vec<ClientState> {
        if payload.lobby_id != self.lobby_id {
            panic!("Wrong lobby id in payload");
        }

        assert!(self
            .player_state
            .players
            .iter()
            .any(|p| { p.user_id == payload.player_id }));

        // TODO: Reset player action on new game loop
        let action_type =
            ActionType::try_from(payload.action.as_ref().unwrap().action_type).unwrap();
        match action_type {
            ActionType::Fold => {
                self.process_fold_action(payload.player_id);
                let result = self.can_determine_winner();
                if result.is_some() && result.unwrap() == true {
                    // TODO: set_winner
                    let winners = self.calculate_winner();
                    // in future: add checks when players are elimanated;

                    self.game_state
                        .next_cycle(&mut self.player_state, &mut self.deck_state);

                    let state = self.create_client_state(winners);
                    return state;
                }
            }
            ActionType::Call => {
                self.process_call_action(payload.player_id, payload.action.unwrap().bet);
            }
            ActionType::Raise => {
                self.process_raise_action(payload.player_id, payload.action.unwrap().bet);
            }
            _ => println!("Illegal move!"),
        }
        // TODO:
        // if only one or less players left with money process game cycle automatically

        let last_active = self
            .game_state
            .calculate_last_active_player_index(&self.player_state)
            .unwrap() as i32;

        if last_active == self.game_state.positions.curr_player_index {
            self.game_state.raiser_index = None;
            let curr_street = self.game_state.street.street_status;
            if curr_street == StreetStatus::River as i32 {
                let winners: Vec<Player> = self.calculate_winner();
                self.game_state
                    .next_cycle(&mut self.player_state, &mut self.deck_state);
                let state = self.create_client_state(winners);
                return state;
            }
            self.game_state
                .next_street(&mut self.deck_state, &mut self.player_state);
        } else {
            self.game_state.next_player(&self.player_state);
        }

        self.create_client_state(Vec::new())
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
    }

    fn process_call_action(&mut self, player_id: i32, bet_amount: i32) {
        // TODO: Refactor repeated code using RefCell?
        let min_call = {
            let player = self
                .player_state
                .players
                .iter()
                .find(|p| p.user_id == player_id)
                .expect("user not found");

            self.calculate_min_call(player)
        };

        if bet_amount < min_call {
            println!("Bet amount is not valid for minimum call");
            return;
        }

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
        // last_bet gonna be equal 1blind at start
        self.game_state.last_bet_action = PlayerAction {
            action_type: ActionType::Call.into(),
            bet: bet_amount,
        };
        player.action = Some(PlayerAction {
            action_type: ActionType::Call.into(),
            bet: bet_amount,
        });
        self.game_state.game_bank += bet_amount;
    }

    fn is_all_in(player: &Player, bet_amount: i32) -> bool {
        player.bank == bet_amount
    }

    // If a player decides to go "all in" by betting all of their chips,
    // but their bet is smaller than the minimum raise allowed,
    // but bigger than the last biggest bet made by another player,
    // then the players who have already placed bets before them cannot raise again unless
    // there's another player who raises after the all-in bet.
    fn can_raise(&self, player: &Player) -> bool {
        if let Some(index) = self.game_state.raiser_index {
            let raiser_bank = self.player_state.players[index as usize].bank;
            return raiser_bank != 0
                || raiser_bank == 0
                    && player.action.as_ref().unwrap().action_type() == ActionType::Empty;
        }
        // untrusted condition for now
        player.bank > self.game_state.biggest_bet_on_curr_street
    }

    fn process_raise_action(&mut self, player_id: i32, bet_amount: i32) {
        if let Some(index) = self
            .player_state
            .players
            .iter()
            .position(|p| p.user_id == player_id)
        {
            let player: &Player = &self.player_state.players[index];

            let valid_min_raise = self.calculate_min_raise(player);

            let bet_is_all_in = Dealer::is_all_in(player, bet_amount);

            if self.can_raise(player) == false {
                println!("Player not eligable to raise");
                return;
            }

            if bet_amount < valid_min_raise
                && (bet_is_all_in == false
                    && bet_amount < self.game_state.biggest_bet_on_curr_street)
            {
                println!("Player raise amount is not valid");
                return;
            }

            if player.action.as_ref().unwrap().action_type == ActionType::Fold.into() {
                println!("Player has folded already and cannot raise");
                return;
            }

            if player.bank < bet_amount {
                println!("Player does not have enough points!");
                return;
            }

            let player = &mut self.player_state.players[index];

            self.game_state.raiser_index = Some(index as i32);

            player.bet_in_current_seed += bet_amount;
            player.bank -= bet_amount;
            self.game_state.game_bank += bet_amount;

            // fix if all in is smaller and valid we dont want to set it as the biggest
            if bet_amount > self.game_state.biggest_bet_on_curr_street {
                self.game_state.raise_amount =
                    bet_amount - self.game_state.biggest_bet_on_curr_street;
                self.game_state.biggest_bet_on_curr_street = bet_amount;
            }

            self.game_state.last_bet_action = PlayerAction {
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
