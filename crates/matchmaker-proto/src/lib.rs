use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub enum ConnectionType {
    Matchmaker,
    Game,
}

pub mod game;
pub mod matchmaker;
