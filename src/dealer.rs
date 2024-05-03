use std::{
    collections::{BTreeMap, HashMap},
    hash::Hash,
};

use crate::{
    card::{Card, CardDeck, CardPair},
    player::{ActionType, Player, PlayerAction, PlayerPayload},
};

use pokereval_cactus::card::Card as PCard;
use pokereval_cactus::evaluator;
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

    pub fn from(players: Vec<Player>, bank_size: i32) -> PlayerState {
        let mut bank_map = HashMap::new();
        for player in players.iter() {
            bank_map.insert(player.id, bank_size);
        }
        PlayerState { bank_map, players }
    }

    pub fn update_player_bank(&mut self, value: i32, id: &i32) {
        let bank = self.bank_map.get_mut(id).unwrap();

        *bank += value;
    }
}

impl Dealer {
    // TODO: implement game settings for bank size etc..
    pub fn new(lobby_id: i32, players: Vec<Player>) -> Dealer {
        Dealer {
            deck_state: DeckState {
                deck: CardDeck::new_random(),
                player_outloads: Vec::new(),
            },
            game_state: GameState::new(),
            lobby_id: lobby_id,
            player_state: PlayerState::from(players, 0),
        }
    }

    fn deal_cards(&mut self) -> Vec<PlayOutload> {
        let mut result = Vec::new();
        if self.game_state.street.street_status == StreetStatus::Preflop as i32 {
            for player in &self.player_state.players {
                let c1 = self.deck_state.deck.deck.pop_front().unwrap();
                let c2 = self.deck_state.deck.deck.pop_front().unwrap();
                result.push(PlayOutload {
                    cards: CardPair {
                        card1: c1,
                        card2: c2,
                    },
                    player_id: player.id,
                })
            }
        } else {
            panic!("Can't deal cards on other street!");
        }
        result
    }
    pub fn start_new_table_loop(&mut self) -> ClientGameState {
        // self = self is not needed?
        self.deck_state.player_outloads = self.deal_cards();
        self.game_state.status = GameStatus::Active;
        // TODO: implement new for Street... and others structs
        ClientGameState {
            game_status: self.game_state.status,
            next_player_id: self
                .player_state
                .players
                .get(self.game_state.next_player_index as usize)
                .unwrap()
                .id,
            street: Street {
                street_status: self.game_state.street.street_status,
                value: self.game_state.street.value.clone(),
            },
            player_out_loads: self.deck_state.player_outloads.clone(),
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
        self.game_state.reset();
    }

    fn get_client_game_state(&mut self) -> ClientGameState {
        let state = ClientGameState {
            game_status: self.game_state.status,
            next_player_id: self
                .player_state
                .players
                .get(self.game_state.next_player_index as usize)
                .unwrap()
                .id,
            street: Street {
                street_status: self.game_state.street.street_status,
                value: self.game_state.street.value.clone(),
            },
            player_out_loads: self.deck_state.player_outloads.clone(),
            lobby_id: self.lobby_id,
            latest_winners: Vec::new(),
        };
        state
    }

    fn calculate_winner(&mut self) -> Vec<i32> {
        let mut winners_map: BTreeMap<i32, Vec<i32>> = BTreeMap::new();

        for outload in self.deck_state.player_outloads.clone() {
            // TODO: will be great to write own 2+2 evaluator;
            let eval = evaluator::Evaluator::new();
            let street: Vec<i32> = self
                .game_state
                .street
                .value
                .iter()
                .map(|card| PCard::new(card.to_string()))
                .collect();

            let result = eval.evaluate(
                vec![
                    PCard::new(outload.cards.card1.to_string()),
                    PCard::new(outload.cards.card2.to_string()),
                ],
                street,
            );

            if winners_map.contains_key(&result) {
                let winners = winners_map.get_mut(&result).unwrap();
                winners.push(outload.player_id);
            } else {
                winners_map.insert(result, vec![outload.player_id]);
            }

            for (k, winners) in winners_map.iter() {
                for w in winners {
                    // TODO:
                    // Winners {
                    // sum_bet: i32,
                    // players
                    // }
                    // let win_points =  (player.bet + ((bank - winners.sumbet) / win.length))
                    // self.state.get_player(w.id).add_bank(win_points);
                    // self.state.game_state.bank.substract(win_points); module substruction;
                    // capacity of player = player.bet * player.len
                    // for player1 capacity = 3000
                    // for player2 capacity = 900
                    // first winner.bank = mod(bank - capacity) > 0 : capacity: bank
                    //bank = mod[bank - capacity] = for player1 result is
                    // if(bank > 0) {
                    // goto second winner
                    // }
                    // player1 all in = 1000
                    // player2 all in = 300
                    // player3 all in = 1000
                    //  if player2 win
                    //   he takes max of 300 from each player
                    // losers foreach
                    // bet - 300 = 700
                }

                if self.game_state.bank == 0 {
                    break;
                }
            }
        }
        winners_map.pop_first().unwrap().1
    }

    pub fn update_game_state(&mut self, payload: PlayerPayload) -> ClientGameState {
        if payload.lobby_id != self.lobby_id {
            panic!("Wrong lobby id in payload");
        }

        assert!(self
            .player_state
            .players
            .iter()
            .any(|p| { p.id == payload.player_id }));
        // TODO: add functionality in case of player folded or sit outed etc.
        let max_index = self.player_state.players.len() - 1;
        let is_last_player_turn = self.game_state.next_player_index as usize == max_index;

        if is_last_player_turn {
            let next_street = self.game_state.street.street_status + 1;

            if next_street > StreetStatus::River as i32 {
                let winners: Vec<i32> = self.calculate_winner();
                let mut state = self.next_game_loop();
                // TODO: set_winner
                state.latest_winners = winners;
                return state;
            }
            // TODO: move_button + increase next player index;
            // TODO: refactor to update state using interface;
            self.game_state.next_player_index = 0;
            self.game_state.street.street_status = next_street;
            if next_street == StreetStatus::Flop as i32 {
                // maybe not needed, TODO: refactor
                self.game_state.street.value.clear();

                self.game_state
                    .street
                    .value
                    .push(self.deck_state.deck.deck.pop_front().unwrap());
                self.game_state
                    .street
                    .value
                    .push(self.deck_state.deck.deck.pop_front().unwrap());
                self.game_state
                    .street
                    .value
                    .push(self.deck_state.deck.deck.pop_front().unwrap());
            } else {
                self.game_state
                    .street
                    .value
                    .push(self.deck_state.deck.deck.pop_front().unwrap());
            }
        } else {
            self.game_state.next_player_index += 1;
        }

        match ActionType::try_from(payload.action.unwrap().action_type).unwrap() {
            ActionType::Fold => println!("Fold"),
            ActionType::Call => println!("Call"),
            ActionType::Raise => println!("Raise"),
            ActionType::Empty => println!("Empty"),
        }

        // TODO: use referenced structure for memory optimization

        self.get_client_game_state()
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Street {
    #[prost(enumeration = "StreetStatus", tag = "1")]
    pub street_status: i32,
    #[prost(message, repeated, tag = "2")]
    pub value: ::prost::alloc::vec::Vec<Card>,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(int32, tag = "1")]
    pub player_id: i32,
    #[prost(message, optional, tag = "2")]
    pub cards: ::core::option::Option<CardPair>,
    #[prost(int32, tag = "3")]
    pub next_player_id: i32,
    #[prost(int32, tag = "4")]
    pub lobby_id: i32,
    #[prost(message, optional, tag = "5")]
    pub street: ::core::option::Option<Street>,
    #[prost(enumeration = "GameStatus", tag = "6")]
    pub game_status: i32,
    #[prost(int32, repeated, tag = "7")]
    pub latest_winners: ::prost::alloc::vec::Vec<i32>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum StreetStatus {
    Preflop = 0,
    Flop = 1,
    Turn = 2,
    River = 3,
}

impl StreetStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            StreetStatus::Preflop => "Preflop",
            StreetStatus::Flop => "Flop",
            StreetStatus::Turn => "Turn",
            StreetStatus::River => "River",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Preflop" => Some(Self::Preflop),
            "Flop" => Some(Self::Flop),
            "Turn" => Some(Self::Turn),
            "River" => Some(Self::River),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum GameStatus {
    Pause = 0,
    None = 1,
    Active = 2,
}
impl GameStatus {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            GameStatus::Pause => "Pause",
            GameStatus::None => "None",
            GameStatus::Active => "Active",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Pause" => Some(Self::Pause),
            "None" => Some(Self::None),
            "Active" => Some(Self::Active),
            _ => None,
        }
    }
}

// it happens that to get deck we need duplicate field: deckstate.deck.deck
// TODO: refactor
pub struct DeckState {
    deck: CardDeck,
    pub player_outloads: Vec<PlayOutload>,
}

impl DeckState {
    pub fn reset(&mut self) {
        self.deck = CardDeck::new_random();
    }

    pub fn new(deck: CardDeck) -> DeckState {
        DeckState {
            deck,
            player_outloads: Vec::new(),
        }
    }
}
struct GameState {
    status: GameStatus,
    last_action: PlayerAction,
    next_player_index: i32,
    next_button_player_index: i32,
    street: Street,
    bank: i32,
}

impl GameState {
    pub fn reset(&mut self) {
        *self = GameState::new();
    }
    pub fn new() -> GameState {
        GameState {
            last_action: PlayerAction {
                action_type: ActionType::Empty.into(),
            },
            status: GameStatus::None,
            next_player_index: 0,
            // TODO: add calculations
            next_button_player_index: 0,
            street: Street {
                street_status: StreetStatus::Preflop.into(),
                value: Vec::new(),
            },
            bank: 0,
        }
    }
}

// change name
#[derive(Clone, Debug)]
pub struct PlayOutload {
    pub player_id: i32,
    pub cards: CardPair,
}
#[derive(Debug)]
pub struct ClientGameState {
    pub player_out_loads: Vec<PlayOutload>,
    pub next_player_id: i32,
    pub lobby_id: i32,
    pub street: Street,
    pub game_status: GameStatus,
    pub latest_winners: Vec<i32>,
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
