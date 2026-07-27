#![forbid(unsafe_code)]
#![deny(warnings)]

use std::fs;
use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rand::Rng;
use tracing::{error, info, warn};

mod analyzer;
mod config;
mod diagnostician;
mod llm;
mod models;
mod prospector;
mod scorer;
mod sender;
mod store;
mod strategist;
mod writer;

use config::AppConfig;
use prospector::search::DuckDuckGoProspector;
use prospector::website_scraper::WebsiteScraper;

#[derive(Parser)]
#[command(name = "global-dev-radar")]
#[command(about = "Global Dev Radar - scan businesses for dev opportunities")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    Search {
        #[arg(short, long)]
        queries_file: Option<String>,
    },
    Diagnose {
        #[arg(short, long, default_value = "10")]
        limit: i32,
    },
    Send {
        #[arg(short, long, default_value = "5")]
        limit: i32,
    },
    CheckReplies,
    Run {
        #[arg(short, long)]
        queries_file: Option<String>,
    },
    DryRun {
        #[arg(short, long)]
        queries_file: Option<String>,
    },
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cli = Cli::parse();

    let runtime = match tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
    {
        Ok(rt) => rt,
        Err(e) => {
            error!(error = %e, "failed to create tokio runtime");
            std::process::exit(1);
        }
    };

    if let Err(e) = runtime.block_on(async { run(cli).await }) {
        error!(error = %e, "fatal error");
        std::process::exit(1);
    }
}

async fn run(cli: Cli) -> Result<()> {
    dotenvy::dotenv().ok();
    let cfg = AppConfig::from_env()?;
    let cfg = Arc::new(cfg);
    let store = store::Store::open("data/radar.db")?;

    match cli.command.unwrap_or(Commands::Run { queries_file: None }) {
        Commands::Search { queries_file } => {
            let queries = load_queries(queries_file)?;
            do_search(&cfg, &store, &queries).await?;
        }
        Commands::Diagnose { limit } => {
            do_diagnose(&cfg, &store, limit).await?;
        }
        Commands::Send { limit } => {
            do_send(&cfg, &store, limit).await?;
        }
        Commands::CheckReplies => {
            do_check_replies(&cfg).await?;
        }
        Commands::Run { queries_file } => {
            let queries = load_queries(queries_file)?;
            do_search(&cfg, &store, &queries).await?;
            do_diagnose(&cfg, &store, 20).await?;
            do_send(&cfg, &store, 5).await?;
            do_check_replies(&cfg).await?;
        }
        Commands::DryRun { queries_file } => {
            let queries = load_queries(queries_file)?;
            do_search(&cfg, &store, &queries).await?;
            do_diagnose(&cfg, &store, 5).await?;
            info!("dry run: skipping send phase");
            let leads = store.fetch_qualified_uncontacted(5)?;
            let llm = llm::LlmClient::new(&cfg)?;
            for lead in leads {
                let email = writer::Writer::generate_email(&llm, &lead).await?;
                info!(company = %lead.company, "DRY RUN email:");
                info!("  Subject: {}", email.subject);
                info!("  Body:\n{}", email.body);
                println!("---");
                println!("To: {} ({})", lead.company, lead.email);
                println!("Subject: {}", email.subject);
                println!();
                println!("{}", email.body);
                println!("---");
            }
        }
    }

    Ok(())
}

fn load_queries(queries_file: Option<String>) -> Result<Vec<String>> {
    let path =
        queries_file.unwrap_or_else(|| "docs/superpowers/specs/100-search-queries.md".to_string());

    let content = fs::read_to_string(&path)
        .with_context(|| format!("failed to read queries file: {path}"))?;
    let queries: Vec<String> = content
        .lines()
        .filter(|l| l.starts_with(|c: char| c.is_ascii_digit()))
        .map(|l| {
            l.find(". ").map_or_else(
                || l.trim().to_string(),
                |idx| l.get(idx + 2..).unwrap_or_default().trim().to_string(),
            )
        })
        .filter(|q| !q.is_empty())
        .collect();

    info!(count = queries.len(), "loaded search queries");
    Ok(queries)
}

async fn do_search(cfg: &Arc<AppConfig>, store: &store::Store, queries: &[String]) -> Result<()> {
    info!("phase: searching for leads");

    let ddg = DuckDuckGoProspector::new(
        cfg.user_agent.clone(),
        cfg.request_delay_min_sec,
        cfg.request_delay_max_sec,
    )?;

    let chunk_size = 10;
    let mut total_new = 0;

    for chunk in queries.chunks(chunk_size) {
        let leads = ddg.discover_leads(chunk).await.unwrap_or_default();

        for lead in leads {
            if lead.email.is_empty() && lead.website.is_empty() {
                continue;
            }
            if store
                .lead_exists(&lead.email, &lead.company)
                .unwrap_or(false)
            {
                continue;
            }
            if store.insert_lead(&lead).unwrap_or(false) {
                total_new += 1;
            }
        }

        info!(processed = chunk.len(), total_new, "batch done");
    }

    store.log("info", &format!("search complete: {total_new} new leads"))?;
    info!(total_new, "search phase complete");
    Ok(())
}

