#[derive(Debug, Default)]
pub struct MatchService;

impl MatchService {
    pub fn status(&self) -> &'static str {
        "server scaffold"
    }
}

