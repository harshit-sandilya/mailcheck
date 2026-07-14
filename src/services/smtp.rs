use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Result, bail};
use hickory_resolver::proto::rr::rdata::MX;
use hickory_resolver::{Resolver, TokioResolver};
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, RootCertStore};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::time::{sleep, timeout};
use tokio_rustls::TlsConnector;

use crate::models::record::OutputRecord;
use crate::services::template;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verdict {
    Valid,
    Invalid,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeStage {
    Greeting,
    Helo,
    StartTls,
    MailFrom,
    Recipient,
}

impl ProbeStage {
    fn label(self) -> &'static str {
        match self {
            Self::Greeting => "server greeting",
            Self::Helo => "EHLO/HELO",
            Self::StartTls => "STARTTLS",
            Self::MailFrom => "MAIL FROM",
            Self::Recipient => "RCPT TO",
        }
    }
}

#[derive(Debug, Clone)]
struct ProbeResult {
    code: u16,
    stage: ProbeStage,
}

#[derive(Debug, Clone)]
struct Verification {
    verdict: Verdict,
    code: u16,
    stage: Option<ProbeStage>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecipientPolicy {
    Selective,
    AcceptAll,
    Inconclusive,
}

#[derive(Debug)]
struct SmtpResponse {
    code: u16,
    text: String,
}

pub struct SmtpService {
    resolver: TokioResolver,
    delay_ms: u64,
    from: String,
    helo: String,
    timeout_secs: u64,
    tls_config: Arc<ClientConfig>,
}

impl SmtpService {
    pub fn new(from: String, delay_ms: u64) -> Result<Self> {
        if from.contains(['\r', '\n']) {
            bail!("MAIL FROM address contains a newline");
        }

        let resolver = Resolver::builder_tokio()?.build()?;
        let native_certs = rustls_native_certs::load_native_certs();
        let mut roots = RootCertStore::empty();
        let (added, _) = roots.add_parsable_certificates(native_certs.certs);
        if added == 0 {
            bail!("unable to load trusted TLS certificates");
        }
        let tls_config = Arc::new(
            ClientConfig::builder()
                .with_root_certificates(roots)
                .with_no_client_auth(),
        );

        Ok(Self {
            resolver,
            delay_ms,
            from,
            helo: "mailcheck.local".to_string(),
            timeout_secs: 8,
            tls_config,
        })
    }

    async fn mx_hosts(&self, domain: &str) -> Vec<String> {
        match self.resolver.mx_lookup(domain).await {
            Ok(lookup) => {
                let mut records = Vec::new();
                for record in lookup.answers() {
                    if let Some(mx_ref) = record.try_borrow::<MX>() {
                        let mx = mx_ref.data();
                        let host = mx.exchange.to_string().trim_end_matches('.').to_string();
                        if !host.is_empty() {
                            records.push((mx.preference, host));
                        }
                    }
                }
                records.sort_by_key(|record| record.0);
                records.into_iter().map(|(_, host)| host).collect()
            }
            Err(_) => match self.resolver.lookup_ip(domain).await {
                Ok(addresses) if addresses.iter().next().is_some() => vec![domain.to_string()],
                _ => Vec::new(),
            },
        }
    }

    async fn read_response<R>(&self, reader: &mut R) -> Option<SmtpResponse>
    where
        R: AsyncBufRead + Unpin,
    {
        let duration = Duration::from_secs(self.timeout_secs);
        let mut text = String::new();
        let mut code = None;

        loop {
            let mut line = String::new();
            let bytes = timeout(duration, reader.read_line(&mut line))
                .await
                .ok()?
                .ok()?;
            if bytes == 0 || line.len() < 3 {
                return None;
            }
            let line_code: u16 = line.get(0..3)?.parse().ok()?;
            code.get_or_insert(line_code);
            let continued = line.as_bytes().get(3) == Some(&b'-');
            text.push_str(&line);
            if !continued {
                break;
            }
        }

        Some(SmtpResponse { code: code?, text })
    }

    async fn send_command<S>(
        &self,
        reader: &mut BufReader<S>,
        command: &str,
    ) -> Option<SmtpResponse>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let duration = Duration::from_secs(self.timeout_secs);
        timeout(duration, reader.get_mut().write_all(command.as_bytes()))
            .await
            .ok()?
            .ok()?;
        self.read_response(reader).await
    }

    async fn greet<S>(&self, reader: &mut BufReader<S>) -> Option<SmtpResponse>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let ehlo = self
            .send_command(reader, &format!("EHLO {}\r\n", self.helo))
            .await?;
        if (200..300).contains(&ehlo.code) {
            return Some(ehlo);
        }

