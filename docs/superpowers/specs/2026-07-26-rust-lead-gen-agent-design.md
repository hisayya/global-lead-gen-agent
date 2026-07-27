# Global Dev Radar - 全球企业问题扫描雷达（Rust）- 设计文档 v2

- 日期：2026-07-27
- 状态：待用户复核
- 作者：WorkBuddy（brainstorming + GPT 优化建议 + 5 轮联网检索）
- 变更：v1 → v2，整合 GPT 三层建议（0 订阅费系统 / 渠道风险分级 / Global Dev Radar 十层架构）与 2026 行业最佳实践

## 1. 项目概述

**不要把自己设计成「全球发广告机器人」，而要设计成「全球企业问题扫描雷达」。发现问题的人，比写代码的人更赚钱。**

系统定位：以独立程序员身份，每天自动扫描全球企业网站，AI 诊断其技术短板与业务痛点，筛出值得联系的目标，自动生成个性化开发建议与冷邮件，自动跟踪回复，**你只负责开会成交**。

核心理念：**让机器负责寻找 1000 个客户，你负责成交那 10 个客户。**

设计原则：在能满足需求的前提下取最简单方案，同时保留充足扩展性（YAGNI + 整洁接口）。

## 2. 目标与约束

| 项 | 内容 |
|---|---|
| 提供的服务 | 承接各类代码开发（web/app/自动化/脚本等） |
| 目标客户 | 全球任何可能需要写代码的人/企业 |
| 触达渠道 | 全渠道：冷邮件（主力 100% 自动化）+ LinkedIn/社区（半自动出稿） |
| 唯一付费项 | AI API（doubao-seed，经火山方舟 OpenAI 兼容接口），其余零花费 |
| 运行位置 | 先本机 Mac 常驻验证，后上 Oracle Cloud Always Free 做 24/7 |
| MVP 成功标准 | 真实约到 1+ 会议/成交 |
| 自动化边界 | 找客户/研究客户/发邮件可自动化 90%；会议/报价/需求确认/成交转人工 |

## 3. 目标市场策略

**不广撒网，先聚焦 1-2 个细分市场打透。**

搜索证实：锁定细分市场后转化率 4.5%-7.5%，广撒网 <1.5%（来源：bangying360 海外运维数据）。Blue ocean selling 策略--找竞争对手没覆盖的细分段。

MVP 首选细分市场（建议，可配置）：
- 德国制造业（数字化滞后、有预算、英语可沟通）
- 美国 SaaS 初创（增长快、需求明确、付费意愿强）

后续扩展：澳洲物流、加拿大医疗软件、英国电商--每个市场验证转化率后再加。

**细分市场由「买家问题」定义，不仅按行业。** 例如不是「制造业」，而是「WordPress 2018 建站 + 无客户门户 + 正在招聘 IT 的中型制造商」。

## 4. 渠道风险分级

| 渠道 | 自动采集 | 自动分析 | 自动发消息 | 策略 |
|---|---|---|---|---|
| Google 搜索 | 是 | 是 | 无需 | 100% 自动化，安全 |
| 企业官网 | 是 | 是 | 联系表单可半自动 | 100% 自动化抓取+分析 |
| DuckDuckGo | 是 | 是 | 无需 | 100% 自动化，比 Google 更宽松 |
| GitHub | 是 | 是 | 无需 | API 免费，安全 |
| Reddit | 是 | 是 | 有风险 | 自动发现+AI 筛选→通知你→人工回复 |
| LinkedIn | 是 | 是 | 高风险 | 仅公开资料分析+AI 出稿→人工点发 |
| Email | 是 | 是 | 最适合 | 100% 自动化（主力通道） |

**核心判断**：Google + 企业官网 + Email 是最稳定的 100% 自动化链路。LinkedIn/Reddit 仅做发现与出稿，不自动发消息。

## 5. 技术栈（Rust，全免费/零 SaaS）

