#[derive(Debug, Clone)]
pub struct SuiteRunRequest {
    pub suite_id: String,
    pub agents: Vec<String>,
}

