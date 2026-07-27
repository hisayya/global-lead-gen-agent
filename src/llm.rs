use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::config::AppConfig;

pub struct LlmClient {
    http: Client,
    base_url: String,
    api_key: String,
    model: String,
    semaphore: Semaphore,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: Vec<ChatMessage<'a>>,
    max_tokens: u32,
    temperature: f32,
    thinking: ThinkingConfig,
    reasoning_effort: &'a str,
}

#[derive(Serialize)]
struct ThinkingConfig {
    #[serde(rename = "type")]
    thinking_type: &'static str,
}

#[derive(Serialize)]
struct ChatMessage<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatResponseMessage,
}

#[derive(Deserialize)]
struct ChatResponseMessage {
    content: Option<String>,
}

impl LlmClient {
    pub fn new(cfg: &AppConfig) -> Result<Self> {
        let http = Client::builder()
            .timeout(Duration::from_mins(3))
            .build()
            .map_err(|e| anyhow!("failed to build reqwest client for LLM: {e}"))?;

        Ok(Self {
            http,
            base_url: cfg.ark_base_url.clone(),
            api_key: cfg.ark_api_key.clone(),
            model: cfg.ark_model.clone(),
            semaphore: Semaphore::new(5),
        })
    }

    pub async fn complete(&self, system: &str, user: &str) -> Result<String> {
        let _permit = self
            .semaphore
            .acquire()
            .await
            .map_err(|_| anyhow!("semaphore closed"))?;

        let req = ChatRequest {
            model: &self.model,
            messages: vec![
                ChatMessage {
                    role: "system",
                    content: system,
                },
                ChatMessage {
                    role: "user",
                    content: user,
                },
            ],
            max_tokens: 262144,
            temperature: 0.1,
            thinking: ThinkingConfig {
                thinking_type: "enabled",
            },
            reasoning_effort: "high",
        };

        let url = format!("{}/chat/completions", self.base_url);

        let resp = self
            .http
            .post(&url)
            .header("Authorization", format!("Bearer {}", self.api_key))
            .json(&req)
            .send()
            .await
            .map_err(|e| anyhow!("LLM request failed: {e}"))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(anyhow!("LLM API error {status}: {body}"));
        }

        let chat_resp: ChatResponse = resp
            .json()
            .await
            .map_err(|e| anyhow!("failed to parse LLM response: {e}"))?;

        let content = chat_resp
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .ok_or_else(|| anyhow!("LLM returned empty response"))?;

        Ok(content)
    }

    pub async fn complete_json<T: for<'de> Deserialize<'de>>(
        &self,
        system: &str,
        user: &str,
    ) -> Result<T> {
        let raw = self.complete(system, user).await?;
        let trimmed = raw
            .trim()
            .strip_prefix("```json")
            .or_else(|| raw.trim().strip_prefix("```"))
            .unwrap_or(&raw)
            .trim()
            .trim_end_matches("```")
            .trim();

        let parsed: T = serde_json::from_str(trimmed)
            .map_err(|e| anyhow!("failed to parse JSON from LLM: {e}\nraw: {trimmed}"))?;
        Ok(parsed)
    }
}
