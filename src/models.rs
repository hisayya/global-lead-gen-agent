use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Lead {
    pub id: Option<i64>,
    pub source: String,
    pub name: String,
    pub company: String,
    pub email: String,
    pub role: String,
    pub website: String,
    pub country: String,
    pub industry: String,
    pub company_pages: String,
    pub diagnosis: String,
    pub tech_stack: String,
    pub digital_maturity: Option<i32>,
    pub pain_points: String,
    pub score: Option<i32>,
    pub qualified: bool,
    pub strategy: String,
    pub status: LeadStatus,
    pub created_at: Option<DateTime<Utc>>,
    pub contacted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LeadStatus {
    New,
    Enriched,
    Contacted,
    Replied,
    Meeting,
    Proposal,
    Won,
    Lost,
}

impl LeadStatus {
    pub fn as_str(&self) -> &str {
        match self {
            LeadStatus::New => "new",
            LeadStatus::Enriched => "enriched",
            LeadStatus::Contacted => "contacted",
            LeadStatus::Replied => "replied",
            LeadStatus::Meeting => "meeting",
            LeadStatus::Proposal => "proposal",
            LeadStatus::Won => "won",
            LeadStatus::Lost => "lost",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "new" => Some(LeadStatus::New),
            "enriched" => Some(LeadStatus::Enriched),
            "contacted" => Some(LeadStatus::Contacted),
            "replied" => Some(LeadStatus::Replied),
            "meeting" => Some(LeadStatus::Meeting),
            "proposal" => Some(LeadStatus::Proposal),
            "won" => Some(LeadStatus::Won),
            "lost" => Some(LeadStatus::Lost),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Outreach {
    pub id: Option<i64>,
    pub lead_id: i64,
    pub channel: String,
    pub sequence_step: i32,
    pub subject: String,
    pub body: String,
    pub sent_at: Option<DateTime<Utc>>,
    pub reply: Option<String>,
    pub reply_class: Option<String>,
    pub booked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiagnosisResult {
    pub industry: String,
    pub business_model: String,
    pub estimated_size: String,
    pub tech_stack: String,
    pub digital_maturity: i32,
    pub pain_points: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScoreResult {
    pub score: i32,
    pub qualified: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrategyResult {
    pub problem_analysis: String,
    pub improvement_suggestion: String,
    pub dev_opportunity: String,
    pub estimated_price_range: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailDraft {
    pub subject: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplyClassification {
    pub class: String,
    pub suggested_action: String,
}
