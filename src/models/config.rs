use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub email: String,
    pub delay: i64,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            email: "default@gmail.com".to_string(),
            delay: 0,
        }
    }
}
