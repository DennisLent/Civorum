use crate::{
    map_components::{terrain::Terrain, tile::Tile},
    pipeline::{biomes::generate_map, map_sizes::MapSizes},
};

/// Map struct that holds all tiles as well as information about itself
pub struct Map {
    seed: Option<u64>,
    size: MapSizes,
    tiles: Vec<Tile>,
}

impl Map {
    /// Instatiate a new map with a given seed (or randomly assigned) and size
    pub fn new(seed: Option<u64>, size: MapSizes) -> Self {
        // use given seed or choose the default seed (13)
        let internal_seed = match seed {
            Some(value) => value,
            None => 12,
        };

        // Create basic landmasses and Terrains
        let (terrain_vec, hill_vec, temp, rain) = generate_map(&internal_seed, &size);

        todo!()
    }

    pub fn debug_terrains(seed: Option<u64>, size: MapSizes) -> (Vec<Terrain>, Vec<bool>) {
        let internal_seed = match seed {
            Some(value) => value,
            None => 12,
        };

        // Create basic landmasses and Terrains
        let (terrain_vec, hill_vec, temp, rain) = generate_map(&internal_seed, &size);

        (terrain_vec, hill_vec)
    }

    pub fn show(self) {
        todo!()
    }
}
