use std::{intrinsics::sqrtf64, u64::MAX};
use noise::{Fbm, MultiFractal, NoiseFn, OpenSimplex};
use rand_chacha::{ChaCha12Rng, rand_core::{Rng, SeedableRng}};

use crate::pipeline::map_sizes::MapSizes;

/// Generate landmasses using random seed and simplex noise.
/// We create a land/water split of 70/30.
/// Landmasses are marked as 1, water is marked as 0.
pub fn generate_landmasses(seed: u64, size: &MapSizes) -> Vec<u8> {
    
    let (width, height) = size.dimensions();

    // Create a seed specifically for landmasses
    let mut rng = ChaCha12Rng::seed_from_u32(seed);
    let noise_seed = rng.next_u64();

    let sea_level = 0.7;
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
pub fn generate_random_255(seed: u64, size: &MapSizes) -> Vec<u8> {
    
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