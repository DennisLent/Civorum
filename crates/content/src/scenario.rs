use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Scenario {
    pub id: String,
    pub game: String,
    pub seed: Option<u64>,
    pub map: String,
    pub factions: Vec<String>,
}

