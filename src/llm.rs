use anyhow::Result;
use async_openai::config::OpenAIConfig;
use async_openai::types::chat::{
    ChatCompletionRequestMessage, ChatCompletionRequestSystemMessage,
    ChatCompletionRequestSystemMessageContent, ChatCompletionRequestUserMessage,
    ChatCompletionRequestUserMessageContent, CreateChatCompletionRequestArgs,
};
use async_openai::Client as OpenAIClient;
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
    pub fn new(cfg: &AppConfig) -> Self {
        let openai_config = OpenAIConfig::new()
            .with_api_base(&cfg.ark_base_url)
            .with_api_key(&cfg.ark_api_key);

        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("failed to build reqwest client for LLM");

        let client = OpenAIClient::with_config(openai_config).with_http_client(http_client);

        Self {
            client,
            model: cfg.ark_model.clone(),
            semaphore: Semaphore::new(5),
        }
    }

    pub async fn complete(&self, system: &str, user: &str) -> Result<String> {
        let _permit = self.semaphore.acquire().await.expect("semaphore closed");

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
            .temperature(0.7)
            .max_completion_tokens(4096u32)
            .build()?;

        let response = self.client.chat().create(request).await?;

        let content = response
            .choices
            .first()
            .and_then(|c| c.message.content.clone())
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