| 关注点 | crate | 用途 |
|---|---|---|
| LLM 调用 | `async-openai` | OpenAI 兼容接口连 doubao-seed（自定义 base_url） |
| 异步运行时 | `tokio` | 全盘异步 |
| HTTP 客户端 | `reqwest` | 抓取网页 / 调 API |
| HTML 解析 | `scraper` + `regex` | 企业官网内容提取、DuckDuckGo 结果解析 |
| 序列化 | `serde` / `serde_json` | 数据结构与 JSON |
| 数据库 | `rusqlite` | 本地 SQLite |
| 发信 | `lettre` | 免费 SMTP 发送冷邮件 |
| 收信分析 | `imap`(rustls) | 拉取回复并由 doubao-seed 分类 |
| 定时调度 | `tokio-cron-scheduler` | 每日运行流水线 |
| 配置 | `dotenvy` | 读取 `.env` |
| CLI | `clap` | 命令与参数 |
| 日志 | `tracing` + `tracing-subscriber` | 结构化日志 |
| 错误处理 | `anyhow`（应用）/ `thiserror`（库） | 错误传播与定义 |

> doubao-seed 接入：base_url = `https://ark.cn-beijing.volces.com/api/v3`，model = Endpoint ID，api_key = `ARK_API_KEY`。完全兼容 OpenAI SDK。

## 6. 架构（十层 Global Dev Radar）

```
L1  目标市场引擎（配置细分市场关键词）
 ↓
L2  企业爬虫系统（DuckDuckGo + GitHub + 直连官网，每日自动运行）
 ↓
L3  AI 诊断系统（分析网站 → 行业/技术栈/数字化成熟度/痛点/评分）
 ↓
L4  决策人发现（官网 About/Team 页 + 公开搜索找 Founder/CEO/CTO）
 ↓
L5  AI 销售策略生成器（痛点→改进建议→开发机会→报价区间估算）
 ↓
L6  邮件工厂（Subject/Opening/PainPoint/Solution/CTA，每封独立生成）
 ↓
L7  自动触达（Day1→Day4→Day10→breakup，纯文本、低量、opt-out）
 ↓
L8  回复分类器（热客户→通知你 / 冷客户→90天后再联系 / 拒绝→永久停止）
 ↓
L9  CRM 管线（Lead→Contacted→Replied→Meeting→Proposal→Won/Lost）
 ↓
L10 成交中心（人工：会议/报价/需求确认/成交，零自动化）
```

每层是独立 Rust 模块，输入/输出为规范化数据结构，可单独替换或扩展。

### 6.1 L1 - 目标市场引擎 `market.rs`
- 配置文件定义细分市场：行业关键词、地区、ICP 特征（如「WordPress 建站 + 无客户门户 + 招聘中」）。
- 输出搜索查询模板：`site:linkedin.com/in CTO "Germany"` / `"manufacturing company" Germany site:.de` / `"contact us" "about us" manufacturing`。
- MVP 先配置 1-2 个市场，后续可加。

### 6.2 L2 - 企业爬虫系统 `prospector/`
统一返回 `Lead { name, company, email, role, website, source }`：
- `search.rs`（主源）：DuckDuckGo HTML 端点 `https://html.duckduckgo.com/html/`，`reqwest` GET + `scraper` 解析 `.result__a` CSS 选择器提取结果链接（已验证可行）。
- `github.rs`：GitHub Search API（免费额度）找「需要帮助/招聘外包」的 issue、repo。
- `website_scraper.rs`：对搜索到的企业官网，抓取首页/About/Services/Careers/Contact/Blog 六个页面，存入 `company_pages` 字段供 L3 诊断。
- 每源独立 try/except + 退避；无效邮箱过滤；按邮箱+公司去重。

### 6.3 L3 - AI 诊断系统 `diagnostician.rs`（新增，核心差异化）
**这是系统的真正护城河，不是「AI 写邮件」，而是「AI 发现问题」。**

把企业网站六页内容扔给 doubao-seed，输出：
```
行业
业务模式
规模估计
技术栈（WordPress/Shopify/React/custom/...）
数字化成熟度（1-5）
痛点分析（UI老旧/无客户门户/无在线报价/手动流程/...）
```

