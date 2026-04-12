#[derive(Debug, Clone)]
pub struct AgentProfile {
    pub id: String,
    pub description: String,
}

#[derive(Debug, Clone)]
pub struct BaselineAgent {
    pub profile: AgentProfile,
}

