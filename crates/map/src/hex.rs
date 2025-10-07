/// We consider the basic size of the circle to be 50 wide for now
pub const SIZE: i32 = 50;

/// Six axial directions in a flat‑top layout
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub enum Direction {
    E,
    NE,
    NW,
    W,
    SW,
    SE,
}

/// Axial hex coordinate (q, r)
/// We are using Flattt-top orientation and and even-r horizontal layout
/// Axial coordinates use two axes q,r that are 120 deg apart
#[derive(Copy, Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct Hex {
    pub q: i32,
    pub r: i32,
}

impl Hex {
    /// Create a new axial coordinate
    pub fn new(q: i32, r: i32) -> Self {
        Self { q, r }
    }

    /// Return the value of q
    pub fn q(&self) -> i32 {
        self.q
    }

    /// Return the value of r
    pub fn r(&self) -> i32 {
        self.r
    }

    pub fn s(&self) -> i32 {
        -self.q - self.r
    }

    /// Manhattan distance to origin (hex length)
    pub fn length(&self) -> i32 {
        (self.q.abs() + self.r.abs() + self.s().abs()) / 2
    }

    /// Manhattan distance in axial space
    pub fn distance(&self, other: Hex) -> i32 {
        hex_distance(*self, other)
    }

    /// Neighbor in a given direction
    pub fn neighbor(&self, dir: Direction) -> Hex {
        hex_add(*self, direction_vector(dir))
    }

    /// All six neighbors in E,NE,NW,W,SW,SE order
    pub fn neighbors(&self) -> [Hex; 6] {
        [
            self.neighbor(Direction::E),
            self.neighbor(Direction::NE),
            self.neighbor(Direction::NW),
            self.neighbor(Direction::W),
            self.neighbor(Direction::SW),
            self.neighbor(Direction::SE),
        ]
    }
}

pub fn hex_add(a: Hex, b: Hex) -> Hex {
    Hex {
        q: a.q + b.q,
        r: a.r + b.r,
    }
}

pub fn hex_subtract(a: Hex, b: Hex) -> Hex {
    Hex {
        q: a.q - b.q,
        r: a.r - b.r,
    }
}

pub fn hex_length(hex: Hex) -> i32 {
    (hex.q.abs() + hex.r.abs() + hex.s().abs()) / 2
}

pub fn hex_distance(a: Hex, b: Hex) -> i32 {
    hex_length(hex_subtract(a, b))
}

/// Direction unit vectors for axial coordinates (flat‑top)
pub const fn direction_vector(dir: Direction) -> Hex {
    match dir {
        Direction::E => Hex { q: 1, r: 0 },
        Direction::NE => Hex { q: 1, r: -1 },
        Direction::NW => Hex { q: 0, r: -1 },
        Direction::W => Hex { q: -1, r: 0 },
        Direction::SW => Hex { q: -1, r: 1 },
        Direction::SE => Hex { q: 0, r: 1 },
    }
}
