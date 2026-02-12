use std::default;

use rand_chacha::{ChaCha12Rng, rand_core::SeedableRng};

use crate::{map_components::tile::Tile, pipeline::map_sizes::MapSizes};

/// Map struct that holds all tiles as well as information about itself
pub struct Map {
    seed: Option<u64>,
    size: MapSizes,
    tiles: Vec<Tile>
}

impl Map {

    /// Instatiate a new map with a given seed (or randomly assigned) and size
    pub fn new(seed: Option<u64>, size: MapSizes) -> Self {
        
        // use given seed or choose the default seed (13)
        let mut rng = ChaCha12Rng::seed_from_u64({
            let seed = match seed {
                Some(value) => value,
                None => 13
            };
            seed
        });


        todo!()
    }

    pub fn show(self) {
        todo!()
    }
}