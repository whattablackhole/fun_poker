#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct Lobby {
    #[prost(int32, tag = "1")]
    pub id: i32,
    #[prost(string, tag = "2")]
    pub name: ::prost::alloc::string::String,
    #[prost(int32, tag = "3")]
    pub author_id: i32,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct LobbyList {
    #[prost(message, repeated, tag = "1")]
    pub list: ::prost::alloc::vec::Vec<Lobby>,
}

pub enum LobbyState {
    Ready,
    NotReady,
}

