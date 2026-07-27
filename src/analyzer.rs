use anyhow::Result;
use async_native_tls::TlsConnector;
use futures::StreamExt;
use mailparse::parse_mail;
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::llm::LlmClient;
use crate::models::ReplyClassification;

const CLASSIFY_SYSTEM: &str = concat!(
    "Classify this email reply. Output JSON:\n",
    "{\n",
    "  \"class\": \"<one of: interested, not_now, objection, reject, ooo>\",\n",
    "  \"suggested_action\": \"<what to do next>\"\n",
    "}\n\n",
    "- interested: wants to talk, book a call, learn more\n",
    "- not_now: maybe later, not right now\n",
    "- objection: pricing concern, need more info, specific question\n",
    "- reject: not interested, stop contacting\n",
    "- ooo: out of office / auto-reply\n\n",
    "Output ONLY valid JSON."
);

pub struct Analyzer {
    host: String,
    port: u16,
    user: String,
    pass: String,
}

impl Analyzer {
    pub fn new(cfg: &AppConfig) -> Self {
        Self {
            host: cfg.imap_host.clone(),
            port: cfg.imap_port,
            user: cfg.imap_user.clone(),
            pass: cfg.imap_pass.clone(),
        }
    }

    pub async fn fetch_and_classify(
        &self,
        llm: &LlmClient,
    ) -> Result<Vec<(String, ReplyClassification)>> {
        info!("connecting to IMAP");

        let tls = TlsConnector::new();
        let tcp = tokio::net::TcpStream::connect((self.host.as_str(), self.port)).await?;
        let tls_stream = tls.connect(&self.host, tcp).await?;

        let client = async_imap::Client::new(tls_stream);

        let mut session = client
            .login(&self.user, &self.pass)
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
                        .map(mailparse::MailHeader::get_value)
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
