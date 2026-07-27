use anyhow::Result;
use tracing::info;

use crate::llm::LlmClient;
use crate::models::{DiagnosisResult, Lead};

const DIAGNOSIS_SYSTEM: &str = r"You are a website diagnostic expert. Analyze the provided website content and output a JSON object with these fields:
- industry: the business industry
- business_model: how they make money
- estimated_size: company size (small/medium/large/enterprise)
- tech_stack: detected technologies (WordPress/Shopify/React/custom/etc.)
- digital_maturity: 1-5 score (1=very outdated, 5=modern)
- pain_points: specific technical problems you can see (old UI, no mobile, no customer portal, manual processes, etc.)

Output ONLY valid JSON, no markdown.";

pub struct Diagnostician;

impl Diagnostician {
    pub async fn diagnose(llm: &LlmClient, lead: &Lead) -> Result<DiagnosisResult> {
        let user_prompt = format!(
            "Website: {}\nCompany: {}\n\nWebsite content:\n{}",
            lead.website,
            lead.company,
            truncate(&lead.company_pages, 15000)
        );

        info!(company = %lead.company, "diagnosing website");

        let result: DiagnosisResult = llm.complete_json(DIAGNOSIS_SYSTEM, &user_prompt).await?;

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
