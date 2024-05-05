use fun_poker::{
    dealer::Dealer,
    protos::{
        self,
        client_request::JoinLobbyRequest,
        player::{Player, PlayerPayload},
        user::User,
    },
    socket_pool::{DealerPool, PlayerChannelClient},
    ThreadPool,
};
use fun_poker::{postgres_database::PostgresDatabase, socket_pool::LobbySocketPool};
use prost::Message;
use protos::client_state::GameStatus;
use std::sync::Arc;
use std::{
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
};
use tungstenite::{accept, Message as TMessage};
trait EncodableMessage {
    fn encode_message(&self) -> Vec<u8>;
}

enum RequestType {
    Http,
    WebSocket,
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
    let dealer_pool = DealerPool::new();
    let arc_dealer_pool = Arc::new(dealer_pool);
    let arc_socket_pool: Arc<LobbySocketPool> = Arc::new(socket_pool);
    let pool = ThreadPool::new(20);
    let arc_repo: Arc<PostgresDatabase> = Arc::new(repository);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        // Think about if it's possible to simplify without repetetive cloning
        let clone_repo = Arc::clone(&arc_repo);
        let clone_dealer_pool = Arc::clone(&arc_dealer_pool);
        let clone_socket_pool = Arc::clone(&arc_socket_pool);
        pool.execute(move || {
            handle_connection(stream, clone_repo, clone_socket_pool, clone_dealer_pool);
        });
    }
}

fn determine_request_type(stream: &TcpStream) -> RequestType {
    let mut buffer = [0; 1024];
    let result = stream.peek(&mut buffer);

    match result {
        Ok(_) => {
            let request_str = String::from_utf8_lossy(&buffer);
            // improve checking for websocket handshake
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
    let mut websocket = accept(stream).unwrap();
    let message = websocket.read().unwrap();

    let bytes = match message {
        TMessage::Binary(bytes) => bytes,
        _ => panic!("Expected binary message"),
    };

    let mut reader = std::io::Cursor::new(bytes);

    // for now we have only joinLobbyRequest send from client on socket opening
    let request = JoinLobbyRequest::decode(&mut reader).unwrap();

    lobby_socket_pool.add(
        request.lobby_id,
        // create new constructor
        PlayerChannelClient {
            player_id: request.player_id,
            socket: websocket,
        },
    );
}

fn handle_http_request(
    mut stream: TcpStream,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<LobbySocketPool>,
    dealer_pool: Arc<DealerPool>,
) {
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();
    let status_line = "HTTP/1.1 200 OK";
    let path = request_line.split(" ").skip(1).next().unwrap();

    if path == "/gameStart" {
        // TODO: if game started return error;
        // TODO: retrieve lobby id, player id from request and validate them;
        // TODO: in future we need to generate tables in lobbies;
        // TODO: move this logic into ~GameOrchestrator
        let ids = socket_pool.get_active_player_ids_by_lobby_id(1);
        let users: Vec<User> = repo.get_users_by_ids(ids);
        let cur_lobby_id = 1;

        let mut dealer = Dealer::new(cur_lobby_id, Player::from_users(users));
        let game_state = dealer.start_new_table_loop();
        let mut cur_player_id = game_state.next_player_id;
        socket_pool.update_clients(game_state);

        loop {
            //TODO: handle the cases where a client is not responding, or has closed the connection;
            let request: PlayerPayload =
                socket_pool.read_client_message(cur_player_id, cur_lobby_id);

            let game_state = dealer.update_game_state(request);

            if game_state.game_status == GameStatus::None {
                break;
            }

            cur_player_id = game_state.next_player_id;

            socket_pool.update_clients(game_state);
        }
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
    dealer_pool: Arc<DealerPool>,
) {
    let reqest_type = determine_request_type(&stream);

    match reqest_type {
        RequestType::WebSocket => handle_web_socket_connection_handshake(stream, socket_pool),
        RequestType::Http => handle_http_request(stream, repo, socket_pool, dealer_pool),
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
