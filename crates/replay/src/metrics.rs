use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ReplayMetrics {
    pub win_rate: f32,
    pub final_score: i32,
    pub turns_played: u32,
}

