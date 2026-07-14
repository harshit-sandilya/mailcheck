# mailcheck

`mailcheck` finds likely company email addresses from a domain and a person's
first and last name. It combines company-specific pattern evidence with a raw
SMTP recipient check. It never sends `DATA`, so it never sends an email.

## How it works

1. Resolve and prioritize the domain's MX servers.
2. Perform a complete SMTP greeting and use STARTTLS when the server offers it.
3. Probe two randomized recipients to classify the server as selective,
   accept-all/opaque, or inconclusive.
4. Prioritize patterns known for that company, then try the global patterns.
5. Report SMTP status separately from pattern confidence.

An accept-all response is not proof that a mailbox exists. It can represent a
real catch-all mailbox, delayed validation, a relay, or recipient-enumeration
protection. In that case, company pattern confidence is the useful signal.

## Build

Requires a current Rust toolchain:

```sh
cargo build --release
```

The binary is written to `target/release/mailcheck`.

## Basic usage

Configure a real envelope sender and a polite delay first:

```sh
mailcheck config set-email you@example.com
mailcheck config set-delay 500
```

Find an address:

```sh
mailcheck find example.com Jane Doe
```

Possible result statuses are:

- `confirmed`: the server rejected randomized recipients and accepted this one.
- `rejected`: the server rejected this recipient.
- `opaque`: the server accepted both randomized recipients, so SMTP cannot
  distinguish mailboxes.
- `unverifiable`: baseline probes produced inconsistent results.
- `unknown`: the SMTP session failed or was temporarily deferred.

`CONF` is the evidence confidence for a company-specific pattern. It is not a
mailbox-delivery guarantee.

## Company pattern data

Release binaries embed the reviewed community registry at
[`data/companies.json`](data/companies.json). Each user can extend or override
it locally:

```sh
mailcheck companies add example.com '{first}.{last}' --confidence 85 --samples 3
mailcheck companies show example.com
mailcheck companies list
mailcheck companies reset example.com
```

Local data is stored in `~/.mailcheck/companies.json`. A local entry with the
same domain and pattern overrides the embedded community entry. Reset removes
only the local data and reveals the community data again.

See [CONTRIBUTING.md](CONTRIBUTING.md) to contribute verified company patterns.

## Global patterns

The built-in fallbacks are:

```text
{first}          {last}           {first}.{last}
{last}.{first}   {first}{last}    {last}{first}
{f}{last}        {f}.{last}       {first}{l}
{first}.{l}      {first}_{last}   {first}-{last}
{f}{l}
```

Manage them with:

```sh
mailcheck patterns list
mailcheck patterns add '{first}.{l}'
mailcheck patterns remove '{first}'
mailcheck patterns reset
```

## CSV batches

Input requires a header row containing `domain,first,last`:

```csv
domain,first,last
example.com,Jane,Doe
```

Run a batch sequentially:

```sh
mailcheck find-all people.csv
mailcheck find-all people.csv --out results.csv
```

Output includes the candidate, SMTP status, pattern confidence/source, boolean
compatibility field, and human-readable reason.

## Operational limitations

- Outbound TCP port 25 must be reachable.
- SMTP servers can intentionally hide recipient existence; no SMTP command can
  force an accept-all server to expose its directory.
- A `confirmed` result means the remote SMTP server distinguished the address
  during this check. It does not guarantee that a human reads the mailbox.
- Probing too quickly can cause throttling or greylisting. Use a delay and keep
  checks targeted.
- Names with punctuation, multiple surnames, transliteration, or collision
  suffixes may require custom patterns.

## Other commands

```sh
mailcheck info
mailcheck update
mailcheck --help
```
