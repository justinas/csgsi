//! GSI types

use serde::Deserialize;
use std::collections::HashMap;

#[derive(Clone, Default, PartialEq, Deserialize)]
pub struct GameState {
    #[serde(rename = "allplayers")]
    pub players: HashMap<u64, Player>,
}

impl GameState {
    pub fn players(&self, team: Team) -> Vec<&Player> {
        let mut ret: Vec<&Player> = self.players.values().filter(|p| p.team == team).collect();
        ret.sort_by_key(|p| p.observer_slot);
        ret
    }
}

#[derive(Clone, PartialEq, Deserialize)]
pub struct Player {
    pub name: String,
    pub observer_slot: u8,
    pub team: Team,
    pub state: PlayerState,
    pub match_stats: PlayerStats,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Deserialize)]
pub enum Team {
    T,
    CT,
}

impl Team {
    pub fn from_log_name(s: &str) -> Option<Team> {
        match s {
            "TERRORIST" => Some(Team::T),
            "CT" => Some(Team::CT),
            _ => None,
        }
    }
}

impl std::fmt::Display for Team {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Team::T => "T",
            Team::CT => "CT",
        })
    }
}

#[derive(Clone, PartialEq, Deserialize)]
pub struct PlayerState {
    pub health: u64,
}

#[derive(Clone, PartialEq, Deserialize)]
pub struct PlayerStats {
    pub kills: u64,
    pub assists: u64,
    pub deaths: u64,
}
