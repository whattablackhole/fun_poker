// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Street {
    #[prost(enumeration = "StreetStatus", tag = "1")]
    pub street_status: i32,
    #[prost(message, repeated, tag = "2")]
    pub cards: ::prost::alloc::vec::Vec<super::card::Card>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Winner {
    #[prost(int32, tag = "1")]
    pub player_id: i32,
    #[prost(int32, tag = "2")]
    pub win_amout: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlayerCards {
    #[prost(int32, tag = "1")]
    pub player_id: i32,
    #[prost(message, optional, tag = "2")]
    pub cards: ::core::option::Option<super::card::CardPair>,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ShowdownOutcome {
    #[prost(message, optional, tag = "1")]
    pub street_history: ::core::option::Option<Street>,
    #[prost(message, repeated, tag = "2")]
    pub winners: ::prost::alloc::vec::Vec<Winner>,
    #[prost(message, repeated, tag = "3")]
    pub players_cards: ::prost::alloc::vec::Vec<PlayerCards>,
    #[prost(bool, tag = "4")]
    pub process_flop_automatically: bool,
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
