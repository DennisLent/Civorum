#[derive(Debug, Clone)]
pub struct ReplayEndpoint {
    pub path: &'static str,
}

#[derive(Debug, Clone)]
pub struct BenchmarkEndpoint {
    pub path: &'static str,
}
