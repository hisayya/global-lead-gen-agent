use anyhow::Result;
use reqwest::Client;
use serde::Deserialize;
use tracing::{debug, warn};

use crate::models::{Lead, LeadStatus};

const GITHUB_API: &str = "https://api.github.com/search/issues";

#[derive(Debug, Deserialize)]
struct GhSearchResponse {
    items: Vec<GhItem>,
}

#[derive(Debug, Deserialize)]
struct GhItem {
    title: String,
    html_url: String,
    body: Option<String>,
    repository_url: String,
}

pub struct GitHubProspector {
    client: Client,
}

impl GitHubProspector {
    pub fn new(user_agent: String) -> Self {
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .expect("failed to build reqwest client");
        Self { client }
    }

    pub async fn search_help_wanted(&self, queries: &[String]) -> Result<Vec<Lead>> {
        let mut leads = Vec::new();

        for query in queries {
            let q = if query.contains("site:github.com") {
                query.replace("site:github.com ", "")
            } else {
                query.to_string()
            };

            debug!(query = q.as_str(), "searching GitHub");

            let resp = self
                .client
                .get(GITHUB_API)
                .query(&[("q", &q), ("per_page", &"5".to_string())])
                .header("Accept", "application/vnd.github.v3+json")
                .send()
                .await;

            match resp {
                Ok(r) => {
                    if !r.status().is_success() {
                        warn!(status = %r.status(), "GitHub API error");
                        continue;
                    }
                    let parsed: GhSearchResponse = match r.json().await {
                        Ok(j) => j,
                        Err(e) => {
                            warn!(error = %e, "failed to parse GitHub response");
                            continue;
                        }
                    };

                    for item in parsed.items {
                        let repo = item
                            .repository_url
                            .rsplit('/')
                            .next()
                            .unwrap_or("unknown")
                            .to_string();

                        leads.push(Lead {
                            id: None,
                            source: "github".to_string(),
                            name: String::new(),
                            company: repo,
                            email: String::new(),
                            role: String::new(),
                            website: item.html_url,
                            country: String::new(),
                            industry: String::new(),
                            company_pages: format!(
                                "{}\n{}",
                                item.title,
                                item.body.unwrap_or_default().chars().take(10000).collect::<String>()
                            ),
                            diagnosis: String::new(),
                            tech_stack: String::new(),
                            digital_maturity: None,
                            pain_points: String::new(),
                            score: None,
                            qualified: false,
                            strategy: String::new(),
                            status: LeadStatus::New,
                            created_at: None,
                            contacted_at: None,
                        });
                    }
                }
                Err(e) => {
                    warn!(error = %e, "GitHub request failed");
                }
            }

            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
        }

        Ok(leads)
    }
}
