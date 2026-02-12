use std::{intrinsics::{ceilf32, sqrtf64}, u64::MAX};
use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex};
use rand_chacha::{ChaCha12Rng, rand_core::{Rng, SeedableRng}};

use crate::{map_components::terrain::Terrain, pipeline::map_sizes::MapSizes};

const MOUNTAIN_THRESHOLD: f32 = 0.05;
const HILL_THRESHOLD: f32 = 0.2;

/// Generate landmasses using random seed and simplex noise.
/// We create a land/water split of 60/40.
/// Landmasses are marked as 1, water is marked as 0.
fn generate_landmasses(seed: u64, size: &MapSizes) -> Vec<u8> {
    
    let (width, height) = size.dimensions();

    // Create a seed specifically for landmasses
    let mut rng = ChaCha12Rng::seed_from_u32(seed);
    let noise_seed = rng.next_u64();

    let sea_level = 0.4;
    let scale = 0.6;
    
    let fbm = Fbm::<OpenSimplex>::new(noise_seed).set_octaves(5).set_frequency(1.0 / scale);

    let mut land = vec![0u8; size.grid_size()];
    let (width, height) = size.dimensions();

    for y in 0..height{
        for x in 0..width {
            let idx = y * width + x;

            // using odd r hexes, we need to perform shifts
            // x shiftrs 0.5 on odd rows
            // y shifts by sqrt(3)/2
            let wx = x as f64 + 0.5 * (y & 1);
            let wy = y as f64 * (sqrtf64(3.)/2.);

            // sample noise
            // scale from [-1.0, 1.0] to [0, 1]
            let n = fbm.get([wx, wy]);
            let landmass_value = (n + 1.0) * 0.5;

            land[idx] = if landmass_value > sea_level { 1 } else { 0 }
        }
    }

    land 
}

/// Generate a random simplex noise scaled to [0, 255]
/// Used for temperature, rainfall and heightmap.
 fn generate_random_255(seed: u64, size: &MapSizes) -> Vec<u8> {
    
    let (width, height) = size.dimensions();

    // Create a seed specifically for landmasses
    let mut rng = ChaCha12Rng::seed_from_u32(seed);
    let noise_seed = rng.next_u64();

    let scale = 0.6;
    
    let fbm = Fbm::<OpenSimplex>::new(noise_seed).set_octaves(5).set_frequency(1.0 / scale);

    let mut temp = vec![0u8; size.grid_size()];
    let (width, height) = size.dimensions();

    for y in 0..height{
        for x in 0..width {
            let idx = y * width + x;

            // using odd r hexes, we need to perform shifts
            // x shiftrs 0.5 on odd rows
            // y shifts by sqrt(3)/2
            let wx = x as f64 + 0.5 * (y & 1);
            let wy = y as f64 * (sqrtf64(3.)/2.);

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

/// Assign terrains based on the landmasses, temperature, rainfall and heightmap
/// Returns (Vec<Terrain>, Vec<bool>) for terrain and defining hills
fn assign_terrain(landmasses: &Vec<u8>, temperature: &Vec<u8>, rainfall: &Vec<u8>, heightmap: &Vec<u8>, size: &MapSizes) -> (Vec<Terrain>, Vec<bool>) {

    let (width, height) = size.dimensions();

    // *************************
    // ** Mountains and hills **
    // *************************

    // Create histogram of heights (only on land)
    let mut height_histogram = [0u8; 256];
    let mut land_count = 0;
    for (i, value) in landmasses.iter().enumerate(){
        if landmasses[i] == 1 {
            height_histogram[heightmap[i] as usize] += 1;
            land_count += 1;
        }
    }

    // Use the histogram to find the top 5% of heights for the mountains
    // The remaining top 20% of heights are for hills
    let cutoff_mountains = ceilf32(land_count * MOUNTAIN_THRESHOLD) as u8;
    let mut k_mountains;
    let cutoff_hills = ceilf32(land_count * HILL_THRESHOLD) as u8;
    let mut k_hills;

    let mut count = 0;
    
    for (index, occurrence) in height_histogram.iter().enumerate().rev(){
        count += occurrence;

        if count >= cutoff_mountains {
            k_mountains = index;
        }

        if count >= cutoff_hills {
            k_hills = cutoff_hills;
            break
        }
    }

    // **********************
    // ** Oceans and lakes **
    // **********************

    let visited: Vec<bool> = vec![false; size.grid_size()];
    

}