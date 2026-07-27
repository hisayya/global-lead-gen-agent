use anyhow::Result;
use tracing::info;

use crate::llm::LlmClient;
use crate::models::{Lead, ScoreResult};

const SCORING_SYSTEM: &str = r#"You are a lead scoring expert for a freelance software developer. Score this company as a potential client.

Scoring signals (each adds points, max 100):
- Outdated tech stack (WordPress 2018, old CMS): +20
- Currently hiring IT/developers: +20
- Business growth signs (expanding, hiring): +15
- Has IT team but website is outdated: +10
- Target market match (US/Germany/Australia/Canada/UK/Europe): +10
- No online booking/customer portal/ticketing: +10
- Has contact form but no clear email: +5
- Manual or paper-based processes visible: +10

Output a JSON object:
{
  "score": <0-100 integer>,
  "qualified": <true if score >= 50>,
  "reason": "<one sentence explanation>"
}

Output ONLY valid JSON, no markdown."#;

pub struct Scorer;

impl Scorer {
    pub async fn score(llm: &LlmClient, lead: &Lead) -> Result<ScoreResult> {
        let user_prompt = format!(
            "Company: {}\nWebsite: {}\nIndustry: {}\nTech stack: {}\nDigital maturity: {}/5\nPain points: {}\nWebsite content excerpt:\n{}",
            lead.company,
            lead.website,
            lead.industry,
            lead.tech_stack,
            lead.digital_maturity.unwrap_or(0),
            lead.pain_points,
            truncate(&lead.company_pages, 5000)
        );

        info!(company = %lead.company, "scoring lead");

        let result: ScoreResult = llm.complete_json(SCORING_SYSTEM, &user_prompt).await?;
        Ok(result)
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() > max {
        s.chars().take(max).collect()
    } else {
        s.to_string()
    }
}
