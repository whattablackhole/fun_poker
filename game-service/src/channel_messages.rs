use crate::{
    game::JoinGameError,
    protos::{player::Player, requests::PlayerActionRequest},
    socket_pool::{ConnectionClosedEvent, ReadMessageError},
};

pub enum GameChannelRequestMessage {
    JoinGame(JoinGameMessage),
    PlayerAction(Result<PlayerActionRequest, ReadMessageError>),
    ConnectionClosed(ConnectionClosedEvent),
}

pub enum GameChannelResponseMessage {
    JoinGame(Result<(), JoinGameError>),
}

pub struct JoinGameMessage {
    pub player: Player,
}
