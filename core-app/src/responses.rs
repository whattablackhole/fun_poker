use prost::Message;

use crate::protos::responses::ResponseMessageType;


pub struct TMessageResponse {
    pub receiver_id: i32,
    pub message_type: ResponseMessageType,
    pub message: Box<dyn EncodableMessage>,
}

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