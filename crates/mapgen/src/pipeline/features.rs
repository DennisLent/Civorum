use itertools::izip;

use crate::{map_components::terrain::Terrain, pipeline::{map, map_sizes::MapSizes}};

/// Find potential spots at which a river can start and end
/// Good river starts are areas with high rainfall. We assign a score to them based on height as well to score them later
/// Good river endings are lakes or coast
/// Returns Vec<(bool, f32)> for starts and Vec<bool> end tiles
fn find_river_potential(terrain_vec: &Vec<Terrain>, rain_vec: &Vec<u8>, height_vec: &Vec<u8>, map_size: &MapSizes) -> (Vec<(bool, f32)>, Vec<bool>){

    let grid_size = map_size.grid_size();

    // top 30% wettest regions on the map
    let rain_threshold = (255 as f32 * 0.7) as u8;

    let mut starting_locations = Vec::with_capacity(grid_size);

    // all locations that are coast are good
    let ending_locations = terrain_vec.iter().map(|terrain| {
        terrain == &Terrain::CoastLake
    }).collect();

    for (terrain, rain, height) in izip!(terrain_vec, rain_vec, height_vec){
        if terrain != &Terrain::Mountain || ((terrain != &Terrain::Ocean || terrain != &Terrain::CoastLake) && rain >= &rain_threshold) {
            let elevation_score = (height / 255) as f32;
            let rainfall_score = (rain / 255) as f32;
            let score = 0.65 * elevation_score + 0.35 * rainfall_score;
            starting_locations.push((true, score))
        }
        else {
            starting_locations.push((false, 0.0));
        }
    }

    (starting_locations, ending_locations)
}

/// Deterministically choose from the starting and ending positions pick pairs
/// For each map size we choose different amounts of pairs
/// Duel: 2
/// Tiny: 3
/// Small: 3
/// Standard: 4
/// Large: 5
/// Huge: 6
fn pick_and_trace_rivers(starting_locations: Vec<bool>, ending_locations: Vec<bool>, terrain_vec: &Vec<Terrain>, seed: u64, map_size: &MapSizes) -> Vec<(usize, usize)> {

    let n_pairs = map_size.number_rivers();

    todo!()
}