use std::{collections::HashMap, convert::Infallible};

use crate::api::games::Games;

use super::models::{Game, GameCode};

pub async fn create_game(
    payload: HashMap<String, usize>,
    games: Games,
) -> Result<impl warp::Reply, Infallible> {
    let game = Game::new(
        *payload.get("players").unwrap(),
        *payload.get("rounds").unwrap(),
    );
    games.lock().unwrap().insert(game.code, game.clone());
    Ok(warp::reply::json(&game))
}

pub async fn get_game(code: GameCode, games: Games) -> Result<Box<dyn warp::Reply>, Infallible> {
    if let Some(game) = games.lock().unwrap().get(&code) {
        Ok(Box::new(warp::reply::json(&game)))
    } else {
        Ok(Box::new(warp::http::StatusCode::NOT_FOUND))
    }
}
