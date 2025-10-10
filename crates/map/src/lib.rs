use std::fmt;
use std::str::FromStr;

pub mod hex;
pub use hex::{Hex, SIZE, direction_vector};

/// Possible high-level map sizes with fixed grid dimensions.
#[derive(Copy, Clone, Debug, Eq, PartialEq, Hash)]
pub enum MapSize {
    Duel,
    Tiny,
    Small,
    Standard,
    Large,
    Huge,
}

impl MapSize {
    pub const VARIANTS: [MapSize; 6] = [
        MapSize::Duel,
        MapSize::Tiny,
        MapSize::Small,
        MapSize::Standard,
        MapSize::Large,
        MapSize::Huge,
    ];

    pub const NAMES: [&'static str; 6] = ["duel", "tiny", "small", "standard", "large", "huge"];

    /// Return the width (columns) and height (rows) in tiles.
    pub const fn dimensions(&self) -> (u32, u32) {
        match self {
            MapSize::Duel => (44, 26),
            MapSize::Tiny => (60, 38),
            MapSize::Small => (74, 46),
            MapSize::Standard => (84, 54),
            MapSize::Large => (96, 60),
            MapSize::Huge => (106, 66),
        }
    }

    /// Lower-case label used for CLI parsing and display.
    pub const fn as_str(&self) -> &'static str {
        match self {
            MapSize::Duel => "duel",
            MapSize::Tiny => "tiny",
            MapSize::Small => "small",
            MapSize::Standard => "standard",
            MapSize::Large => "large",
            MapSize::Huge => "huge",
        }
    }
}

impl fmt::Display for MapSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Error raised when parsing a map size from string fails.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct ParseMapSizeError;

impl fmt::Display for ParseMapSizeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("invalid map size")
    }
}

impl std::error::Error for ParseMapSizeError {}

impl FromStr for MapSize {
    type Err = ParseMapSizeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_ascii_lowercase();

        match normalized.as_str() {
            "duel" => Ok(MapSize::Duel),
            "tiny" => Ok(MapSize::Tiny),
            "small" => Ok(MapSize::Small),
            "standard" => Ok(MapSize::Standard),
            "large" => Ok(MapSize::Large),
            "huge" => Ok(MapSize::Huge),
            _ => Err(ParseMapSizeError),
        }
    }
}

/// A collection of hex tiles laid out in an even-r horizontal grid.
#[derive(Clone, Debug, PartialEq)]
pub struct Map {
    size: MapSize,
    width: u32,
    height: u32,
    tiles: Vec<Hex>,
}

impl Map {
    /// Create a new map for the given size with all axial coordinates generated.
    pub fn new(size: MapSize) -> Self {
        let (width, height) = size.dimensions();
        let tiles = generate_even_r_hexes(width, height);

        Self {
            size,
            width,
            height,
            tiles,
        }
    }

    pub fn size(&self) -> MapSize {
        self.size
    }

    pub fn width(&self) -> u32 {
        self.width
    }

    pub fn height(&self) -> u32 {
        self.height
    }

    pub fn tiles(&self) -> &[Hex] {
        &self.tiles
    }
}

fn generate_even_r_hexes(width: u32, height: u32) -> Vec<Hex> {
    let mut tiles = Vec::with_capacity(width as usize * height as usize);

    for row in 0..height {
        for col in 0..width {
            tiles.push(even_r_to_axial(col, row));
        }
    }

    tiles
}

fn even_r_to_axial(col: u32, row: u32) -> Hex {
    let row = row as i32;
    let col = col as i32;
    let q = col - ((row + (row & 1)) / 2);
    let r = row;

    Hex::new(q, r)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn duel_map_dimensions() {
        let map = Map::new(MapSize::Duel);
        assert_eq!(map.width(), 44);
        assert_eq!(map.height(), 26);
        assert_eq!(map.tiles().len(), 44 * 26);
    }

    #[test]
    fn parse_accepts_all_variants() {
        for (name, expected) in MapSize::NAMES.iter().zip(MapSize::VARIANTS.iter()) {
            let parsed = (*name).parse::<MapSize>().expect("parse variant");
            assert_eq!(parsed, *expected);
        }
    }

    #[test]
    fn parse_rejects_unknown() {
        assert!("gigantic".parse::<MapSize>().is_err());
    }
}
