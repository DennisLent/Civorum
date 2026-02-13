use crate::map_components::{
    hex_coords::{CompassDirection, HexCoord},
    resources::ResourceType,
    terrain::{Feature, Terrain},
    yields::Yields,
};

/// Base implementation of a tile, that hold all the main information about the raw state, yields and appeal.
pub struct Tile {
    // coordinations
    hex_coords: HexCoord,
    // tile information
    base_terrain: Terrain,
    feature: Option<Feature>,
    hill: bool,
    passable: bool,
    yields: Yields,
    // rivers and water
    river_edges: u8,
    freshwater: bool,
    ocean_acces: bool,
    // map related information
    resource: Option<ResourceType>,
    landmass: String,
    // tile improvements todo
    owner: Option<String>,
}
