use fun_poker::{
    dealer_pool::DealerPool,
    game::GameSettings,
    game_orchestrator::GameOrchestrator,
    postgres_database::PostgresDatabase,
    protos::{
        requests::{CreateLobbyRequest, JoinLobbyRequest, ObserveLobbyRequest, StartGameRequest},
        user::User,
    },
    responses::EncodableMessage,
    socket_pool::{PlayerChannelClient, SocketPool},
    thread_pool::ThreadPool
};

use prost::{DecodeError, Message};

use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Cursor, Read, Seek, SeekFrom, Write},
    net::{TcpListener, TcpStream},
    sync::Arc,
};

use tungstenite::accept;

enum RequestType {
    Http,
    WebSocket,
}

#[allow(clippy::derive_partial_eq_without_eq)]
#[derive(Clone, PartialEq, ::prost::Message)]
pub struct EmptyMessage {}

fn main() {
    // TODO: add multiple db connections for concurrency
    // r2d2 or deadpool-postgres or self implementation
    let repository: PostgresDatabase = PostgresDatabase::new().unwrap();
    let listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let socket_pool = SocketPool::new();
    let dealer_pool = DealerPool::new();
    let game_orchestrator = GameOrchestrator::new();

    let arc_dealer_pool = Arc::new(dealer_pool);
    let arc_socket_pool: Arc<SocketPool> = Arc::new(socket_pool);
    let arc_game_orchestrator: Arc<GameOrchestrator> = Arc::new(game_orchestrator);

    let pool = ThreadPool::new(20);
    let arc_thread_pool: Arc<ThreadPool> = Arc::new(pool);
    let arc_repo: Arc<PostgresDatabase> = Arc::new(repository);
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let clone_game_orchestrator = Arc::clone(&arc_game_orchestrator);
        let clone_pool = Arc::clone(&arc_thread_pool);
        let clone_repo = Arc::clone(&arc_repo);
        let clone_dealer_pool = Arc::clone(&arc_dealer_pool);
        let clone_socket_pool = Arc::clone(&arc_socket_pool);
        arc_thread_pool.execute(move || {
            handle_connection(
                stream,
                clone_repo,
                clone_socket_pool,
                clone_dealer_pool,
                clone_pool,
                clone_game_orchestrator,
            );
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
            // TODO: implement JWT verifier
            let user_id = map.get("user_id").unwrap();
            i32::from_str_radix(user_id, 10).unwrap()
        }
        Err(_) => todo!("Bad request"),
    };

    let websocket = accept(stream).unwrap();

    socket_pool.add(PlayerChannelClient {
        client_id: user_id,
        socket: websocket,
    });
}

fn handle_http_request(
    mut stream: TcpStream,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<SocketPool>,
    _dealer_pool: Arc<DealerPool>,
    thread_pool: Arc<ThreadPool>,
    game_orchestrator: Arc<GameOrchestrator>,
) {
    let mut buf_reader = BufReader::new(&stream);

    let request_line = buf_reader.by_ref().lines().next().unwrap().unwrap();

    let status_line = "HTTP/1.1 200 OK";

    let path = request_line.split(" ").skip(1).next().unwrap();

    let (message, status_line): (Box<dyn EncodableMessage>, &str) = match path {
        "/createLobby" => create_lobby_handler(buf_reader, repo, game_orchestrator),
        "/getLobbies" => (Box::new(repo.get_lobbies()), &status_line),
        "/startGame" => start_game_request_handler(
            buf_reader,
            repo,
            socket_pool,
            thread_pool,
            game_orchestrator,
        ),
        "/joinLobby" => join_lobby_request_handler(buf_reader, repo),
        // "/observeLobby" => observe_lobby_request_handler(buff_reader),
        _ => (Box::new(EmptyMessage {}), "HTTP/1.1 400 Bad Request"),
    };

    let response = construct_response(status_line, message);

    stream.write_all(&response).unwrap();
}

fn _observe_lobby_request_handler(buf_reader: BufReader<&TcpStream>) {
    let decode_fn = |cursor: &mut Cursor<&[u8]>| ObserveLobbyRequest::decode(cursor);

    let result = parse_message(buf_reader, decode_fn);

    let _request = match result {
        Ok(v) => v,
        _ => todo!(),
    };
}

