# 全球自动获客 Agent（Rust）— 设计文档

- 日期：2026-07-26
- 状态：待用户复核
- 作者：WorkBuddy（brainstorming 产出）

## 1. 项目概述

构建一个**全球自动获客 Agent**：以独立程序员身份，在全球范围自动寻找需要软件开发服务的客户（创业者/非技术创始人、海外中小企业、其他开发者/工作室转包、代理公司白标开发），并通过多通道触达将其转化为会议/成交。

设计原则：**在能满足需求的前提下取最简单方案，同时保留充足扩展性**（YAGNI + 整洁接口）。

## 2. 目标与约束

| 项 | 内容 |
|---|---|
| 提供的服务 | 承接各类代码开发（web/app/自动化/脚本等） |
| 目标客户 | 全球任何可能需要写代码的人/企业（见上四类） |
| 触达渠道 | 全渠道：冷邮件 + LinkedIn + 社区内容 + Google Maps 抓邮箱 |
| 唯一付费项 | AI API（doubao-seed，经火山方舟 OpenAI 兼容接口），其余零花费 |
| 运行位置 | 先本机 Mac 常驻验证，后上 Oracle Cloud Always Free 做 24/7 |
| MVP 成功标准 | 真实约到 1+ 会议/成交（非仅跑通链路） |

## 3. 技术栈（Rust，全免费/零 SaaS）

| 关注点 | crate | 用途 |
|---|---|---|
| LLM 调用 | `async-openai` | OpenAI 兼容接口连 doubao-seed（自定义 base_url） |
| 异步运行时 | `tokio` | 全盘异步 |
| HTTP 客户端 | `reqwest` | 抓取网页 / 调 API |
| HTML 解析 | `scraper` + `regex` | Google Maps / 网页提取邮箱与企业信息 |
| 序列化 | `serde` / `serde_json` | 数据结构与 JSON |
| 数据库 | `rusqlite` | 本地 SQLite（最简单，无编译期依赖） |
| 发信 | `lettre` | 免费 SMTP 发送冷邮件 |
| 收信分析 | `imap`(rustls) | 拉取回复并由 doubao-seed 分类 |
| 定时调度 | `tokio-cron-scheduler` | 每日运行流水线 |
| 配置 | `dotenvy` | 读取 `.env` |
| CLI | `clap` | 命令与参数 |
| 日志 | `tracing` + `tracing-subscriber` | 结构化日志 |
| 错误处理 | `anyhow`（应用）/ `thiserror`（库） | 错误传播与定义 |

> doubao-seed 接入：base_url = `https://ark.cn-beijing.volces.com/api/v3`，model = 你的 Endpoint ID，api_key = `ARK_API_KEY`。完全兼容 OpenAI SDK，无需改造成本。

## 4. 架构

线性流水线 + 可插拔模块（借鉴 `AI-SDR-System` 的 6-agent 拆解，但用 Rust 结构体实现，不引入重框架）：

```
prospector(google_maps / github / search)
   → enricher → scorer → writer → sender → analyzer(IMAP)
        ↓                ↓                      ↓
   rusqlite(leads)  rusqlite(score)      rusqlite(outreach)
```

每个环节是一个独立模块，输入/输出为规范化数据结构（`Lead` / `Outreach`），新增渠道 = 新增一个 prospector 适配器，不动主干。

## 5. 借鉴的开源项目与设计

| 来源 | 借鉴点 |
|---|---|
| `AG2AI-Admin/AI-SDR-System` | 6-agent 职责拆解（Prospector/Outreach/Qualification/Conversation/Scheduler/HumanHandoff）+ cold-email / scoring prompt 模板 |
| `Sama-ndari/autonomous-sdr-agent` | 状态机（写→分类→决策）用于回复处理 |
| `laramies/theHarvester` | OSINT 多源邮箱抓取思路（Google Maps / 搜索引擎） |
| `sahil9001/customer-outreach-campaign-agent` | CrewAI 双智能体角色/目标设定范式（仅作概念参考） |
| `filip-michalsky/SalesGPT` | 上下文感知对话阶段管理（用于 analyzer 的回复分类） |

