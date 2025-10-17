use std::str::FromStr;

use hexx::{Hex, conversions::OffsetHexMode, HexOrientation};
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

/// Generate continental map with elevation (water/land), then biomes from
/// temperature (latitudinal) and rainfall (noise).
pub fn generate_continents(axials: &[Hex], seed: u64, map_height: u32) -> Vec<Tile> {
    // Elevation: low-frequency FBM
    let elev = Fbm::<OpenSimplex>::new(seed as u32);
    let elev_freq = 0.0225_f64; // slightly wider continents
    let sea_level = 0.0_f64;

    // Rainfall: combine a base field with multiple high‑frequency peak fields
    // and a slow amplitude mask; all sampled in normalized map space.
    let rain_base = Fbm::<OpenSimplex>::new((seed ^ 0x9E37_79B9) as u32);
    let rain_peaks_a = Fbm::<OpenSimplex>::new((seed ^ 0xC2B2_AE35) as u32);
    let rain_peaks_b = Fbm::<OpenSimplex>::new((seed ^ 0x27D4_EB2F) as u32);
    let rain_amp     = Fbm::<OpenSimplex>::new((seed ^ 0x1656_7EAD) as u32);

    // Temperature gradient based on row index distance from map center (0 at edges, 1 at center)
    let half = ((map_height as f64) - 1.0) / 2.0;
    // Small temperature noise to add diversity (sampled in normalized space)
    let temp_noise = Fbm::<OpenSimplex>::new((seed ^ 0xA5A5_5A5A) as u32);
    let temp_noise_amp = 0.15_f64;

    // Compute axial bounds to normalize coordinates into [0,1]
    let (mut min_q, mut max_q) = (i32::MAX, i32::MIN);
    let (mut min_r, mut max_r) = (i32::MAX, i32::MIN);
    for h in axials.iter().copied() {
        min_q = min_q.min(h.x());
        max_q = max_q.max(h.x());
        min_r = min_r.min(h.y());
        max_r = max_r.max(h.y());
    }
    let span_q = (max_q - min_q).max(1) as f64;
    let span_r = (max_r - min_r).max(1) as f64;

    axials
        .iter()
        .copied()
        .map(|h| {
            let q = h.x() as f64;
            let r = h.y() as f64;

            // Elevation
            let e = elev.get([q * elev_freq, r * elev_freq]);
            if e < sea_level {
                return Tile::new(h, Terrain::Water, e as f32, 0.0, 0.0);
            }

            // Temperature: hot near center row, cold near top/bottom + small noise
            let row = h
                .to_offset_coordinates(OffsetHexMode::Odd, HexOrientation::Flat)[1] as f64;
            let dist = (row - half).abs() / half.max(1.0); // 0 at center, 1 at edges
            let base_temp = (1.0 - dist).clamp(0.0, 1.0);
            let uq = (q - min_q as f64) / span_q;
            let ur = (r - min_r as f64) / span_r;
            // ~3 cycles across the map for temperature noise
            let tn = temp_noise.get([uq * 3.0, ur * 3.0]); // [-1,1]
            let temp = (base_temp + tn * temp_noise_amp).clamp(0.0, 1.0);

            // Rainfall: create small intense pockets
            let rb = (rain_base.get([uq * 2.0, ur * 2.0]) + 1.0) * 0.5; // soft background
            let rpa = (rain_peaks_a.get([uq * 10.0, ur * 10.0]) + 1.0) * 0.5; // [0,1]
            let rpb = (rain_peaks_b.get([uq * 14.0, ur * 14.0]) + 1.0) * 0.5; // [0,1]
            let rpockets = (rpa * rpb).powf(4.0); // intersect & sharpen pockets
            let ramp = ((rain_amp.get([uq * 3.0, ur * 3.0]) + 1.0) * 0.5).clamp(0.0, 1.0);
            let rf = (rb * 0.2 + rpockets * (0.9 * ramp)).clamp(0.0, 1.0);

            // Simple thresholds
            let cold_thr = 0.25; // temp below → snow
            let warm_thr = 0.7; // temp above → warm band
            let dry_thr = 0.35;  // rain below → desert if warm
            let wet_thr = 0.6;  // rain above → forest if warm

            let terrain = if temp < cold_thr {
                Terrain::Snow
            } else if temp > warm_thr && rf < dry_thr {
                Terrain::Desert
            } else if temp > warm_thr && rf >= wet_thr {
                Terrain::Forest
            } else {
                Terrain::Grass
            };
            Tile::new(h, terrain, e as f32, temp as f32, rf as f32)
        })
        .collect()
}