        let helo = self
            .send_command(reader, &format!("HELO {}\r\n", self.helo))
            .await?;
        (200..300).contains(&helo.code).then_some(helo)
    }

    async fn transaction<S>(&self, mut reader: BufReader<S>, to: &str) -> Option<ProbeResult>
    where
        S: AsyncRead + AsyncWrite + Unpin,
    {
        let mail = self
            .send_command(&mut reader, &format!("MAIL FROM:<{}>\r\n", self.from))
            .await?;
        if !(200..300).contains(&mail.code) {
            return Some(ProbeResult {
                code: mail.code,
                stage: ProbeStage::MailFrom,
            });
        }

        let recipient = self
            .send_command(&mut reader, &format!("RCPT TO:<{to}>\r\n"))
            .await?;
        let _ = timeout(
            Duration::from_secs(self.timeout_secs),
            reader.get_mut().write_all(b"QUIT\r\n"),
        )
        .await;
        Some(ProbeResult {
            code: recipient.code,
            stage: ProbeStage::Recipient,
        })
    }

    /// Performs an SMTP handshake through RCPT TO. DATA is never issued.
    async fn probe(&self, mx_host: &str, to: &str) -> Option<ProbeResult> {
        if to.contains(['\r', '\n']) {
            return None;
        }

        let duration = Duration::from_secs(self.timeout_secs);
        let stream = timeout(duration, TcpStream::connect(format!("{mx_host}:25")))
            .await
            .ok()?
            .ok()?;
        let mut reader = BufReader::new(stream);

        let greeting = self.read_response(&mut reader).await?;
        if greeting.code != 220 {
            return Some(ProbeResult {
                code: greeting.code,
                stage: ProbeStage::Greeting,
            });
        }

        let ehlo = match self.greet(&mut reader).await {
            Some(response) => response,
            None => {
                return Some(ProbeResult {
                    code: 0,
                    stage: ProbeStage::Helo,
                });
            }
        };

        let supports_starttls = ehlo.text.lines().any(|line| {
            line.get(4..)
                .is_some_and(|capability| capability.trim().eq_ignore_ascii_case("STARTTLS"))
        });

        if supports_starttls {
            let starttls = self.send_command(&mut reader, "STARTTLS\r\n").await?;
            if starttls.code != 220 {
                return Some(ProbeResult {
                    code: starttls.code,
                    stage: ProbeStage::StartTls,
                });
            }

            let server_name = ServerName::try_from(mx_host.to_string()).ok()?;
            let connector = TlsConnector::from(self.tls_config.clone());
            let tls = timeout(
                duration,
                connector.connect(server_name, reader.into_inner()),
            )
            .await
            .ok()?
            .ok()?;
            let mut tls_reader = BufReader::new(tls);
            if self.greet(&mut tls_reader).await.is_none() {
                return Some(ProbeResult {
                    code: 0,
                    stage: ProbeStage::Helo,
                });
            }
            return self.transaction(tls_reader, to).await;
        }

        self.transaction(reader, to).await
    }

    async fn verify(&self, mx_hosts: &[String], to: &str) -> Verification {
        let mut stage_failure = None;
        let mut recipient_unknown = None;
        for mx in mx_hosts {
            if let Some(result) = self.probe(mx, to).await {
                if result.stage != ProbeStage::Recipient {
                    stage_failure.get_or_insert(result);
                    continue;
                }
                let verdict = match result.code {
                    200..=299 => Verdict::Valid,
                    500..=599 => Verdict::Invalid,
                    _ => Verdict::Unknown,
                };
                let verification = Verification {
                    verdict,
                    code: result.code,
                    stage: Some(result.stage),
                };
                if verification.verdict == Verdict::Unknown {
                    recipient_unknown.get_or_insert(verification);
                    continue;
                }
                return verification;
            }
        }

        if let Some(verification) = recipient_unknown {
            return verification;
        }
        match stage_failure {
            Some(result) => Verification {
                verdict: Verdict::Unknown,
                code: result.code,
                stage: Some(result.stage),
            },
            None => Verification {
                verdict: Verdict::Unknown,
                code: 0,
                stage: None,
            },
        }
    }

    fn reason_for(verification: &Verification) -> String {
        if let Some(stage) = verification.stage
            && stage != ProbeStage::Recipient
        {
            return if verification.code == 0 {
                format!("SMTP session failed during {}", stage.label())
            } else {
                format!(
                    "SMTP session rejected during {} (code {})",
                    stage.label(),
                    verification.code
                )
            };
        }

        match (&verification.verdict, verification.code) {
            (Verdict::Valid, _) => "email accepted by server".to_string(),
            (Verdict::Invalid, 550) => "mailbox rejected by server (code 550)".to_string(),
            (Verdict::Invalid, 551) => "user not local, wrong server".to_string(),
            (Verdict::Invalid, 553) => "mailbox name invalid".to_string(),
            (Verdict::Invalid, code) => format!("rejected by server (code {code})"),
            (Verdict::Unknown, 0) => "no response — connection failed or timed out".to_string(),
            (Verdict::Unknown, code) if (400..500).contains(&code) => {
                format!("temporarily deferred (code {code}) — retry later")
            }
            (Verdict::Unknown, code) => format!("unexpected response (code {code})"),
        }
    }

    async fn recipient_policy(&self, mx_hosts: &[String], domain: &str) -> RecipientPolicy {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let mut verdicts = Vec::new();
        for index in 0..2 {
            let address = format!("mailcheck-probe-{nonce:x}-{index}@{domain}");
            verdicts.push(self.verify(mx_hosts, &address).await.verdict);
            self.wait_between_probes().await;
        }

        Self::classify_recipient_policy(&verdicts)
    }

    fn classify_recipient_policy(verdicts: &[Verdict]) -> RecipientPolicy {
        if verdicts.iter().all(|verdict| *verdict == Verdict::Valid) {
            RecipientPolicy::AcceptAll
        } else if verdicts.iter().all(|verdict| *verdict == Verdict::Invalid) {
            RecipientPolicy::Selective
        } else {
            RecipientPolicy::Inconclusive
        }
    }

    async fn wait_between_probes(&self) {
        if self.delay_ms > 0 {
            sleep(Duration::from_millis(self.delay_ms)).await;
        }
    }

    fn output(
        domain: &str,
        first: &str,
        last: &str,
        email: String,
        status: &str,
        passed: bool,
        reason: String,
    ) -> OutputRecord {
        OutputRecord {
            domain: domain.to_string(),
            first_name: first.to_string(),
            last_name: last.to_string(),
            email,
            confidence: None,
            pattern_source: String::new(),
            status: status.to_string(),
            passed,
            reason,
        }
    }

    pub async fn check_all(
        &self,
        domain: &str,
        first: &str,
        last: &str,
        patterns: &[String],
    ) -> Result<Vec<OutputRecord>> {
        if [domain, first, last]
            .iter()
            .any(|value| value.contains(['\r', '\n']))
        {
            bail!("domain and names must not contain newlines");
        }

        let mx_hosts = self.mx_hosts(domain).await;
        if mx_hosts.is_empty() {
            return Ok(vec![Self::output(
                domain,
                first,
                last,
                String::new(),
                "unknown",
                false,
                "no usable MX records found for domain".to_string(),
            )]);
        }

        let locals = template::candidates(patterns, first, last);
        let policy = self.recipient_policy(&mx_hosts, domain).await;
        if policy == RecipientPolicy::AcceptAll {
            return Ok(locals
                .into_iter()
                .map(|local| {
                    Self::output(
                        domain,
                        first,
                        last,
                        format!("{local}@{domain}"),
                        "opaque",
                        false,
                        "SMTP accepts randomized recipients — mailbox is opaque; use pattern confidence"
                            .to_string(),
                    )
                })
                .collect());
        }

        let mut results = Vec::new();
        for local in locals {
            let address = format!("{local}@{domain}");
            let verification = self.verify(&mx_hosts, &address).await;
            let (status, passed, reason) = if policy == RecipientPolicy::Inconclusive {
                (
                    "unverifiable",
                    false,
                    format!(
                        "catch-all probes were inconclusive; {}",
                        Self::reason_for(&verification)
                    ),
                )
            } else {
                (
                    match verification.verdict {
                        Verdict::Valid => "confirmed",
                        Verdict::Invalid => "rejected",
                        Verdict::Unknown => "unknown",
                    },
                    verification.verdict == Verdict::Valid,
                    Self::reason_for(&verification),
                )
            };
            results.push(Self::output(
                domain, first, last, address, status, passed, reason,
            ));
            self.wait_between_probes().await;
        }
        Ok(results)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn baseline_probes_classify_recipient_policy_conservatively() {
        assert_eq!(
            SmtpService::classify_recipient_policy(&[Verdict::Valid, Verdict::Valid]),
            RecipientPolicy::AcceptAll
        );
        assert_eq!(
            SmtpService::classify_recipient_policy(&[Verdict::Invalid, Verdict::Invalid]),
            RecipientPolicy::Selective
        );
        assert_eq!(
            SmtpService::classify_recipient_policy(&[Verdict::Invalid, Verdict::Unknown]),
            RecipientPolicy::Inconclusive
        );
    }
}
