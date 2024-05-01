use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};

use fun_poker::ThreadPool;
use fun_poker::{postgres_database::PostgresDatabase, socket_pool::LobbySocketPool};
use prost::Message;
use std::sync::Arc;
use tungstenite::accept;
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
    let socket_pool = LobbySocketPool::new();
    let arc_socket_pool: Arc<LobbySocketPool> = Arc::new(socket_pool);
    let pool = ThreadPool::new(20);
    let arc_repo: Arc<PostgresDatabase> = Arc::new(repository);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let clone_repo = Arc::clone(&arc_repo);
        let clone_socket_pool = Arc::clone(&arc_socket_pool);
        pool.execute(move || {
            handle_connection(stream, clone_repo, clone_socket_pool);
        });
    }
}

enum RequestType {
    Http,
    WebSocket,
}
fn determine_request_type(stream: &TcpStream) -> RequestType {
    let mut buffer = [0; 1024];
    let result = stream.peek(&mut buffer);

    match result {
        Ok(_) => {
            let request_str = String::from_utf8_lossy(&buffer);
            // improve checking
            if request_str.contains("websocket") {
                RequestType::WebSocket
            } else {
                RequestType::Http
            }
        }
        Err(_) => RequestType::Http,
    }
}

fn handle_web_socket_connection_handshake(
    stream: TcpStream,
    lobby_socket_pool: Arc<LobbySocketPool>,
) {
    let websocket = accept(stream).unwrap();
    lobby_socket_pool.add(String::from("1"), websocket);
}

fn handle_http_request(
    mut stream: TcpStream,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<LobbySocketPool>,
) {
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let status_line = "HTTP/1.1 200 OK";
    let path = request_line.split(" ").skip(1).next().unwrap();

    if path == "/sendMessageToAll" {
        socket_pool.send_message_to_all(String::from("Hello guys"));
    }

    let message: Box<dyn EncodableMessage> = match path {
        "/getLobbies" => Box::new(repo.get_lobbies()),
        _ => Box::new(EmptyMessage {}),
    };

    let response = construct_response(status_line, message);

    stream.write_all(&response).unwrap();
}

fn handle_connection(
    stream: TcpStream,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<LobbySocketPool>,
) {
    let reqest_type = determine_request_type(&stream);

    match reqest_type {
        RequestType::WebSocket => handle_web_socket_connection_handshake(stream, socket_pool),
        RequestType::Http => handle_http_request(stream, repo, socket_pool),
    };
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
