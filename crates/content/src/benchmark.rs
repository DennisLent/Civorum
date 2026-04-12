use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BenchmarkSuite {
    pub name: String,
    pub game: String,
    pub scenarios: Vec<String>,
    pub seeds: Vec<u64>,
    pub max_turns: u32,
    pub metrics: Vec<String>,
}

