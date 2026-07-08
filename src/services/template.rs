/// Renders a pattern like "{f}.{last}" into a candidate local-part, e.g. "j.doe".
/// Tokens: {f} = first-name initial, {l} = last-name initial,
///         {first} = full first name, {last} = full last name.
/// Names are lowercased before substitution.
pub fn render(pattern: &str, first: &str, last: &str) -> String {
    let first = first.to_lowercase();
    let last = last.to_lowercase();
    let fi = first
        .chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_default();
    let li = last
        .chars()
        .next()
        .map(|c| c.to_string())
        .unwrap_or_default();

    pattern
        .replace("{first}", &first)
        .replace("{last}", &last)
        .replace("{f}", &fi)
        .replace("{l}", &li)
}

/// Renders every pattern for a given name, deduplicating the result.
pub fn candidates(patterns: &[String], first: &str, last: &str) -> Vec<String> {
    let mut out: Vec<String> = patterns.iter().map(|p| render(p, first, last)).collect();
    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renders_all_tokens() {
        assert_eq!(render("{first}.{last}", "Jane", "Doe"), "jane.doe");
        assert_eq!(render("{f}{last}", "Jane", "Doe"), "jdoe");
        assert_eq!(render("{first}{l}", "Jane", "Doe"), "janed");
        assert_eq!(render("{f}{l}", "Jane", "Doe"), "jd");
    }
}