fn parse_message<T, F>(mut buf_reader: BufReader<&TcpStream>, decode: F) -> Result<T, DecodeError>
where
    T: Message,
    F: for<'a> FnOnce(&'a mut Cursor<&'a [u8]>) -> Result<T, DecodeError>,
{
    let body_start = get_body_buffer_position(&buf_reader);

    let buffer = buf_reader.fill_buf().unwrap();

    let mut cursor = Cursor::new(buffer);
    cursor.seek(SeekFrom::Start(body_start as u64)).unwrap();

    let result: Result<T, DecodeError> = decode(&mut cursor);

    result
}

fn create_lobby_handler(
    buf_reader: BufReader<&TcpStream>,
    repo: Arc<PostgresDatabase>,
    game_orchestrator: Arc<GameOrchestrator>,
) -> (Box<dyn EncodableMessage>, &str) {
    let decode_fn = |cursor: &mut Cursor<&[u8]>| CreateLobbyRequest::decode(cursor);

    let result = parse_message(buf_reader, decode_fn);

    let create_lobby_request = match result {
        Ok(v) => v,
        _ => return (Box::new(EmptyMessage {}), "HTTP/1.1 400 Bad Request"),
    };

    let lobby_id = repo.create_lobby(create_lobby_request.payload.unwrap());

    let created = game_orchestrator.create_game(lobby_id, GameSettings { blind_size: 100 });

    if created {
        (Box::new(EmptyMessage {}), "HTTP/1.1 200 OK")
    } else {
        (Box::new(EmptyMessage {}), "HTTP/1.1 400 Bad Request")
    }
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

fn _parse_body(mut buf_reader: BufReader<&TcpStream>) -> Vec<u8> {
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

fn get_body_buffer_position(buf_reader: &BufReader<&TcpStream>) -> usize {
    let buffer = buf_reader.buffer();

    let headers_end = buffer
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|pos| pos + 4)
        .expect("Headers not found");

    headers_end
}

fn start_game_request_handler(
    buf_reader: BufReader<&TcpStream>,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<SocketPool>,
    thread_pool: Arc<ThreadPool>,
    game_orchestrator: Arc<GameOrchestrator>,
) -> (Box<dyn EncodableMessage>, &str) {
    let decode_fn = |cursor: &mut Cursor<&[u8]>| StartGameRequest::decode(cursor);

    let request = parse_message(buf_reader, decode_fn).unwrap();

    // TODO: if game started return error;
    // TODO: retrieve lobby id, player id from request and validate them;

    // THINK ABOUT HOW AND WHEN USER SHOULD BE ABLE TO JOIN THE GAME
    let _users: Vec<User> = repo.get_users_by_lobby_id(request.lobby_id);

    game_orchestrator.start_game(request.lobby_id, thread_pool, socket_pool);

    (Box::new(EmptyMessage {}), "HTTP/1.1 200 OK")
}

fn join_lobby_request_handler(
    buf_reader: BufReader<&TcpStream>,
    repo: Arc<PostgresDatabase>,
    // socket_pool: Arc<SocketPool>,
    // dealer_pool: Arc<DealerPool>,
) -> (Box<dyn EncodableMessage>, &str) {
    let decode_fn = |cursor: &mut Cursor<&[u8]>| JoinLobbyRequest::decode(cursor);

    let result = parse_message(buf_reader, decode_fn);

    let request = match result {
        Ok(v) => v,
        _ => return (Box::new(EmptyMessage {}), "HTTP/1.1 400 Bad Request"),
    };

    repo.add_user_to_lobby(request.lobby_id, request.player_id);

    (Box::new(EmptyMessage {}), "HTTP/1.1 200 OK")
}

fn handle_connection(
    stream: TcpStream,
    repo: Arc<PostgresDatabase>,
    socket_pool: Arc<SocketPool>,
    dealer_pool: Arc<DealerPool>,
    pool: Arc<ThreadPool>,
    game_orchestrator: Arc<GameOrchestrator>,
) {
    let reqest_type = determine_request_type(&stream);

    match reqest_type {
        RequestType::WebSocket => handle_web_socket_connection_handshake(stream, socket_pool),
        RequestType::Http => handle_http_request(
            stream,
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
