use base64::{engine::general_purpose::STANDARD, Engine as _};
use fun_poker::{
    dealer_pool::DealerPool,
    game::GameSettings,
    game_orchestrator::GameOrchestrator,
    postgres_database::PostgresDatabase,
    protos::{
        requests::{CreateLobbyRequest, ObserveLobbyRequest, SpawnBotRequest, StartGameRequest},
        user::User,
    },
    responses::EncodableMessage,
    socket_pool::{ConnectionClosedEvent, SocketClient, SocketPool},
    thread_pool::ThreadPool,
};

use prost::{DecodeError, Message};
use rustls::{ServerConfig, ServerConnection, StreamOwned};
use sha1::{Digest, Sha1};

use rustls_pemfile::{certs, private_key};
use serde_json::Value;
use std::{collections::HashMap, env, fs::File};
use std::{
    io::{BufReader, Cursor, Read, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};
use tungstenite::{
    http::StatusCode,
    protocol::{Role, WebSocketConfig},
    WebSocket,
};
enum RequestType {
    Http,
    WebSocket,
}

use dotenv::dotenv;

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EmptyMessage {}

#[derive(Debug, Default)]
struct QueryParams {
    lobby_id: Option<i32>,
}
pub struct Request {
    uri: String,
    headers: Vec<String>,
    body: Vec<u8>,
}

pub struct Configuration {
    address: String,
    db_connection: String,
}

enum ClaimTypesEnum {
    Country,
    Anonymous,
    NameIdentifier,
    UniqueName,
}

impl ClaimTypesEnum {
    fn as_str(&self) -> &'static str {
        match self {
            ClaimTypesEnum::Country => {
                "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/country"
            }
            ClaimTypesEnum::Anonymous => {
                "http://schemas.xmlsoap.org/ws/2005/05/identity/claims/anonymous"
            }
            ClaimTypesEnum::NameIdentifier => "nameid",
            ClaimTypesEnum::UniqueName => "unique_name",
        }
    }
}

pub const STATUS_BAD_REQUEST: &str = "HTTP/1.1 400 Bad Request";
pub const STATUS_OK: &str = "HTTP/1.1 200 OK";
pub const STATUS_INTERNAL_ERROR: &str = "HTTP/1.1 500 Internal Server Error";

fn load_tls_config(cert_path: &str, key_path: &str) -> Arc<ServerConfig> {
    let cert_file =
        &mut BufReader::new(File::open(cert_path).expect("cannot open certificate file"));
    let cert_chain = certs(cert_file).map(|i| i.unwrap()).collect();

    let key_file = &mut BufReader::new(File::open(key_path).expect("cannot open private key file"));
    let private_key = private_key(key_file).unwrap().unwrap();

    let config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, private_key)
        .expect("bad certificate/key");
    Arc::new(config)
}

fn main() {
    dotenv().ok();

    let configuration = load_configuration();

    let tls_config: Arc<ServerConfig> =
        load_tls_config("./src/localhost.crt", "./src/localhost.key");

    // TODO: add multiple db connections for concurrency
    // r2d2 or deadpool-postgres or self implementation
    let repository: PostgresDatabase = PostgresDatabase::new(&configuration.db_connection).unwrap();
    let listener = TcpListener::bind(&configuration.address).unwrap();
    let socket_pool = SocketPool::new();
    let dealer_pool = DealerPool::new();

    let game_orchestrator = GameOrchestrator::new();
    let arc_dealer_pool = Arc::new(dealer_pool);
    let arc_socket_pool: Arc<SocketPool> = Arc::new(socket_pool);
    let arc_game_orchestrator: Arc<GameOrchestrator> = Arc::new(game_orchestrator);

    let pool = ThreadPool::new(20);
    let arc_thread_pool: Arc<ThreadPool> = Arc::new(pool);
    let arc_repo: Arc<PostgresDatabase> = Arc::new(repository);

    setup_sockets_health_checker(&arc_game_orchestrator, &arc_socket_pool);

    for stream in listener.incoming() {
        // TODO: think about dependecy injection pattern to decrease arguments amount
        let tls_config = tls_config.clone();
        let stream = stream.unwrap();
        let tls_stream = rustls::StreamOwned::new(
            rustls::ServerConnection::new(tls_config).expect("failed to create server connection"),
            stream,
        );

        let clone_game_orchestrator = Arc::clone(&arc_game_orchestrator);
        let clone_pool = Arc::clone(&arc_thread_pool);
        let clone_repo = Arc::clone(&arc_repo);
        let clone_dealer_pool = Arc::clone(&arc_dealer_pool);
        let clone_socket_pool = Arc::clone(&arc_socket_pool);
        arc_thread_pool.execute(move || {
            handle_connection(
                tls_stream,
                clone_repo,
                clone_socket_pool,
                clone_dealer_pool,
                clone_pool,
                clone_game_orchestrator,
            );
        });
    }
}

