use std::path::Path;

use civorum_mapgen::pipeline::{map::Map, map_sizes::MapSizes};

pub mod debug_render;

pub use debug_render::render_map_png;

pub fn render_debug_map(
    seed: Option<u64>,
    size: MapSizes,
    cell_px: u32,
    out_path: &Path,
) -> Result<(), Box<dyn std::error::Error>> {
    let (width, height) = size.dimensions();
    let panic_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let terrain_result = std::panic::catch_unwind(|| Map::debug_terrains(seed, size));
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
