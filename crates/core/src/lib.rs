use std::path::Path;

use civorum_content::{GameDefinition, Scenario};
use civorum_generators::pipeline::{map::Map, map_sizes::MapSizes, map_types::MapTypes};
use civorum_replay::ReplayRecord;
use civorum_rules::RuleSet;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod debug_render;

pub use debug_render::render_map_png;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct State {
    pub turn: u32,
    pub active_player: usize,
    pub scenario_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Action {
    pub actor_id: String,
    pub action_id: String,
    pub target: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Effect {
    pub summary: String,
}

#[derive(Debug, Clone)]
pub struct EngineContract {
    pub game: GameDefinition,
    pub scenario: Scenario,
    pub rules: RuleSet,
}

impl EngineContract {
    pub fn replay_stub(&self, seed: u64) -> ReplayRecord {
        ReplayRecord {
            id: Uuid::new_v4(),
            game_id: self.game.id.clone(),
            scenario_id: self.scenario.id.clone(),
            seed,
            actions: Vec::new(),
            events: Vec::new(),
        }
    }
}

pub fn render_debug_map(
    seed: Option<u64>,
    size: MapSizes,
    map_type: MapTypes,
    cell_px: u32,
    out_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = size.dimensions();
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let terrain_result = std::panic::catch_unwind(|| Map::debug_terrains(seed, size, map_type));
    std::panic::set_hook(panic_hook);
    let (terrain_vec, hill_vec) = terrain_result.map_err(|_| {
        "map generation panicked while building debug terrain data (check mapgen biome indexing)"
    })?;

    render_map_png(
        &terrain_vec,
        &hill_vec,
        i32::try_from(width)?,
        i32::try_from(height)?,
        cell_px,
        out_path,
    )
}