**prompt 资产**：直接复用上述开源项目的 cold-email 与 ICP-scoring 提示词模板，改为适配「程序员接开发外包」的语境，并加入英文自检（QC）环节兜底 doubao-seed 的英文质量。

## 6. 模块详细设计

### 6.1 `llm.rs` — LLM 客户端封装
- 基于 `async-openai` 构造 `Client`，注入自定义 `OpenAIConfig`（base_url / api_key / model）。
- 提供 `complete(system, user) -> String` 与 `complete_json(...)`（结构化输出，用于评分/分类）。
- 统一超时、重试（指数退避）、token 预算控制。

### 6.2 `prospector/` — 线索获取（可插拔适配器）
每个适配器返回统一 `Lead { name, company, email, role, website, source }`：
- `google_maps.rs`：借鉴 theHarvester，HTTP GET + `scraper`/`regex` 从 Google Maps 搜索结果提取企业网站与邮箱，按行业/地区参数化。
- `github.rs`：调用 GitHub Search API（免费额度）找「需要帮助/招聘外包」的 issue、repo、组织。
- `search.rs`：DuckDuckGo HTML / Serper 免费层，按「we need a developer / build MVP / outsource software」等信号检索。
- 每源独立 try/except + 退避；无效邮箱过滤；按邮箱+公司去重。

### 6.3 `enricher.rs` — 清洗与背调
- doubao-seed 校验邮箱有效性、提取公司背景与「购买信号」（如刚融资、招人、发外包需求）。

### 6.4 `scorer.rs` — ICP 分级
- 规则（行业/规模/角色）+ doubao-seed 评分，标 `qualified` 与否，输出分数与理由。

### 6.5 `writer.rs` — 个性化冷邮件生成
- 基于背调生成英文 subject + body；**含一轮自检（QC）**确保通顺、不像机器人。
- 复用开源 cold-email prompt 模板，按客户类型（创始人/SME/工作室/代理）切换角度。

### 6.6 `sender.rs` — 发送
- `lettre` 通过免费 Gmail / 现有邮箱 SMTP 发送；纯文本优先、随机延时、每日上限（MVP ≤30 封/天）保护送达率。

### 6.7 `linkedin_drafter.rs` / `community_drafter.rs` — 半自动出稿（MVP 不激活）
- 仅生成草稿供人工点发（规避 LinkedIn 封号与社区 spam 规则），接口已留好。

### 6.8 `analyzer.rs` — 回复分析与约会议
- `imap` 拉取回复 → doubao-seed 分类（interested / not_now / objection / ooo）→ 正回应则产出「约会议」动作（输出日历链接或通知用户）。

### 6.9 `store.rs` — 存储
- `rusqlite` 封装，表：`leads`、`outreach`、`logs`。

### 6.10 `main.rs` — 编排与调度
- `tokio-cron-scheduler` 每日触发 `runner`；`runner` 顺序执行上述模块，状态落 SQLite。

## 7. 数据模型（SQLite）

```sql
CREATE TABLE leads (
  id INTEGER PRIMARY KEY,
  source TEXT,
  name TEXT,
  company TEXT,
  email TEXT UNIQUE,
  role TEXT,
  website TEXT,
  context TEXT,
  score INTEGER,
  status TEXT,
  created_at TEXT,
  contacted_at TEXT
);

CREATE TABLE outreach (
  id INTEGER PRIMARY KEY,
  lead_id INTEGER,
  channel TEXT,
  subject TEXT,
  body TEXT,
  sent_at TEXT,
  reply TEXT,
  reply_class TEXT,
  booked_at TEXT
);

CREATE TABLE logs (
  id INTEGER PRIMARY KEY,
  ts TEXT,
  level TEXT,
  msg TEXT
);
```

