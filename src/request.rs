#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct JoinLobbyRequest {
    #[prost(int32, tag = "1")]
    pub lobby_id: i32,
    #[prost(int32, tag = "2")]
    pub player_id: i32,
}