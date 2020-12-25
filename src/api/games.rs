use std::{
    collections::HashMap,
    convert::Infallible,
    sync::{Arc, Mutex},
};

use warp::Filter;

mod handlers;
mod models;

pub type Games = Arc<Mutex<HashMap<models::GameCode, models::Game>>>;

pub fn game_routes(
    games: Games,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("games" / ..).and(create_game(games.clone()).or(get_game(games)))
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
        .and(with_games(games))
        .and_then(handlers::get_game)
}

fn with_games(games: Games) -> impl Filter<Extract = (Games,), Error = Infallible> + Clone {
    warp::any().map(move || games.clone())
}
