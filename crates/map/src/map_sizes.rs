use std::fmt;


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