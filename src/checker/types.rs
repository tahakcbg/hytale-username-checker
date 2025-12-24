use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct CheckResult {
    pub username: String,
    pub status: ResultStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ResultStatus {
    Available,
    Taken,
    Error(String),
    Invalid,
}

#[derive(Debug, Deserialize)]
pub struct ApiResponse {
    pub available: Option<bool>,
}

#[derive(Default, Clone)]
pub struct Stats {
    pub total: usize,
    pub checked: usize,
    pub available: usize,
    pub taken: usize,
    pub errors: usize,
}
