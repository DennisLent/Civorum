#[derive(Debug, Clone, Copy, PartialEq, Eq)]
/// Base yields enums for easier comparisons
pub enum BaseYields {
    Food,
    Production,
    Gold,
    Science,
    Culture,
    Faith,
    Appeal,
}

/// Base yields in the game of civ.
/// This also includes the appeal.
pub struct Yields {
    food: i32,
    production: i32,
    gold: i32,
    science: i32,
    culture: i32,
    faith: i32,
    appeal: i32,
}

impl Yields {
    /// Create a new istance of the yield object
    pub fn new(
        food: i32,
        production: i32,
        gold: i32,
        science: i32,
        culture: i32,
        faith: i32,
        appeal: i32,
    ) -> Self {
        return Yields {
            food,
            production,
            gold,
            science,
            culture,
            faith,
            appeal,
        };
    }

    /// Return a specific yield for a tile
    pub fn get_yield(&self, yield_type: BaseYields) -> i32 {
        match yield_type {
            BaseYields::Food => self.food,
            BaseYields::Production => self.production,
            BaseYields::Gold => self.gold,
            BaseYields::Science => self.science,
            BaseYields::Culture => self.culture,
            BaseYields::Faith => self.faith,
            BaseYields::Appeal => self.appeal,
        }
    }

    /// Return a vector with all the yields of a single tile in this order:
    /// 1. Food
    /// 2. Production
    /// 3. Gold
    /// 4. Science
    /// 5. Culture,
    /// 6. Faith
    /// 7. Appeal
    pub fn get_yields(&self) -> Vec<i32> {
        vec![
            self.food,
            self.production,
            self.gold,
            self.science,
            self.culture,
            self.faith,
            self.appeal,
        ]
    }

    /// Set the yields based on modifiers
    pub fn set_yields(
        &mut self,
        yield_types: Vec<BaseYields>,
        modifiers: Vec<i32>,
    ) -> Result<(), &'static str> {
        for (yield_type, modifier) in yield_types.iter().zip(modifiers.iter()) {
            match yield_type {
                BaseYields::Food => self.food += modifier,
                BaseYields::Production => self.production += modifier,
                BaseYields::Gold => self.gold += modifier,
                BaseYields::Science => self.science += modifier,
                BaseYields::Culture => self.culture += modifier,
                BaseYields::Faith => self.faith += modifier,
                BaseYields::Appeal => self.appeal += modifier,
            }
        }

        Ok(())
    }
}
