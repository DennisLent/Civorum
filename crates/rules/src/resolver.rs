use civorum_content::GameDefinition;

#[derive(Debug, Clone)]
pub struct RuleSet {
    pub game_id: String,
    pub primitives: Vec<super::RulePrimitive>,
}

impl RuleSet {
    pub fn from_definition(definition: &GameDefinition) -> Self {
        Self {
            game_id: definition.id.clone(),
            primitives: Vec::new(),
        }
    }
}