fn load_configuration() -> Configuration {
    let is_docker_env = env::var("RUN_IN_DOCKER").is_ok();

    if is_docker_env {
        let address = env::var("IP_PORT").expect("IP_PORT must be set in Docker environment");
        let db_connection =
            env::var("DATABASE_URL").expect("DATABASE_URL must be set in Docker environment");

        Configuration {
            address,
            db_connection,
        }
    } else {
        let args: Vec<String> = env::args().collect();

        let address = match env::var("ADDRESS") {
            Ok(addr) => addr,
            Err(_) => {
                if args.len() < 2 {
                    eprintln!("Usage: {} <IP:PORT> <DATABASE_URL>", args[0]);
                    std::process::exit(1);
                }
                args[1].clone()
            }
        };

        let db_connection = match env::var("DATABASE_URL") {
            Ok(db_con) => db_con,
            Err(_) => {
                if args.len() < 3 {
                    eprintln!("Usage: {} <IP:PORT> <DATABASE_URL>", args[0]);
                    std::process::exit(1);
                }
                args[2].clone()
            }
        };

        Configuration {
            address,
            db_connection,
        }
    }
}

fn setup_sockets_health_checker(
    game_orchestrator: &Arc<GameOrchestrator>,
    socket_pool: &Arc<SocketPool>,
) {
    let socket_event_listener = socket_pool.spawn_health_checker();

    let game_o = Arc::clone(game_orchestrator);
    let socket_o = Arc::clone(socket_pool);

    let on_connection_closed = Box::new(move |e: ConnectionClosedEvent| {
        game_o.update_player_connection_status(e, &socket_o);
    });

    socket_event_listener
        .lock()
        .unwrap()
        .push(on_connection_closed);
}

fn determine_request_type(request: &Request) -> RequestType {
    let socket = request.headers.contains(&"Upgrade: websocket".to_string());

    if socket {
        RequestType::WebSocket
    } else {
        RequestType::Http
    }
}

fn get_path_from_uri(uri: &str) -> &str {
    uri.split("?").next().unwrap()
}

fn parse_user_from_claims(claims: Value) -> User {
    let mut user = User::default();
    user.country = claims[ClaimTypesEnum::Country.as_str()]
        .as_str()
        .unwrap()
        .to_string();
    user.name = claims[ClaimTypesEnum::UniqueName.as_str()]
        .as_str()
        .unwrap()
        .to_string();
    user.id = str::parse::<i32>(
        claims[ClaimTypesEnum::NameIdentifier.as_str()]
            .as_str()
            .unwrap(),
    )
    .unwrap();
    user
}

fn parse_claims_from_request(request: &Request) -> Option<Value> {
    let claims = {
        let jwt_payload = request
            .headers
            .iter()
            .find(|p| p.starts_with("X-JWT-Payload"));

        let value = if let Some(jwt_payload) = jwt_payload {
            let json = jwt_payload.replacen("X-JWT-Payload:", "", 1);

            let json_data: Result<Value, serde_json::Error> = serde_json::from_str(&json);
            if json_data.is_ok() {
                json_data.unwrap()
            } else {
                return None;
            }
        } else {
            return None;
        };
        value
    };
    Some(claims)
}

