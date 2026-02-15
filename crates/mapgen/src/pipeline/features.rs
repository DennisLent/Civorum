use itertools::Itertools;

use crate::{
    map_components::terrain::Terrain,
    pipeline::{helpers::neighbors_odd_r, map_sizes::MapSizes},
};

/// Find potential spots at which a river can start and end
/// Good river starts are areas with high rainfall. We assign a score to them based on height as well to score them later
/// Good river endings are lakes or coast
/// Returns Vec<(bool, f32)> for starts and Vec<bool> end tiles
fn find_river_potential(terrain_vec: &Vec<Terrain>, rain_vec: &Vec<u8>, height_vec: &Vec<u8>, map_size: &MapSizes) -> (Vec<f32>, Vec<bool>){

    let grid_size = map_size.grid_size();
    let (width, height) = map_size.dimensions();

    let mut starting_locations = Vec::with_capacity(grid_size);

    // all locations that are coast are good
    let ending_locations = terrain_vec.iter().map(|terrain| {
        terrain == &Terrain::CoastLake
    }).collect();

    // check each tile
    // a mountain has good elevation, but since rivers run on tiles, we need to check for a mountain with a normal tile next to it (also no coast)
    for x in 0..width {
        for y in 0..height {

            let tile_idx = y * width + x;
            
            // mountain as strong source only if it borders non-mountain, non-coast land
            let elevation_score = if terrain_vec[tile_idx] == Terrain::Mountain {
                for (nx, ny) in neighbors_odd_r(x, y, width, height) {
                    let nid = ny * width + nx;
                    if terrain_vec[nid] != Terrain::Mountain && terrain_vec[nid] != Terrain::CoastLake {
                        break;
                    }
                }
                1.0
            }
            else {
                height_vec[tile_idx] as f32/ 255.0 
            };

            // assign score based on height and rainfall
            let rain_score = (rain_vec[tile_idx] as f32/ 255.0);
            let score = 0.65*elevation_score + 0.35*rain_score;
            starting_locations.push(score);

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
fn pick_and_trace_rivers(starting_locations: Vec<f32>, ending_locations: Vec<bool>, terrain_vec: &Vec<Terrain>, map_size: &MapSizes) -> Vec<Vec<usize>> {

    let n_pairs = map_size.number_rivers();
    let river_vec = Vec::new();

    river_vec
    
}


pub fn place_features(terrain_vec: &Vec<Terrain>, rain_vec: &Vec<u8>, height_vec: &Vec<u8>, map_size: &MapSizes) {

    let (river_starts, river_ends) = find_river_potential(terrain_vec, rain_vec, height_vec, map_size);

    let _ = pick_and_trace_rivers(river_starts, river_ends, terrain_vec, map_size);
}
