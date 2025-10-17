mod map_sizes;
mod parser;
mod r#gen;
pub mod terrain;
pub mod tile;

pub use map_sizes::MapSize;
pub use r#gen::MapKind;
pub use terrain::Terrain;
pub use tile::Tile;

use hexx::{conversions::OffsetHexMode, Hex, HexLayout, HexOrientation, Vec2};

/// Visual hex size used by the viewer (circumradius in world units)
pub const SIZE: i32 = 50;

/// Flat‑top (odd‑q) rectangular hex map stored in row‑major order.
#[derive(Clone, Debug)]
pub struct Map {
    size: MapSize,
    width: u32,
    height: u32,
    tiles: Vec<Hex>,          // axial coordinates (grid)
    cells: Vec<Tile>,         // per‑tile data (aligned with tiles)
}

impl Map {
    /// Create a new map for the given size and precompute all axial coordinates.
    ///
    /// The internal layout is flat‑top using OddColumns (odd‑q) offset.
    pub fn new(size: MapSize) -> Self {
        let (width, height) = size.dimensions();
        let tiles = generate_odd_q_hexes(width, height);
        let cells = tiles
            .iter()
            .copied()
            .map(|h| Tile::new(h, Terrain::Grass, 0.0, 0.0, 0.0))
            .collect();
        Self {
            size,
            width,
            height,
            tiles,
            cells,
        }
    }

    /// Map size label (for CLI and display); dimensions are given by `width/height`.
    pub fn size(&self) -> MapSize {
        self.size
    }

    /// Columns count (q offset columns)
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Rows count
    pub fn height(&self) -> u32 {
        self.height
    }

    /// All axial tile coordinates in row‑major order (row 0..h, col 0..w).
    pub fn tiles(&self) -> &[Hex] {
        &self.tiles
    }

    /// All tile data in row‑major order (aligned with `tiles()`).
    pub fn cells(&self) -> &[Tile] {
        &self.cells
    }

    /// Convert axial -> (col,row) in odd‑q and return linear index if in‑bounds.
    pub fn axial_to_index(&self, hex: Hex) -> Option<usize> {
        let [col, row] = hex.to_offset_coordinates(OffsetHexMode::Odd, HexOrientation::Flat);
        self.offset_to_index(col, row)
    }

    /// Convert (col,row) -> linear index if in‑bounds (row‑major).
    pub fn offset_to_index(&self, col: i32, row: i32) -> Option<usize> {
        if col < 0 || row < 0 {
            return None;
        }
        let (col, row) = (col as u32, row as u32);
        if col >= self.width || row >= self.height {
            None
        } else {
            Some((row * self.width + col) as usize)
        }
    }

    /// Convert linear index -> axial hex
    pub fn index_to_axial(&self, index: usize) -> Option<Hex> {
        if index >= self.tiles.len() {
            None
        } else {
            Some(self.tiles[index])
        }
    }

    /// Return in‑bounds axial neighbors (6‑connectivity, flat‑top)
    pub fn neighbors(&self, hex: Hex) -> impl Iterator<Item = Hex> + '_ {
        const NEIGH: [Hex; 6] = Hex::NEIGHBORS_COORDS;
        NEIGH.into_iter().filter_map(move |d| {
            let n = hex + d;
            let [c, r] = n.to_offset_coordinates(OffsetHexMode::Odd, HexOrientation::Flat);
            self.offset_to_index(c, r).map(|_| n)
        })
    }

    /// Hex layout used by this map (flat‑top, SCALE = `SIZE`).
    pub fn layout(&self) -> HexLayout {
        HexLayout {
            orientation: HexOrientation::Flat,
            origin: Vec2::ZERO,
            scale: Vec2::splat(SIZE as f32),
            ..Default::default()
        }
    }

    /// World/pixel space center for a given axial coordinate according to this map's layout.
    pub fn hex_to_world_pos(&self, hex: Hex) -> Vec2 {
        self.layout().hex_to_world_pos(hex)
    }

    /// Size of the bounding rectangle of a single hex for this map's layout.
    pub fn rect_size(&self) -> Vec2 {
        self.layout().rect_size()
    }

    /// Hex circumradius in world units (distance from center to vertex).
    ///
    /// For flat‑top orientation, `rect_size().x == 2 * circumradius` and
    /// `rect_size().y == √3 * circumradius` (see hexx docs).
    pub fn hex_circumradius(&self) -> f32 {
        // For flat‑top, layout.scale.x == R when scale is isotropic
        self.layout().scale.x
    }

    /// Hex apothem in world units (distance from center to edge).
    /// For flat‑top orientation, `apothem = √3/2 * R = rect_size().y / 2`.
    pub fn hex_apothem(&self) -> f32 {
        self.rect_size().y * 0.5
    }

    /// Hex diameter in world units (vertex‑to‑vertex width) for flat‑top.
    pub fn hex_diameter(&self) -> f32 {
        self.rect_size().x
    }

    /// Uniform scale factor to fit a 3D model whose unit is a circumradius.
    ///
    /// If your model was authored such that a circumradius of `model_r` maps to 1.0
    /// world unit in the DCC tool, this returns the uniform scale `s` you should apply
    /// to match the current hex layout size: `s = R_world / model_r`.
    pub fn scale_for_model_circumradius(&self, model_r: f32) -> f32 {
        let r = self.hex_circumradius();
        if model_r <= 0.0 { 1.0 } else { r / model_r }
    }

    /// Uniform scale factor to fit a model authored with a vertex‑to‑vertex diameter.
    /// Returns `s = diameter_world / model_diameter`.
    pub fn scale_for_model_diameter(&self, model_diameter: f32) -> f32 {
        let d = self.hex_diameter();
        if model_diameter <= 0.0 { 1.0 } else { d / model_diameter }
    }

    /// Generate a map with tile data using the provided seed and kind.
    pub fn generate(size: MapSize, seed: u64, kind: MapKind) -> Self {
        let (width, height) = size.dimensions();
        let tiles = generate_odd_q_hexes(width, height);
        let cells = match kind {
            MapKind::Continents => r#gen::generate_continents(&tiles, seed, height),
        };

        Self { size, width, height, tiles, cells }
    }
}

fn generate_odd_q_hexes(width: u32, height: u32) -> Vec<Hex> {
    let mut tiles = Vec::with_capacity(width as usize * height as usize);
    for row in 0..height {
        for col in 0..width {
            let h = Hex::from_offset_coordinates(
                [col as i32, row as i32],
                OffsetHexMode::Odd,
                HexOrientation::Flat,
            );
            tiles.push(h);
        }
    }
    tiles
}

// Generation logic moved to `gen` module.
