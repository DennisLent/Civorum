use itertools::izip;
use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex};
use rand_chacha::{
    ChaCha12Rng,
    rand_core::{Rng, SeedableRng},
};
use serde::Deserialize;
use std::{collections::VecDeque, f64::consts::PI, fs, path::PathBuf, sync::OnceLock, vec};

use crate::{map_components::terrain::Terrain, pipeline::map_sizes::MapSizes};

#[derive(Debug, Clone, Deserialize)]
/// Config for the biome settings being loaded from the yaml file
struct BiomesConfig {
    terrain: TerrainThresholds,
    landmasses: LandmassesConfig,
    temperature: TemperatureConfig,
    rainfall: NoiseConfig,
    heightmap: NoiseConfig,
}

#[derive(Debug, Clone, Deserialize)]
/// Config for the terrain settings being loaded from the yaml file
struct TerrainThresholds {
    mountain_threshold: f32,
    hill_threshold: f32,
    snow_temp_threshold: u8,
    tundra_temp_threshold: u8,
    desert_temp_threshold: u8,
    desert_rain_threshold: u8,
    grassland_rain_threshold: u8,
}

#[derive(Debug, Clone, Deserialize)]
/// Config for landmass settings being loaded from the yaml file
struct LandmassesConfig {
    base_factor: usize,
    base_land_percent: u32,
    fuzzy_flip_percent: u32,
    coast_island_percent: u32,
    smoothing_passes: usize,
}

#[derive(Debug, Clone, Deserialize)]
/// Config for temperature settings being loaded from the yaml file
struct TemperatureConfig {
    continental_octaves: usize,
    continental_scale: f64,
    detail_octaves: usize,
    detail_scale: f64,
    continental_weight: f64,
    detail_weight: f64,
    base_amplitude: f64,
    latitude_amp_floor: f64,
}

#[derive(Debug, Clone, Deserialize)]
/// Config for noise seetings being loaded from the yaml file
struct NoiseConfig {
    octaves: usize,
    scale: f64,
}

/// Default implementation of all config settings in case reading fails
fn default_biomes_config() -> BiomesConfig {
    BiomesConfig {
        terrain: TerrainThresholds {
            mountain_threshold: 0.05,
            hill_threshold: 0.2,
            snow_temp_threshold: 40,
            tundra_temp_threshold: 85,
            desert_temp_threshold: 150,
            desert_rain_threshold: 85,
            grassland_rain_threshold: 155,
        },
        landmasses: LandmassesConfig {
            base_factor: 16,
            base_land_percent: 6,
            fuzzy_flip_percent: 4,
            coast_island_percent: 6,
            smoothing_passes: 2,
        },
        temperature: TemperatureConfig {
            continental_octaves: 4,
            continental_scale: 120.0,
            detail_octaves: 5,
            detail_scale: 35.0,
            continental_weight: 0.7,
            detail_weight: 0.3,
            base_amplitude: 0.18,
            latitude_amp_floor: 0.5,
        },
        rainfall: NoiseConfig {
            octaves: 5,
            scale: 60.0,
        },
        heightmap: NoiseConfig {
            octaves: 5,
            scale: 40.0,
        },
    }
}

fn config_path() -> PathBuf {
    if let Ok(path) = std::env::var("CIVORUM_BIOMES_CONFIG") {
        return PathBuf::from(path);
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../biomes.yaml")
}

fn load_biomes_config() -> BiomesConfig {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(raw) => match serde_yaml::from_str::<BiomesConfig>(&raw) {
            Ok(config) => config,
            Err(err) => {
                eprintln!(
                    "Failed to parse biome config at '{}': {err}. Falling back to defaults.",
                    path.display()
                );
                default_biomes_config()
            }
        },
        Err(err) => {
            eprintln!(
                "Failed to read biome config at '{}': {err}. Falling back to defaults.",
                path.display()
            );
            default_biomes_config()
        }
    }
}

fn biomes_config() -> &'static BiomesConfig {
    static CONFIG: OnceLock<BiomesConfig> = OnceLock::new();
    CONFIG.get_or_init(load_biomes_config)
}

