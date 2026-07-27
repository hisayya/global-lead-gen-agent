# Global Dev Radar - 100 个搜索查询模式

- 日期：2026-07-27
- 用途：系统每日自动扫描全球企业的搜索查询池
- 原则：不预设行业/地区，基于「买家实际问题信号」发现线索
- 搜索引擎：DuckDuckGo HTML 端点（主）+ GitHub API（辅）

---

## A. 主动招聘信号（25 个）

企业正在招聘开发者 = 有开发需求且愿意付费。

1. `"we are hiring" "software developer" -site:linkedin.com`
2. `"now hiring" "web developer" -site:linkedin.com`
3. `"looking for" "full stack developer" -site:linkedin.com`
4. `"hiring" "frontend developer" -site:linkedin.com`
5. `"hiring" "backend developer" -site:linkedin.com`
6. `"we need" "React developer" -site:linkedin.com`
7. `"we need" "Python developer" -site:linkedin.com`
8. `"job opening" "software engineer" -site:linkedin.com`
9. `"careers" "developer" "apply" -site:linkedin.com -site:indeed.com`
10. `"join our team" "engineer" -site:linkedin.com`
11. `"we are expanding" "engineering team"`
12. `"growing team" "developer" "apply"`
13. `"contract" "developer" "remote" -site:upwork.com -site:fiverr.com`
14. `"freelance" "developer" "needed" -site:upwork.com -site:fiverr.com`
15. `"seeking" "developer" "part-time" OR "contract"`
16. `"help wanted" "programmer" OR "developer"`
17. `"we are looking to hire" "developer" OR "engineer"`
18. `"open position" "software" "developer"`
19. `"job vacancy" "web developer" OR "app developer"`
20. `"recruiting" "developer" "join"`
21. `"we are building" "team" "developer" "hiring"`
22. `"need someone" "build" "website" OR "app" OR "platform"`
23. `"looking to outsource" "development" OR "software"`
24. `"seeking development partner" OR "seeking development agency"`
25. `"request for proposal" "software development" OR "web development"`

## B. 网站技术落后信号（20 个）

企业网站技术栈老旧 = 升级/重建机会。

26. `"powered by WordPress" "© 2018" OR "© 2019" OR "© 2020"`
27. `"powered by WordPress" "© 2021" -site:wordpress.com`
28. `inurl:"/wp-admin" "login" -site:wordpress.com`
29. `inurl:"/wp-login.php" -site:wordpress.com`
30. `"powered by Shopify" "© 2019" OR "© 2020"`
31. `"powered by Wix" -site:wix.com`
32. `"powered by Squarespace" -site:squarespace.com`
33. `"powered by Joomla" -site:joomla.org`
34. `"powered by Drupal" "© 2020" OR "© 2021"`
35. `inurl:"/administrator/" "Joomla"`
36. `inurl:"/user/login" "Drupal"`
37. `"this site uses cookies" "WordPress" -site:wordpress.com "© 2019"`
38. `"website by" "© 2018" OR "© 2019" "contact us"`
39. `"designed by" "© 2018" OR "© 2019" "about us"`
40. `"last updated" "2019" OR "2020" "contact us"`
41. `inurl:"/old/" "contact us"`
42. `"site under construction" "contact us"`
43. `"best viewed in" "Internet Explorer" "contact us"`
44. `"powered by WordPress" "http://" -inurl:https -site:wordpress.com`
45. `inurl:"/page/" "powered by WordPress" "© 2019" -site:wordpress.com`

## C. 缺少现代功能信号（15 个）

企业网站缺关键功能 = 开发机会。