加权评分模型：
| 信号 | 分数 |
|---|---|
| 技术栈落后（如 WordPress 2018） | +20 |
| 正在招聘 IT/开发人员 | +20 |
| 业务增长迹象（招聘/扩张） | +15 |
| 有 IT 团队但网站老旧 | +10 |
| 目标市场匹配（美国/德国/澳洲） | +10 |
| 无在线预约/客户门户/工单系统 | +10 |
| 有联系表单但无邮箱 | +5 |

总分 0-100，>=50 标为 `qualified`。

### 6.4 L4 - 决策人发现 `decider.rs`（新增）
从官网 About/Team 页面提取决策人：
- Founder / CEO / CTO / Operations Manager
- 记录：姓名、职位、邮箱（regex 从页面提取）、LinkedIn（如公开）
- 若官网无决策人信息，标记为 `needs_manual_research`

### 6.5 L5 - AI 销售策略生成器 `strategist.rs`（新增）
基于 L3 诊断结果，生成：
```
问题分析（具体到页面/功能级别）
改进建议（可落地的技术方案）
开发机会（能卖什么服务）
报价区间估算（rough range，供你参考）
```

例如：
```
问题：客户支持流程高度手动，官网无工单系统
建议：建立自助工单系统 + 自动报表
机会：工单系统开发 + 内部仪表盘
报价：$3,000-$8,000（视复杂度）
```

### 6.6 L6 - 邮件工厂 `writer.rs`
基于 L3 诊断 + L5 策略，生成英文 subject + body：
- 结构：Subject / Opening（引用具体观察）/ PainPoint / Solution / CTA
- 每封独立生成，非模板群发
- 含一轮 QC 自检确保英文通顺
- 首封 50-125 词、0 链接、纯文本
- 内置 opt-out 语句与寄件人身份/地址
- 按客户类型（创始人/SME/工作室/代理）切换角度

### 6.7 L7 - 自动触达 `sender.rs` + `sequence.rs`
**4 封序列（10-16 天），基于 2026 实测最佳实践：**

| 触达 | 日期 | 目标 | 内容 |
|---|---|---|---|
| Email 1 | Day 1 | 开场（问题+相关性） | 引用具体观察，1 个 proof point，1 个软 CTA，<60 词 |
| Email 2 | Day 4 | bump（同线程回复） | 25-35 词，简短提及上次，换一个角度 |
| Email 3 | Day 9-10 | 新角度/案例 | 不同 hook，社会证明（案例研究），<50 词 |
| Email 4 | Day 14-16 | breakup | 「我将关闭此线索」，留 calendar link。**breakup 回复率最高（8-14%）** |

- `lettre` 通过免费 Gmail SMTP 发送；纯文本；同线程回复（Reply-To 原邮件，不新建线程，提升 12-18% 回复率）。
- 每日上限 ≤25-30 封/邮箱；随机延时；周二至周四发送效果最佳。

### 6.8 L8 - 回复分类器 `analyzer.rs`
IMAP 拉取回复 -> doubao-seed 分类：
| 分类 | 动作 |
|---|---|
| 热客户（interested/let's talk/book a call） | 即时通知你（推送通知/日志告警），转人工 |
| 冷客户（not now/maybe later） | 90 天后自动重新进入序列 |
| 拒绝（no thanks） | 永久停止，标记 `lost` |
| 异议（objection/pricing） | AI 生成 rebuttal 草稿供你审核后发送 |
| OOO | 自动延后重试 |

### 6.9 L9 - CRM 管线 `store.rs`
状态流转：
```
Lead → Contacted → Replied → Meeting → Proposal → Won
                                        ↘ Lost
```

### 6.10 L10 - 成交中心（人工，零自动化）
客户表达兴趣后，全部转人工：会议、报价、需求确认、成交。

### 6.11 `llm.rs` - LLM 客户端封装
- `async-openai` + 自定义 `OpenAIConfig`（base_url / api_key / model）。
- `complete(system, user) -> String` 与 `complete_json(...)`。
- 超时、指数退避重试、Semaphore 并发限制（5-10）、token 预算。

### 6.12 `main.rs` - 编排与调度
- `tokio-cron-scheduler` 每日触发 runner。
- runner 顺序执行 L1→L7；L8 独立 IMAP 轮询。

## 7. 公开资产层（被动获客，零成本）

