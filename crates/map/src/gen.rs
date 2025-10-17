use std::str::FromStr;

use hexx::Hex;
use noise::{Fbm, NoiseFn, OpenSimplex};

use crate::{Terrain, Tile};

/// The available high‑level map kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MapKind {
    Continents,
}

impl FromStr for MapKind {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_ascii_lowercase().as_str() {
            "continents" | "continent" | "" => Ok(MapKind::Continents),
            _ => Err(()),
        }
    }
}

/// Generate a simple continental map: heightmap → water/land only.
/// Mountains/other terrains are added in subsequent passes.
pub fn generate_continents(axials: &[Hex], seed: u64) -> Vec<Tile> {
    let fbm = Fbm::<OpenSimplex>::new(seed as u32);
    let freq = 0.050_f64; // low frequency for continental shapes
    let sea_level = 0.0_f64;

    axials
        .iter()
        .copied()
        .map(|h| {
            let q = h.x() as f64;
            let r = h.y() as f64;
            let nx = q * freq;
            let ny = r * freq;
            let val = fbm.get([nx, ny]); // [-1, 1]

            let terrain = if val < sea_level {
                Terrain::Water
            } else {
                Terrain::Grass
            };
            Tile::new(h, terrain)
        })
        .collect()
}