fn handle_web_socket_request(
    mut stream: StreamOwned<ServerConnection, TcpStream>,
    request: self::Request,
    socket_pool: Arc<SocketPool>,
    thread_pool: Arc<ThreadPool>,
    game_orchestrator: Arc<GameOrchestrator>,
    repo: Arc<PostgresDatabase>,
) {
    let headers = generate_websocket_accept_headers(&request);

    send_websocket_handshake_response(&mut stream, headers);

    stream.sock.set_nonblocking(true).unwrap();

    let websocket =
        WebSocket::from_raw_socket(stream, Role::Server, Some(WebSocketConfig::default()));

    let claims = parse_claims_from_request(&request).unwrap();
    let user = parse_user_from_claims(claims);
    let uri = request.uri.split(" ").skip(1).next().unwrap();
    let path = get_path_from_uri(uri);
    let query_params = parse_queries_from_uri(uri);

    let result = match path {
        "/ws" => root_socket_connection_handler(user, websocket, socket_pool),
        "/join_lobby" => join_lobby_request_socket_handler(
            user,
            query_params,
            websocket,
            repo,
            game_orchestrator,
            socket_pool,
            thread_pool,
        ),
        _ => Err((websocket, STATUS_BAD_REQUEST)),
    };

    match result {
        Ok(_) => {}
        Err(mut e) => {
            e.0.write(e.1.into()).unwrap();
            e.0.write(tungstenite::Message::Close(None)).unwrap();
        }
    }
}

fn send_websocket_handshake_response(
    stream: &mut StreamOwned<ServerConnection, TcpStream>,
    headers: HashMap<String, String>,
) -> bool {
    let response_str = format!("HTTP/1.1 {} \r\n", StatusCode::SWITCHING_PROTOCOLS);
    let headers_str = headers
        .iter()
        .map(|(key, value)| format!("{}:{}\r\n", key, value))
        .collect::<String>();
    let response_data = format!("{}{}\r\n", response_str, headers_str);
    stream.write_all(response_data.as_bytes()).unwrap();

    true
}

fn parse_websocket_headers(request: &Request) -> HashMap<String, String> {
    let mut map = HashMap::new();
    request
        .headers
        .iter()
        .filter(|p| p.contains("Sec-WebSocket"))
        .for_each(|h| {
            let key_value_string = h.split_once(": ").unwrap();
            map.insert(
                key_value_string.0.to_string(),
                key_value_string.1.to_string(),
            );
        });
    map
}

fn generate_websocket_accept_headers(request: &Request) -> HashMap<String, String> {
    let mut headers = HashMap::new();

    let mut request_headers = parse_websocket_headers(&request);

    let key = request_headers.remove("Sec-WebSocket-Key").unwrap();
    let version = request_headers.remove("Sec-WebSocket-Version").unwrap();

    // TODO: add when will be supported
    // let extenion_header_value =request_headers.remove("Sec-WebSocket-Extensions").unwrap();

    let combined_key = format!("{}{}", key, "258EAFA5-E914-47DA-95CA-C5AB0DC85B11");

    let mut hasher = Sha1::new();

    hasher.update(combined_key.as_bytes());
    let result = hasher.finalize();

    let base64_encoded = STANDARD.encode(&result);

    headers.insert("Upgrade".to_string(), "websocket".to_string());
    headers.insert("Connection".to_string(), "Upgrade".to_string());
    headers.insert("Sec-WebSocket-Accept".to_string(), base64_encoded);
    headers.insert("Sec-WebSocket-Version".to_string(), version);
    headers
}

fn root_socket_connection_handler(
    user: User,
    websocket: WebSocket<StreamOwned<ServerConnection, TcpStream>>,
    socket_pool: Arc<SocketPool>,
) -> Result<
    (),
    (
        WebSocket<StreamOwned<ServerConnection, TcpStream>>,
        &'static str,
    ),
> {
    socket_pool.add(SocketClient {
        client_id: user.id,
        socket: websocket,
        path: String::from("ws"),
    });

    Ok(())
}

