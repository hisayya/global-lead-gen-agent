use anyhow::{Result, anyhow};
use async_openai::Client as OpenAIClient;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
};
use serde::Deserialize;
use std::time::Duration;
use tokio::sync::Semaphore;

use crate::config::AppConfig;

pub struct LlmClient {
    client: OpenAIClient<OpenAIConfig>,
    model: String,
    semaphore: Semaphore,
}

impl LlmClient {
    pub fn new(cfg: &AppConfig) -> Result<Self> {
        let openai_config = OpenAIConfig::new()
            .with_api_base(&cfg.ark_base_url)
            .with_api_key(&cfg.ark_api_key);

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_mins(2))
            .build()
            .map_err(|e| anyhow!("failed to build reqwest client for LLM: {e}"))?;

        let client = OpenAIClient::with_config(openai_config).with_http_client(http_client);

        Ok(Self {
            client,
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

        let system_msg = ChatCompletionRequestMessage::System(ChatCompletionRequestSystemMessage {
            content: ChatCompletionRequestSystemMessageContent::Text(system.to_string()),
            name: None,
        });

        let user_msg = ChatCompletionRequestMessage::User(ChatCompletionRequestUserMessage {
            content: ChatCompletionRequestUserMessageContent::Text(user.to_string()),
            name: None,
        });

        let request = CreateChatCompletionRequestArgs::default()
            .model(&self.model)
            .messages(vec![system_msg, user_msg])
            .temperature(0.7_f32)
            .max_completion_tokens(4096u32)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .into_iter()
            .next()
            .and_then(|c| c.message.content)
            .unwrap_or_default();

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

        let parsed: T = serde_json::from_str(trimmed)?;
        Ok(parsed)
    }
}
