use std::collections::{BTreeMap, HashSet};

use anyhow::{Context, Result};

use crate::models::company::{CompanyPattern, CompanyRegistry, normalize_domain};
use crate::models::record::OutputRecord;
use crate::services::template;

const COMMUNITY_REGISTRY: &str = include_str!("../../data/companies.json");

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RankedPattern {
    pub pattern: String,
    pub confidence: u8,
    pub samples: u32,
    pub source: &'static str,
}

pub fn community_registry() -> Result<CompanyRegistry> {
    CompanyRegistry::from_json(COMMUNITY_REGISTRY)
        .map_err(anyhow::Error::msg)
        .context("embedded company registry is invalid")
}

pub fn ranked_patterns(domain: &str, local: &CompanyRegistry) -> Result<Vec<RankedPattern>> {
    let community = community_registry()?;
    let mut merged = BTreeMap::<String, RankedPattern>::new();

    for candidate in community.patterns_for(domain) {
        merged.insert(candidate.pattern.clone(), ranked(candidate, "community"));
    }
    for candidate in local.patterns_for(domain) {
        merged.insert(candidate.pattern.clone(), ranked(candidate, "local"));
    }

    let mut patterns: Vec<_> = merged.into_values().collect();
    patterns.sort_by(|a, b| {
        b.confidence
            .cmp(&a.confidence)
            .then_with(|| b.samples.cmp(&a.samples))
            .then_with(|| a.pattern.cmp(&b.pattern))
    });
    Ok(patterns)
}

fn ranked(candidate: CompanyPattern, source: &'static str) -> RankedPattern {
    RankedPattern {
        pattern: candidate.pattern,
        confidence: candidate.confidence,
        samples: candidate.samples,
        source,
    }
}

pub fn merge_with_fallbacks(ranked: &[RankedPattern], fallbacks: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    ranked
        .iter()
        .map(|candidate| candidate.pattern.clone())
        .chain(fallbacks.iter().cloned())
        .filter(|pattern| seen.insert(pattern.clone()))
        .collect()
}

pub fn annotate_results(
    results: &mut [OutputRecord],
    ranked: &[RankedPattern],
    first: &str,
    last: &str,
) {
    let mut evidence = BTreeMap::<String, &RankedPattern>::new();
    for candidate in ranked {
        evidence
            .entry(template::render(&candidate.pattern, first, last))
            .or_insert(candidate);
    }

    for result in results {
        let local = result.email.split('@').next().unwrap_or_default();
        if let Some(candidate) = evidence.get(local) {
            result.confidence = Some(candidate.confidence);
            result.pattern_source = candidate.source.to_string();
        }
    }
}

pub fn all_domains(local: &CompanyRegistry) -> Result<Vec<(String, &'static str)>> {
    let community = community_registry()?;
    let mut domains = BTreeMap::<String, &'static str>::new();
    for company in community.companies {
        domains.insert(normalize_domain(&company.domain), "community");
    }
    for company in &local.companies {
        let domain = normalize_domain(&company.domain);
        let source = if domains.contains_key(&domain) {
            "community+local"
        } else {
            "local"
        };
        domains.insert(domain, source);
    }
    Ok(domains.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_registry_is_valid() {
        community_registry().unwrap();
    }

    #[test]
    fn local_pattern_overrides_community_shape() {
        let mut local = CompanyRegistry::default();
        local.upsert(
            "example.com".to_string(),
            "{first}.{last}".to_string(),
            88,
            4,
        );

        let ranked = ranked_patterns("example.com", &local).unwrap();
        assert_eq!(ranked[0].confidence, 88);
        assert_eq!(ranked[0].source, "local");
    }
}
