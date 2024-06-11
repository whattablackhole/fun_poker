use fun_poker::{
    dealer::Dealer,
    dealer_pool::DealerPool,
    player::PlayerPayloadError,
    postgres_database::PostgresDatabase,
    protos::{
        client_request::JoinLobbyRequest,
        player::{Player, PlayerPayload}, user::User,
    },
    socket_pool::{PlayerChannelClient, ReadMessageError, SocketPool},
    ThreadPool,
};

use prost::Message;

use std::{
    collections::HashMap,
    io::{prelude::*, BufReader},
    net::{TcpListener, TcpStream},
    sync::Arc,
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
    let socket_pool = SocketPool::new();
    let dealer_pool = DealerPool::new();
    let arc_dealer_pool = Arc::new(dealer_pool);
    let arc_socket_pool: Arc<SocketPool> = Arc::new(socket_pool);
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



fn handle_web_socket_connection_handshake(stream: TcpStream, socket_pool: Arc<SocketPool>) {
    let mut buffer = [0; 1024];
    let result: Result<usize, std::io::Error> = stream.peek(&mut buffer);
    let request_str = String::from_utf8_lossy(&buffer);

    let request_uri = request_str.lines().next().unwrap();
    let query_uri = request_uri.split(" ").skip(1).next().unwrap();

    let user_id: i32 = match result {
        Ok(_) => {
            let map = parse_queries_from_url(query_uri);
            let user_id = map.get("user_id").unwrap();
            i32::from_str_radix(user_id, 10).unwrap()
        }
        Err(_) => todo!("Bad request"),
    };

    let websocket = accept(stream).unwrap();


    socket_pool.add(PlayerChannelClient {
        player_id: user_id,
        socket: websocket,
    });
}

fn handle_http_request(
    mut stream: TcpStream,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<SocketPool>,
    dealer_pool: Arc<DealerPool>,
) {
    let mut buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader.by_ref().lines().next().unwrap().unwrap();
    
    let status_line = "HTTP/1.1 200 OK";

    let path = request_line.split(" ").skip(1).next().unwrap();


    let message: Box<dyn EncodableMessage> = match path {
        "/getLobbies" => Box::new(repo.get_lobbies()),
        "/startGame" => Box::new(start_game_request_handler(repo, socket_pool)),
        "/joinLobby" => Box::new(join_lobby_request_handler(parse_body(buf_reader), repo)),
        _ => Box::new(EmptyMessage {}),
    };

    let response = construct_response(status_line, message);

    stream.write_all(&response).unwrap();
}


fn parse_queries_from_url(url: &str) -> HashMap<&str, &str> {
    let query_start = url.find("?").unwrap();

    let queries = &url[query_start + 1..];

    let pairs = queries.split("&");

    let mut keys_values = HashMap::new();

    for pair in pairs {
        let mut key_value = pair.split("=");

        let key = key_value.next().unwrap();
        let value = key_value.next().unwrap();

        keys_values.insert(key, value);
    }
    keys_values
}

fn parse_body(mut buf_reader: BufReader<&mut TcpStream>) -> Vec<u8> {
    let mut headers = Vec::new();

    loop {
        let mut line = Vec::new();
        buf_reader.read_until(b'\n', &mut line).unwrap();

        if line == b"\r\n" {
            break;
        }
        headers.extend_from_slice(&line);
    }

    let headers_str = String::from_utf8_lossy(&headers);

    let content_length = headers_str
        .lines()
        .find(|line| line.starts_with("Content-Length:"))
        .and_then(|line| line.split(':').nth(1))
        .and_then(|len| len.trim().parse::<usize>().ok())
        .unwrap_or(0);

    let mut body = vec![0; content_length];

    if content_length > 0 {
        buf_reader.read_exact(&mut body).unwrap();
    }

    body
}

fn start_game_request_handler(repo: Arc<PostgresDatabase>, socket_pool: Arc<SocketPool>) {
  // TODO: if game started return error;
        // TODO: retrieve lobby id, player id from request and validate them;
        // TODO: in future we need to generate tables in lobbies;
        // TODO: move this logic into ~GameOrchestrator
        let ids = Vec::new();
        let users: Vec<User> = repo.get_users_by_ids(ids);
        let cur_lobby_id = 1;

        let mut dealer = Dealer::new(cur_lobby_id, Player::from_users(users));

        let game_states = dealer.start_new_game().unwrap();

        socket_pool.update_clients(game_states);

        loop {
            //TODO: handle the cases where a client is not responding, or has closed the connection;
            //TODO: use seperate messages for separated responses to decrease memory load and bandwidth
            let request: Result<PlayerPayload, ReadMessageError> =
                socket_pool.read_client_message(dealer.get_next_player_id(), cur_lobby_id);

            let result: Result<PlayerPayload, PlayerPayloadError> = match request {
                Ok(p) => Ok(p),
                Err(ReadMessageError::Disconnected) => {
                    socket_pool.close_connection(dealer.get_next_player_id());
                    Err(PlayerPayloadError::Disconnected {
                        id: dealer.get_next_player_id(),
                        lobby_id: cur_lobby_id,
                    })
                }
                Err(ReadMessageError::Iddle) => Err(PlayerPayloadError::Iddle {
                    id: dealer.get_next_player_id(),
                    lobby_id: cur_lobby_id,
                }),
            };

            let updated_state = dealer.update_game_state(result);

            socket_pool.update_clients(updated_state.client_states);

            let mut is_ready = updated_state.is_ready_for_next_hand;

            if updated_state.should_complete_game_cycle_automatically {
                let updated_state = dealer.complete_game_cycle_automatically();
                socket_pool.update_clients(updated_state.client_states);
                is_ready = updated_state.is_ready_for_next_hand;
            }

            while is_ready {
                let updated_state = dealer.setup_next_hand();

                socket_pool.update_clients(updated_state.client_states);

                if updated_state.should_complete_game_cycle_automatically {
                    let updated_state = dealer.complete_game_cycle_automatically();
                    is_ready = updated_state.is_ready_for_next_hand;
                    socket_pool.update_clients(updated_state.client_states);
                } else {
                    is_ready = updated_state.is_ready_for_next_hand;
                }
            }

            // use dealer api instead
            // if game_state.game_status == GameStatus::None {
            //     break;
            // }
        }
}


fn join_lobby_request_handler(
    body: Vec<u8>,
    repo: Arc<PostgresDatabase>,
    // socket_pool: Arc<SocketPool>,
    // dealer_pool: Arc<DealerPool>,
) -> String {
    let mut reader = std::io::Cursor::new(body);

    let request = JoinLobbyRequest::decode(&mut reader).unwrap();

    repo.add_user_to_lobby(request.lobby_id, request.player_id);

    String::from("Ok")
}

fn handle_connection(
    stream: TcpStream,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<SocketPool>,
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
