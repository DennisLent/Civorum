mod map_sizes;
mod parser;

use hexx::HexLayout;
pub use map_sizes::MapSize;
use hexx::Vec2;


/// Visual hex size used by the viewer (circumradius in world units)
pub const SIZE: i32 = 50;

/// A collection of hex tiles laid out in an even-q (column offset) grid
/// using flat-top orientation.
#[derive(Clone, Debug)]
pub struct Map {
    size: MapSize,
    width: u32,
    height: u32,
    layout: HexLayout
}

impl Map {
    /// Create a new map for the given size with all axial coordinates generated.
    pub fn new(size: MapSize) -> Self {
        let (width, height) = size.dimensions();
        // Generate coordinates using hexx's flat rectangle helper, which
        // corresponds to an even‑q column offset layout for flat‑top hexes.
        let layout = HexLayout {
            orientation: hexx::HexOrientation::Flat,
            origin: Vec2::new(0.0, 0.0),
            scale: Vec2::new(1.0, 1.0)
        };

        Self {
            size,
            width,
            height,
            layout,
        }
    }
    
}
