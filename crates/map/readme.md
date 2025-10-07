# Map information and guidelines

This document includes information about the maps that I am trying to create, this also holds information about the biomes, sizes, terrains etc.


## Map sizes

According to the wiki, Civ VI uses the following map sizes:

| Map Size | Dimensions | Players |
|----------|------------|---------|
| Duel     | 44x26      | 2–4     |
| Tiny     | 60x38      | 4/6     |
| Small    | 74x46      | 6/10    |
| Standard | 84x54      | 8/14    |
| Large    | 96x60      | 10/16   |
| Huge     | 106x66     | 12/20   |

## Terrain & biomes

The map uses a coarse `Terrain` mask (`Land`/`Water`) plus detailed land
`Biome` classes inspired by the Whittaker diagram:

- TropicalForest, Savanna, Desert
- TemperateForest, Temperate (shrub/woodland), Grassland, Prairie
- Taiga, Tundra, Snow

Water tiles are categorized by depth: `Shallow`, `Ocean`, `DeepOcean`.
Height-map generation distinguishes oceans from lakes: water
connected to the boundary is ocean; enclosed water remains `Shallow`
and behaves like a lake.
