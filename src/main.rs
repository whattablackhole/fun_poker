use std::{
    io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}
};

use fun_poker::postgres_database::PostgresDatabase;
use fun_poker::ThreadPool;
use prost::Message;
use std::sync::Arc;


trait EncodableMessage {
    fn encode_message(&self) -> Vec<u8>;
}

impl<M: Message> EncodableMessage for M {
    fn encode_message(&self) -> Vec<u8> {
        let mut buf = Vec::new();
        self.encode(&mut buf).unwrap();
        buf
    }
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EmptyMessage {}

fn main() {
    let repository: PostgresDatabase = PostgresDatabase::new().unwrap();
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);
    let arc_repo: Arc<PostgresDatabase>  = Arc::new(repository);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let clone_repo = Arc::clone(&arc_repo);
        pool.execute(move || {
            handle_connection(stream, clone_repo);
        });
    }
}

fn handle_connection(mut stream: TcpStream,  repo: Arc<PostgresDatabase>) {
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let status_line = "HTTP/1.1 200 OK";
    let path = request_line.split(" ").skip(1).next().unwrap();
    
    println!("{path}");
    
    let message: Box<dyn EncodableMessage> = match path  {
        "/getLobbies" => Box::new(repo.get_lobbies()),
        _ => Box::new(EmptyMessage{})
    };

    let response = construct_response(status_line, message);

    stream.write_all(&response).unwrap();
}


fn construct_response(status_line: &str, message: Box<dyn EncodableMessage>) -> Vec<u8> {
    
    let buf = message.encode_message();
    let content_length = buf.len();

    let mut response = Vec::new();
    response.extend_from_slice(status_line.as_bytes());
    response.extend_from_slice(b"\r\nContent-Length: ");
    response.extend_from_slice(content_length.to_string().as_bytes());
    response.extend_from_slice(b"\r\nContent-Type: application/octet-stream\r\n");
    response.extend_from_slice(b"Access-Control-Allow-Origin: *\r\n\r\n");
    
    response.extend_from_slice(&buf);

    response
}