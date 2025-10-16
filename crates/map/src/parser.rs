use std::fmt;
use std::str::FromStr;
use crate::MapSize;

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