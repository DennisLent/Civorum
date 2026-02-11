/// Compass Directions for pointy top representation
/// Use odd-r indentation (odd rows indented) 
/// NW  / \  NE
/// W   |  | E
/// SW  \  / SE
pub enum CompassDirection {
    NE,
    E,
    SE,
    SW,
    W,
    NW
}

/// Basic struct to store hex coordinates
pub struct HexCoord {
    x: i32,
    y: i32,
}

impl HexCoord{
    /// Instantiate a new HexCoord
    pub fn new(x: i32, y: i32) -> Self {
        HexCoord { x, y }
    }
    
    /// Return the x coord
    pub fn x(&self) -> i32 {
        self.x
    }

    /// Return the y coord
    pub fn y(&self) -> i32 {
        self.y
    }

    /// Return the coordinate of the neighbor in a given direction
    pub fn neighbor(&self, direction: CompassDirection) -> HexCoord {
        let p = self.y & 1;
        if p == 0{
            match direction {
                CompassDirection::NE => HexCoord::new(self.x, self.y - 1),
                CompassDirection::E => HexCoord::new(self.x + 1, self.y),
                CompassDirection::SE => HexCoord::new(self.x, self.y + 1),
                CompassDirection::SW => HexCoord::new(self.x - 1 , self.y + 1),
                CompassDirection::W => HexCoord::new(self.x - 1, self.y),
                CompassDirection:: NW => HexCoord::new(self.x - 1, self.y - 1)
            }
        } else {
            match direction {
                CompassDirection::NE => HexCoord::new(self.x + 1, self.y - 1),
                CompassDirection::E => HexCoord::new(self.x + 1, self.y),
                CompassDirection::SE => HexCoord::new(self.x + 1, self.y + 1),
                CompassDirection::SW => HexCoord::new(self.x, self.y + 1),
                CompassDirection::W => HexCoord::new(self.x - 1, self.y),
                CompassDirection:: NW => HexCoord::new(self.x, self.y - 1)
            }
        }
    }
}


