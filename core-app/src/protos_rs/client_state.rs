// This file is @generated by prost-build.
#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct ClientState {
    #[prost(int32, tag = "1")]
    pub player_id: i32,
    #[prost(int32, tag = "2")]
    pub lobby_id: i32,
    /// Optional because the game might not be started
    #[prost(message, optional, tag = "3")]
    pub curr_player_id: ::core::option::Option<super::google::protobuf::Int32Value>,
    /// Optional because the game might not be started
    #[prost(message, optional, tag = "4")]
    pub curr_button_id: ::core::option::Option<super::google::protobuf::Int32Value>,
    /// Optional because the game might not be started
    #[prost(message, optional, tag = "5")]
    pub curr_small_blind_id: ::core::option::Option<super::google::protobuf::Int32Value>,
    /// Optional because the game might not be started
    #[prost(message, optional, tag = "6")]
    pub curr_big_blind_id: ::core::option::Option<super::google::protobuf::Int32Value>,
    /// Optional because it might not be dealt yet
    #[prost(message, optional, tag = "7")]
    pub cards: ::core::option::Option<super::card::CardPair>,
    /// Optional because the game might not be started
    #[prost(message, optional, tag = "8")]
    pub street: ::core::option::Option<super::game_state::Street>,
    /// GameStatus could be WAITING_FOR_PLAYERS or similar
    #[prost(enumeration = "super::game_state::GameStatus", tag = "9")]
    pub game_status: i32,
    /// This would include only the joined player(s)
    #[prost(message, repeated, tag = "10")]
    pub players: ::prost::alloc::vec::Vec<super::player::Player>,
    /// Optional because the game might not have reached this stage
    #[prost(message, optional, tag = "11")]
    pub showdown_outcome: ::core::option::Option<super::game_state::ShowdownOutcome>,
    /// Optional because the game might not be started
    #[prost(message, optional, tag = "12")]
    pub amount_to_call: ::core::option::Option<super::google::protobuf::Int32Value>,
    /// Optional because the game might not be started
    #[prost(message, optional, tag = "13")]
    pub min_amount_to_raise: ::core::option::Option<super::google::protobuf::Int32Value>,
    /// Optional because the game might not be started
    #[prost(message, optional, tag = "14")]
    pub can_raise: ::core::option::Option<super::google::protobuf::BoolValue>,
    /// Might be empty if the game hasn't started
    #[prost(message, repeated, tag = "15")]
    pub action_history: ::prost::alloc::vec::Vec<super::game_state::Action>,
}
