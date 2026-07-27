use anyhow::Result;
use lettre::message::header::ContentType;
use lettre::transport::smtp::authentication::Credentials;
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use tracing::{info, warn};

use crate::config::AppConfig;
use crate::models::{EmailDraft, Lead};

pub struct Sender {
    mailer: AsyncSmtpTransport<Tokio1Executor>,
    from_name: String,
    from_addr: String,
    physical_addr: String,
}

impl Sender {
    pub fn new(cfg: &AppConfig) -> Self {
        let creds = Credentials::new(cfg.smtp_user.clone(), cfg.smtp_pass.clone());

        let mailer = AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.smtp_host)
            .expect("failed to create SMTP transport")
            .port(cfg.smtp_port)
            .credentials(creds)
            .build();

        Self {
            mailer,
            from_name: cfg.smtp_from_name.clone(),
            from_addr: cfg.smtp_from_addr.clone(),
            physical_addr: cfg.sender_physical_addr.clone(),
        }
    }

    pub async fn send_email(&self, lead: &Lead, draft: &EmailDraft) -> Result<()> {
        if lead.email.is_empty() {
            warn!(company = %lead.company, "no email, skipping send");
            anyhow::bail!("lead has no email");
        }

        let from_header = format!("{} <{}>", self.from_name, self.from_addr);
        let to_header = format!("{} <{}>", lead.name, lead.email);

        let body_with_footer = format!(
            "{}\n\n--\n{}\n{}\nReply STOP to opt out",
            draft.body, self.from_name, self.physical_addr
        );

        let email = Message::builder()
            .from(from_header.parse()?)
            .to(to_header.parse()?)
            .subject(&draft.subject)
            .header(ContentType::TEXT_PLAIN)
            .body(body_with_footer)?;

        info!(to = %lead.email, subject = %draft.subject, "sending email");

        match self.mailer.send(email).await {
            Ok(_) => {
                info!(to = %lead.email, "email sent successfully");
                Ok(())
            }
            Err(e) => {
                warn!(error = %e, to = %lead.email, "failed to send email");
                Err(anyhow::anyhow!("SMTP error: {e}"))
            }
        }
    }
}
