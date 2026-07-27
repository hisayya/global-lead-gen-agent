use anyhow::{Result, anyhow};
use rand::Rng;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::{debug, warn};

use crate::models::{Lead, LeadStatus};

pub struct DuckDuckGoProspector {
    client: Client,
    delay_min: u64,
    delay_max: u64,
}

impl DuckDuckGoProspector {
    pub fn new(user_agent: String, delay_min: u64, delay_max: u64) -> Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(30))
            .build()
            .map_err(|e| anyhow!("failed to build reqwest client: {e}"))?;
        Ok(Self {
            client,
            delay_min,
            delay_max,
        })
    }

    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<SearchResult>> {
        let url = format!(
            "https://html.duckduckgo.com/html/?q={}",
            urlencoding::encode(query)
        );

        debug!(query = query, "searching DuckDuckGo");

        let resp = self.client.get(&url).send().await?;

        if !resp.status().is_success() {
            warn!(status = %resp.status(), "DuckDuckGo returned non-success");
            return Ok(Vec::new());
        }

        let body = resp.text().await?;
        let document = Html::parse_document(&body);

        let result_selector = Selector::parse(".result__a")
            .map_err(|e| anyhow!("invalid selector for result__a: {e}"))?;

        let mut results = Vec::new();
        for element in document.select(&result_selector).take(limit) {
            let title = element.text().collect::<String>();
            let href = element.attr("href").unwrap_or_default();

            let link = parse_ddg_url(href);
            if link.is_empty() {
                continue;
            }

            results.push(SearchResult {
                title: title.trim().to_string(),
                url: link,
            });
        }

        Ok(results)
    }

    async fn random_delay(&self) {
        if self.delay_max > self.delay_min {
            let delay = rand::rng().random_range(self.delay_min..=self.delay_max);
            tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
        }
    }

    pub async fn discover_leads(&self, queries: &[String]) -> Result<Vec<Lead>> {
        let mut leads = Vec::new();
        let email_re = Regex::new(r"\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b")
            .map_err(|e| anyhow!("bad regex: {e}"))?;

        for query in queries {
            let results = self.search(query, 5).await.unwrap_or_default();
            self.random_delay().await;

            for result in results {
                let domain = extract_domain(&result.url);
                if domain.is_empty() {
                    continue;
                }

                let website = if result.url.starts_with("http") {
                    result.url.clone()
                } else {
                    format!("https://{domain}")
                };

                let page_content = match self.client.get(&website).send().await {
                    Ok(r) if r.status().is_success() => r.text().await.unwrap_or_default(),
                    _ => String::new(),
                };
                let emails = extract_emails(&page_content, &email_re);
                let email = emails.into_iter().next().unwrap_or_default();

                leads.push(Lead {
                    id: None,
                    source: "duckduckgo".to_string(),
                    name: String::new(),
                    company: domain.clone(),
                    email,
                    role: String::new(),
                    website,
                    country: String::new(),
                    industry: String::new(),
                    company_pages: page_content.chars().take(50000).collect(),
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

        Ok(leads)
    }
}

pub struct SearchResult {
    pub title: String,
    pub url: String,
}

fn parse_ddg_url(href: &str) -> String {
    if let Some(start) = href.find("uddg=") {
        let rest = href.get(start + 5..).unwrap_or_default();
        let end = rest.find('&').unwrap_or(rest.len());
        let encoded = rest.get(..end).unwrap_or_default();
        return urlencoding::decode(encoded)
            .map(std::borrow::Cow::into_owned)
            .unwrap_or_default();
    }
    if href.starts_with("http") {
        href.to_string()
    } else {
        String::new()
    }
}

fn extract_domain(url: &str) -> String {
    let no_proto = url.split_once("://").map_or(url, |(_, rest)| rest);
    let domain = no_proto.split('/').next().unwrap_or("");
    let domain = domain.split(':').next().unwrap_or(domain);
    if domain.contains('.') {
        domain.to_string()
    } else {
        String::new()
    }
}

fn extract_emails(text: &str, re: &Regex) -> Vec<String> {
    let mut seen = std::collections::HashSet::new();
    let mut result = Vec::new();
    for cap in re.captures_iter(text) {
        let email = cap
            .get(0)
            .map(|m| m.as_str().to_lowercase())
            .unwrap_or_default();
        if email.is_empty() {
            continue;
        }
        if email.contains("example.com")
            || email.contains("sentry")
            || email.contains("w3.org")
            || email.contains("schema")
        {
            continue;
        }
        if seen.insert(email.clone()) {
            result.push(email);
        }
    }
    result
}
