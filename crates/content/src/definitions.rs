use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GameDefinition {
    pub id: String,
    pub name: String,
    pub observation_schema: Vec<String>,
    pub metrics: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct UnitDefinition {
    pub id: String,
    pub name: String,
    pub tags: Vec<String>,
    pub actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TileDefinition {
    pub id: String,
    pub tags: Vec<String>,
    pub passable: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResourceDefinition {
    pub id: String,
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionDefinition {
    pub id: String,
    pub description: String,
}