fn join_lobby_request_socket_handler(
    user: User,
    query_params: QueryParams,
    websocket: WebSocket<StreamOwned<ServerConnection, TcpStream>>,
    repo: Arc<PostgresDatabase>,
    game_orchestrator: Arc<GameOrchestrator>,
    socket_pool: Arc<SocketPool>,
    thread_pool: Arc<ThreadPool>,
) -> Result<
    (),
    (
        WebSocket<StreamOwned<ServerConnection, TcpStream>>,
        &'static str,
    ),
> {
    let lobby_id = if let Some(lobby_id) = query_params.lobby_id {
        lobby_id
    } else {
        return Err((websocket, STATUS_BAD_REQUEST));
    };

    // TODO: validate lobby id
    // if !is_temp_user {
    //     repo.add_user_to_lobby(lobby_id, user.id);
    // }

    let game_created = if !game_orchestrator.is_game_exists(lobby_id) {
        let created = game_orchestrator.create_game(lobby_id, GameSettings { blind_size: 100 });
        created
    } else {
        true
    };

    if !game_created {
        return Err((websocket, STATUS_INTERNAL_ERROR));
    }

    // TODO: think about sending messages to game_orchestrator...
    socket_pool.add(SocketClient {
        client_id: user.id,
        socket: websocket,
        path: String::from("join_lobby"),
    });

    game_orchestrator.join_game(lobby_id, user, &socket_pool);

    let should_start = game_orchestrator.should_start_game(lobby_id);

    if should_start {
        game_orchestrator.start_game(lobby_id, thread_pool, socket_pool)
    }

    Ok(())
}

fn handle_http_request(
    mut stream: StreamOwned<ServerConnection, TcpStream>,
    request: Request,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<SocketPool>,
    _dealer_pool: Arc<DealerPool>,
    thread_pool: Arc<ThreadPool>,
    game_orchestrator: Arc<GameOrchestrator>,
) {
    let status_line = STATUS_OK;
    let path = request.uri.split(" ").skip(1).next().unwrap();

    let (message, status_line): (Box<dyn EncodableMessage>, &str) = match path {
        "/createLobby" => create_lobby_handler(request, repo, game_orchestrator),
        "/getLobbies" => (Box::new(repo.get_lobbies()), &status_line),
        "/startGame" => {
            start_game_request_handler(request, repo, socket_pool, thread_pool, game_orchestrator)
        }
        "/spawnAIBot" => spawn_ai_bot_handler(request, game_orchestrator, socket_pool),
        // "/observeLobby" => observe_lobby_request_handler(buff_reader),
        _ => (Box::new(EmptyMessage {}), STATUS_BAD_REQUEST),
    };

    let response = construct_response(status_line, message);
    stream.write_all(&response).unwrap();
}

fn _observe_lobby_request_handler(request: Request) {
    let decode_fn = |cursor: &mut Cursor<Vec<u8>>| ObserveLobbyRequest::decode(cursor);

    let result = parse_message(request.body, decode_fn);

    let _request = match result {
        Ok(v) => v,
        _ => todo!(),
    };
}

fn spawn_ai_bot_handler(
    request: Request,
    game_orchestrator: Arc<GameOrchestrator>,
    socket_pool: Arc<SocketPool>,
) -> (Box<dyn EncodableMessage>, &'static str) {
    let decode_fn = |cursor: &mut Cursor<Vec<u8>>| SpawnBotRequest::decode(cursor);

    let result = parse_message(request.body, decode_fn);

    let request = match result {
        Ok(v) => v,
        _ => return (Box::new(EmptyMessage {}), STATUS_BAD_REQUEST),
    };

    game_orchestrator.spawn_bot(request.lobby_id, &socket_pool);

    (Box::new(EmptyMessage {}), STATUS_OK)
}

fn parse_message<T, F>(body: Vec<u8>, decode: F) -> Result<T, DecodeError>
where
    T: Message,
    F: for<'a> FnOnce(&'a mut Cursor<Vec<u8>>) -> Result<T, DecodeError>,
{
    let mut cursor = Cursor::new(body);

    let result: Result<T, DecodeError> = decode(&mut cursor);

    result
}

