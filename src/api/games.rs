use std::{
    collections::HashMap,
    convert::Infallible,
    sync::atomic::AtomicUsize,
    sync::{atomic::Ordering, Arc, Mutex},
};

use futures::{Stream, StreamExt};
use tokio::sync::mpsc;
use warp::{sse::ServerSentEvent, Filter};

mod handlers;
mod models;

pub type Games = Arc<Mutex<HashMap<models::GameCode, models::Game>>>;
pub type GameClients = Arc<Mutex<HashMap<usize, (usize, mpsc::UnboundedSender<Message>)>>>;

static NEXT_CLIENT_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug)]
pub enum Message {
    PlayerJoined(String),
}

pub fn game_routes(
    games: Games,
    clients: GameClients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("games" / ..).and(
        create_game(games.clone())
            .or(game_events(clients.clone()))
            .or(get_game(games.clone()))
            .or(player_join(games, clients)),
    )
}

fn create_game(
    games: Games,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::end()
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and(with_games(games))
        .and_then(handlers::create_game)
}

fn get_game(
    games: Games,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path::param()
        .and(warp::get())
        .and(with_games(games))
        .and_then(handlers::get_game)
}

fn player_join(
    games: Games,
    clients: GameClients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(usize / "join")
        .and(warp::post())
        .and(warp::body::content_length_limit(1024 * 32))
        .and(warp::body::json())
        .and(with_games(games))
        .and(with_clients(clients))
        .and_then(handlers::join_game)
}

fn game_events(
    clients: GameClients,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!(usize / "events")
        .and(warp::get())
        .and(with_clients(clients))
        .map(|game_code, clients| {
            let stream = client_connected(game_code, clients);
            warp::sse::reply(warp::sse::keep_alive().stream(stream))
        })
}

fn client_connected(
    game_code: usize,
    clients: GameClients,
) -> impl Stream<Item = Result<impl ServerSentEvent + Send + 'static, warp::Error>> + Send + 'static
{
    let id = NEXT_CLIENT_ID.fetch_add(1, Ordering::Relaxed);

    log::debug!("New game client: {}", id);

    let (tx, rx) = mpsc::unbounded_channel();

    clients.lock().unwrap().insert(id, (game_code, tx));

    rx.map(|msg| match msg {
        Message::PlayerJoined(player_id) => {
            Ok((warp::sse::event("message"), warp::sse::data(player_id)).into_a())
        }
        _ => Ok(warp::sse::data("Unknown message").into_b()),
    })
}

fn with_games(games: Games) -> impl Filter<Extract = (Games,), Error = Infallible> + Clone {
    warp::any().map(move || games.clone())
}

fn with_clients(
    clients: GameClients,
) -> impl Filter<Extract = (GameClients,), Error = Infallible> + Clone {
    warp::any().map(move || clients.clone())
}
