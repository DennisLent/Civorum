use hexx::Hex;

use crate::terrain::Terrain;

pub struct Tile {
    hex: Hex,
    terrain: Terrain
}

impl Tile {
    pub fn new(hex: Hex, terrain: Terrain) -> Self {
        Tile{hex: hex, terrain: terrain}
    }

    pub fn hex(&self) -> &Hex {
        &self.hex
    }

    pub fn terrain(&self) -> &Terrain {
        &self.terrain
    }

    pub fn terrain_to_file(&self) -> &str {
        self.terrain.clone().terrain_to_file()
    }
}