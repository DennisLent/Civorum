use hexx::Hex;

use crate::terrain::Terrain;

#[derive(Debug, Clone)]
pub struct Tile {
    hex: Hex,
    terrain: Terrain,
    elevation: f32,   // [-1,1] approx
    temperature: f32, // [0,1]
    rainfall: f32,    // [0,1]
}

impl Tile {
    pub fn new(hex: Hex, terrain: Terrain, elevation: f32, temperature: f32, rainfall: f32) -> Self {
        Tile { hex, terrain, elevation, temperature, rainfall }
    }

    pub fn hex(&self) -> &Hex { &self.hex }
    pub fn terrain(&self) -> &Terrain { &self.terrain }
    pub fn elevation(&self) -> f32 { self.elevation }
    pub fn temperature(&self) -> f32 { self.temperature }
    pub fn rainfall(&self) -> f32 { self.rainfall }

    pub fn terrain_to_file(&self) -> &str { self.terrain.terrain_to_file() }
}
