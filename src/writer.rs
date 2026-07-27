use anyhow::Result;
use tracing::info;

use crate::llm::LlmClient;
use crate::models::{EmailDraft, Lead};

const WRITER_SYSTEM: &str = r#"You are an expert cold email writer for a freelance software developer. Write a personalized cold email based on the company diagnosis.

Rules:
- Under 125 words
- Plain text, no HTML
- No links in the first email
- Reference a SPECIFIC problem you observed on their website
- One clear, low-friction CTA (e.g. "open to a 15-min chat?")
- Professional but not stiff
- Include a P.S. line with "Reply STOP to opt out" at the very end
- Sign off with a real name

Output a JSON object:
{
  "subject": "<3-5 words, slightly vague, relevant>",
  "body": "<the email body>"
}

Output ONLY valid JSON."#;

const QC_SYSTEM: &str = r#"You are an English quality checker. Review this cold email. If it has grammar issues, awkward phrasing, or sounds like a robot, rewrite it. If it's already good, return it as-is.

Output a JSON object:
{
  "subject": "<final subject>",
  "body": "<final body>"
}

Output ONLY valid JSON."#;

pub struct Writer;

impl Writer {
    pub async fn generate_email(llm: &LlmClient, lead: &Lead) -> Result<EmailDraft> {
        let user_prompt = format!(
            "Recipient: {} at {}\nTheir problem: {}\nYour solution: {}\nDev opportunity: {}\nEstimated value: {}\n\nDiagnosis:\n{}",
            if lead.name.is_empty() { "there" } else { &lead.name },
            lead.company,
            lead.pain_points,
            lead.strategy,
            if lead.industry.is_empty() { "custom development" } else { &lead.industry },
            "varies",
            lead.diagnosis
        );

        info!(company = %lead.company, "writing cold email");

        let draft: EmailDraft = llm.complete_json(WRITER_SYSTEM, &user_prompt).await?;

        let qc_prompt = format!(
            "Subject: {}\n\nBody:\n{}",
            draft.subject, draft.body
        );

        let final_draft: EmailDraft = llm.complete_json(QC_SYSTEM, &qc_prompt).await?;

        Ok(final_draft)
    }

    pub async fn generate_followup(
        llm: &LlmClient,
        lead: &Lead,
        step: i32,
        previous_body: &str,
    ) -> Result<EmailDraft> {
        let prompt = match step {
            2 => format!(
                "Write follow-up email #1 (Day 4) for this lead. Reply in same thread. 25-35 words. Bump the topic, add one new angle.\n\nPrevious email:\n{previous_body}\n\nCompany: {}\nProblem: {}",
                lead.company, lead.pain_points
            ),
            3 => format!(
                "Write follow-up email #2 (Day 10) for this lead. Share a different angle or mini case study. Under 50 words.\n\nCompany: {}\nProblem: {}",
                lead.company, lead.pain_points
            ),
            4 => format!(
                "Write the breakup email (Day 16). Tell them you'll stop reaching out. Leave a calendar link placeholder. Under 40 words.\n\nCompany: {}",
                lead.company
            ),
            _ => {
                return Ok(EmailDraft {
                    subject: String::new(),
                    body: String::new(),
                })
            }
        };

        info!(company = %lead.company, step, "writing follow-up");

        let system = "You are an expert cold email follow-up writer. Output JSON {\"subject\": \"...\", \"body\": \"...\"}. Output ONLY JSON.";
        let draft: EmailDraft = llm.complete_json(system, &prompt).await?;
        Ok(draft)
    }
}
