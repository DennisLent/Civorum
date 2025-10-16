# Map Crate

The `map` crate provides a simple, rectangular hex-map for Civorum built on
top of the excellent [`hexx`](https://docs.rs/hexx) library. It focuses on
coordinate/layout correctness and a minimal API you can build gameplay on.

Key properties
- Orientation: flat-top hexagons
- Indexing/layout: Odd-Columns (odd‑q) offset for (col,row) addressing
- Storage: axial coordinates (`hexx::Hex`) in row‑major order
- World mapping: `hexx::HexLayout` constructed via the map for consistent
  pixel/world coordinates

References
- Hexx crate docs: https://docs.rs/hexx
- Coordinate systems: https://www.redblobgames.com/grids/hexagons

## Coordinates & Layout

- Axial coordinates `(x, y)` (sometimes written `(q, r)`), with `z = -x - y`.
- For odd‑q flat‑top indexing:
  - axial → offset: `hex.to_offset_coordinates(OffsetHexMode::Odd, HexOrientation::Flat)`
  - offset → axial: `Hex::from_offset_coordinates([col,row], OffsetHexMode::Odd, HexOrientation::Flat)`

The crate exposes a `HexLayout` through `Map::layout()`, which the viewer uses
to convert axial coordinates to world positions.

## Sizes & Dimensions

`MapSize` enumerates Civ‑like presets (`Duel`, `Tiny`, `Small`, `Standard`,
`Large`, `Huge`). Each size maps to a fixed `(width, height)` in tiles, where
width is columns and height is rows in odd‑q offset.

## Basic Usage

```rust
use map::{Map, MapSize};

// Create a map from a preset
let m = Map::new(MapSize::Standard);
assert_eq!(m.tiles().len() as u32, m.width() * m.height());

// Iterate axial tiles (hexx::Hex)
for h in m.tiles() {
    let _ = (h.x(), h.y(), h.z());
}

// In‑bounds neighbors (6‑connectivity)
if let Some(center) = m.index_to_axial(0) {
    for n in m.neighbors(center) {
        // e.g., distance using hexx
        let _d = center.unsigned_distance_to(n);
    }
}

// Convert axial → offset index
let some_hex = m.index_to_axial(0).unwrap();
let idx = m.axial_to_index(some_hex).unwrap();
assert_eq!(m.tiles()[idx], some_hex);

// Axial → world using flat‑top layout
let layout = m.layout();
let world = layout.hex_to_world_pos(some_hex);
let _ = (world.x, world.y);
```

## Rendering with Hexx

For visualization (e.g., in the `viewer` crate), use hexx’s mesh helpers.
This keeps geometry/math consistent with the layout:

```rust
use hexx::{PlaneMeshBuilder, MeshInfo};

let layout = m.layout();
let hex = m.index_to_axial(0).unwrap();
let info: MeshInfo = PlaneMeshBuilder::new(&layout).at(hex).build();
// Convert MeshInfo → engine mesh (see hexx docs for Bevy example)
```

Hexx provides both `PlaneMeshBuilder` (flat hex) and `ColumnMeshBuilder`
(3D column). See: https://docs.rs/hexx/latest/hexx/mesh/index.html

## Notes

- Odd‑q (odd columns) is used for the flat‑top rectangle; be sure to pass
  `(OffsetHexMode::Odd, HexOrientation::Flat)` when converting.
- Axial coordinates are signed; don’t treat `(x,y)` like a 0‑based grid.
- The world layout (`HexLayout`) controls scale and origin for all mapping.
