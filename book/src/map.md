# Map Crate

The `map` crate provides minimal axial hex-grid primitives and a
deterministic in-memory map backed by `BTreeMap` for stable iteration.
Maps are rectangular axial regions (not hex-disks).

## Coordinates
- Axial coordinates `(q, r)` with implicit `s = -q - r`.
- Directions: `E, NE, NW, W, SW, SE`.

## Types
- `Hex`: coordinate helpers (neighbors, distance).
- `Terrain`: coarse mask (`Land`, `Water`).
- `Biome`: detailed land biome (Whittaker-inspired: Desert, Savanna, TropicalForest, TemperateForest, Temperate, Grassland, Prairie, Taiga, Tundra, Snow).
- `WaterDepth`: water categories (`Shallow`, `Ocean`, `DeepOcean`).
- `Tile`: carries `terrain` plus `biome` or `water` detail, and `elevation`.
- `Map`: rectangular constructor, getters, bounds, and stable iteration.
- `worldgen`: deterministic world generators (`realistic_map`, `random_map`, `from_height_fn`).

## Example
```rust
use map::{Hex, Map, Tile};

let (w,h) = (8,6);
let mut m = Map::new_rect(0, 0, w, h, Tile::default());
let p = Hex::new(0, 0);
assert!(m.contains(p));
assert_eq!(m.get(p).unwrap().terrain, map::Terrain::Land);

// Realistic generator:
let seed = 42;
let sea_level = 0.0;
let m2 = map::worldgen::realistic_map(w, h, seed, sea_level);

// Random generator with scale tuned to radius:
let scale = map::worldgen::feature_scale_for_dims(w, h);
let m3 = map::worldgen::random_map(w, h, seed, sea_level, scale);

// From height map:
let m4 = map::worldgen::from_height_fn(w, h, sea_level, |h| {
    // simple dome: negative is water, positive is land
    let d = Hex::new(0,0).distance(h) as f32;
    50.0 - d * 5.0
});
```
