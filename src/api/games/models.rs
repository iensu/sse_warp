use std::collections::HashMap;

use serde::{Deserialize, Serialize};

pub type GameCode = usize;
pub type PlayerId = usize;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Player {
    pub id: PlayerId,
    pub status: PlayerStatus,
    pub name: String,
    pub points: usize,
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
    pub code: GameCode,
    pub players: HashMap<PlayerId, Player>,
    pub status: GameStatus,
    pub total_players: usize,
    pub rounds: Vec<GameRound>,
}

impl Game {
    pub fn new(players: usize, _rounds: usize) -> Self {
        Self {
            code: 1234,
            players: HashMap::new(),
            status: GameStatus::Active,
            total_players: players,
            rounds: vec![],
        }
    }
}
