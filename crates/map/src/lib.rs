mod map_sizes;
mod parser;

pub use map_sizes::MapSize;

use hexx::{conversions::OffsetHexMode, Hex, HexLayout, HexOrientation, Vec2};

/// Visual hex size used by the viewer (circumradius in world units)
pub const SIZE: i32 = 50;

/// Flat‑top (odd‑q) rectangular hex map stored in row‑major order.
#[derive(Clone, Debug)]
pub struct Map {
    size: MapSize,
    width: u32,
    height: u32,
    tiles: Vec<Hex>,
}

impl Map {
    /// Create a new map for the given size and precompute all axial coordinates.
    ///
    /// The internal layout is flat‑top using OddColumns (odd‑q) offset.
    pub fn new(size: MapSize) -> Self {
        let (width, height) = size.dimensions();
        let tiles = generate_odd_q_hexes(width, height);
        Self {
            size,
            width,
            height,
            tiles,
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
