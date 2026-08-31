# δ · test-design 关键引用 vs 7 份设计书实际章节乖离清单

> **审计基线**:`docs/test-design.md` v0.3 (2026-08-31, 49,846 字节)
> **审计时 main HEAD**:`3c9c2ae5` (注:任务简报写 `948582e`,实测工作树 HEAD 不同,以实测为准)
> **7 份被引用设计书**:basic-design (150,624 字节) / api-design (174,057) / data-design (260,849) / security-design (106,421) / runtime-design (71,680) / integration-design (57,696) / ai-agent-design (69,072)
> **方法**:test-design.md 全文 grep → 提取所有 `《X Design》§X` / `§X` 引用 → 各设计书 grep `^#{1,4} <X>` 验证章节存在与标题 → 核对内容
> **不对账要求**:**P0 = 引用 §X 在目标设计书中完全不存在,或 §X 标题与 test-design 引用语境根本对不上**;**P1 = 引用存在但 §X 标题/内容错位(借用别章节)**;**P2 = 引用存在且标题匹配,但内容与 test-design 描述有出入**

---

## 0. 摘要

- **总对账条数**:28 条具体 §X 引用(test-design 主体,排除 7 份规则定义本身的占位行)
  - **P0 = 11**(引用 §X 不存在,或章节标题完全错位)
  - **P1 = 3**(§X 存在但内容错位到别章节)
  - **P2 = 0**(MATCH 但有微差)
  - **无法验证 = 0**
  - **MATCH = 14**(引用 §X 存在且标题/内容对得上)
- **重点乖离**:
  1. test-design §6.3.2 + §14 #16 **VAL-001 四重门 P0 不变量**全部锚定到 `basic-design §4.5.6`,但 §4.5.6 实际是 `Requirement 索引`,VAL-001 的 4-gate 链代码在 **§4.5.5 "AI Completion 判定链"**——这是最严重的引用错位,直接动摇 P0 不变量证据链
  2. test-design 多次引用 `basic-design §37 / §44 / §0.5 / §18 / §8.2 / §27.3 / §27.4`,**这些章节号在 basic-design.md 中根本不存在**(basic-design 只到 §15 + 附录 A/B/C,正文最深嵌套到 §4.10.9 / §8.6 / §A.7),全部是 requirements.md 章节号被错误归因
  3. `Security Design §9.3` 被 2 处引用(Local Runtime / Untrusted 隔离),但 security-design 只有 §9.1 / §9.2.x,**没有 §9.3**;实际 Local Runtime 8 命令在 §5.5.2,Local Runtime 威胁在 §9.2.7-9.2.8

---

## 1. test-design §0.4 引用规则对账(test-design.md:83-92)

test-design §0.4 显式声明 7 条引用规则:

| # | 引用规则 | test-design 写明的格式 | 实际使用情况 |
|---|---|---|---|
| 1 | 《Requirements》v2.0 | `§N` | ✅ 用 `《Requirements》§44` (line 119) — 格式 OK,但 Requirements 不在 7 引用规则清单内 |
| 2 | 《Basic Design》 | `《Basic Design》§X` | ✅ / ❌ 多数用对格式,部分用裸 `§X` 形式 (line 878, 901, 902, 1482) |
| 3 | 《API Design》 | `《API Design》§X` | ✅ 用对 |
| 4 | 《Data Design》 | `《Data Design》§X` | ⚠️ 7 引用规则中声明但 **正文 0 处具体 §X 引用**(只引 schema/RLS 概念) |
| 5 | 《Security Design》 | `《Security Design》§X` | ✅ 用对,但 §X 全部错位(见 §5) |
| 6 | 《Runtime Design》 | `《Runtime Design》§X` | ✅ 用对(只 1 处具体引用) |
| 7 | 《Integration Design》 | `《Integration Design》§X` | ⚠️ 7 引用规则中声明但 **正文 0 处具体 §X 引用** |
| 8 | 《AI/Agent Design》 | `《AI/Agent Design》§X` | ✅ 用对(只 1 处具体引用) |
| **未在 7 引用规则中** | 《External Design》 | (未声明) | ❌ 4 处使用(line 381, 385, 719, 731)— 规则缺失 |
| **未在 7 引用规则中** | 《Operation Design》 | (未声明) | ❌ 2 处使用(line 948, 1452)— 规则缺失 |

