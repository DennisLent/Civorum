use crate::map_components::{resources::ResourceType, terrain::{Feature, Terrain}, yields::Yields};

/// Base implementation of a tile, that hold all the main information about the raw state, yields and appeal.
pub struct Tile {
    // coordinations
    hex_coords: (i32, i32),
    // tile information
    base_terrain: Terrain,
    feature: Option<Feature>,
    hill: bool,
    passable: bool,
    yields: Yields,
    // rivers and water
    river: bool,
    river_edge: Option<i32>,
    freshwater: bool,
    ocean_acces: bool,
    // map related information
    resource: Option<ResourceType>,
    landmass: String,
    elevation: i32,
    climate: i32,
    // tile improvements todo
    owner: Option<String>

}
