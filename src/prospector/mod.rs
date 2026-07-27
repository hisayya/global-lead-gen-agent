pub mod github;
pub mod search;
pub mod website_scraper;

use anyhow::Result;
use async_trait::async_trait;

use crate::models::Lead;

#[async_trait]
pub trait Prospector: Send + Sync {
    async fn discover(&self, queries: &[String], limit_per_query: usize) -> Result<Vec<Lead>>;
}
