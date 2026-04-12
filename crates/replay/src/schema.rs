use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayRecord {
    pub id: Uuid,
    pub game_id: String,
    pub scenario_id: String,
    pub seed: u64,
    pub actions: Vec<ReplayAction>,
    pub events: Vec<ReplayEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayAction {
    pub turn: u32,
    pub player: String,
    pub action_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplayEvent {
    pub turn: u32,
    pub summary: String,
}

