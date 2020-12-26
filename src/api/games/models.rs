use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type GameCode = usize;
pub type PlayerId = String;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub status: PlayerStatus,
    pub id: String,
    pub points: usize,
}

impl Player {
    pub fn new(id: String) -> Self {
        Self {
            id,
            status: PlayerStatus::Joined,
            points: 0,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PlayerStatus {
    Joined,
    Ready,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GameRound {
    pub order: usize,
    pub status: GameRoundStatus,
    pub caption: String,
    pub present_image: String,
    pub images: HashMap<PlayerId, Image>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameRoundStatus {
    NotStarted,
    SelectGif,
    Present,
    Vote,
    Finished,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Image {
    id: usize,
    url: String,
    player: PlayerId,
    votes: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GameStatus {
    Active,
    Finished,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Game {
    code: GameCode,
    players: HashMap<PlayerId, Player>,
    status: GameStatus,
    total_players: usize,
    rounds: Vec<GameRound>,
}

impl Game {
    pub fn new(code: usize, players: usize, _rounds: usize) -> Self {
        Self {
            code,
            players: HashMap::new(),
            status: GameStatus::Active,
            total_players: players,
            rounds: vec![],
        }
    }

    pub fn add_player(&mut self, p: &Player) {
        self.players.insert(p.id.clone(), p.clone());
    }

    pub fn player_exists(&self, id: &String) -> bool {
        self.players.keys().into_iter().find(|&k| k == id).is_some()
    }
}
