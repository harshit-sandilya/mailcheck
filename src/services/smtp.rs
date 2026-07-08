use anyhow::Result;
use hickory_resolver::Resolver;
use hickory_resolver::TokioResolver;
use hickory_resolver::proto::rr::rdata::MX;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};

use crate::models::record::OutputRecord;
use crate::services::template;

#[derive(Debug, Clone, PartialEq)]
pub enum Verdict {
    Valid,
    Invalid,
    Unknown,
}

pub struct SmtpService {
    resolver: TokioResolver,
    delay_ms: u64,
    from: String,
    helo: String,
    timeout_secs: u64,
}

impl SmtpService {
    pub fn new(from: String, delay_ms: u64) -> Result<Self> {
        let resolver = Resolver::builder_tokio()?.build()?;
        Ok(Self {
            resolver,
            delay_ms,
            from,
            helo: "mailcheck.local".to_string(),
            timeout_secs: 8,
        })
    }

    async fn mx_hosts(&self, domain: &str) -> Vec<String> {
        match self.resolver.mx_lookup(domain).await {
            Ok(lookup) => {
                let mut records = Vec::new();

                for record in lookup.answers() {
                    if let Some(mx_ref) = record.try_borrow::<MX>() {
                        let mx = mx_ref.data();

                        records.push((
                            mx.preference,
                            mx.exchange.to_string().trim_end_matches('.').to_string(),
                        ));
                    }
                }

                records.sort_by_key(|r| r.0);
                records.into_iter().map(|(_, host)| host).collect()
            }
            Err(_) => vec![],
        }
    }

    /// Single SMTP handshake up to RCPT TO. Never sends DATA, so no email is transmitted.
    async fn probe(&self, mx_host: &str, to: &str) -> Option<u16> {
        let dur = Duration::from_secs(self.timeout_secs);
        let stream = timeout(dur, TcpStream::connect(format!("{mx_host}:25")))
            .await
            .ok()?
            .ok()?;
        let (rd, mut wr) = stream.into_split();
        let mut reader = BufReader::new(rd);
        let mut line = String::new();

        timeout(dur, reader.read_line(&mut line)).await.ok()?.ok()?;
        line.clear();

        wr.write_all(format!("EHLO {}\r\n", self.helo).as_bytes())
            .await
            .ok()?;
        loop {
            line.clear();
            timeout(dur, reader.read_line(&mut line)).await.ok()?.ok()?;
            if line.len() < 4 || &line[3..4] != "-" {
                break;
            }
        }

        wr.write_all(format!("MAIL FROM:<{}>\r\n", self.from).as_bytes())
            .await
            .ok()?;
        line.clear();
        timeout(dur, reader.read_line(&mut line)).await.ok()?.ok()?;

        wr.write_all(format!("RCPT TO:<{to}>\r\n").as_bytes())
            .await
            .ok()?;
        line.clear();
        timeout(dur, reader.read_line(&mut line)).await.ok()?.ok()?;
        let code: u16 = line.get(0..3)?.parse().ok()?;

        let _ = wr.write_all(b"QUIT\r\n").await;
        Some(code)
    }

    async fn verify(&self, mx_hosts: &[String], to: &str) -> (Verdict, u16) {
        for mx in mx_hosts {
            if let Some(code) = self.probe(mx, to).await {
                let verdict = match code {
                    200..=299 => Verdict::Valid,
                    500..=599 => Verdict::Invalid,
                    _ => Verdict::Unknown,
                };
                return (verdict, code);
            }
        }
        (Verdict::Unknown, 0)
    }

    fn reason_for(code: u16, verdict: &Verdict) -> String {
        match (verdict, code) {
            (Verdict::Valid, _) => "email accepted by server".to_string(),
            (Verdict::Invalid, 550) => "mailbox doesn't exist".to_string(),
            (Verdict::Invalid, 551) => "user not local, wrong server".to_string(),
            (Verdict::Invalid, 553) => "mailbox name invalid".to_string(),
            (Verdict::Invalid, c) => format!("rejected by server (code {c})"),
            (Verdict::Unknown, 0) => "no response — connection failed or timed out".to_string(),
            (Verdict::Unknown, c) if (400..500).contains(&c) => {
                format!("temporarily deferred (code {c}) — possibly greylisted, retry later")
            }
            (Verdict::Unknown, c) => format!("unexpected response (code {c})"),
        }
    }

    pub async fn check_all(
        &self,
        domain: &str,
        first: &str,
        last: &str,
        patterns: &[String],
    ) -> Result<Vec<OutputRecord>> {
        let mx_hosts = self.mx_hosts(domain).await;
        if mx_hosts.is_empty() {
            return Ok(vec![OutputRecord {
                domain: domain.to_string(),
                first_name: first.to_string(),
                last_name: last.to_string(),
                email: String::new(),
                passed: false,
                reason: "no MX records found for domain".to_string(),
            }]);
        }

        let probe_addr = format!("zzz-notreal-{first}{last}@{domain}").to_lowercase();
        let (probe_verdict, _) = self.verify(&mx_hosts, &probe_addr).await;
        let is_catch_all = probe_verdict == Verdict::Valid;

        let mut results = Vec::new();
        for local in template::candidates(patterns, first, last) {
            let addr = format!("{local}@{domain}");
            let (verdict, code) = self.verify(&mx_hosts, &addr).await;

            let (passed, reason) = if is_catch_all {
                (
                    false,
                    "domain is catch-all (accepts any address) — result unconfirmed".to_string(),
                )
            } else {
                (verdict == Verdict::Valid, Self::reason_for(code, &verdict))
            };

            results.push(OutputRecord {
                domain: domain.to_string(),
                first_name: first.to_string(),
                last_name: last.to_string(),
                email: addr,
                passed,
                reason,
            });

            if self.delay_ms > 0 {
                sleep(Duration::from_millis(self.delay_ms)).await;
            }
        }
        Ok(results)
    }
}