**很多人忽略这一层。客户买的不是代码，而是信任。**

| 资产 | 平台 | 内容 |
|---|---|---|
| Demo 项目 | GitHub + GitHub Pages（免费托管） | CRM Demo / 库存系统 Demo / 工单系统 Demo / 自动报表 Demo |
| 个人主页 | GitHub Pages | 定位声明 + 3-5 个精选项目 + 案例研究 + 联系方式 |
| 技术内容 | Reddit / dev.to / LinkedIn | 分享自动化/AI 工作流/企业数字化经验 |

MVP 阶段：先在 GitHub 建 3 个可运行的 Demo 项目 + 一个 GitHub Pages 个人页，在冷邮件中附链接作为信任信号。搜索证实：有 live demo link + public repo 的开发者，回复率显著高于无公开资产者。

## 8. 数据模型（SQLite）

```sql
CREATE TABLE leads (
  id INTEGER PRIMARY KEY,
  source TEXT,
  name TEXT,
  company TEXT,
  email TEXT UNIQUE,
  role TEXT,
  website TEXT,
  country TEXT,
  industry TEXT,
  company_pages TEXT,
  diagnosis TEXT,
  tech_stack TEXT,
  digital_maturity INTEGER,
  pain_points TEXT,
  score INTEGER,
  qualified INTEGER,
  strategy TEXT,
  status TEXT,
  created_at TEXT,
  contacted_at TEXT
);

CREATE TABLE outreach (
  id INTEGER PRIMARY KEY,
  lead_id INTEGER,
  channel TEXT,
  sequence_step INTEGER,
  subject TEXT,
  body TEXT,
  sent_at TEXT,
  reply TEXT,
  reply_class TEXT,
  booked_at TEXT
);

CREATE TABLE market_config (
  id INTEGER PRIMARY KEY,
  name TEXT,
  keywords TEXT,
  region TEXT,
  icp_criteria TEXT,
  active INTEGER
);

CREATE TABLE logs (
  id INTEGER PRIMARY KEY,
  ts TEXT,
  level TEXT,
  msg TEXT
);
```

## 9. 数据流

```
L1(market_config) → L2(prospector: search/github/website) → store(leads:new)
  → L3(diagnostician: AI 诊断) → L4(decider: 决策人发现) → store(leads:enriched)
  → L5(strategist: 策略生成) → store(leads:strategy)
  → L6(writer: 邮件生成) → L7(sender: 发送序列) → store(outreach:sent, leads:contacted)
  → L8(analyzer: IMAP 回复分类) → store(outreach:reply)
    → hot → 通知用户(人工)
    → cold → 90天后重入序列
    → reject → 永久停止
```

## 10. 错误处理 / 合规 / 限流 / 送达率

### 10.1 容错与数据质量
- 每源独立 try/except + 指数退避；单一源失败不影响整体。
- 无效邮箱过滤、按邮箱+公司去重、HTTP HEAD 校验网站存在。

### 10.2 发送限流与送达率（2026 实测基准）
- 单邮箱 25-30 封/天安全上限；MVP ≤25-30/天。
- 首封纯文本、0 链接、无图片；同线程回复；序列 4 封、Day 1/4/10/16。
- 投诉率 <0.1%（红线 0.3%）；退信率 <2%。
- 可选升级：独立发信域名（约 70-100 元/年）+ SPF/DKIM/DMARC；非 MVP 必须。

### 10.3 合规（CAN-SPAM / GDPR / CASL）
- 每封含：真实发件人姓名与公司、物理/邮寄地址（footer）、清晰 opt-out。
- GDPR/PECR：仅工作邮箱、保留相关性记录、退订即删。
- CASL：仅公开列出工作邮箱、信息与角色直接相关。
- `writer` 模板内置 opt-out 语句与寄件人身份/地址占位符。

### 10.4 渠道安全
- LinkedIn/Reddit：绝不自动发消息，仅 AI 出稿 + 人工点发。

### 10.5 Rust 工程最佳实践
- 并发限流：`tokio::sync::Semaphore`（5-10 并发）。
- 重试：429/5xx 指数退避。
- 边界：合理 `max_tokens`；输入长度校验；async 内禁 `thread::sleep`。
- 可观测：每次调用打 `tracing` span。
- 护栏：`max_steps` 上限；高风险操作加确认/白名单。

