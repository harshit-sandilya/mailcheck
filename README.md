# mailcheck

CLI that guesses a person's work email from first/last name + company domain, then verifies each guess via a raw SMTP handshake (MAIL FROM / RCPT TO) — it never actually sends an email, it just asks the mail server "would you accept mail for this address?" and reads the response code.

## Build

Requires Rust (install via https://rustup.rs if you don't have it):

```
cd mailcheck
cargo build --release
```

Binary will be at `target/release/mailcheck`.

## Usage

```
./target/release/mailcheck --domain example.com --first Jane --last Doe
```

Optional flags:

- `--from` — the MAIL FROM address used in the handshake. Use a real address of yours (e.g. your Gmail) since some servers reject obviously fake senders.
- `--helo` — hostname presented in EHLO. Default is fine for most cases.
- `--timeout-secs` — per-connection timeout (default 8s).

## Important notes

- **Port 25 must be reachable.** Many home ISPs and cloud providers (AWS, GCP, most VPS) block outbound port 25 by default to fight spam. If every result comes back "unknown", this is almost certainly why. Test from a normal home or office network first. If it's blocked, options are: run it from a machine/VPS with port 25 allowed (some providers unblock it on request), or fall back to a free web-based checker for spot checks.
- **Catch-all domains**: some company mail servers accept mail for _any_ address at that domain (so they can't be validated at SMTP level). The tool detects this by probing a near-certainly-fake address first, and marks results as "catch-all (unconfirmed)" if so — meaning the address might work but SMTP alone can't confirm it.
- **Rate limit yourself.** Hammering a mail server with lots of RCPT TO attempts in quick succession can get your IP greylisted or blocked. Add delays between different domains if you're checking many companies, and don't run this against the same domain in a tight loop.
- **Be a good citizen.** This is for finding a real human to send one thoughtful, personalized email to — not for building a spam list. Use it for a handful of targeted people at companies you're actually applying to.

## How the candidate list works

For "Jane Doe" @ example.com it tries: `jane`, `doe`, `jane.doe`, `doe.jane`, `janedoe`, `doejane`, `jdoe`, `j.doe`, `janed`, `jane.d`, `jane_doe`, `jane-doe`, `jd` — covering the vast majority of real-world corporate patterns. Easy to extend in `candidates()` in `src/main.rs` if a company uses something unusual (e.g. `jane.d@`, `d.jane@`).
