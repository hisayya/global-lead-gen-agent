use anyhow::Result;
use chrono::Utc;
use rusqlite::Connection;

use crate::models::{Lead, LeadStatus, Outreach};

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: &str) -> Result<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch(
            "PRAGMA journal_mode=WAL;
             PRAGMA busy_timeout=5000;",
        )?;
        let store = Self { conn };
        store.init_schema()?;
        Ok(store)
    }

    fn init_schema(&self) -> Result<()> {
        self.conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS leads (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                source TEXT NOT NULL DEFAULT '',
                name TEXT NOT NULL DEFAULT '',
                company TEXT NOT NULL DEFAULT '',
                email TEXT NOT NULL DEFAULT '',
                role TEXT NOT NULL DEFAULT '',
                website TEXT NOT NULL DEFAULT '',
                country TEXT NOT NULL DEFAULT '',
                industry TEXT NOT NULL DEFAULT '',
                company_pages TEXT NOT NULL DEFAULT '',
                diagnosis TEXT NOT NULL DEFAULT '',
                tech_stack TEXT NOT NULL DEFAULT '',
                digital_maturity INTEGER,
                pain_points TEXT NOT NULL DEFAULT '',
                score INTEGER,
                qualified INTEGER NOT NULL DEFAULT 0,
                strategy TEXT NOT NULL DEFAULT '',
                status TEXT NOT NULL DEFAULT 'new',
                created_at TEXT NOT NULL DEFAULT '',
                contacted_at TEXT
            );

            CREATE UNIQUE INDEX IF NOT EXISTS idx_leads_email_company ON leads(email, company);

            CREATE TABLE IF NOT EXISTS outreach (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                lead_id INTEGER NOT NULL,
                channel TEXT NOT NULL DEFAULT 'email',
                sequence_step INTEGER NOT NULL DEFAULT 0,
                subject TEXT NOT NULL DEFAULT '',
                body TEXT NOT NULL DEFAULT '',
                sent_at TEXT,
                reply TEXT,
                reply_class TEXT,
                booked_at TEXT,
                FOREIGN KEY (lead_id) REFERENCES leads(id)
            );

            CREATE TABLE IF NOT EXISTS market_config (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                keywords TEXT NOT NULL DEFAULT '',
                region TEXT NOT NULL DEFAULT '',
                icp_criteria TEXT NOT NULL DEFAULT '',
                active INTEGER NOT NULL DEFAULT 1
            );

            CREATE TABLE IF NOT EXISTS logs (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                ts TEXT NOT NULL DEFAULT '',
                level TEXT NOT NULL DEFAULT 'info',
                msg TEXT NOT NULL DEFAULT ''
            );
            ",
        )?;
        Ok(())
    }

    pub fn insert_lead(&self, lead: &Lead) -> Result<bool> {
        let now = Utc::now().to_rfc3339();
        let result = self.conn.execute(
            "INSERT OR IGNORE INTO leads
                (source, name, company, email, role, website, country, industry,
                 company_pages, diagnosis, tech_stack, digital_maturity, pain_points,
                 score, qualified, strategy, status, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17, ?18)",
            rusqlite::params![
                lead.source,
                lead.name,
                lead.company,
                lead.email,
                lead.role,
                lead.website,
                lead.country,
                lead.industry,
                lead.company_pages,
                lead.diagnosis,
                lead.tech_stack,
                lead.digital_maturity,
                lead.pain_points,
                lead.score,
                lead.qualified as i32,
                lead.strategy,
                lead.status.as_str(),
                now,
            ],
        )?;
        Ok(result > 0)
    }

    pub fn lead_exists(&self, email: &str, company: &str) -> Result<bool> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM leads WHERE email = ?1 AND company = ?2",
            rusqlite::params![email, company],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    pub fn update_lead_diagnosis(
        &self,
        id: i64,
        diagnosis: &str,
        tech_stack: &str,
        digital_maturity: i32,
        pain_points: &str,
        industry: &str,
    ) -> Result<()> {
        self.conn.execute(
            "UPDATE leads SET diagnosis = ?1, tech_stack = ?2, digital_maturity = ?3,
             pain_points = ?4, industry = ?5, status = 'enriched' WHERE id = ?6",
            rusqlite::params![diagnosis, tech_stack, digital_maturity, pain_points, industry, id],
        )?;
        Ok(())
    }

    pub fn update_lead_score(&self, id: i64, score: i32, qualified: bool, reason: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE leads SET score = ?1, qualified = ?2 WHERE id = ?3",
            rusqlite::params![score, qualified as i32, id],
        )?;
        if !reason.is_empty() {
            let existing: String = self.conn.query_row(
                "SELECT strategy FROM leads WHERE id = ?1",
                rusqlite::params![id],
                |row| row.get(0),
            ).unwrap_or_default();
            let combined = if existing.is_empty() {
                format!("Score reason: {reason}")
            } else {
                format!("{existing}\nScore reason: {reason}")
            };
            self.conn.execute(
                "UPDATE leads SET strategy = ?1 WHERE id = ?2",
                rusqlite::params![combined, id],
            )?;
        }
        Ok(())
    }

    pub fn update_lead_strategy(&self, id: i64, strategy: &str) -> Result<()> {
        self.conn.execute(
            "UPDATE leads SET strategy = ?1 WHERE id = ?2",
            rusqlite::params![strategy, id],
        )?;
        Ok(())
    }

    pub fn mark_contacted(&self, id: i64) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "UPDATE leads SET status = 'contacted', contacted_at = ?1 WHERE id = ?2",
            rusqlite::params![now, id],
        )?;
        Ok(())
    }

    pub fn update_lead_status(&self, id: i64, status: LeadStatus) -> Result<()> {
        self.conn.execute(
            "UPDATE leads SET status = ?1 WHERE id = ?2",
            rusqlite::params![status.as_str(), id],
        )?;
        Ok(())
    }

    pub fn fetch_unenriched_leads(&self, limit: i32) -> Result<Vec<Lead>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, name, company, email, role, website, country, industry,
                    company_pages, diagnosis, tech_stack, digital_maturity, pain_points,
                    score, qualified, strategy, status, created_at, contacted_at
             FROM leads WHERE status = 'new' LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit], |row| {
            let status_str: String = row.get(17)?;
            let status = LeadStatus::from_str(&status_str).unwrap_or(LeadStatus::New);
            Ok(Lead {
                id: Some(row.get(0)?),
                source: row.get(1)?,
                name: row.get(2)?,
                company: row.get(3)?,
                email: row.get(4)?,
                role: row.get(5)?,
                website: row.get(6)?,
                country: row.get(7)?,
                industry: row.get(8)?,
                company_pages: row.get(9)?,
                diagnosis: row.get(10)?,
                tech_stack: row.get(11)?,
                digital_maturity: row.get(12)?,
                pain_points: row.get(13)?,
                score: row.get(14)?,
                qualified: row.get::<_, i32>(15)? != 0,
                strategy: row.get(16)?,
                status,
                created_at: parse_dt(row.get(18)?),
                contacted_at: parse_dt_opt(row.get(19)?),
            })
        })?;
        let mut leads = Vec::new();
        for row in rows {
            leads.push(row?);
        }
        Ok(leads)
    }

    pub fn fetch_qualified_uncontacted(&self, limit: i32) -> Result<Vec<Lead>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, source, name, company, email, role, website, country, industry,
                    company_pages, diagnosis, tech_stack, digital_maturity, pain_points,
                    score, qualified, strategy, status, created_at, contacted_at
             FROM leads WHERE qualified = 1 AND status = 'enriched' LIMIT ?1",
        )?;
        let rows = stmt.query_map(rusqlite::params![limit], |row| {
            let status_str: String = row.get(17)?;
            let status = LeadStatus::from_str(&status_str).unwrap_or(LeadStatus::Enriched);
            Ok(Lead {
                id: Some(row.get(0)?),
                source: row.get(1)?,
                name: row.get(2)?,
                company: row.get(3)?,
                email: row.get(4)?,
                role: row.get(5)?,
                website: row.get(6)?,
                country: row.get(7)?,
                industry: row.get(8)?,
                company_pages: row.get(9)?,
                diagnosis: row.get(10)?,
                tech_stack: row.get(11)?,
                digital_maturity: row.get(12)?,
                pain_points: row.get(13)?,
                score: row.get(14)?,
                qualified: row.get::<_, i32>(15)? != 0,
                strategy: row.get(16)?,
                status,
                created_at: parse_dt(row.get(18)?),
                contacted_at: parse_dt_opt(row.get(19)?),
            })
        })?;
        let mut leads = Vec::new();
        for row in rows {
            leads.push(row?);
        }
        Ok(leads)
    }

    pub fn insert_outreach(&self, o: &Outreach) -> Result<i64> {
        let now = o.sent_at.map(|t| t.to_rfc3339());
        self.conn.execute(
            "INSERT INTO outreach (lead_id, channel, sequence_step, subject, body, sent_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![o.lead_id, o.channel, o.sequence_step, o.subject, o.body, now],
        )?;
        Ok(self.conn.last_insert_rowid())
    }

    pub fn count_sent_today(&self) -> Result<i32> {
        let today_prefix = Utc::now().format("%Y-%m-%d").to_string();
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM outreach WHERE sent_at LIKE ?1",
            rusqlite::params![format!("{today_prefix}%")],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn count_sequence_for_lead(&self, lead_id: i64) -> Result<i32> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM outreach WHERE lead_id = ?1",
            rusqlite::params![lead_id],
            |row| row.get(0),
        )?;
        Ok(count)
    }

    pub fn log(&self, level: &str, msg: &str) -> Result<()> {
        let now = Utc::now().to_rfc3339();
        self.conn.execute(
            "INSERT INTO logs (ts, level, msg) VALUES (?1, ?2, ?3)",
            rusqlite::params![now, level, msg],
        )?;
        Ok(())
    }
}

fn parse_dt(s: String) -> Option<chrono::DateTime<chrono::Utc>> {
    chrono::DateTime::parse_from_rfc3339(&s)
        .ok()
        .map(|t| t.with_timezone(&chrono::Utc))
}

fn parse_dt_opt(s: Option<String>) -> Option<chrono::DateTime<chrono::Utc>> {
    s.and_then(|v| parse_dt(v))
}