/// Generate landmasses in three stages
/// 1. coarse mostly-water seed map
/// 2. repeated zoom with coast decisions,
/// 3. hex-neighborhood smoothing passes.
/// Landmasses are marked as 1, water is marked as 0.
fn generate_landmasses(seed: u64, size: &MapSizes) -> Vec<u8> {
    let config = biomes_config();
    let land_cfg = &config.landmasses;
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let (width, height) = size.dimensions();

    let mut w = width.div_ceil(land_cfg.base_factor).max(2);
    let mut h = height.div_ceil(land_cfg.base_factor).max(2);
    let mut grid = vec![0u8; w * h];

    // Stage 1. coarse seed, mostly water, guaranteed water borders.
    for y in 0..h {
        for x in 0..w {
            let idx = y * w + x;
            let border = x == 0 || x + 1 == w || y == 0 || y + 1 == h;
            if border {
                grid[idx] = 0;
                continue;
            }

            let roll = rng.next_u32() % 100;
            grid[idx] = if roll < land_cfg.base_land_percent {
                1
            } else {
                0
            };
        }
    }

    // Stage 2. repeated 2x
    while w < width || h < height {
        let new_w = (w * 2).min(width);
        let new_h = (h * 2).min(height);
        let mut next = vec![0u8; new_w * new_h];

        for ny in 0..new_h {
            for nx in 0..new_w {
                let px = (nx / 2).min(w - 1);
                let py = (ny / 2).min(h - 1);

                let pe = (px + 1).min(w - 1);
                let ps = (py + 1).min(h - 1);

                let parent = grid[py * w + px];
                let east = grid[py * w + pe];
                let south = grid[ps * w + px];
                let diag = grid[ps * w + pe];

                let land_votes = parent + east + south + diag;
                let mut value = if land_votes > 2 {
                    1
                } else if land_votes < 2 {
                    0
                } else {
                    parent
                };

                let mixed = land_votes > 0 && land_votes < 4;
                if mixed {
                    if rng.next_u32() % 100 < land_cfg.fuzzy_flip_percent {
                        value = 1 - value;
                    }

                    if value == 0 && rng.next_u32() % 100 < land_cfg.coast_island_percent {
                        value = 1;
                    }
                }

                next[ny * new_w + nx] = value;
            }
        }

        for x in 0..new_w {
            next[x] = 0;
            next[(new_h - 1) * new_w + x] = 0;
        }
        for y in 0..new_h {
            next[y * new_w] = 0;
            next[y * new_w + (new_w - 1)] = 0;
        }

        grid = next;
        w = new_w;
        h = new_h;
    }

    // Stage 3. hex-neighborhood smoothing.
    for _ in 0..land_cfg.smoothing_passes {
        let mut next = grid.clone();
        for y in 0..height {
            for x in 0..width {
                let idx = y * width + x;
                let mut water_neighbors = 0usize;
                let mut land_neighbors = 0usize;

                for (nx, ny) in neighbors_odd_r(x, y, width, height) {
                    if grid[ny * width + nx] == 0 {
                        water_neighbors += 1;
                    } else {
                        land_neighbors += 1;
                    }
                }

                if water_neighbors >= 4 {
                    next[idx] = 0;
                } else if land_neighbors >= 4 {
                    next[idx] = 1;
                }
            }
        }

        for x in 0..width {
            next[x] = 0;
            next[(height - 1) * width + x] = 0;
        }
        for y in 0..height {
            next[y * width] = 0;
            next[y * width + (width - 1)] = 0;
        }

        grid = next;
    }

    grid
}