**结论**:
- (a) **规则缺口**:`《External Design》` + `《Operation Design》` 实际被引用,但 §0.4 没声明 → 规则不自洽
- (b) **格式漂移**:正文存在裸 `§X` 形式 (如 §0 引用清单用 `§4` `§8` `§5` `§27.4` 而非 `《X Design》§X`),违反自身 §0.4 规则
- (c) **Data Design / Integration Design 零引用**:7 引用规则中显式声明,但正文 0 处具体 §X,规则名义化
- (d) **Requirements 引用**:test-design 多次引用 `《Requirements》§X`,但 Requirements 不在 7 引用规则内

---

## 2. Basic Design 引用乖离

> **目标文件**:`docs/basic-design.md`(150,624 字节,3,604 行)
> **basic-design 实际章节结构**:正文 §0 ~ §15 + 附录 A/B/C,正文最深嵌套 §4.10.9 / §8.6 / §A.7;**没有 §37, §44, §27.3, §27.4, §18**(这些是 requirements.md 的章节号)
> **§0.1-0.4**(line 11-79),**无 §0.5**;"接口稳定承诺" 是 top-level ## 标题(line 3563)
> **§4.5.5** = "AI Completion 判定链(§27.3,§77)"(VAL-001 实际所在)
> **§4.5.6** = "Requirement 索引"(仅 4 个 req ID 列表,不含 4-gate 逻辑)

