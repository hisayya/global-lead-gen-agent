# global-lead-gen-agent (Global Dev Radar)

> **A Rust agent that scans businesses worldwide for software-development opportunities.**

`Global Dev Radar` discovers, enriches, and scores companies that are likely to need software development services — then helps you engage them. Think of it as a **lead-generation pipeline for freelancers and dev shops**, written in strict, safe Rust.

---

## ✨ What it does

1. **Prospect discovery** — finds candidate companies from GitHub profiles and web search.
2. **Website enrichment** — scrapes company websites to build a richer picture (stack, size, activity).
3. **Scoring** — `scorer.rs` ranks prospects by how likely they are to need (and pay for) dev work.
4. **Analysis & diagnostics** — `analyzer.rs` + `diagnostician.rs` explain *why* each lead scored the way it did, so you don't waste outreach on duds.
5. **LLM-assisted** — optional LLM layer (`llm.rs`) for smarter enrichment and outreach drafting.

---

## 🧱 Architecture

```
┌───────────────┐   ┌───────────────┐   ┌───────────────┐
│  prospector   │   │  prospector   │   │  prospector   │
│  github.rs    │   │  search.rs    │   │  website_...  │
└──────┬────────┘   └──────┬────────┘   └──────┬────────┘
       └───────────────────┼───────────────────┘
                           ▼
                 ┌───────────────────┐
                 │      models.rs     │  lead data model
                 └─────────┬─────────┘
                           ▼
        ┌───────────────┐  ┌───────────────┐  ┌───────────────┐
        │   analyzer    │  │    scorer     │  │ diagnostician │
        └───────────────┘  └───────────────┘  └───────────────┘
                           ▼
        ┌───────────────┐  (optional)   ┌───────────────┐
        │     main.rs   │──────────────►│     llm.rs    │
        └───────────────┘               └───────────────┘
```

---

## 🚀 Getting started

```bash
# 1. Configure (copy the template, fill in your keys)
cp .env.example .env

# 2. Run
cargo run --release
```

### Environment variables (`.env`)

| Variable | Purpose |
|---|---|
| `ARK_API_KEY` | LLM provider key (Volcano ARK, OpenAI-compatible) |
| `ARK_MODEL` | Model endpoint id |
| `SMTP_HOST` / `SMTP_PORT` / `SMTP_USER` / `SMTP_PASS` | Outreach email sending |
| `IMAP_HOST` / `IMAP_PORT` / `IMAP_USER` / `IMAP_PASS` | Inbox monitoring |
| `DAILY_SEND_LIMIT` | Cap on outbound emails per day (default 25) |
| `REQUEST_DELAY_MIN_SEC` / `REQUEST_DELAY_MAX_SEC` | Polite request throttling |

**Note:** All values in `.env.example` are placeholders — never commit real keys.

---

## 🛠️ Tech stack

- **Rust** (2024 edition) — strict linting: `-D warnings` + `-D unsafe-code` + deny-by-default lints, no `unsafe` anywhere
- **tokio / async** — non-blocking IO for scraping & outreach
- **serde** — typed config & payloads

---

## 📂 Project layout

```
src/
  main.rs             # CLI entry
  config.rs           # env config
  models.rs           # lead data model
  prospector/
    mod.rs
    github.rs         # GitHub-based discovery
    search.rs         # web-search-based discovery
    website_scraper.rs# company website enrichment
  analyzer.rs         # per-prospect analysis
  scorer.rs           # lead scoring
  diagnostician.rs    # scoring explanations
  llm.rs              # optional LLM enrichment
(design specs available on request)
```

---

## ⚖️ Usage notes

- **Respect rate limits** — built-in request throttling; tune the delay env vars.
- **Outreach email** — use SMTP with a dedicated sender; keep `DAILY_SEND_LIMIT` low to stay polite and avoid spam flags.
- **Use with a real business** — leads are suggestions; always verify before contacting.

---

## 📄 License

MIT — free to use, modify, and reuse.