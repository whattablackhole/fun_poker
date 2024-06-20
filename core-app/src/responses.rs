use std::error::Error;

use prost::Message;

use crate::{
    game_orchestrator::JoinGameMessage,
    protos::{
        client_state::ClientState,
        player::PlayerPayload,
        responses::{ResponseMessageType, StartGameResponse},
        user::User,
    },
    socket_pool::{ConnectionClosedEvent, ReadMessageError},
};

pub struct TMessageResponse {
    pub receiver_id: i32,
    pub message_type: ResponseMessageType,
    pub message: Box<dyn EncodableMessage>,
}

pub type DynMessage = Box<dyn Message>;
pub type DynMessageError = Box<dyn Error + Send + Sync>;
pub type DynMessageResult = Result<DynMessage, DynMessageError>;

pub trait EncodableMessage {
    fn encode_message(&self) -> Vec<u8>;
}

impl<M: Message> EncodableMessage for M {
    fn encode_message(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode(&mut buf).unwrap();
        buf
    }
}

pub enum SocketSourceMessage {
    ConnectionClosed(ConnectionClosedEvent),
    PlayerPayload(Result<PlayerPayload, ReadMessageError>),
}


pub enum GameChannelMessage {
    HttpRequestSource(JoinGameMessage),
    SocketSource(SocketSourceMessage),
    InnerSource(PlayerPayload)
}

fn create_message_response<T>(
    message: T,
    t: ResponseMessageType,
    receiver_id: i32,
) -> TMessageResponse
where
    T: Message + 'static,
{
    return TMessageResponse {
        message: Box::new(message),
        receiver_id,
        message_type: t,
    };
}

pub fn generate_client_state_responses(states: Vec<ClientState>) -> Vec<TMessageResponse> {
    return states
        .into_iter()
        .map(|state| {
            let receiver_id = state.player_id;
            create_message_response(state, ResponseMessageType::ClientState, receiver_id)
        })
        .collect();
}

pub fn generate_game_started_responses(
    lobby_id: i32,
    users: &Vec<User>,
    delay: i32,
) -> Vec<TMessageResponse> {
    return users
        .into_iter()
        .map(|user| {
            let receiver_id = user.id;
            create_message_response(
                StartGameResponse {
                    game_started_delay: delay,
                    lobby_id,
                },
                ResponseMessageType::StartGame,
                receiver_id,
            )
        })
        .collect();
}
