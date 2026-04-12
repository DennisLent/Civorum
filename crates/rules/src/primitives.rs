#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RulePrimitive {
    Move,
    Attack,
    Gather,
    FoundSettlement,
    ProduceEntity,
    EndTurn,
    UpdateVisibility,
    UpdateScore,
}

