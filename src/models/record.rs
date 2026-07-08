use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Deserialize)]
pub struct InputRecord {
    pub domain: String,
    pub first: String,
    pub last: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct OutputRecord {
    pub domain: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub passed: bool,
    pub reason: String,
}
