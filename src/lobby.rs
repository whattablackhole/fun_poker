#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Lobby {
    #[prost(int32, tag = "1")]
    pub id: i32,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(int32, tag = "3")]
    pub author_id: i32,
    #[prost(int32, tag = "4")]
    pub players_registered: i32,
    #[prost(enumeration = "GameType", tag = "5")]
    pub game_type: i32,
    #[prost(enumeration = "GameName", tag = "6")]
    pub game_name: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LobbyList {
    #[prost(message, repeated, tag = "1")]
    pub list: ::prost::alloc::vec::Vec<Lobby>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum GameName {
    Holdem = 0,
}
impl GameName {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            GameName::Holdem => "Holdem",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Holdem" => Some(Self::Holdem),
            _ => None,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum GameType {
    Tournament = 0,
    Cash = 1,
}
impl GameType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            GameType::Tournament => "Tournament",
            GameType::Cash => "Cash",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Tournament" => Some(Self::Tournament),
            "Cash" => Some(Self::Cash),
            _ => None,
        }
    }
}

pub enum LobbyState {
    Ready,
    NotReady,
}

