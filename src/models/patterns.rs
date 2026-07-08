use serde::{Deserialize, Serialize};

pub const BUILTIN_PATTERNS: &[&str] = &[
    "{first}",
    "{last}",
    "{first}.{last}",
    "{last}.{first}",
    "{first}{last}",
    "{last}{first}",
    "{f}{last}",
    "{f}.{last}",
    "{first}{l}",
    "{first}.{l}",
    "{first}_{last}",
    "{first}-{last}",
    "{f}{l}",
];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternOverrides {
    pub added: Vec<String>,
    pub removed: Vec<String>,
}

impl Default for PatternOverrides {
    fn default() -> Self {
        Self {
            added: Vec::new(),
            removed: Vec::new(),
        }
    }
}

impl PatternOverrides {
    pub fn effective(&self) -> Vec<String> {
        BUILTIN_PATTERNS
            .iter()
            .map(|p| p.to_string())
            .filter(|p| !self.removed.contains(p))
            .chain(self.added.iter().cloned())
            .collect()
    }

    pub fn add(&mut self, pattern: String) {
        if let Some(pos) = self.removed.iter().position(|p| p == &pattern) {
            self.removed.remove(pos);
            return;
        }
        if !self.added.contains(&pattern) {
            self.added.push(pattern);
        }
    }

    pub fn remove(&mut self, pattern: &str) {
        self.added.retain(|p| p != pattern);
        if BUILTIN_PATTERNS.contains(&pattern) && !self.removed.iter().any(|p| p == pattern) {
            self.removed.push(pattern.to_string());
        }
    }

    pub fn reset(&mut self) {
        self.added.clear();
        self.removed.clear();
    }
}
