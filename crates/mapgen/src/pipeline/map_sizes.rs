/// The types of map sizes that exist for generating a map
pub enum MapSizes {
    Duel,
    Tiny,
    Small,
    Standard,
    Large,
    Huge
}

impl MapSizes {
    /// Return the dimensions (width, height) based on the size
    pub fn dimensions(&self) -> (i32, i32) {
        match self {
            Self::Duel => (44, 26),
            Self::Tiny => (60, 38),
            Self::Small => (74, 46),
            Self::Standard => (84, 54),
            Self::Large => (96, 60),
            Self::Huge => (106, 66)
        }
    }
}