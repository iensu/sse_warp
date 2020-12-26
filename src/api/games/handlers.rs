use std::{
    collections::{hash_map::Entry, HashMap},
    convert::Infallible,
};

use crate::{api::games::Games, util::random_number};

use super::{
    models::{Game, GameCode, Player},
    GameClients, Message,
};

pub async fn create_game(
    payload: HashMap<String, usize>,
    games: Games,
) -> Result<impl warp::Reply, Infallible> {
    let code = (random_number() % 8999) + 1000;
    let game = Game::new(
        code as usize,
        *payload.get("players").unwrap(),
        *payload.get("rounds").unwrap(),
    );
    games.lock().unwrap().insert(code as usize, game.clone());
    Ok(warp::reply::json(&game))
}

pub async fn get_game(code: GameCode, games: Games) -> Result<Box<dyn warp::Reply>, Infallible> {
    if let Some(game) = games.lock().unwrap().get(&code) {
        Ok(Box::new(warp::reply::json(&game)))
    } else {
        Ok(Box::new(warp::http::StatusCode::NOT_FOUND))
    }
}

pub async fn join_game(
    code: GameCode,
    payload: HashMap<String, String>,
    games: Games,
    clients: GameClients,
) -> Result<Box<dyn warp::Reply>, Infallible> {
    let name = payload.get("name");

    if name.is_none() {
        return Ok(Box::new(warp::http::StatusCode::BAD_REQUEST));
    }

    let name = name.unwrap();

    match games.lock().unwrap().entry(code) {
        Entry::Occupied(mut game) => {
            let game = game.get_mut();

            if game.player_exists(&name) {
                return Ok(Box::new(warp::http::StatusCode::BAD_REQUEST));
            }

            let p = Player::new(name.clone());
            game.add_player(&p);

            clients
                .lock()
                .unwrap()
                .retain(|_client_id, (game_code, tx)| {
                    if *game_code == code {
                        tx.send(Message::PlayerJoined(name.clone())).is_ok()
                    } else {
                        true
                    }
                });

            Ok(Box::new(warp::reply::json(&p)))
        }
        _ => Ok(Box::new(warp::http::StatusCode::NOT_FOUND)),
    }
}
