use itertools::izip;
use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex};
use rand_chacha::{
    ChaCha12Rng,
    rand_core::{Rng, SeedableRng},
};
use std::{collections::VecDeque, f64::consts::PI, vec};

use crate::{map_components::terrain::Terrain, pipeline::map_sizes::MapSizes};

const MOUNTAIN_THRESHOLD: f32 = 0.05;
const HILL_THRESHOLD: f32 = 0.2;
const SNOW_TEMP_THRESHOLD: u8 = 40;
const TUNDRA_TEMP_THRESHOLD: u8 = 85;
const DESERT_TEMP_THRESHOLD: u8 = 170;
const DESERT_RAIN_THRESHOLD: u8 = 85;
const GRASSLAND_RAIN_THRESHOLD: u8 = 155;

/// Generate landmasses using random seed and simplex noise.
/// We create a land/water split of 60/40.
/// Landmasses are marked as 1, water is marked as 0.
fn generate_landmasses(seed: u64, size: &MapSizes) -> Vec<u8> {
    // Create a seed specifically for landmasses
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let noise_seed = rng.next_u64();

    let sea_level = 0.4;
    let scale = 0.6;

    let fbm = Fbm::<OpenSimplex>::new(noise_seed as u32)
        .set_octaves(5)
        .set_frequency(1.0 / scale);

    let mut land = vec![0u8; size.grid_size()];
    let (width, height) = size.dimensions();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;

            // using odd r hexes, we need to perform shifts
            // x shiftrs 0.5 on odd rows
            // y shifts by sqrt(3)/2
            let wx = x as f64 + 0.5 * (y & 1) as f64;
            let wy = y as f64 * ((3_f64).sqrt() / 2.);

            // sample noise
            // scale from [-1.0, 1.0] to [0, 1]
            let n = fbm.get([wx, wy]);
            let landmass_value = (n + 1.0) * 0.5;

            land[idx] = if landmass_value > sea_level { 1 } else { 0 }
        }
    }

    land
}

/// Use a seed to generate a temperature distribution.
/// Temperate varies throughout, but is coldest at the north and south.
/// Warmer areas towards the center of the map.
fn generate_temperature(seed: u64, size: &MapSizes) -> Vec<u8> {
    let (width, height) = size.dimensions();

    // Create a seed specifically for random generation
    // We use continental noise (overall change of temperature) and detail noise for some variation
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let noise_seed_cont = rng.next_u64();
    let noise_seed_det = rng.next_u64();

    let cont = Fbm::<OpenSimplex>::new(noise_seed_cont as u32)
        .set_octaves(4)
        .set_frequency(1.0 / 120.0);
    let det = Fbm::<OpenSimplex>::new(noise_seed_det as u32)
        .set_octaves(5)
        .set_frequency(1.0 / 35.0);

    let mut out = vec![0u8; (width * height) as usize];

    for y in 0..height {
        let lat = if height <= 1 {
            0.0
        } else {
            y as f64 / (height as f64 - 1.0)
        };
        let d = ((lat - 0.5).abs() * 2.0).min(1.0);

        let base = ((PI * d).cos() + 1.0) * 0.5;

        for x in 0..width {
            let idx = y * width + x;

            let wx = x as f64 + 0.5 * (y & 1) as f64;
            let wy = y as f64 * ((3_f64).sqrt() / 2.);

            // Sample noise and add to eachother (70/30 split)
            let n_cont = cont.get([wx, wy]);
            let n_det = det.get([wx, wy]);
            let noise = 0.7 * n_cont + 0.3 * n_det;

            // vary amplitude by latitude
            let amp = 0.5 + 0.5 * base;
            let mut temp = base + amp * noise;

            // clamp & switch to u8
            temp = temp.clamp(0.0, 1.0);
            out[idx] = (temp * 255.0).round() as u8;
        }
    }

    out
}

/// Generate a random simplex noise scaled to [0, 255]
/// Used for rainfall and heightmap.
fn generate_random_255(seed: u64, size: &MapSizes) -> Vec<u8> {
    // Create a seed specifically for random generation
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let noise_seed = rng.next_u64();

    let scale = 0.6;

    let fbm = Fbm::<OpenSimplex>::new(noise_seed as u32)
        .set_octaves(5)
        .set_frequency(1.0 / scale);

    let mut temp = vec![0u8; size.grid_size()];
    let (width, height) = size.dimensions();

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;

            // using odd r hexes, we need to perform shifts
            // x shiftrs 0.5 on odd rows
            // y shifts by sqrt(3)/2
            let wx = x as f64 + 0.5 * (y & 1) as f64;
            let wy = y as f64 * ((3_f64).sqrt() / 2.);

            // sample noise
            // scale from [-1.0, 1.0] to [0, 255]
            // NewValue = int((((OldValue - OldMin) * NewRange) / OldRange) + NewMin)
            let n = fbm.get([wx, wy]);
            let temp_value = (((n + 1.0) * 255.0) / 2.0) as u8;

            temp[idx] = temp_value;
        }
    }

    temp
}

