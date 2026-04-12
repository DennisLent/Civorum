use crate::{
    map_components::{terrain::Terrain, tile::Tile},
    pipeline::{
        biomes::{generate_map, generate_map_with_type},
        features::place_features,
        map_sizes::MapSizes,
        map_types::MapTypes,
    },
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
        let (terrain_vec, height, hill_vec, temp, rain) = generate_map(&internal_seed, &size);

        todo!()
    }

    pub fn debug_terrains(seed: Option<u64>, size: MapSizes, map_type: MapTypes) -> (Vec<Terrain>, Vec<bool>) {
        let internal_seed = match seed {
            Some(value) => value,
            None => 12,
        };

        // Create basic landmasses and Terrains
        let (terrain_vec, height, hill_vec, _temp, rain) =
            generate_map_with_type(&internal_seed, &size, map_type);

        let _ = place_features(&terrain_vec, &rain, &height, &size);

        (terrain_vec, hill_vec)
    }

    pub fn show(self) {
        todo!()
    }
}