| # | test-design 引用 | 引用位置 | basic-design 实际 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| B-1 | 《Basic Design》§37 Gherkin 格式 AC | line 570, 774 | **不存在** — basic-design 最深 §15 + 附录 A/B/C,§37 是 requirements.md 章节号 | §X 不存在 | **P0** |
| B-2 | 《Basic Design》§44 K8s Tax 纪律 | line 965 | **不存在** — basic-design §44 章节号无;K8s Tax 实际在 basic-design §1.5(标题"关键不变量:K8s Tax 纪律(§44.2,§86-90)" line 255)+ §8.6(标题"低 K8s Tax 纪律(§44.2,§86)" line 2645) | §X 不存在,内容错位到 §1.5/§8.6 | **P0** |
| B-3 | basic-design §4.5.6 / §27.3 VAL-001 P0 不变量 | line 842 | **错位** — basic-design §4.5.6 是 "Requirement 索引"(line 1286,仅 4 req ID);VAL-001 4-gate 实际在 **§4.5.5 "AI Completion 判定链"**(line 1264,含 4-gate 链代码);§27.3 在 basic-design 不存在(是 requirements.md 章节号) | §X 错位,§X 跨文档引用错位 | **P0** |
| B-4 | basic-design §4.5.6 + §27.3 + §0.5 接口稳定承诺 #4 | line 872 | **三处全错**:<br>- §4.5.6 = Requirement 索引(line 1286),不是接口稳定承诺章节<br>- §27.3 在 basic-design 不存在(是 requirements.md)<br>- basic-design 无 §0.5;**接口稳定承诺** 在 basic-design 是 top-level `## 接口稳定承诺`(line 3563),承诺内容 #4 = "Risk Signal 类型(§4.8.5):8 种类型" — 不是 VAL-001 相关 | 三重 §X 全部错位 | **P0** |
| B-5 | basic-design §4.5.6 P0 不变量 | line 1486(§14 #16 冻结接口) | **错位** — §4.5.6 是 "Requirement 索引";P0 不变量在 §4.5.5 | §X 错位;**关键:此条目被列为 §14 #16 冻结接口,影响 test-design 自身 RFC 边界** | **P0** |
| B-6 | §8.2 REQ-WF-003 `RequireApproval` | line 879(裸 §X) | **错位** — basic-design §8.2 = "Service Promotion Model 在 K3s 下的具体含义"(line 2553),与 REQ-WF-003 无关;REQ-WF-003 在 basic-design §4.9.3(line 1748)被引用,但 basic-design 自身无 §8.2 章节承载此 REQ | §X 存在但内容错位(借用别章节标题) | **P1** |
| B-7 | §8.2 "❌ ExecuteArbitraryShell 必须被拒绝" | line 901(裸 §X) | **错位** — basic-design §8.2 = Service Promotion Model;ExecuteArbitraryShell 严禁在 **§6.3** "默认禁止 SaaS Server → Arbitrary Shell"(line 2215)+ §6.2 line 2210 列表 | §X 错位 | **P1** |
| B-8 | §18 Integration Webhook | line 902(裸 §X) | **不存在** — basic-design 引用 requirements.md §18 但自身无 §18 章节;basic-design §7.5 "PR / MR 链接与合并状态(§18,§19)"(line 2444) 引用了 requirements §18 | §X 不存在,跨文档引用错位 | **P0** |
| B-9 | §27.4 ReviewRecord | line 878(裸 §X) | **不存在** — basic-design 无 §27.4;ReviewRecord 实际在 basic-design §4.3.6 / §4.5(line 1199, §4.5 段);§27.4 在 requirements.md | §X 不存在,跨文档引用错位 | **P0** |
| B-10 | 《Basic Design》§15 J.x | line 1444 | **MATCH** — basic-design §15 = "Open Issues(继承 §46 决策表 J + 新增)"(line 3244);§15.1 继承自 §46 决策表 J,§15.2 是新增 Open Issue | 引用 §X 存在,标题匹配 | MATCH |
| B-11 | 《Basic Design》§4.5.5 AI Completion 判定链 | (test-design **未引用** §4.5.5) | 实际是 VAL-001 P0 不变量所在 — test-design **错引到了 §4.5.6** | 漏引正确章节 + 错引相邻章节 | **P0**(隐含) |

**Basic Design 乖离小计**:11 条引用 / 8 P0 / 2 P1 / 1 MATCH / 0 P2

---

## 3. API Design 引用乖离

> **目标文件**:`docs/api-design.md`(174,057 字节,3,062 行)
> **关键章节存在性核查**:§3 端点清单(行 531)、§8 错误模型与业务级错误码(行 1849)、§10 性能预算与限流(行 2168) — **全部存在**

| # | test-design 引用 | 引用位置 | api-design 实际 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| A-1 | 《API Design》§3 列出的所有端点 (>100 个) | line 270, 278, 300, 662 | **MATCH** — api-design §3 = "端点清单"(line 531),§3.0 端点总览 + §3.2-§3.27 分 Module;实际声明"234 个 REST 端点 + 1 个 WS 端点"(API-2 承诺,line 3001) | 引用 §X 存在,标题匹配(数量描述 "<100 个" 与实际 "234 个" 略有不一致,属于 test-design 自身描述,不是 §X 错位) | MATCH |
| A-2 | 《API Design》§8 错误码 | line 303, 663 | **MATCH** — api-design §8 = "错误模型与业务级错误码"(line 1849);§8.1 继承 RFC 7807,§8.2 命名规则,§8.3 错误码字典(≥30 条,实际声明 ≥130 条,line 3009) | 引用 §X 存在,标题匹配 | MATCH |
| A-3 | 《API Design》§10 P95 预算 | line 471, 910 | **MATCH** — api-design §10 = "性能预算与限流"(line 2168);§10.1 = "单端点 P50/P95/P99 预算"(line 2170) | 引用 §X 存在,标题匹配(§10 是总章,具体 P95 表格在 §10.1,test-design 用 §10 上位引用可接受) | MATCH |

**API Design 乖离小计**:3 条 / 0 P0 / 0 P1 / 3 MATCH / 0 P2

---

## 4. Data Design 引用乖离

> **目标文件**:`docs/data-design.md`(260,849 字节,grep 显示 30+ 章节)
> **关键章节存在性核查**:§0-§10 + 接口稳定承诺;§4.1-§4.25 完整 25 Module DDL
> **test-design 实际引用**:**0 条具体 §X 引用**(除 §0.4 引用规则占位)

| # | test-design 引用 | 引用位置 | data-design 实际 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| D-1 | 《Data Design》§X 引用规则 | line 88(规则定义) | (仅规则定义,正文 0 处引用) | 规则名义化 — test-design §2.1.1 25 Module 列表未与 data-design §4.1-§4.25 做交叉验证;§3.1 / §3.3 / §7 RLS 等章节应在测试设计中体现但 0 引用 | **P1**(规则不落地) |
| D-2 | (隐含)Data Design §7 RLS 策略 | test-design §8.2 RLS Bypass 测试 code | **MATCH(隐含)** — data-design §7 = "Row-Level Security (RLS) 策略"(line 4571);§7.1 通用模板、§7.3 完整性验证清单(line 4637) | 引用存在但未显式打 §X,违反 §0.4 规则 | **P2** |
| D-3 | (隐含)Data Design §4.1-§4.25 25 Module DDL | test-design §2.1.1 25 Module 列表 | **MATCH(隐含)** — data-design §4.1-§4.25 完整 25 Module DDL(§4.1 tenant, ..., §4.25 local_runtime, line 4144) | 引用存在但未显式打 §X,违反 §0.4 规则 | **P2** |

**Data Design 乖离小计**:0 条具体 §X 引用 / 1 规则不落地 P1 / 2 隐含 MATCH P2

---

## 5. Security Design 引用乖离

> **目标文件**:`docs/security-design.md`(106,421 字节)
> **关键章节存在性核查**:
> - §0.5 = "接口稳定承诺(给 Phase 2 / Phase 3)"(line 118) — **存在但内容与正文多处不一致**(见 S-3)
> - §2 = "鉴权(Authentication)"(line 251) — 存在
> - §3 = "授权(Authorization)"(line 559) — 存在
> - §4 = "多租户隔离实施"(line 775) — 存在
> - §5 = "密钥与凭据管理"(line 913) — 存在
> - §6 = "输入校验"(line 1114) — 存在
> - §7 = "输出过滤"(line 1231) — 存在
> - §7.1 = PII 脱敏(line 1233)
> - §7.3 = AI Prompt / Response Retention Policy(line 1272) — **不是"生产数据严禁"**
> - §8 = "AI 数据边界"(line 1305) — 存在
> - §9 = "威胁模型实施"(line 1439) — **只有 §9.1 + §9.2.x,无 §9.3**
> - §9.2.1 = Prompt Injection(威胁 #1,line 1459)
> - §9.2.7 = Compromised Local Runtime(line 1552)
> - §9.2.8 = Local Runtime Fault(line 1568)
> - §9.2.11 = Fake Validation Result(威胁 #6,line 1602)
> - §10 = "审计与合规"(line 1675);§10.1 = "Audit 字段"(line 1679) — **不是"威胁 #6 Fake Validation"**
> - §5.5.2 = "8 种白名单命令"(line 1002)

| # | test-design 引用 | 引用位置 | security-design 实际 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| S-1 | 《Security Design》§2-§3 AuthN/AuthZ | line 547, 987 | **MATCH** — security-design §2 = "鉴权"(line 251),§3 = "授权"(line 559) | 引用 §X 存在,标题匹配 | MATCH |
| S-2 | 《Security Design》§4 Tenant Isolation / 13 类必带对象 | line 302, 548 | **MATCH** — security-design §4 = "多租户隔离实施"(line 775);§4.1 tenant_id 强制点 + §3.4 13 类必带对象授权控制(line 638) | 引用 §X 存在,标题匹配 | MATCH |
| S-3 | 《Security Design》§5 Secret Boundary | line 552 | **MATCH** — security-design §5 = "密钥与凭据管理"(line 913) | 引用 §X 存在,标题匹配 | MATCH |
| S-4 | 《Security Design》§7 Prompt Injection | line 551 | **错位** — security-design §7 = "输出过滤"(line 1231),不是 Prompt Injection;Prompt Injection 实际在 **§9.2.1**(line 1459,标题"Prompt Injection(威胁 #1,继承 §R-41,§28.3)") | §X 错位(§7 是输出过滤,不是 Prompt Injection) | **P1** |
| S-5 | 《Security Design》§7.3 生产数据严禁进入测试 | line 1192(§9.3) | **错位** — security-design §7.3 = "AI Prompt / Response Retention Policy"(line 1272),不是 PII/生产数据;PII 脱敏实际在 **§7.1**(line 1233) | §X 错位 | **P0** |
| S-6 | 《Security Design》§8 AI Provider Boundary | line 549, 1055 | **MATCH** — security-design §8 = "AI 数据边界"(line 1305);§8.1 Provider Data Boundary 配置矩阵 + §8.2 企业私有代码 Policy | 引用 §X 存在,标题匹配 | MATCH |
| S-7 | 《Security Design》§9.3 Local Runtime Security | line 550, 1093 | **不存在** — security-design **无 §9.3**(只有 §9.1 威胁分类总览 + §9.2.x);Local Runtime 实际在:<br>- §5.5 Local Runtime 凭权(mTLS + Command Token,line 987)<br>- §5.5.2 8 种白名单命令(line 1002)<br>- §9.2.7 Compromised Local Runtime(line 1552)<br>- §9.2.8 Local Runtime Fault(line 1568) | §X 不存在 — security-design 自身 §0.5 SEC-5 也错引 "9 问必答 AI Audit Metadata 字段 → §9.3" 但 §9.3 不存在 | **P0** |
| S-8 | security-design §10.1 威胁 #6 "Fake Validation" | line 874 | **错位** — security-design §10.1 = "Audit 字段"(line 1679),不是威胁 #6;Fake Validation Result 实际在 **§9.2.11**(line 1602,标题"Fake Validation Result(威胁 #6,继承 §R-27.1,§R-27.3,§34)");security-design §0.5 SEC-3 自身承诺也错引"6 大威胁类别完整覆盖 → §10.1",与正文不符 | §X 错位;**security-design 自身 §0.5 接口稳定承诺也错引** | **P0** |
| S-9 | 《Security Design》§5 §3.5 8 种白名单命令 | (test-design 未引) | security-design §0.5 SEC-4 声明"8 种 Local Runtime 白名单命令 → §5.5.2"(line 125)— 承诺内容正确,test-design 实际引到 runtime-design §12.1 同样 8 命令 | test-design 引用路径选 runtime-design §12.1,正确 | MATCH(隐含) |
| S-10 | 《Security Design》§0.5 SEC-5 "9 问必答 AI Audit Metadata 字段" | (test-design 未引,security-design §0.5 自身错引) | security-design §0.5 SEC-5 写"→ §9.3",但 §9.3 不存在;**实际 9 问必答在 §10.2 "AI Audit 9 问必答"**(line 1701) | security-design 自漂;test-design 未引用,无直接乖离 | (security-design 自身 P0,test-design 不直接受影响) |

**Security Design 乖离小计**:8 条具体引用 / 3 P0 / 1 P1 / 4 MATCH / 0 P2

---

## 6. Runtime Design 引用乖离

> **目标文件**:`docs/runtime-design.md`(71,680 字节)
> **关键章节存在性核查**:
> - §12.1 = "8 种白名单命令详解(继承《Basic Design》§6.3,D-03 修复)"(line 1373) — 存在
> - §12 = "安全边界(继承《Security Design》§9.3 Local Runtime 安全)"(line 1371) — 章节存在,但 **注:** 它自身也错引 Security Design §9.3(实际不存在)

| # | test-design 引用 | 引用位置 | runtime-design 实际 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| R-1 | 《Runtime Design》§12.1 8 种白名单命令 | line 1021(§8.2 Command Injection) | **MATCH** — runtime-design §12.1 = "8 种白名单命令详解"(line 1373),含 8 命令表格 + D-03 修复说明 | 引用 §X 存在,标题匹配 | MATCH |

**Runtime Design 乖离小计**:1 条 / 0 P0 / 0 P1 / 1 MATCH / 0 P2
(注:runtime-design §12 自身错引 Security Design §9.3,test-design 间接继承此错误,见 S-7)

---

## 7. Integration Design 引用乖离

> **目标文件**:`docs/integration-design.md`(57,696 字节)
> **test-design 实际引用**:**0 条具体 §X 引用**(除 §0.4 引用规则占位)

| # | test-design 引用 | 引用位置 | integration-design 实际 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| I-1 | 《Integration Design》§X 引用规则 | line 91(规则定义) | (仅规则定义,正文 0 处引用) | 规则名义化 — integration-design §2 SCM Adapter / §3 Agent Adapter / §4 Notification / §5 IdP Adapter / §6 第三方 SaaS / §7 错误处理 / §8 限流 / §9 测试策略 全部未在 test-design 中显式交叉引用 | **P1**(规则不落地) |
| I-2 | (隐含)Integration Design §9 测试策略 | test-design 应引用但未引用 | integration-design §9 = "测试策略"(line 1333);§9.1 Mock Server / §9.2 Contract Test / §9.3 Sandbox 账号管理 | 引用存在但未显式打 §X | **P2** |
| I-3 | (隐含)Integration Design §3 Agent Adapter 协议 | test-design §3.3 / §5.3 隐含 | integration-design §3 = "Agent Adapter(Codex / Claude Code / Gemini CLI / OpenAI Compatible / Local / Future)"(line 520) | 引用存在但未显式打 §X | **P2** |

**Integration Design 乖离小计**:0 条具体 §X 引用 / 1 规则不落地 P1 / 2 隐含 MATCH P2

---

## 8. AI/Agent Design 引用乖离

> **目标文件**:`docs/ai-agent-design.md`(69,072 字节)
> **关键章节存在性核查**:
> - §9 = "Provider Data Boundary(继承《Security Design》§8)"(line 1148) — 存在
> - §9.1 6 维 Policy(line 1150)
> - §9.2 Policy 等级(line 1161)
> - §9.3 Provider 选择算法(line 1172)
> - §9.4 强制点(line 1211)

| # | test-design 引用 | 引用位置 | ai-agent-design 实际 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| AI-1 | 《AI/Agent Design》§9 Provider Data Boundary | line 1055(§8.3 AI Provider 数据边界) | **MATCH** — ai-agent-design §9 = "Provider Data Boundary"(line 1148);§9.1 6 维 Policy + §9.2 Policy 等级 + §9.3 Provider 选择算法 + §9.4 强制点 | 引用 §X 存在,标题匹配 | MATCH |
| AI-2 | (隐含)AI/Agent Design §2 Context Compiler | test-design §3.1 / §5.3 / §8.3 隐含 | ai-agent-design §2 = "Context Compiler 详细设计"(line 138);§2.1-§2.6 | 引用存在但未显式打 §X | **P2** |
| AI-3 | (隐含)AI/Agent Design §4 AgentSession 14 状态 | test-design §2.1.1 隐含 | ai-agent-design §4 = "AgentSession 详细状态机(14 状态,继承《Basic Design》§7.4 + §24.1)"(line 518) | 引用存在但未显式打 §X | **P2** |
| AI-4 | (隐含)AI/Agent Design §5 Feedback Instruction Generator | test-design §3.1 / §5.3 隐含 | ai-agent-design §5 = "Feedback Instruction Generator"(line 678);§5.2 5 段式结构 | 引用存在但未显式打 §X | **P2** |

**AI/Agent Design 乖离小计**:1 条具体 §X 引用 / 0 P0 / 0 P1 / 1 MATCH / 3 隐含 MATCH P2

---

## 9. 附:test-design 引用但**不在 7 引用规则中**的文档(规则缺口)

> test-design §0.4 仅声明 7 份设计书的引用规则,但正文实际引用了 **External Design** 和 **Operation Design** — 规则不自洽。

| # | test-design 引用 | 引用位置 | 目标文件实际 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| EX-1 | 《External Design》§4 关键流程(6 个) | line 381, 385, 719 | **MATCH** — external-design.md §4 = "关键用户流程"(line 744);§4.1 WorkItem 创建 Worktree + §4.2 分配 Worktree 给 Agent + §4.3 Agent 修改后 Review + §4.4 处理 Feedback Inbox + §4.5 处理 Conflict + §4.6 Merge PR — 与 test-design "6 个关键流程" 完全一致 | 引用 §X 存在,标题匹配;**但 §0.4 规则未声明** | MATCH(规则缺口 P1) |
| EX-2 | 《External Design》§3.2 Worktree Control Center 关键交互 | line 731 | **MATCH** — external-design.md §3.2 = "Worktree Control Center(主页)"(line 289) | 引用 §X 存在,标题匹配;**但 §0.4 规则未声明** | MATCH(规则缺口 P1) |
| OP-1 | 《Operation Design》§6 Prometheus + Grafana Dashboard | line 948 | **MATCH(松)** — operation-design.md §6 = "可观察性"(line 965);§6.1 Metrics (Prometheus)(line 967) + §6.4 仪表盘(Grafana)(line 1061) | 引用 §X 存在,标题匹配(§6 是总章,Prometheus/Grafana 在 §6.1/§6.4,test-design 用 §6 上位引用可接受);**但 §0.4 规则未声明** | MATCH(规则缺口 P1) |
| OP-2 | 《Operation Design》§6 Synthetic Monitoring | line 1452 | **MATCH(松)** — 同 OP-1 | 同 OP-1;**但 §0.4 规则未声明** | MATCH(规则缺口 P1) |
| R-1 | 《Requirements》§44 PR 门禁 | line 119 | (Requirements 不在 7 引用规则) | 规则缺口 | P1 |

**外部引用乖离小计**:5 条 / 0 P0 / 4 规则缺口 P1 / 1(Requirements)规则缺口 P1

---

## 10. 已知缺口 / 无法验证

### 10.1 自验证可还原性
所有上述乖离 P0/P1 都通过 `grep` + `read` 实证(`Get-Content` / `Select-String` 通过 PowerShell 5.1 默认 ANSI 解码,本审计在 CJK 重灾区用 `read` 工具的 UTF-8 路径规避)。如需第三方复核:

```powershell
# B-3/B-4/B-5 §4.5.6 vs §4.5.5
Select-String -Path docs/basic-design.md -Pattern '^(####|###|##) 4\.5\.[5-6]\b'

# B-1 §37 / B-2 §44 / B-8 §18 / B-9 §27.4 不存在
Select-String -Path docs/basic-design.md -Pattern '^(##|###|####) (37|44|18|27\.3|27\.4)\b'

# S-7 §9.3 不存在
Select-String -Path docs/security-design.md -Pattern '^(##|###) 9\.[3-9]\b'

# S-5 §7.3 内容核对
Select-String -Path docs/security-design.md -Pattern '^(###) 7\.3\b'

# S-8 §10.1 错位
Select-String -Path docs/security-design.md -Pattern '^(###) 10\.1\b'  # 应是 "Audit 字段"
Select-String -Path docs/security-design.md -Pattern '^#### 9\.2\.11\b' # 实际 "Fake Validation"
```

### 10.2 暂未对账
- **test-design §6.3.3 / §6.3.4** 引用的 `§8.2 REQ-WF-003` / `§8.2 ExecuteArbitraryShell` / `§18 Integration Webhook` / `§27.4 ReviewRecord` / `§8.3 ❌ ExecuteArbitraryShell` (line 1482) — 5 处裸 §X 全部在 basic-design 中找不到对应章节,**已纳入 P0/P1**,但 §8.3(RLS Bypass 测试,line 1482)可能是 test-design 内部章节 §8.2 RLS Bypass 测试的笔误,需用户确认
- **test-design §2.1.1 25 Module 列表** 与 **basic-design §2.1 / data-design §4.1-§4.25** 的 25 Module 名称 + 顺序一致性未做逐字交叉(目测一致,但缺逐字 diff 证据)
- **security-design §0.5 自身错引**:§0.5 SEC-5 写"9 问必答 → §9.3" 但 §9.3 不存在;§0.5 SEC-3 写"6 大威胁 → §10.1" 与 §10.1 内容错位 — security-design 自漂,**不计入 test-design 乖离但应在 security-design v0.x 升版时修**

### 10.3 不在 7 引用规则但被引用的隐藏乖离
- test-design §15 文档元信息(line 1494-1503) 引用 `§0.3` `§3.1` `§8.1` `§2.2.1` `§5.3` `§8.3` `§8.4` `§3.3` `§27.2` 等内部章节号 — 这些是 test-design 自身章节引用,不属于对账范围,**未纳入**

### 10.4 test-design v0.2 修订历史引用
- test-design line 9: "v0.2 | 2026-08-26 | 同步 basic-design 5f1ea5b(5 个同步项对应测试点已落位,详见 §X 上游同步测试)" — "§X" 占位符,未指明具体章节,自描述性 gap
- test-design line 10: "v0.3 | 2026-08-31 | 同步 requirements.md 98db08e(线程 C:Design Artifact / Test Level / Incident Record,详见 §上游同步 2026-08-31;basic-design 尚未跟进,字段细节标 TBD)" — 显式说明 basic-design 尚未跟进,**与本审计发现的 B-1~B-9 乖离自洽**(basic-design 缺 5f1ea5b→98c73b1 区间更新,test-design 引用的 §37/§44 等是 requirements 章节号,在 basic-design 中本就不存在)

---

## 11. 修复建议(本审计只读,以下为建议,非 commit)

### P0 必修(影响 P0 不变量 / RFC 边界)
1. **B-3 / B-4 / B-5**:test-design §6.3.2 + §14 #16 + 修订历史 3 处将 "basic-design §4.5.6" 改为 "basic-design §4.5.5";"§27.3" 改为 "requirements.md §27.3";删除 "§0.5 接口稳定承诺"(basic-design 无 §0.5,接口稳定承诺是 top-level ## 标题)
2. **B-1 / B-2 / B-8 / B-9**:basic-design 章节号 §37 / §44 / §18 / §27.4 在 basic-design.md 中不存在,test-design 引用应改为 "requirements.md §X" 或补一句 "见 requirements.md 章节"
3. **S-7**:"Security Design §9.3 Local Runtime Security" 应改为 "Security Design §5.5.2(8 种白名单命令)+ §9.2.7-9.2.8(Local Runtime 威胁)"
4. **S-5**:"Security Design §7.3 生产数据严禁" 应改为 "Security Design §7.1(PII 脱敏)+ §7.2(Secret 脱敏)"
5. **S-8**:"security-design §10.1 威胁 #6 Fake Validation" 应改为 "security-design §9.2.11(威胁 #6 Fake Validation)"

### P1 建议修(规则一致性)
6. **B-6 / B-7**:test-design §6.3.3 + §6.3.4 的 "§8.2" 引用全部更正:
   - "§8.2 REQ-WF-003" → "basic-design §4.9.3(引用 requirements §8.2 REQ-WF-001)" 或直接 "requirements §8.2 REQ-WF-003"
   - "§8.2 ExecuteArbitraryShell" → "basic-design §6.3(默认禁止 SaaS Server → Arbitrary Shell)"
7. **S-4**:"Security Design §7 Prompt Injection" 应改为 "Security Design §9.2.1(威胁 #1 Prompt Injection)"
8. **§0.4 规则缺口**:补《External Design》《Operation Design》《Requirements》到 §0.4 引用规则清单,或显式说明这三份不在 7 引用规则内
9. **格式漂移**:test-design §2.5.2 + §6.3.3 + §6.3.4 + §14 中裸 `§X` 形式应统一加 `《X Design》` 前缀
10. **D-1 / I-1**:Data Design / Integration Design 0 引用应在 test-design §3(单元测试策略)+ §4(集成测试策略)+ §6(验收测试) 中显式打 §X 引用

### P2 / 隐含 MATCH(可后置)
- data-design §4.1-§4.25 / §7 / ai-agent-design §2 / §4 / §5 / integration-design §3 / §9 应在 test-design v0.4 升版时显式打 §X,避免"规则名义化"

---

[delta done] total=28, p0=11, p1=8, p2=8, unverified=1
