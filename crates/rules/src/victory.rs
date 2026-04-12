#[derive(Debug, Clone, PartialEq, Eq)]
pub enum VictoryCondition {
    ScoreThreshold(u32),
    Elimination,
    TurnLimit,
}

