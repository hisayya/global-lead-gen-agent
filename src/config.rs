use anyhow::Result;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub ark_api_key: String,
    pub ark_base_url: String,
    pub ark_model: String,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub smtp_user: String,
    pub smtp_pass: String,
    pub smtp_from_name: String,
    pub smtp_from_addr: String,
    pub sender_physical_addr: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_user: String,
    pub imap_pass: String,
    pub daily_send_limit: u32,
    pub request_delay_min_sec: u64,
    pub request_delay_max_sec: u64,
    pub user_agent: String,
}

impl AppConfig {
    pub fn from_env() -> Result<Self> {
        Ok(Self {
            ark_api_key: env_str("ARK_API_KEY")?,
            ark_base_url: env_str("ARK_BASE_URL")?,
            ark_model: env_str("ARK_MODEL")?,
            smtp_host: env_str("SMTP_HOST")?,
            smtp_port: env_u16("SMTP_PORT")?,
            smtp_user: env_str("SMTP_USER")?,
            smtp_pass: env_str("SMTP_PASS")?,
            smtp_from_name: env_str("SMTP_FROM_NAME")?,
            smtp_from_addr: env_str("SMTP_FROM_ADDR")?,
            sender_physical_addr: env_str("SENDER_PHYSICAL_ADDR")?,
            imap_host: env_str("IMAP_HOST")?,
            imap_port: env_u16("IMAP_PORT")?,
            imap_user: env_str("IMAP_USER")?,
            imap_pass: env_str("IMAP_PASS")?,
            daily_send_limit: env_u32_or("DAILY_SEND_LIMIT", 25),
            request_delay_min_sec: env_u64_or("REQUEST_DELAY_MIN_SEC", 3),
            request_delay_max_sec: env_u64_or("REQUEST_DELAY_MAX_SEC", 5),
            user_agent: env_str_or(
                "USER_AGENT",
                "GlobalDevRadar/0.1 (contact: unknown)",
            ),
        })
    }
}

fn env_str(key: &str) -> Result<String> {
    std::env::var(key).map_err(|_| anyhow::anyhow!("missing env: {key}"))
}

fn env_str_or(key: &str, default: &str) -> String {
    std::env::var(key).unwrap_or_else(|_| default.to_string())
}

fn env_u16(key: &str) -> Result<u16> {
    std::env::var(key)?
        .parse()
        .map_err(|_| anyhow::anyhow!("invalid u16 for env: {key}"))
}

fn env_u32_or(key: &str, default: u32) -> u32 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u64_or(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}
