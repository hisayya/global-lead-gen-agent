use anyhow::Result;
use tracing::info;

use crate::llm::LlmClient;
use crate::models::{Lead, StrategyResult};

const STRATEGY_SYSTEM: &str = r#"You are a technical sales strategist. Based on the company diagnosis, generate a personalized development proposal.

Output a JSON object:
{
  "problem_analysis": "<specific problems observed on their website/business>",
  "improvement_suggestion": "<concrete technical solution>",
  "dev_opportunity": "<what service you can sell them>",
  "estimated_price_range": "<rough USD range, e.g. $3,000-$8,000>"
}

Be specific to their actual problems. Do not be generic. Output ONLY valid JSON."#;

pub struct Strategist;

impl Strategist {
    pub async fn generate_strategy(llm: &LlmClient, lead: &Lead) -> Result<StrategyResult> {
        let user_prompt = format!(
            "Company: {}\nWebsite: {}\nIndustry: {}\nTech stack: {}\nDigital maturity: {}/5\nPain points: {}\n\nDiagnosis:\n{}",
            lead.company,
            lead.website,
            lead.industry,
            lead.tech_stack,
            lead.digital_maturity.unwrap_or(0),
            lead.pain_points,
            lead.diagnosis
        );

        info!(company = %lead.company, "generating strategy");

        let result: StrategyResult = llm.complete_json(STRATEGY_SYSTEM, &user_prompt).await?;
        Ok(result)
    }
}
