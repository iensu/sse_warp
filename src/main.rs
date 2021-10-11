use api::games::Games;

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

mod api;
mod util;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let game_clients = Arc::new(Mutex::new(HashMap::new()));
    let games: Games = Arc::new(Mutex::new(HashMap::new()));

    let routes = api::games::game_routes(games, game_clients);

    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
}
