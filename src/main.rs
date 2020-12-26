use api::games::Games;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use warp::Filter;

mod api;
mod util;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let chat_clients = Arc::new(Mutex::new(HashMap::new()));
    let game_clients = Arc::new(Mutex::new(HashMap::new()));
    let games: Games = Arc::new(Mutex::new(HashMap::new()));

    let routes =
        api::chat::chat_routes(chat_clients).or(api::games::game_routes(games, game_clients));

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
