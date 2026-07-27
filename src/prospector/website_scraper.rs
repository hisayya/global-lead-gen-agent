use anyhow::{Result, anyhow};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use tracing::debug;

pub struct WebsiteScraper {
    client: Client,
}

impl WebsiteScraper {
    pub fn new(user_agent: String) -> Result<Self> {
        let client = Client::builder()
            .user_agent(user_agent)
            .timeout(std::time::Duration::from_secs(20))
            .build()
            .map_err(|e| anyhow!("failed to build reqwest client: {e}"))?;
        Ok(Self { client })
    }

    pub async fn fetch_pages(&self, base_url: &str) -> Result<WebsiteContent> {
        let base = normalize_base_url(base_url);

        let mut pages = Vec::new();

        let home = self.fetch_page(&base).await.unwrap_or_default();
        pages.push(PageContent {
            path: "/".to_string(),
            text: home,
        });

        let sub_paths = [
            "/about",
            "/about-us",
            "/team",
            "/services",
            "/contact",
            "/careers",
        ];
        for path in sub_paths {
            let url = format!("{base}{path}");
            if let Some(content) = self.fetch_page(&url).await {
                pages.push(PageContent {
                    path: path.to_string(),
                    text: content,
                });
            }
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }

        let combined = pages
            .iter()
            .map(|p| format!("--- {} ---\n{}", p.path, p.text))
            .collect::<Vec<_>>()
            .join("\n\n");

        Ok(WebsiteContent {
            base_url: base,
            pages: combined,
            raw_pages: pages,
        })
    }

    async fn fetch_page(&self, url: &str) -> Option<String> {
        debug!(url = url, "fetching page");

        let resp = self.client.get(url).send().await.ok()?;

        if !resp.status().is_success() {
            return None;
        }

        let html = resp.text().await.ok()?;
        let text = extract_text(&html);

        if text.trim().is_empty() {
            None
        } else {
            Some(text.chars().take(20000).collect())
        }
    }
}

pub struct WebsiteContent {
    pub base_url: String,
    pub pages: String,
    pub raw_pages: Vec<PageContent>,
}

pub struct PageContent {
    pub path: String,
    pub text: String,
}

fn normalize_base_url(url: &str) -> String {
    let url = if url.starts_with("http") {
        url.to_string()
    } else {
        format!("https://{url}")
    };
    url.trim_end_matches('/').to_string()
}

fn extract_text(html: &str) -> String {
    let document = Html::parse_document(html);

    let mut parts = Vec::new();

    if let Ok(title_sel) = Selector::parse("title")
        && let Some(title) = document.select(&title_sel).next()
    {
        let t: String = title.text().collect();
        if !t.trim().is_empty() {
            parts.push(format!("TITLE: {}", t.trim()));
        }
    }

    if let Ok(h_sel) = Selector::parse("h1, h2, h3") {
        for h in document.select(&h_sel) {
            let text: String = h.text().collect();
            let text = text.trim();
            if !text.is_empty() {
                parts.push(text.to_string());
            }
        }
    }

    if let Ok(p_sel) = Selector::parse("p, li, span, div") {
        for p in document.select(&p_sel) {
            let text: String = p.text().collect();
            let text = text.trim();
            if text.len() > 20 {
                parts.push(text.to_string());
            }
        }
    }

    if let Ok(a_sel) = Selector::parse("a") {
        for a in document.select(&a_sel) {
            if let Some(href) = a.attr("href")
                && href.contains("mailto:")
            {
                parts.push(format!("EMAIL: {}", href.replace("mailto:", "")));
            }
        }
    }

    if let Ok(email_re) = Regex::new(r"\b[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}\b") {
        for cap in email_re.captures_iter(html) {
            if let Some(m) = cap.get(0) {
                let email = m.as_str().to_lowercase();
                if !email.contains("example.com") && !email.contains("sentry") {
                    parts.push(format!("EMAIL: {email}"));
                }
            }
        }
    }

    parts.dedup();
    parts.join("\n")
}
