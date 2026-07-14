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

pub const VALID_TOKENS: [&str; 4] = ["{first}", "{last}", "{f}", "{l}"];

pub fn validate_pattern(pattern: &str) -> Result<(), String> {
    if pattern.is_empty() || pattern.contains(['\r', '\n']) {
        return Err("pattern must be non-empty and contain no newlines".to_string());
    }
    let mut cursor = 0;
    while cursor < pattern.len() {
        let remaining = &pattern[cursor..];
        match (remaining.find('{'), remaining.find('}')) {
            (None, None) => break,
            (None, Some(_)) => return Err(format!("unmatched '}}' in pattern: {pattern}")),
            (Some(open), Some(close)) if close < open => {
                return Err(format!("unmatched '}}' in pattern: {pattern}"));
            }
            (Some(open), Some(close)) => {
                let token = &remaining[open..=close];
                if !VALID_TOKENS.contains(&token) {
                    return Err(format!(
                        "unknown token '{token}' — valid tokens are {}",
                        VALID_TOKENS.join(", ")
                    ));
                }
                cursor += close + 1;
            }
            (Some(_), None) => return Err(format!("unmatched '{{' in pattern: {pattern}")),
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PatternOverrides {
    pub added: Vec<String>,
    pub removed: Vec<String>,
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_tokens_and_balanced_braces() {
        assert!(validate_pattern("{first}.{last}").is_ok());
        assert!(validate_pattern("{unknown}.{last}").is_err());
        assert!(validate_pattern("{first}.{last").is_err());
        assert!(validate_pattern("{first}.last}").is_err());
        assert!(validate_pattern("").is_err());
    }
}