### 10.6 抓取伦理与法律
- 查 `robots.txt` 与 ToS；设置 descriptive User-Agent；尊重 429 + Retry-After。
- 速率：新域名 1 请求/3-5 秒；缓存结果；收到停止请求立即配合。
- GDPR：合法利益 + LIA + 数据最小化 + 保留 TTL + 删除请求支持。
- DuckDuckGo HTML 端点比 Google 更宽松，作为主搜索源。

## 11. 运行与部署

- 本机：`cargo run` + `tokio-cron-scheduler` 每日跑；日志落盘。
- 上云：Oracle Cloud Always Free（永久免费 2xARM 4vCPU/24G），同套代码。

## 12. 测试与验收

- 单测（`cargo test`）：scraper 用 mock HTML；diagnostician/scorer 用样例 lead。
- dry-run 模式：只生成诊断+邮件不发送，先肉眼看质量。
- MVP 成功标准：真实约到 1+ 会议/成交。

## 13. 项目结构

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
    market.rs
    prospector/{mod,search,github,website_scraper}.rs
    diagnostician.rs
    decider.rs
    strategist.rs
    writer.rs
    sender.rs
    sequence.rs
    analyzer.rs
    prompts.rs
    linkedin_drafter.rs
    community_drafter.rs
  tests/
  docs/superpowers/specs/
```

## 14. 实施阶段（里程碑）

1. **M0 脚手架**：Cargo 工程、`.env`、llm 封装、store、models、日志。
2. **M1 目标市场 + 爬虫**：market.rs 配置 + DuckDuckGo/GitHub/官网抓取适配器 + 去重。
3. **M2 AI 诊断**：diagnostician.rs（网站分析+评分模型）+ decider.rs（决策人发现）+ strategist.rs（策略生成）。
4. **M3 邮件工厂 + 触达**：writer.rs（含 QC）+ sender.rs + sequence.rs（4 封序列）。
5. **M4 回复分析 + CRM**：analyzer.rs（IMAP 分类）+ store 状态流转 + 通知机制。
6. **M5 调度与验证**：cron 编排 + dry-run + 真实小批量发信 -> 约到会议。
7. **M6 公开资产**：GitHub Pages 个人页 + 3 个 Demo 项目。
8. **M7 半自动扩展**：linkedin_drafter / community_drafter 出稿。
9. **M8 上云**：Oracle Always Free 部署。

## 15. 真正的护城河

不是「AI 写邮件」，而是：
- **企业数据库**：一年后可能有 50,000 家公司的结构化诊断数据。
- **痛点识别模型**：基于积累的诊断结果持续优化评分模型与 prompt。
- **行业知识库**：按行业积累常见痛点与解决方案模板。

这些是竞争对手难以复制的资产。邮件模板可以被抄，但一个包含 50,000 家企业痛点诊断的数据库不能。

## 16. 范围边界（YAGNI）

- MVP 不买域名、不接付费数据 API、不做 Web UI、LinkedIn/社区只留接口。
- 不引入 CrewAI/AutoGen 等重框架。
- 不做自动成交（L10 永远人工）。

## 17. 风险与缓解

| 风险 | 缓解 |
|---|---|
| doubao-seed 英文质量波动 | writer 内置 QC 自检轮 |
| 免费邮箱送达率/限流 | ≤25-30 封/天、纯文本、opt-out、投诉率<0.1% |
| LinkedIn 封号 | 仅半自动出稿，绝不自动操作 |
| 抓取被反爬 | 限流+延时+多源兜底 |
| 合规风险 | 内置 opt-out + 寄件人身份/地址 |
| Google Maps 抓取违反 ToS | 以 DuckDuckGo/GitHub/官网为主源 |
| GDPR 个人数据 | legitimate interest + LIA + 数据最小化 + TTL |
| LLM 429/5xx | Semaphore 并发限制 + 指数退避 |
| 目标市场过宽导致资源分散 | MVP 锁定 1-2 个细分市场 |