async fn do_diagnose(cfg: &Arc<AppConfig>, store: &store::Store, limit: i32) -> Result<()> {
    info!(limit, "phase: diagnosing leads");

    let llm = llm::LlmClient::new(cfg)?;
    let scraper_client = WebsiteScraper::new(cfg.user_agent.clone())?;

    let leads = store.fetch_unenriched_leads(limit)?;

    info!(count = leads.len(), "leads to diagnose");

    for lead in leads {
        let Some(id) = lead.id else { continue };

        if !lead.company_pages.is_empty() {
            if let Err(e) = diagnose_one(&llm, store, &lead, id).await {
                warn!(error = %e, company = %lead.company, "diagnosis failed");
            }
        } else if !lead.website.is_empty() {
            match scraper_client.fetch_pages(&lead.website).await {
                Ok(content) => {
                    let mut updated = lead.clone();
                    updated.company_pages = content.pages;
                    if let Err(e) = diagnose_one(&llm, store, &updated, id).await {
                        warn!(error = %e, company = %lead.company, "diagnosis failed");
                    }
                }
                Err(e) => {
                    warn!(error = %e, website = %lead.website, "failed to scrape website");
                }
            }
        }

        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }

    store.log("info", "diagnose phase complete")?;
    info!("diagnose phase complete");
    Ok(())
}

async fn diagnose_one(
    llm: &llm::LlmClient,
    store: &store::Store,
    lead: &models::Lead,
    id: i64,
) -> Result<()> {
    let diagnosis = diagnostician::Diagnostician::diagnose(llm, lead).await?;

    store.update_lead_diagnosis(
        id,
        &serde_json::to_string(&diagnosis)?,
        &diagnosis.tech_stack.as_string(),
        diagnosis.digital_maturity,
        &diagnosis.pain_points.as_string(),
        &diagnosis.industry,
    )?;

    let mut scored = lead.clone();
    scored.industry = diagnosis.industry.clone();
    scored.tech_stack = diagnosis.tech_stack.as_string();
    scored.digital_maturity = Some(diagnosis.digital_maturity);
    scored.pain_points = diagnosis.pain_points.as_string();

    let score_result = scorer::Scorer::score(llm, &scored).await?;
    store.update_lead_score(
        id,
        score_result.score,
        score_result.qualified,
        &score_result.reason,
    )?;

    if score_result.qualified {
        let strategy = strategist::Strategist::generate_strategy(llm, &scored).await?;
        store.update_lead_strategy(id, &serde_json::to_string(&strategy)?)?;
        info!(company = %lead.company, score = score_result.score, "lead qualified + strategy generated");
    } else {
        info!(company = %lead.company, score = score_result.score, "lead not qualified");
    }

    Ok(())
}

async fn do_send(cfg: &Arc<AppConfig>, store: &store::Store, limit: i32) -> Result<()> {
    info!(limit, "phase: sending outreach");

    let sent_today = store.count_sent_today()?;
    let remaining = cfg.daily_send_limit.saturating_sub(sent_today as u32) as i32;
    let actual_limit = remaining.min(limit);

    if actual_limit <= 0 {
        info!(
            sent_today,
            limit = cfg.daily_send_limit,
            "daily limit reached"
        );
        return Ok(());
    }

    let leads = store.fetch_qualified_uncontacted(actual_limit)?;
    info!(count = leads.len(), "leads to contact");

    let llm = llm::LlmClient::new(cfg)?;
    let sender = sender::Sender::new(cfg)?;

    for lead in leads {
        let Some(id) = lead.id else { continue };

        let email = match writer::Writer::generate_email(&llm, &lead).await {
            Ok(e) => e,
            Err(e) => {
                warn!(error = %e, company = %lead.company, "failed to generate email");
                continue;
            }
        };

        match sender.send_email(&lead, &email).await {
            Ok(()) => {
                store.insert_outreach(&models::Outreach {
                    id: None,
                    lead_id: id,
                    channel: "email".to_string(),
                    sequence_step: 1,
                    subject: email.subject.clone(),
                    body: email.body,
                    sent_at: Some(chrono::Utc::now()),
                    reply: None,
                    reply_class: None,
                    booked_at: None,
                })?;
                store.mark_contacted(id)?;
            }
            Err(e) => {
                warn!(error = %e, "send failed for {}", lead.company);
            }
        }

        let delay = rand::rng().random_range(30..60u64);
        tokio::time::sleep(std::time::Duration::from_secs(delay)).await;
    }

    store.log("info", "send phase complete")?;
    info!("send phase complete");
    Ok(())
}

async fn do_check_replies(cfg: &Arc<AppConfig>) -> Result<()> {
    info!("phase: checking replies");

    let llm = llm::LlmClient::new(cfg)?;
    let analyzer = analyzer::Analyzer::new(cfg);

    match analyzer.fetch_and_classify(&llm).await {
        Ok(results) => {
            for (email, classification) in &results {
                info!(
                    class = %classification.class,
                    action = %classification.suggested_action,
                    "reply classified"
                );

                match classification.class.as_str() {
                    "interested" => {
                        warn!(
                            "HOT LEAD detected! Action: {}",
                            classification.suggested_action
                        );
                        println!("\n=== HOT LEAD ===\n{email}\n");
                    }
                    "objection" => {
                        info!("Objection received - draft rebuttal needed");
                    }
                    "reject" => {
                        info!("Recipient opted out - stopping contact");
                    }
                    "not_now" => {
                        info!("Not now - schedule 90-day follow-up");
                    }
                    "ooo" => {
                        info!("Out of office - retry later");
                    }
                    _ => {}
                }
            }
            info!(count = results.len(), "replies processed");
        }
        Err(e) => {
            error!(error = %e, "failed to check replies");
        }
    }

    Ok(())
}