fn create_lobby_handler(
    request: Request,
    repo: Arc<PostgresDatabase>,
    game_orchestrator: Arc<GameOrchestrator>,
) -> (Box<dyn EncodableMessage>, &'static str) {
    let decode_fn = |cursor: &mut Cursor<Vec<u8>>| CreateLobbyRequest::decode(cursor);

    let result = parse_message(request.body, decode_fn);

    let create_lobby_request = match result {
        Ok(v) => v,
        _ => return (Box::new(EmptyMessage {}), STATUS_BAD_REQUEST),
    };

    let lobby_id = repo.create_lobby(create_lobby_request.payload.unwrap());

    let created = game_orchestrator.create_game(lobby_id, GameSettings { blind_size: 100 });

    if created {
        return (Box::new(EmptyMessage {}), STATUS_OK);
    } else {
        return (Box::new(EmptyMessage {}), STATUS_BAD_REQUEST);
    }
}

fn parse_queries_from_uri(uri: &str) -> QueryParams {
    let query_start: usize = match uri.find("?") {
        Some(pos) => pos,
        None => return QueryParams::default(),
    };

    let queries = &uri[query_start + 1..];
    let pairs = queries.split("&");

    let mut keys_values = QueryParams::default();

    for pair in pairs {
        let mut key_value = pair.split("=");
        if let (Some(key), Some(value)) = (key_value.next(), key_value.next()) {
            match key {
                "lobby_id" => {
                    keys_values.lobby_id = value.parse::<i32>().ok();
                }
                _ => {}
            }
        }
    }

    keys_values
}

fn get_body_buffer_position(buffer: &Vec<u8>) -> usize {
    let headers_end = buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
        .expect("Headers not found");

    headers_end
}

fn start_game_request_handler(
    request: Request,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<SocketPool>,
    thread_pool: Arc<ThreadPool>,
    game_orchestrator: Arc<GameOrchestrator>,
) -> (Box<dyn EncodableMessage>, &'static str) {
    let decode_fn = |cursor: &mut Cursor<Vec<u8>>| StartGameRequest::decode(cursor);

    let request = parse_message(request.body, decode_fn).unwrap();

    let _users: Vec<User> = repo.get_users_by_lobby_id(request.lobby_id);

    game_orchestrator.start_game(request.lobby_id, thread_pool, socket_pool);

    (Box::new(EmptyMessage {}), STATUS_OK)
}

fn parse_request(stream: &mut StreamOwned<ServerConnection, TcpStream>) -> Request {
    let mut buffer = vec![0; 3000];

    let bytes_read = stream.read(&mut buffer).unwrap();

    buffer.resize(bytes_read, 0);

    let bodystart = get_body_buffer_position(&buffer);

    let request_str = String::from_utf8_lossy(&buffer[..bodystart]);

    let mut request_lines = request_str.lines();

    let uri = request_lines.next().unwrap_or("").to_string();

    let headers: Vec<String> = request_lines
        .take_while(|l| !l.is_empty())
        .map(str::to_string)
        .collect();

    let mut body = Vec::new();

    body.extend_from_slice(&buffer[bodystart..buffer.len()]);

    Request { uri, headers, body }
}

fn handle_connection(
    mut stream: StreamOwned<ServerConnection, TcpStream>,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<SocketPool>,
    dealer_pool: Arc<DealerPool>,
    pool: Arc<ThreadPool>,
    game_orchestrator: Arc<GameOrchestrator>,
) {
    let request = parse_request(&mut stream);
    let reqest_type = determine_request_type(&request);

    match reqest_type {
        RequestType::WebSocket => {
            handle_web_socket_request(stream, request, socket_pool, pool, game_orchestrator, repo)
        }
        RequestType::Http => handle_http_request(
            stream,
            request,
            repo,
            socket_pool,
            dealer_pool,
            pool,
            game_orchestrator,
        ),
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