46. `"contact us" "email us" -inurl:form -site:linkedin.com`
47. `"call us" "phone" "contact" -inurl:form -inurl:booking`
48. `"request a quote" -inurl:form`
49. `"book an appointment" "call us" -inurl:online -inurl:booking`
50. `"schedule" "call us" -inurl:online -inurl:booking`
51. `"our services" "contact us" -inurl:cart -inurl:shop`
52. `"about us" "contact" "email" -inurl:portal -inurl:login`
53. `"customer portal" "coming soon"`
54. `"online ordering" "coming soon"`
55. `"new website" "coming soon" "contact us"`
56. `"we are updating" "website" "contact us"`
57. `"under maintenance" "contact us"`
58. `"login" "coming soon" -site:github.com`
59. `"dashboard" "coming soon" -site:github.com`
60. `"mobile app" "coming soon" -site:github.com`

## D. 社区求救信号（15 个）

Reddit / 论坛 / X 上有人公开求助找开发者。

61. `site:reddit.com "need a developer" "budget"`
62. `site:reddit.com "looking for" "developer" "hire"`
63. `site:reddit.com "recommendations" "web developer"`
64. `site:reddit.com "anyone know" "app developer" "hire"`
65. `site:reddit.com "outsourcing" "development" "experience"`
66. `site:reddit.com "freelance" "developer" "budget" "project"`
67. `site:reddit.com "how much" "cost" "build" "website" OR "app"`
68. `site:reddit.com "need help" "custom software" OR "automation"`
69. `site:reddit.com "looking for" "agency" OR "freelancer" "build"`
70. `site:reddit.com "MVP" "developer" "budget" OR "hire"`
71. `site:reddit.com "startup" "need" "technical" "co-founder" OR "developer"`
72. `site:reddit.com "legacy" "system" "modernize" OR "migrate"`
73. `site:reddit.com "CRM" OR "ERP" "custom" "build" OR "develop"`
74. `site:reddit.com "automation" "workflow" "build" "help"`
75. `site:reddit.com "dashboard" "build" "developer" "hire"`

## E. 企业增长/扩张信号（10 个）

企业正在扩张 = 可能需要新系统/工具/自动化。

76. `"we are expanding" "new office" "contact us"`
77. `"now open" "new location" "careers" "contact us"`
78. `"we are growing" "hiring" "contact us"`
79. `"series A" OR "series B" OR "funding" "contact us" -site:crunchbase.com`
80. `"we just raised" "contact us" -site:crunchbase.com`
81. `"acquired" "merger" "contact us" "careers"`
82. `"new product" "launching" "contact us"`
83. `"rebranding" "new" "website" "contact us"`
84. `"scaling" "team" "hiring" "contact us"`
85. `"doubling" "team" "hiring" "contact us"`

## F. GitHub 求助信号（15 个）

GitHub Issues / Discussions 上有企业团队公开求助。

86. `site:github.com "help wanted" "production" label:bug`
87. `site:github.com "good first issue" "enterprise" OR "production"`
88. `site:github.com "need help" "deployment" "urgent"`
89. `site:github.com "how to" "integrate" "API" "help" in:issues`
90. `site:github.com "migration" "guide" "help" in:issues`
91. `site:github.com "performance" "issue" "help" in:issues`
92. `site:github.com "security" "vulnerability" "help" in:issues`
93. `site:github.com "outdated" "dependency" "help" in:issues`
94. `site:github.com "legacy" "code" "refactor" "help"`
95. `site:github.com "automation" "workflow" "help" in:discussions`
96. `site:github.com "dashboard" "build" "help" in:discussions`
97. `site:github.com "CRM" "custom" "build" in:discussions`
98. `site:github.com "bot" "build" "help" in:discussions`
99. `site:github.com "scraping" "build" "help" in:discussions`
100. `site:github.com "API" "integration" "looking for" "developer" in:discussions`

---

## 使用说明

- 系统每日轮换执行这 100 个查询（DuckDuckGo HTML 端点，每次取前 5 个结果）。
- 每个结果 URL -> 抓取页面内容 -> AI 诊断（L3）-> 评分（>=50 才进入下一步）。
- 去重：按域名+邮箱去重，同一企业不重复处理。
- 限流：每查询间隔 3-5 秒 + 随机延时；每日总查询量控制在 50-80 个（避免触发反爬）。
- 查询池可动态扩展：系统积累诊断数据后，可基于高频痛点关键词自动生成新查询。
