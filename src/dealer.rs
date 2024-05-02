use crate::{
    card::{Card, CardDeck, CardPair},
    player::{ActionType, PlayerAction, PlayerPayload},
};

pub struct Dealer {
    lobby_id: i32,
    game_state: GameState,
    deck_state: DeckState,
    player_ids: Vec<i32>,
}

impl Dealer {
    //
    pub fn new(lobby_id: i32, player_ids: Vec<i32>) -> Dealer {
        Dealer {
            deck_state: DeckState {
                deck: CardDeck::new_random(),
                player_outloads: Vec::new(),
            },
            game_state: GameState::new(),
            lobby_id: lobby_id,
            player_ids: Vec::from(player_ids),
        }
    }

    fn deal_cards(&mut self) -> Vec<PlayOutload> {
        let mut result = Vec::new();
        if self.game_state.street.street_status == StreetStatus::Preflop as i32 {
            for id in &self.player_ids {
                let c1 = self.deck_state.deck.deck.pop_front().unwrap();
                let c2 = self.deck_state.deck.deck.pop_front().unwrap();
                result.push(PlayOutload {
                    cards: CardPair {
                        card1: c1,
                        card2: c2,
                    },
                    player_id: *id,
                })
            }
        } else {
            panic!("Can't deal cards on other street!");
        }
        result
    }
    pub fn start_new_game(&mut self) -> UpdatedGameState {
        self.deck_state.player_outloads = self.deal_cards();
        self.game_state.status = GameStatus::Active;
        // TODO: implement new for Street... and others structs
        UpdatedGameState {
            game_status: self.game_state.status,
            next_player_id: *self
                .player_ids
                .get(self.game_state.next_player_index as usize)
                .unwrap(),
            street: Street {
                street_status: self.game_state.street.street_status,
                value: self.game_state.street.value.clone(),
            },
            player_out_loads: self.deck_state.player_outloads.clone(),
            lobby_id: self.lobby_id,
        }
    }

    fn next_game_loop(&mut self) -> UpdatedGameState {
        self.to_default_state();
        return self.start_new_game();
    }

    fn to_default_state(&mut self) {
        self.deck_state.reset();
        self.game_state.reset();
    }

    pub fn update_game_state(&mut self, payload: PlayerPayload) -> UpdatedGameState {
        if payload.lobby_id != self.lobby_id {
            panic!("Wrong lobby id in payload");
        }

        assert!(self
            .player_ids
            .iter()
            .any(|id| { *id == payload.player_id }));

        let max_index = self.player_ids.len() - 1;
        let is_last_player_turn = self.game_state.next_player_index as usize == max_index;

        if is_last_player_turn {
            let next_street = self.game_state.street.street_status + 1;

            if next_street > StreetStatus::River as i32 {
                return self.next_game_loop();
            }
            // TODO: move_button + increase next player index;

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

        let state = UpdatedGameState {
            game_status: self.game_state.status,
            next_player_id: *self
                .player_ids
                .get(self.game_state.next_player_index as usize)
                .unwrap(),
            street: Street {
                street_status: self.game_state.street.street_status,
                value: self.game_state.street.value.clone(),
            },
            player_out_loads: self.deck_state.player_outloads.clone(),
            lobby_id: self.lobby_id,
        };
        state
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
pub struct UpdatedGameState {
    pub player_out_loads: Vec<PlayOutload>,
    pub next_player_id: i32,
    pub lobby_id: i32,
    pub street: Street,
    pub game_status: GameStatus,
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
