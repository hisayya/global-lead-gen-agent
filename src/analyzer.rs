use anyhow::Result;
use async_native_tls::TlsConnector;
use futures::StreamExt;
use mailparse::parse_mail;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::llm::LlmClient;
use crate::models::ReplyClassification;

const CLASSIFY_SYSTEM: &str = r#"Classify this email reply. Output JSON:
{
  "class": "<one of: interested, not_now, objection, reject, ooo>",
  "suggested_action": "<what to do next>"
}

- interested: wants to talk, book a call, learn more
- not_now: maybe later, not right now
- objection: pricing concern, need more info, specific question
- reject: not interested, stop contacting
- ooo: out of office / auto-reply

Output ONLY valid JSON."#;

pub struct Analyzer {
    imap_host: String,
    imap_port: u16,
    imap_user: String,
    imap_pass: String,
}

impl Analyzer {
    pub fn new(cfg: &AppConfig) -> Self {
        Self {
            imap_host: cfg.imap_host.clone(),
            imap_port: cfg.imap_port,
            imap_user: cfg.imap_user.clone(),
            imap_pass: cfg.imap_pass.clone(),
        }
    }

    pub async fn fetch_and_classify(
        &self,
        llm: &LlmClient,
    ) -> Result<Vec<(String, ReplyClassification)>> {
        info!("connecting to IMAP");

        let tls = TlsConnector::new();
        let tcp = tokio::net::TcpStream::connect((self.imap_host.as_str(), self.imap_port))
            .await?;
        let tls_stream = tls.connect(&self.imap_host, tcp).await?;

        let client = async_imap::Client::new(tls_stream);

        let mut session = client
            .login(&self.imap_user, &self.imap_pass)
            .await
            .map_err(|e| anyhow::anyhow!("IMAP login failed: {:?}", e.0))?;

        session.select("INBOX").await?;

        let uids = session.uid_search("UNSEEN").await?;

        let mut results = Vec::new();

        for uid in uids.iter().take(50) {
            let fetch_stream = session.uid_fetch(uid.to_string(), "RFC822").await?;

            let messages: Vec<_> = fetch_stream.collect::<Vec<_>>().await;

            for msg in messages {
                let msg = msg.map_err(|e| anyhow::anyhow!("fetch error: {e}"))?;

                if let Some(body) = msg.body() {
                    let parsed =
                        parse_mail(body).map_err(|e| anyhow::anyhow!("parse error: {e}"))?;

                    let subject = parsed
                        .headers
                        .iter()
                        .find(|h| h.get_key() == "Subject")
                        .map(|h| h.get_value())
                        .unwrap_or_default();

                    let body_text = parsed.get_body().unwrap_or_default();
                    let combined = format!("Subject: {subject}\n\n{body_text}");

                    match self.classify_reply(llm, &combined).await {
                        Ok(class) => {
                            results.push((combined, class));
                        }
                        Err(e) => {
                            warn!(error = %e, "failed to classify reply");
                        }
                    }
                }
            }

            let _ = session.uid_store(uid.to_string(), "+FLAGS (\\Seen)").await;
        }

        session.logout().await?;
        info!(count = results.len(), "classified replies");
        Ok(results)
    }

    async fn classify_reply(
        &self,
        llm: &LlmClient,
        email_content: &str,
    ) -> Result<ReplyClassification> {
        let truncated: String = email_content.chars().take(3000).collect();
        let result: ReplyClassification = llm.complete_json(CLASSIFY_SYSTEM, &truncated).await?;
        Ok(result)
    }
}
