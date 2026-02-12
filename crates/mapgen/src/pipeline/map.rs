use rand_chacha::{ChaCha12Rng, rand_core::SeedableRng};

use crate::{map_components::tile::Tile, pipeline::map_sizes::MapSizes};

/// Map struct that holds all tiles as well as information about itself
pub struct Map {
    seed: Option<u64>,
    size: MapSizes,
    tiles: Vec<Tile>
}

impl Map {

    pub fn new(seed: Option<u64>, size: MapSizes) -> Self {
        
        let seed = seed.unwrap_or(0);
        let mut rng = ChaCha12Rng::seed_from_u64(seed);
        
        todo!()
    }

    pub fn show(self) {
        todo!()
    }
}