## 8. 数据流

```
prospector → store(leads:new) → enricher → scorer → store(leads:enriched/score)
  → writer → sender → store(outreach:sent, leads:contacted)
  → analyzer(IMAP) → classify → store(outreach:reply, leads:replied/booked)
```

## 9. 错误处理 / 合规 / 限流 / 送达率

- **每源独立容错**：try/except + 指数退避；单一源失败不影响整体。
- **数据质量**：无效邮箱过滤、按邮箱+公司去重、HTTP HEAD 校验网站存在。
- **限流**：每日发送上限（MVP ≤30）、token 预算、随机延时、并发受限。
- **LinkedIn / 社区**：绝不自动化，仅 AI 出稿 + 人工点发，规避封号。
- **送达率**：纯文本、低量、真实签名；**可选升级**——后续可加独立域名（约 70–100 元/年，非 MVP 必须，需用户另行决定）。
- **合规**：冷邮件需含退订/opt-out 说明，遵守目标地区法规（CAN-SPAM / GDPR 等）；在 `writer` 模板中内置 opt-out 语句。

## 10. 运行与部署

- **本机（验证期）**：`cargo run`（或编译后后台进程）+ `tokio-cron-scheduler` 每日跑；日志落盘。
- **上云（24/7）**：Oracle Cloud Always Free（永久免费 2×ARM 4vCPU/24G），同套代码，仅把 `rusqlite` 换成 `sqlx`+Postgres 或仍用 SQLite 文件（接口不变）。

## 11. 测试与验收

- **单测**（`cargo test`）：scraper 用 mock HTML；enricher/scorer 用样例 lead。
- **dry-run 模式**：只生成邮件不发送，先肉眼看质量。
- **MVP 成功标准**：真实约到 1+ 会议/成交。

## 12. 项目结构

```
agent全球自动获客/
  Cargo.toml
  .env.example
  src/
    main.rs
    config.rs
    llm.rs
    models.rs
    store.rs
    prospector/{mod,google_maps,github,search}.rs
    enricher.rs
    scorer.rs
    writer.rs
    sender.rs
    linkedin_drafter.rs
    community_drafter.rs
    analyzer.rs
    prompts.rs
  tests/
  docs/superpowers/specs/
```

## 13. 实施阶段（建议里程碑）

1. **M0 脚手架**：Cargo 工程、`.env`、llm 封装、store、models、日志。
2. **M1 线索源**：google_maps + github + search 适配器 + 去重/过滤。
3. **M2 AI 链路**：enricher + scorer + writer（含 QC）+ 复用开源 prompt。
4. **M3 触达闭环**：sender（lettre）+ analyzer（imap 分类）+ 约会议通知。
5. **M4 调度与验证**：cron 编排 + dry-run + 真实小批量发信 → 约到会议。
6. **M5 半自动扩展**：linkedin_drafter / community_drafter 出稿（人工点发）。
7. **M6 上云**：Oracle Always Free 部署。

## 14. 范围边界（YAGNI）

- MVP 不买域名、不接付费数据 API、不做 Web UI（先日志+终端）、LinkedIn/社区只留接口。
- 不引入 CrewAI/AutoGen 等重框架，仅借鉴其设计思想与 prompt 模板。

## 15. 风险与缓解

| 风险 | 缓解 |
|---|---|
| doubao-seed 英文质量波动 | writer 内置 QC 自检轮；必要时升级强模型（用户后续决定） |
| 免费邮箱送达率/限流 | 低量高质、纯文本、opt-out；域名列为可选升级 |
| LinkedIn 封号 | 仅半自动出稿，绝不自动操作 |
| 抓取被反爬 | 限流+延时+多源兜底；失败不影响整体 |
| 合规风险 | 内置 opt-out，遵守目标地区法规 |