/// Helper function for Odd-r neighbors for pointy-top hexes
/// Returns only in-bounds neighbors
fn neighbors_odd_r(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let p = y & 1;

    let candidates: [(usize, usize); 6] = if p == 0 {
        // even row
        [
            (x, y - 1),
            (x + 1, y),
            (x, y + 1),
            (x - 1, y + 1),
            (x - 1, y),
            (x - 1, y - 1),
        ]
    } else {
        // odd row
        [
            (x + 1, y - 1),
            (x + 1, y),
            (x + 1, y + 1),
            (x, y + 1),
            (x - 1, y),
            (x, y - 1),
        ]
    };

    let mut out = Vec::with_capacity(6);

    for (nx, ny) in candidates {
        // y must be in-bounds
        if ny < 0 || ny >= height {
            continue;
        }

        if nx < 0 || nx >= width {
            continue;
        }

        out.push((nx, ny));
    }

    out
}

/// Returns a mask where true is ocean
/// Assumes landmask of 1 = land & 0 = water.
fn ocean_mask(landmasses: &Vec<u8>, size: &MapSizes) -> Vec<bool> {
    let (width, height) = size.dimensions();
    let mut ocean = vec![false; size.grid_size()];
    let mut queue = VecDeque::new();

    // check north and south corners
    // do not neeed to check east-west since we wrap
    for x in 0..width {
        for y in [0, height - 1] {
            let idx = y * width + x;
            if landmasses[idx] == 0 && !ocean[idx] {
                ocean[idx] == true;
                queue.push_back((x, y));
            }
        }
    }

    // BFS flood fill across connected water (hex neighbors).
    while let Some((x, y)) = queue.pop_front() {
        for (nx, ny) in neighbors_odd_r(x, y, width, height) {
            let nidx = (ny * width + nx) as usize;
            if landmasses[nidx] == 0 && !ocean[nidx] {
                ocean[nidx] = true;
                queue.push_back((nx, ny));
            }
        }
    }

    ocean
}

/// Assign terrains based on the landmasses, temperature, rainfall and heightmap
/// Returns (Vec<Terrain>, Vec<bool>) for terrain and defining hills
fn assign_terrain(
    landmasses: &Vec<u8>,
    temperature: &Vec<u8>,
    rainfall: &Vec<u8>,
    heightmap: &Vec<u8>,
    size: &MapSizes,
) -> (Vec<Terrain>, Vec<bool>) {
    // *************************
    // ** Mountains and hills **
    // *************************

    // Create histogram of heights (only on land)
    let mut height_histogram = [0u32; 256];
    let mut land_count = 0;
    for (i, value) in landmasses.iter().enumerate() {
        if landmasses[i] == 1 {
            height_histogram[heightmap[i] as usize] += 1;
            land_count += 1;
        }
    }

    // Use the histogram to find the top 5% of heights for the mountains
    // The remaining top 20% of heights are for hills
    let cutoff_mountains = f32::ceil(land_count as f32 * MOUNTAIN_THRESHOLD) as u32;
    let mut k_mountains = 255;
    let cutoff_hills = f32::ceil(land_count as f32 * HILL_THRESHOLD) as u32;
    let mut k_hills = 170;

    let mut count = 0;

    for (index, occurrence) in height_histogram.iter().enumerate().rev() {
        count += occurrence;

        if count >= cutoff_mountains {
            k_mountains = index as u8;
        }

        if count >= cutoff_hills {
            k_hills = cutoff_hills as u8;
            break;
        }
    }

    // **********************
    // ** Oceans and lakes **
    // **********************

    let ocean_mask = ocean_mask(&landmasses, &size);

    // **************
    // ** Terrains **
    // **************

    let n = size.grid_size();
    let mut terrain_vec = Vec::with_capacity(n);
    let mut hill_vec = Vec::with_capacity(n);

    for (l, o, h, t, r) in izip!(landmasses, ocean_mask, heightmap, temperature, rainfall) {
        // oceans
        if *l == 0 {
            terrain_vec.push(if o {
                Terrain::Ocean
            } else {
                Terrain::CoastLake
            });
            hill_vec.push(false);
            continue;
        }

        if h >= &k_mountains {
            terrain_vec.push(Terrain::Mountain);
            hill_vec.push(false);
            continue;
        }

        let is_hill = h >= &k_hills;
        hill_vec.push(is_hill);

        let terrain = if t <= &SNOW_TEMP_THRESHOLD {
            Terrain::Snow
        } else if t <= &TUNDRA_TEMP_THRESHOLD {
            Terrain::Tundra
        } else if t >= &DESERT_TEMP_THRESHOLD && r <= &DESERT_RAIN_THRESHOLD {
            Terrain::Desert
        } else if r >= &GRASSLAND_RAIN_THRESHOLD {
            Terrain::Grassland
        } else {
            Terrain::Plains
        };

        terrain_vec.push(terrain);
    }

    (terrain_vec, hill_vec)
}

/// Creates landmasses, temperature, rainfall, height and ocean masks for the map.
/// Assigns the respective terrains to each tile
/// Returns a vec for the terrain, hills, temperatire and rain
pub fn generate_map(seed: &u64, size: &MapSizes) -> (Vec<Terrain>, Vec<bool>, Vec<u8>, Vec<u8>) {
    let land_seed = seed.clone();
    let land = generate_landmasses(land_seed, size);

    let temp_seed = seed + 1;
    let temp = generate_temperature(temp_seed, size);

    let rain_seed = seed + 2;
    let rain = generate_random_255(rain_seed, size);

    let height_seed = seed + 3;
    let height = generate_random_255(height_seed, size);

    let (terrain_vec, hill_vec) = assign_terrain(&land, &temp, &rain, &height, size);

    (terrain_vec, hill_vec, temp, rain)
}
