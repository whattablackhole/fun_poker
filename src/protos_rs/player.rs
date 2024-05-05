// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Player {
    #[prost(string, tag = "1")]
    pub user_name: ::prost::alloc::string::String,
    #[prost(int32, tag = "2")]
    pub user_id: i32,
    #[prost(string, tag = "3")]
    pub country: ::prost::alloc::string::String,
    #[prost(message, optional, tag = "4")]
    pub action: ::core::option::Option<PlayerAction>,
    #[prost(int32, tag = "5")]
    pub bank: i32,
    #[prost(message, optional, tag = "6")]
    pub cards: ::core::option::Option<super::card::CardPair>,
    #[prost(int32, tag = "7")]
    pub bet_in_current_seed: i32,
}
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlayerPayload {
    #[prost(int32, tag = "1")]
    pub player_id: i32,
    #[prost(int32, tag = "2")]
    pub lobby_id: i32,
    #[prost(message, optional, tag = "3")]
    pub action: ::core::option::Option<PlayerAction>,
}
/// ? INTRODUCE ACTION HISTORY ?
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct PlayerAction {
    #[prost(enumeration = "ActionType", tag = "1")]
    pub action_type: i32,
    #[prost(int32, tag = "2")]
    pub bet: i32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, ::prost::Enumeration)]
#[repr(i32)]
pub enum ActionType {
    Fold = 0,
    Call = 1,
    Raise = 2,
    Empty = 3,
}
impl ActionType {
    /// String value of the enum field names used in the ProtoBuf definition.
    ///
    /// The values are not transformed in any way and thus are considered stable
    /// (if the ProtoBuf definition does not change) and safe for programmatic use.
    pub fn as_str_name(&self) -> &'static str {
        match self {
            ActionType::Fold => "Fold",
            ActionType::Call => "Call",
            ActionType::Raise => "Raise",
            ActionType::Empty => "Empty",
        }
    }
    /// Creates an enum from field names used in the ProtoBuf definition.
    pub fn from_str_name(value: &str) -> ::core::option::Option<Self> {
        match value {
            "Fold" => Some(Self::Fold),
            "Call" => Some(Self::Call),
            "Raise" => Some(Self::Raise),
            "Empty" => Some(Self::Empty),
            _ => None,
        }
    }
}
