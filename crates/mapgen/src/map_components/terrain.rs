#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// The types of base terrain that exist in the game.
/// All five terrain types have their Hill variants, where the hill denotes a difference in relief.
/// There are two more types of base terrain, related to water.
/// Also added mountain to make it easier to distinguish, in the base game, mountain is just the same tile, but impassable
pub enum Terrain {
    Plains,
    Grassland,
    Desert,
    Tundra,
    Snow,
    CoastLake,
    Ocean,
    Mountain,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// These are commonly-met special formations of some sort that enrich the yields of the base terrain.
/// Most features basically become part of the tile underneath.
pub enum Feature {
    Woods,
    Rainforest,
    Marsh,
    Floodplains,
    Oasis,
    Fissure,
    VolanicSoil,
    Reef,
    Ice,
}
