use civorum_content::BenchmarkSuite;

#[derive(Debug, Default)]
pub struct BenchmarkRunner;

impl BenchmarkRunner {
    pub fn describe(&self, suite: &BenchmarkSuite) -> String {
        format!(
            "benchmark '{}' across {} scenarios and {} seeds",
            suite.name,
            suite.scenarios.len(),
            suite.seeds.len()
        )
    }
}

