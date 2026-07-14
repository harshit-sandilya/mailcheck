use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::models::patterns::validate_pattern;

pub const REGISTRY_VERSION: u32 = 1;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyPattern {
    pub pattern: String,
    pub confidence: u8,
    #[serde(default)]
    pub samples: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyRecord {
    pub domain: String,
    #[serde(default)]
    pub patterns: Vec<CompanyPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompanyRegistry {
    pub version: u32,
    #[serde(default)]
    pub companies: Vec<CompanyRecord>,
}

impl Default for CompanyRegistry {
    fn default() -> Self {
        Self {
            version: REGISTRY_VERSION,
            companies: Vec::new(),
        }
    }
}

pub fn normalize_domain(domain: &str) -> String {
    domain.trim().trim_end_matches('.').to_lowercase()
}

pub fn validate_domain(domain: &str) -> Result<(), String> {
    let domain = normalize_domain(domain);
    if domain.is_empty()
        || !domain.contains('.')
        || domain.starts_with('.')
        || domain.ends_with('.')
        || domain.contains("..")
        || domain
            .chars()
            .any(|c| !(c.is_ascii_alphanumeric() || c == '-' || c == '.'))
    {
        return Err(format!("invalid company domain: {domain}"));
    }
    Ok(())
}

impl CompanyRegistry {
    pub fn from_json(content: &str) -> Result<Self, String> {
        let registry: Self = serde_json::from_str(content).map_err(|e| e.to_string())?;
        registry.validate()?;
        Ok(registry)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.version != REGISTRY_VERSION {
            return Err(format!(
                "unsupported company registry version {} (expected {REGISTRY_VERSION})",
                self.version
            ));
        }

        let mut seen = BTreeMap::<(&str, &str), ()>::new();
        for company in &self.companies {
            validate_domain(&company.domain)?;
            if company.domain != normalize_domain(&company.domain) {
                return Err(format!(
                    "company domain must be normalized as '{}'",
                    normalize_domain(&company.domain)
                ));
            }
            if company.patterns.is_empty() {
                return Err(format!("company '{}' has no patterns", company.domain));
            }

            for candidate in &company.patterns {
                validate_pattern(&candidate.pattern)?;
                if candidate.confidence > 100 {
                    return Err(format!(
                        "confidence for '{}:{}' must be between 0 and 100",
                        company.domain, candidate.pattern
                    ));
                }
                if candidate.samples == 0 {
                    return Err(format!(
                        "samples for '{}:{}' must be at least 1",
                        company.domain, candidate.pattern
                    ));
                }
                if seen
                    .insert((&company.domain, &candidate.pattern), ())
                    .is_some()
                {
                    return Err(format!(
                        "duplicate company pattern '{}:{}'",
                        company.domain, candidate.pattern
                    ));
                }
            }
        }
        Ok(())
    }

    pub fn patterns_for(&self, domain: &str) -> Vec<CompanyPattern> {
        let domain = normalize_domain(domain);
        self.companies
            .iter()
            .filter(|company| company.domain == domain)
            .flat_map(|company| company.patterns.iter().cloned())
            .collect()
    }

    pub fn upsert(&mut self, domain: String, pattern: String, confidence: u8, samples: u32) {
        let domain = normalize_domain(&domain);
        let company = match self
            .companies
            .iter_mut()
            .find(|company| company.domain == domain)
        {
            Some(company) => company,
            None => {
                self.companies.push(CompanyRecord {
                    domain: domain.clone(),
                    patterns: Vec::new(),
                });
                self.companies
                    .last_mut()
                    .expect("company was just inserted")
            }
        };

        match company
            .patterns
            .iter_mut()
            .find(|candidate| candidate.pattern == pattern)
        {
            Some(candidate) => {
                candidate.confidence = confidence;
                candidate.samples = samples;
            }
            None => company.patterns.push(CompanyPattern {
                pattern,
                confidence,
                samples,
            }),
        }
    }

    pub fn reset_domain(&mut self, domain: &str) -> bool {
        let domain = normalize_domain(domain);
        let before = self.companies.len();
        self.companies.retain(|company| company.domain != domain);
        self.companies.len() != before
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_confidence_and_duplicate_patterns() {
        let invalid_score = r#"{
            "version": 1,
            "companies": [{
                "domain": "example.com",
                "patterns": [{"pattern": "{first}.{last}", "confidence": 101, "samples": 2}]
            }]
        }"#;
        assert!(CompanyRegistry::from_json(invalid_score).is_err());

        let duplicate = r#"{
            "version": 1,
            "companies": [{
                "domain": "example.com",
                "patterns": [
                    {"pattern": "{first}.{last}", "confidence": 90, "samples": 2},
                    {"pattern": "{first}.{last}", "confidence": 80, "samples": 1}
                ]
            }]
        }"#;
        assert!(CompanyRegistry::from_json(duplicate).is_err());
    }

    #[test]
    fn local_upsert_replaces_the_same_pattern() {
        let mut registry = CompanyRegistry::default();
        registry.upsert(
            "Example.COM.".to_string(),
            "{first}.{last}".to_string(),
            75,
            1,
        );
        registry.upsert(
            "example.com".to_string(),
            "{first}.{last}".to_string(),
            90,
            4,
        );

        let patterns = registry.patterns_for("EXAMPLE.COM");
        assert_eq!(patterns.len(), 1);
        assert_eq!(patterns[0].confidence, 90);
        assert_eq!(patterns[0].samples, 4);
    }
}