/// Use a seed to generate a temperature distribution.
/// Temperate varies throughout, but is coldest at the north and south.
/// Warmer areas towards the center of the map.
fn generate_temperature(seed: u64, size: &MapSizes) -> Vec<u8> {
    let cfg = &biomes_config().temperature;
    let (width, height) = size.dimensions();

    // Create a seed specifically for random generation
    // We use continental noise (overall change of temperature) and detail noise for some variation
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let noise_seed_cont = rng.next_u64();
    let noise_seed_det = rng.next_u64();

    let cont = Fbm::<OpenSimplex>::new(noise_seed_cont as u32)
        .set_octaves(cfg.continental_octaves)
        .set_frequency(1.0 / cfg.continental_scale);
    let det = Fbm::<OpenSimplex>::new(noise_seed_det as u32)
        .set_octaves(cfg.detail_octaves)
        .set_frequency(1.0 / cfg.detail_scale);

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
            let noise = cfg.continental_weight * n_cont + cfg.detail_weight * n_det;

            // vary amplitude by latitude
            let amp = cfg.base_amplitude * (cfg.latitude_amp_floor + 0.5 * base);
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
fn generate_random_255(seed: u64, size: &MapSizes, noise_config: &NoiseConfig) -> Vec<u8> {
    // Create a seed specifically for random generation
    let mut rng = ChaCha12Rng::seed_from_u64(seed);
    let noise_seed = rng.next_u64();

    let fbm = Fbm::<OpenSimplex>::new(noise_seed as u32)
        .set_octaves(noise_config.octaves)
        .set_frequency(1.0 / noise_config.scale);

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
pub fn neighbors_odd_r(x: usize, y: usize, width: usize, height: usize) -> Vec<(usize, usize)> {
    let p = y & 1;

    let x = x as isize;
    let y = y as isize;
    let width = width as isize;
    let height = height as isize;

    let candidates: [(isize, isize); 6] = if p == 0 {
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

        out.push((nx as usize, ny as usize));
    }

    out
}

/// Returns a mask where true is ocean
/// Assumes landmask of 1 = land & 0 = water.
/// This will only mark the oceans and the lakes, coastal tiles need to be marked separately
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
                ocean[idx] = true;
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

/// Function to mark the coastal tiles i.e. "ocean" tiles with at least one land neighbor
fn coastal_water_mask(landmasses: &[u8], ocean: &[bool], size: &MapSizes) -> Vec<bool> {
    let (width, height) = size.dimensions();
    let mut coast = vec![false; width * height];

    for y in 0..height {
        for x in 0..width {
            let idx = y * width + x;

            // only ocean water can be coast
            if landmasses[idx] != 0 || !ocean[idx] {
                continue;
            }

            // if any neighbor is land mark as coast
            if neighbors_odd_r(x, y, width, height)
                .into_iter()
                .any(|(nx, ny)| landmasses[ny * width + nx] == 1)
            {
                coast[idx] = true;
            }
        }
    }

    coast
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
    let terrain_cfg = &biomes_config().terrain;
    // *************************
    // ** Mountains and hills **
    // *************************

    // Create histogram of heights (only on land)
    let mut height_histogram = [0u32; 256];
    let mut land_count = 0;
    for (i, _) in landmasses.iter().enumerate() {
        if landmasses[i] == 1 {
            height_histogram[heightmap[i] as usize] += 1;
            land_count += 1;
        }
    }

    // Use the histogram to find the top 5% of heights for the mountains
    // The remaining top 20% of heights are for hills
    let cutoff_mountains = f32::ceil(land_count as f32 * terrain_cfg.mountain_threshold) as u32;
    let mut k_mountains: Option<u8> = None;
    let cutoff_hills = f32::ceil(
        land_count as f32 * (terrain_cfg.mountain_threshold + terrain_cfg.hill_threshold),
    ) as u32;
    let mut k_hills: Option<u8> = None;

    let mut count = 0;

    for (index, occurrence) in height_histogram.iter().enumerate().rev() {
        count += occurrence;

        if k_mountains.is_none() && count >= cutoff_mountains {
            k_mountains = Some(index as u8);
        }

        if k_hills.is_none() && count >= cutoff_hills {
            k_hills = Some(index as u8);
            break;
        }
    }

    let k_mountains = k_mountains.expect("did not find mountain threshold");
    let k_hills = k_hills.expect("did not find hill threshold");

    // **********************
    // ** Oceans and lakes **
    // **********************

    let ocean_mask = ocean_mask(&landmasses, &size);
    let coast_mask = coastal_water_mask(&landmasses, &ocean_mask, &size);

    // **************
    // ** Terrains **
    // **************

    let n = size.grid_size();
    let mut terrain_vec = Vec::with_capacity(n);
    let mut hill_vec = Vec::with_capacity(n);

    for i in 0..n {
        let l = landmasses[i];
        
        //water
        if l == 0{
            let o = ocean_mask[i];

            let is_coast = coast_mask[i];
            let is_lake = !o;

            terrain_vec.push(if is_lake || is_coast {
                Terrain::CoastLake
            } else {
                Terrain::Ocean
            });
            hill_vec.push(false);
            continue;
        }

        // hills
        let h = heightmap[i];
        if h >= k_mountains {
            terrain_vec.push(Terrain::Mountain);
            hill_vec.push(false);
            continue;
        }

        let is_hill = h >= k_hills;
        hill_vec.push(is_hill);


        let t = temperature[i];
        let r = rainfall[i];
        let terrain = if t <= terrain_cfg.snow_temp_threshold {
            Terrain::Snow
        } else if t <= terrain_cfg.tundra_temp_threshold {
            Terrain::Tundra
        } else if t >= terrain_cfg.desert_temp_threshold && r <= terrain_cfg.desert_rain_threshold
        {
            Terrain::Desert
        } else if r >= terrain_cfg.grassland_rain_threshold {
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
/// Returns a vec for the terrain, height, hills, temperatire and rain
pub fn generate_map(seed: &u64, size: &MapSizes) -> (Vec<Terrain>, Vec<u8>, Vec<bool>, Vec<u8>, Vec<u8>) {
    let config = biomes_config();
    let land_seed = seed.clone();
    let land = generate_landmasses(land_seed, size);

    let temp_seed = seed + 1;
    let temp = generate_temperature(temp_seed, size);

    let rain_seed = seed + 2;
    let rain = generate_random_255(rain_seed, size, &config.rainfall);

    let height_seed = seed + 3;
    let height = generate_random_255(height_seed, size, &config.heightmap);

    let (terrain_vec, hill_vec) = assign_terrain(&land, &temp, &rain, &height, size);

    (terrain_vec, height, hill_vec, temp, rain)
}