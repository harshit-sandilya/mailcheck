# Contributing to mailcheck

## Company pattern contributions

Community company patterns live in `data/companies.json` and are embedded into
release binaries. Add or update the smallest possible record and open a pull
request.

Schema:

```json
{
  "version": 1,
  "companies": [
    {
      "domain": "example.com",
      "patterns": [
        {
          "pattern": "{first}.{last}",
          "confidence": 85,
          "samples": 3
        }
      ]
    }
  ]
}
```

Rules:

- Domains must be lowercase registrable mail domains without a trailing dot.
- Keep company records sorted alphabetically by domain.
- Valid tokens are `{first}`, `{last}`, `{f}`, and `{l}`.
- `samples` is the number of independently observed employee addresses that
  match the pattern.
- Confidence describes evidence that the company uses the pattern; it does not
  claim that every employee has that address.
- Use 50–74 for one credible observation, 75–89 for two to four consistent
  observations, and 90–99 for five or more consistent observations.
- Do not use 100: aliases, name collisions, and regional systems always leave
  room for exceptions.
- Explain the public evidence in the pull request. Do not commit personal data,
  scraped address lists, private correspondence, or credentials.
- Prefer evidence published by the company itself, such as staff, press,
  careers, or contact pages.

Before opening the pull request, run:

```sh
cargo fmt -- --check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

The test suite validates the embedded registry, including domain formatting,
tokens, confidence bounds, and duplicate patterns.

## Local-only data

Patterns that should not be contributed can remain private:

```sh
mailcheck companies add example.com '{first}.{last}' --confidence 80 --samples 2
```

This writes only to `~/.mailcheck/companies.json`.
