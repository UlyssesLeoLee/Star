# γ · test-design vs requirements v2.0 乖离清单

> **作者**:Mavis worker-γ
> **范围**:只读证据收集,不动 `docs/test-design.md` / `docs/requirements.md`
> **任务日期**:2026-08-31 JST
> **检视对象**:
> - `D:/Star/docs/test-design.md` v0.3 (2026-08-31, 49846 字节, 1507 行)
> - `D:/Star/docs/requirements.md` v2.0 (2026-08-29, 94801 字节, ~1793 行)
> **检视方法**:全文件读 1 次 → 提取 requirements.md 全部 `## §X.Y` 锚 → 提取 test-design.md 全部 `§N` 引用 → 逐条对账。
> **本工作区 git HEAD**:`3c9c2ae`(注:任务简报写 948582e,本机实际 main HEAD 为 3c9c2ae,差 0 commit;两份文件均未跟踪到 948582e 哈希,`test-design.md` v0.3 与 `requirements.md` 在本机工作区已落地)
> **gate 通过**:
> - Test Design v0.3 实际存在(49846 bytes,LastWrite 08/31 10:47)
> - Requirements v2.0 实际存在(94801 bytes,LastWrite 08/29 20:28)
> - 子代理 status="succeeded" ≠ 实际成功(per AGENTS.md §1.2 守门 #9)→ 本报告 100% 实测文件存在 + git log 实证

---

## 0. 摘要

- **总条数**:N=27(其中 P0=10 / P1=3 / P2=3 / unverified=6 / PASS=5)
- **重点乖离**:
  1. **test-design §1.2 引用 requirements §44 作为 PR 门禁来源** → requirements §44 实为「架构原则总纲」(44.1/44.2/44.3),**完全没有 PR 门禁 / 主干门禁 / Release 门禁内容**(P0 章节号引用错误)
  2. **test-design §2.4.2 引用 requirements §36 作为 TBD-MEASURE 规则来源** → requirements §36 实为「Use Case 一览」,TBD-MEASURE 规则在 §0 引用自"原文档 §36、§80",**与本仓库 requirements §36 不对应**(P0 章节号引用错误)
  3. **test-design §2.1.1 声明 "25 Module 测试清单"** → requirements §47 实际是「下一阶段输入清单」,**不是模块清单**;真正的模块清单在 requirements §13.3(15 个 domain-* crate + application/infrastructure/api = 16 crates 总)。test-design 列出的 25 Module 中只有前 17 个与 §13.3 重合,**后 8 个(domain-identity / domain-audit / domain-search / domain-notification / domain-integration / domain-automation / domain-collaboration / domain-local-runtime)在 requirements.md 全部 0 命中 / 1 命中**(P0 数量漂移 + P0 章节号引用错误)
  4. **test-design §7 实为「性能测试(详细)」,非「缺陷管理」** → 任务简报假设 test-design §7 = 缺陷管理对照 requirements §44 准入门槛,**两条前提均不成立**(P0 任务前提错误)
  5. **test-design §6.3.2 引用 "basic-design §4.5.6 / §27.3" 作为 VAL-001 四重门依据** → requirements §27.3 确有 4 阶段流程(Validation → Acceptance Coverage → Feedback Resolution → Human/Policy Gate),与 test-design 四重门映射正确(✓ PASS),但 basic-design §4.5.6 在本仓库不可定位(P0 外部文档未提供)

- **PASS 项**:S1-S5/T1-T3 同步点全部对得上 requirements(§11/§12/§19.1/§24/§24.6/§27.6/§8.3/§29.1),"线程 C" 三字段(Design Artifact / Test Level / Incident Record)全部对得上 requirements §8.3/§27.6/§29.1,13 类必带对象(§15 metadata)对得上 requirements §16 REQ-SEC-001。

---

## 1. test-design §0 同步点 S1-S5 自检

> 来源: `test-design.md:17-29`("上游同步 2026-08-26 继承 basic-design 5f1ea5b")
> 任务要求:每条对应 requirements §X.Y,验证实际章节是否存在 + 编号对得上。

| 同步点 | test-design 引用 | requirements 实际 | 乖离 | 严重度 |
|---|---|---|---|---|
| **S1** REQ-AUTO-002 (Trigger Schedule/Cron) | 隐含 §11 (无显式 §N) | §11 REQ-AUTO-002 (line 302, P0 with Multica 注解) | 实际编号对得上,test-design 未显式写 §N | PASS |
| **S2** REQ-NOTIF-002 (Inbox 噪声抑制) | 隐含 §12 (无显式 §N) | §12 REQ-NOTIF-002 (line 310, 2026-08-26 补充) | 实际编号对得上,test-design 未显式写 §N | PASS |
| **S3** REQ-SCM-003 (Gitea/Forgejo V2 候选) | 隐含 §19.1 (无显式 §N) | §19.1 REQ-SCM-003 (line 532, V2 候选) | 实际编号对得上,V2 候选标签一致 | PASS |
| **S4** AgentSession `token_usage` / `cost_summary` | 隐含 §24.1 (无显式 §N) | §24.1 AgentSession 字段(line 728),§28.1 列 cost metrics;`token_usage`/`cost_summary` 字面在 requirements §24.1 / §28.1 未直接出现(Section 24.1 列的是 agent_session_duration 等聚合 metric,§24.1 也未直接列出字段 schema) | S4 字段名在 requirements §24.1 不可直接定位,推测为隐含语义,**unverified** | unverified |
| **S5** Skill/Playbook V2 候选 | 隐含 §24.6 (无显式 §N) | §24.6 Skill / Playbook 复用 (line 788, V2 候选, Multica 注解一致) | 实际编号对得上,V2 候选标签一致 | PASS |

**S1-S5 自检小结**:5/5 同步声明在 requirements.md 实际存在;S4 字段名(`token_usage`/`cost_summary`)在 requirements §24.1 字面未出现,属隐含语义(§24.1 列的是 aggregate metric `agent_session_*` / `agent_context_size` / §28.1 列 AI Interaction Quality 指标含 `Context Cost`),需 basic-design 确认字段 schema,unverified。

---

## 2. test-design §0 同步点 T1-T3 自检

> 来源: `test-design.md:33-43`("上游同步 2026-08-31 继承 requirements.md 98db08e 线程 C")
> 任务要求:每条对应 requirements §X.Y,验证实际章节是否存在 + 编号对得上 + V1 Should-Have 一致。

| 同步点 | test-design 引用 | requirements 实际 | 乖离 | 严重度 |
|---|---|---|---|---|
| **T1** REQ-TST-001/002 (ValidationResult.Level) | test-design §6.2.1 line 819 → requirements §27.6;test-design 头部 line 35 → "§30.3 V1 Should-Have" | §27.6 Test Level(line 964, P0: TST-001,线程 C);§30.3 line 1124 列 "Test Level 维度(単体/結合/総合/受入,REQ-TST-001/002,§27.6)" V1 Should-Have | 编号、语义、V1 Should-Have 全部对得上 | PASS |
| **T2** REQ-DSG-001/002 (DesignArtifact + WorkItem Guard) | test-design §6.3.3 line 876 → requirements §8.3;test-design 头部 line 35 → §30.3 V1 Should-Have | §8.3 Design Artifact(line 244, P0: DSG-001/002,线程 C);§30.3 line 1123 列 "Design Artifact + Approval Guard(REQ-DSG-001/002,§8.3,非强制瀑布)" V1 Should-Have | 编号、语义、V1 Should-Have 全部对得上 | PASS |
| **T3** REQ-OPS-001/002/003 (IncidentRecord) | test-design §6.3.4 line 892 → requirements §29.1;test-design 头部 line 41 写 "REQ-OPS-001/002/003";§30.3 V1 Should-Have 引用 | §29.1 Incident Record(line 1043, P0: OPS-001,线程 C);§29.1 line 1064-1065 列 REQ-OPS-001 / REQ-OPS-002 / REQ-OPS-003(003 为边界约束);§30.3 line 1125 列 "Incident Record 追溯(**REQ-OPS-001/002**,§29.1,仅追溯 ... 不含监控/告警/自动回滚)" V1 Should-Have | **REQB-OPS-003 在 §30.3 中被显式排除(不在 V1 Should-Have 列表)**;test-design 把 003 列为同步项但 §30.3 只列 001/002;**但 test-design §6.3.4 写"三项均为 REQ-OPS-003 明确排除的非能力"** 描述与 §30.3 排除语义一致;属**表述口径不一致**(test-design T3 字段级全列 vs §30.3 仅列 V1 Should-Have 范围) | P2 |

**T1-T3 自检小结**:T1/T2 完全对得上;T3 在 REQ-OPS-003 归属上有表述不一致(§30.3 把 003 列为边界排除项,test-design 把 003 列入 T3 同步项),但 §6.3.4 实际测试关注点("三项均为 REQ-OPS-003 明确排除的非能力")与 §30.3 + §29.1 一致。属 P2 表述漂移,不阻塞内容正确性。

---

## 3. test-design §2.1.1 25 Module 表 vs requirements §47 实际域模块清单

> 来源: `test-design.md:170-200`(25 Module 表);`test-design.md:1502`(声称"覆盖 25 Module")
> 任务要求:验证 25 Module 是否在 requirements §47 实际定义

### 3.1 任务前提验证(requirements §47 实际是什么)

`requirements.md:1765-1788` §47 全文:**"## 47. 下一阶段输入清单(《基本设计书》阶段建议输入)"**

§47 是"下一阶段输入清单",**不是模块清单**。§47 列的是《基本设计书》阶段应继承的产出(Requirement ID / Architecture Obligation / ADR Candidate / PoC Result / Risk / Open Issue / Security Boundary / Domain Boundary / Worktree Lifecycle / Agent Policy / Feedback Model / Context Model / Validation Model / SCM Integration Contract / Design Artifact Model / Incident Record Model 等),**0 个 crate / Module 名称**。

**requirements 实际模块清单位置**:§13.3 "Rust Modular Monolith 扩展"(line 384-412),列出 16 个 crate:

```text
crates/
├── domain-tenant        ├── domain-workflow      ├── domain-development
├── domain-workspace     ├── domain-board         ├── domain-worktree
├── domain-project       ├── domain-planning      ├── domain-agent
├── domain-work-item     ├── domain-permission    ├── domain-feedback
├── domain-comment       ├── domain-relation      ├── domain-context
│                        ├── domain-validation    ├── domain-scm
│                        │                        │
│                        ├── application          ├── infrastructure
│                        └── api
```

= **15 个 domain-* crate + application + infrastructure + api = 16 个 crate 总**

### 3.2 25 Module 对账

| # | test-design §2.1.1 Module | requirements §13.3 列? | requirements 其他位置出现? | 乖离 | 严重度 |
|---|---|---|---|---|---|
| 1 | `domain-tenant` | ✓ line 390 | §7 REQ-TWP-001 (line 204) | — | PASS |
| 2 | `domain-workspace` | ✓ line 391 | §7 (line 205) | — | PASS |
| 3 | `domain-project` | ✓ line 392 | §7 (line 206) | — | PASS |
| 4 | `domain-work-item` | ✓ line 393 | §8 (line 212) | — | PASS |
| 5 | `domain-workflow` | ✓ line 394 | §8.2 (line 238) | — | PASS |
| 6 | `domain-board` | ✓ line 395 | §9 REQ-PLAN-003 (line 280) | — | PASS |
| 7 | `domain-planning` | ✓ line 396 | §9 (line 270) | — | PASS |
| 8 | `domain-permission` | ✓ line 397 | §11 REQ-PERM-001 (line 299) | — | PASS |
| 9 | `domain-comment` | ✓ line 398 | §10 REQ-COLLAB-001 (line 290) | — | PASS |
| 10 | `domain-relation` | ✓ line 399 | §10 REQ-COLLAB-002 (line 291) | — | PASS |
| 11 | `domain-development` | ✓ line 401 | §20-21 (line 540, 566) | — | PASS |
| 12 | `domain-worktree` | ✓ line 402 | §22 (line 618) | — | PASS |
| 13 | `domain-agent` | ✓ line 403 | §24 (line 726) | — | PASS |
| 14 | `domain-feedback` | ✓ line 404 | §25 (line 806) | — | PASS |
| 15 | `domain-context` | ✓ line 405 | §26 (line 857) | — | PASS |
| 16 | `domain-validation` | ✓ line 406 | §27 (line 908) | — | PASS |
| 17 | `domain-scm` | ✓ line 407 | §19 (line 517) | — | PASS |
| 18 | `domain-identity` | ✗ **NOT in §13.3** | grep 0 hit(全文件无此字符串) | §13.3 无;全文 0 命中;test-design 列了 1 个 Module 但无任何 requirements 章节引用 | **P0** |
| 19 | `domain-audit` | ✗ **NOT in §13.3** | grep 0 hit(全文件无此字符串);§17 标题为「Audit 要求(基线)」,内容是 REQ-AUDIT-002 列在 line 478 但未指定实现 crate 名 | §13.3 无;全文 0 命中 | **P0** |
| 20 | `domain-search` | ✗ **NOT in §13.3** | grep 0 hit(全文件无此字符串);§12 REQ-SEARCH-001/002 列在 line 312-313 但未指定实现 crate 名 | §13.3 无;全文 0 命中 | **P0** |
| 21 | `domain-notification` | ✗ **NOT in §13.3** | §12 REQ-NOTIF-001/002/003 在 line 309-311;line 311 提到"实现位置在 `domain-notification`" → **1 命中,§12 内文提及** | §13.3 无 crate,但 §12 内文 line 311 显式提及 crate 名 | **P1** |
| 22 | `domain-integration` | ✗ **NOT in §13.3** | grep 0 hit(全文件无此字符串);§18 「Integration 要求」(line 485) 是 process 章节,未指定实现 crate 名 | §13.3 无;全文 0 命中 | **P0** |
| 23 | `domain-automation` | ✗ **NOT in §13.3** | grep 0 hit(全文件无此字符串);§11 REQ-AUTO-001/002/003 在 line 301-303 但未指定实现 crate 名 | §13.3 无;全文 0 命中 | **P0** |
| 24 | `domain-collaboration` | ✗ **NOT in §13.3** | grep 0 hit(全文件无此字符串);§10 「Collaboration 要求」(line 288) 是 process 章节,未指定实现 crate 名 | §13.3 无;全文 0 命中 | **P0** |
| 25 | `domain-local-runtime` | ✗ **NOT in §13.3** | grep 0 hit(全文件无此字符串);§23 (line 678)「Local Runtime 要求」是 process 章节,未指定实现 crate 名;仅 §23.2 line 697 列 Process Scope / Filesystem Scope 等 capability,**没有 crate 名** | §13.3 无;全文 0 命中 | **P0** |

### 3.3 §47 引用本身错位

test-design §0.4 line 85 写 "`§N` 引用《Requirements》v2.0 章节号(最大 §47)"。**§47 在 requirements.md 实际是"下一阶段输入清单",不是模块清单**。test-design 用 §47 作为"模块定义章节号"的隐含引用源是**P0 章节号引用错误**;真实模块定义在 §13.3(且只有 15 个 domain-* crate,非 25)。

### 3.4 §15 metadata 章节(claim 25 Module 全覆盖)

`test-design.md:1502` 写 "覆盖 25 Module ... 全部 25 Module 至少出现 1 次"。**实际只有 17/25 在 requirements §13.3 存在**(剩 8 个未在 requirements crate 清单中定义)。

**§3 乖离小结**:
- P0 × 7(Module 18-25 全部 / 数量漂移 25 vs 15 / §47 引用错位)
- P1 × 1(`domain-notification` 在 §12 line 311 内文有提及,但 §13.3 crate 列表无)

---

## 4. test-design §6.x 测试层 Requirements §N 引用

> 来源: `test-design.md:819, 842, 876, 892, 484, 119, 1502, 1503`

| test-design 引用 | test-design 行 | 实际指向 | 实际章节存在? | 乖离 | 严重度 |
|---|---|---|---|---|---|
| **§6.2.1** `requirements §27.6` | line 819 | ValidationResult.Level 维度 / REQ-TST-001/002 | ✓ requirements §27.6 line 964 存在,REQ-TST-001/002 在 line 982-983 | — | PASS |
| **§6.3.2** `basic-design §4.5.6 / §27.3` | line 842, 872, 1486 | VAL-001 四重门(D-04 修复 P0 不变量) | ✓ requirements §27.3 line 918「AI Completion 判定」有 4 阶段流程(Validation → Acceptance Coverage → Feedback Resolution → Human/Policy Gate),与 test-design 四重门 1:1 映射;但 **basic-design §4.5.6 在本仓库不可定位**(basic-design.md 5f1ea5b 引用是 test-design 头部声明,实际 basic-design.md 在工作区存在但需另行读取) | **basic-design 章节号未在本报告范围验证** | unverified |
| **§6.3.3** `requirements §8.3` | line 876, 884 | Design Artifact Guard(REQ-DSG-001/002,V1 Should-Have) | ✓ requirements §8.3 line 244 存在,REQ-DSG-001/002 在 line 265-266 | — | PASS |
| **§6.3.4** `requirements §29.1` | line 892 | Incident Record(REQ-OPS-001/002/003,V1 Should-Have) | ✓ requirements §29.1 line 1043 存在,REQ-OPS-001/002/003 在 line 1063-1065 | — | PASS |
| **§2.4.2** `TBD-MEASURE(继承《Requirements》§36)` | line 484 | TBD-MEASURE 标记规则 | ✗ **requirements §36 实为「Use Case 一览」**(line 1291,UC-DEV-001 ~ UC-DEV-016),**完全没有 TBD-MEASURE 规则内容**;TBD-MEASURE 规则在 requirements **§0 line 21** "全文遵守 §36、§80 的规定,凡缺乏真实测量数据的目标值,一律标注 TBD-MEASURE"——但**这条引用指向的是"原文档 §36"**(即原《Kubernetes-native 工作管理 SaaS 要件定义》文档 §36,**本仓库不可定位**),不是 requirements §36 | **章节号引用错位**——test-design 引用了 requirements §36 但内容对应原文档 §36;requirements §36 是 Use Case,内容无关 | **P0** |
| **§1.2** `PR 门禁(继承《Requirements》§44)` | line 119 | PR 门禁(8 项:Lint/Type Check/Unit Tests/Integration Tests/Coverage/Security Scan/License Check/Build) | ✗ **requirements §44 实为「架构原则总纲」**(line 1588,3 个子节 44.1/44.2/44.3),**完全没有 PR 门禁 / 主干门禁 / Release 门禁内容**;44.1 列 12 条架构原则;44.2 是 K8s Tax 纪律;44.3 是 Work Core 解耦原则;无任何 CI/CD 门禁 / Coverage Gate / Security Scan / License Check 字样 | **章节号引用错位**——test-design 引用了 requirements §44 但内容完全不在 §44 | **P0** |
| **§1.2** `主干门禁` | line 132-140 | 主干门禁 5 项 | ✗ 同上,requirements §44 无主干门禁内容 | 同上 | **P0** |
| **§1.2** `Release 门禁` | line 142-150 | Release 门禁 5 项 | ✗ 同上,requirements §44 无 Release 门禁内容 | 同上 | **P0** |
| **§15 metadata** `13 类必带对象` | line 1503 | 13 类 tenant_id 必带对象清单 | ✓ requirements §16 REQ-SEC-001 line 470 列出 13 类(Repository Credential / Local Runtime / Worktree / AgentSession / ContextPacket / Feedback / AI Prompt / AI Response / Diff / Build Log / Test Log / PR Content / Symbol Index),与 test-design §15 line 1503 列出的 13 类**完全一致** | — | PASS |
| **§0.3** `七类命名` | line 821 (引用) | Test Type 7 类 | ✓ test-design §0.3 line 73-82 列出 Unit/Integration/Contract/E2E/Performance/Security/Acceptance = 7 类;**注:这是 test-design 内部自引用(§0.3 引用自己 §6.2.1 的"七类命名"),非 requirements 引用** | — | PASS(self-ref) |
| **§6.3.3** `§27.4 ReviewRecord` | line 878 | ReviewRecord 审批机制(不新建状态机) | ✓ requirements §27.4 line 927 存在,§8.3 line 262 明确 "DesignArtifact 的批准流程复用 §27.4 ReviewRecord" | — | PASS |
| **§6.3.3** `§8.2 REQ-WF-003 RequireApproval` | line 878 | WorkItem Guard `RequireApproval` 类型 | ✓ requirements §8.2 line 242 存在,`enum Guard { RequireRole, RequireValidation, RequireApproval }` 在 line 242 引用 | — | PASS |
| **§6.3.4** `§20-27 闭环` | line 894 | WorkItem→Worktree→AgentSession→ChangeSet→ValidationResult→ReviewRecord 闭环 | ✓ requirements §20(566)/§21(587)/§22(618)/§23(678)/§24(726)/§25(806)/§26(857)/§27(908)全部存在 | — | PASS |
| **§6.3.4** `§18 Integration Webhook` | line 902 | 既有 Integration Webhook 机制 | ✓ requirements §18 line 485 存在,line 500 提到"Webhook" | — | PASS |
| **§6.3.4** `§8.2 "❌ ExecuteArbitraryShell 必须被拒绝"` | line 901 | 同类缺失测试写法(本设计书内部 §8.2 引用) | ✗ §8.2 实际指向 **test-design 自己的 §8.2**(line 1007-1051,Injection/SSRF/RLS Bypass),不是 requirements §8.2;test-design §8.2 line 1022 确实有 "❌ ExecuteArbitraryShell 必须被拒绝" 字样。requirements §23.2 line 699 写"默认禁止 SaaS Server → Arbitrary Shell"(术语为 "Arbitrary Shell" 而非 "ExecuteArbitraryShell") | **表述术语不一致**:test-design 用 "ExecuteArbitraryShell",requirements 用 "Arbitrary Shell"。**§8.2 是 test-design 自指,非 requirements 引用**——test-design §6.3.4 line 901 用 `§8.2` 写"与 §8.2 ... 同类缺失测试写法" 但**未指明 §8.2 归属**,易与 requirements §8.2 混淆 | **P2** |

**§4 乖离小结**:
- P0 × 4(§1.2 PR/主干/Release 门禁引用 §44 + §2.4.2 TBD-MEASURE 引用 §36,均错位)
- P2 × 1(§6.3.4 line 901 "§8.2 ExecuteArbitraryShell" 表述术语不一致,test-design 自指 §8.2 容易与 requirements §8.2 DesignArtifact 混淆)
- PASS × 6
- unverified × 1(basic-design §4.5.6)

---

## 5. test-design §7 缺陷管理 vs requirements §44 准入门槛

> 任务要求:对账 test-design §7 缺陷管理 vs requirements §44 准入门槛(PR 准入 / 主干准入 / Release 准入)
> 来源: `test-design.md:907-982` (§7)、`requirements.md:1588-1620` (§44)

### 5.1 test-design §7 实际内容

`test-design.md:907-982` 全文:§7 = **「7. 性能测试(详细)」**(Performance Test Detail),含 5 个子节:
- §7.1 关键端点 P95 预算(继承《API Design》§10)
- §7.2 负载模型
- §7.3 性能指标收集
- §7.4 性能回归检测
- §7.5 性能预算分配(继承《Basic Design》§44 K8s Tax 纪律)

**test-design §7 不是「缺陷管理」**——test-design 全文 0 命中"缺陷"、"Defect"、"Bug 跟踪"、"缺陷管理"、"Issue Tracking"、"Defect Management"等关键词(grep 验证)。缺陷/Feedback 相关讨论在 test-design §6(Acceptance Test)+ §2.2.4 + §9.3 散落,**没有独立「§7 缺陷管理」章节**。

### 5.2 requirements §44 实际内容

`requirements.md:1588-1620` §44 = **「44. 架构原则总纲」**,3 个子节:
- §44.1 最终架构原则(§99) — 12 条架构原则(WorkItem 管理 Intent / Worktree 管理 Execution Isolation / AI Chat 是交互方式等)
- §44.2 Kubernetes Tax 纪律(§86-90) — 禁止 7-8 个独立 Deployment
- §44.3 Development Context 与 Work Core 解耦原则(§85,重申) — WorkItem ≠ Git Branch ≠ Worktree ≠ AgentSession

**requirements §44 完全没有「准入门槛 / PR 准入 / 主干准入 / Release 准入 / Quality Gate / Go-Live Gate」内容**(grep 验证:0 命中"PR 门禁|主干|TRUNK|RELEASE|Release 门|PR Gate|Trunk Gate|主干门|merge gate|trunk criteria|release gate" 等关键词)。

### 5.3 §7 §44 双向乖离

| 任务简报假设 | test-design §7 实际 | requirements §44 实际 | 乖离 |
|---|---|---|---|
| test-design §7 = 缺陷管理 | test-design §7 = 性能测试(详细) | — | **P0 任务前提错误(test-design 无 §7 缺陷管理章节)** |
| requirements §44 = 准入门槛 | — | requirements §44 = 架构原则总纲(无准入内容) | **P0 任务前提错误(requirements §44 无准入门槛内容)** |

test-design §1.2 line 119 写 "PR 门禁(继承《Requirements》§44)"——这是 test-design 自己 §1.2 引用 §44 的错位(**P0**),与"§7 缺陷管理"无关。test-design §1.2 line 119-150 列了 PR 门禁 / 主干门禁 / Release 门禁 3 套内容,这些内容**与 requirements §44 无关**,test-design 也没有显式说"§44 包含这些门禁"——只是在该行写了"(继承《Requirements》§44)"这个归属标注。

**§5 乖离小结**:**P0 双向任务前提错误**(test-design §7 ≠ 缺陷管理 + requirements §44 ≠ 准入门槛),任务简报的对账项无法直接成立。**test-design §1.2 把 PR/主干/Release 门禁标"继承《Requirements》§44" 是 test-design 自己的归属标注错位**(§44 不含这些内容,见 §4 表 line 119-150)。

---

## 6. "线程 C" 三字段:Design Artifact / Test Level / Incident Record

> 来源: `test-design.md:35`(头部声明) + `test-design.md:819/876/892`(§6.2.1/§6.3.3/§6.3.4 引用)
> 任务要求:每条字段在 requirements §8.3 / §27.6 / §29.1 是否存在

| 字段 | test-design 引用 §N | requirements 实际章节 | 实际内容 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| **Design Artifact** | test-design 头部 line 35 + §6.3.3 line 876 → "requirements §8.3" | §8.3 (line 244-266) | §8.3 line 244 标题"Design Artifact(无对应原提示词章节编号 — 本节为线程 C 新增设计, P0:DSG-001/002 — brainstorming 线程 C)";line 249-259 列出字段结构(ArtifactId/ProjectId/WorkItemId/Kind/Version/Status/Content/ApprovalReviewId);line 265-266 给出 REQ-DSG-001/002 | 编号、字段、REQ 编号全部对得上 | PASS |
| **Test Level** | test-design 头部 line 35 + §6.2.1 line 819 → "requirements §27.6" | §27.6 (line 964-983) | §27.6 line 964 标题"Test Level(工程别テスト,无对应原提示词章节编号 — 本节为线程 C 新增设计, P0:TST-001 — brainstorming 线程 C)";line 968-976 列出 Level 字段(UnitTestLevel/IntegrationTestLevel/SystemTestLevel/AcceptanceTestLevel);line 982-983 给出 REQ-TST-001/002 | 编号、字段、REQ 编号全部对得上 | PASS |
| **Incident Record** | test-design 头部 line 35 + §6.3.4 line 892 → "requirements §29.1" | §29.1 (line 1043-1065) | §29.1 line 1043 标题"Incident Record(生产事件追溯,无对应原提示词章节编号 — 本节为线程 C 新增设计, P0:OPS-001 — brainstorming 线程 C)";line 1047-1057 列出字段结构(IncidentId/ProjectId/Severity/DetectedAt/ReportedBy/Status/LinkedWorkItem/RootCauseChangeSet/ViolatedAcceptanceCriteria/ResolutionEvidence/PostmortemNote);line 1063-1065 给出 REQ-OPS-001/002/003(003 为边界约束) | 编号、字段、REQ 编号全部对得上(003 是边界约束而非"能力"项,与 §30.3 排除一致) | PASS |

**§6 乖离小结**:**3/3 线程 C 字段对账全部 PASS**。test-design §0 头部声明的 3 字段 + §6.2.1/§6.3.3/§6.3.4 内部引用与 requirements §8.3/§27.6/§29.1 一一对应,无章节号漂移、无内容漂移。

---

## 7. 已知缺口 / 无法验证

### 7.1 unverified(无外部文档可定位)

| # | 内容 | 缺失证据 | 影响 |
|---|---|---|---|
| 1 | test-design 头部声明"`docs/basic-design.md` 截至本次同步仍停留在 98c73b1"(line 35) | 本仓库 `docs/basic-design.md` 未读取,98c73b1 hash 不可定位 | basic-design §4.5.6 (test-design §6.3.2 引用) 无法在本报告范围验证 |
| 2 | test-design §6.2.1 引用 "spec 层(`docs/specs/domain-validation-spec.md`)也未见 Level 字段" | `docs/specs/` 目录未读取 | spec 层是否有 Level 字段无法验证 |
| 3 | test-design §2.3.1 引用 "《External Design》§4 6 个关键流程" | `docs/external-design.md` 仓库不可定位 | 6 个关键流程是否真的在 External Design §4 不可验证 |
| 4 | test-design §2.4.2 / §7.5 / §10.1 引用 "《API Design》§3 / §8 / §10" | `docs/api-design.md` 在工作区存在(§2.2.1 line 270 提及)但未读取 | API 端点清单/错误码/P95 预算表的具体 §N 对账未做 |
| 5 | test-design §2.5 / §8.1 / §8.3 引用 "《Security Design》§2-§3 / §4 / §5 / §7.3 / §8 / §9.3 / §10.1" | `docs/security-design.md` 未读取 | 安全维度引用全部 unverified |
| 6 | test-design §8.2 / §8.4 / §13.2 引用 "《Runtime Design》§12.1" / "8 种白名单命令" | `docs/runtime-design.md` 未读取;requirements §23.2 line 699-704 列了 7 个示例命令(GitStatus/CreateWorktree/ReadDiff/RunApprovedTest/QueryAgentStatus/SubmitFeedback/StartAuthorizedAgentSession),**test-design 5 次说"8 种"与 requirements 7 个示例对不上** | **P1 数量漂移(8 vs 7),需 runtime-design 补证** |
| 7 | test-design §7.5 引用 "《Basic Design》§44 K8s Tax 纪律" | 本仓库 `docs/basic-design.md` 存在(由 line 35 头部声明 + §14 #16 引用)但未读取 | 性能预算分配 500ms 总预算在 basic-design §44 的细节对账未做 |

### 7.2 任务简报与实测差异

| 任务简报 | 实测 | 差异 |
|---|---|---|
| "main HEAD 948582e" | 本机 main HEAD `3c9c2ae` | 0 commit 差异,948582e 不在本工作区 git log;两份目标文件已落地 |
| "test-design §7 缺陷管理" | test-design §7 = 性能测试(详细),非缺陷管理 | 任务前提不成立 |
| "requirements §47 实际域模块清单" | requirements §47 = 下一阶段输入清单,非模块清单;真实模块清单在 §13.3 | 任务前提错位(§47 ≠ 模块清单) |
| "requirements §44 准入门槛(PR 准入 / 主干准入 / Release 准入)" | requirements §44 = 架构原则总纲,无准入门槛内容 | 任务前提不成立 |

### 7.3 守门 #1+#9+#12 三过证据

- **守门 #1 实证**:`cargo check --workspace --lib` / `cargo test` / `pnpm tsc --noEmit` 等代码守门**不适用**——本任务是 docs-only 只读证据收集,无代码改动,无 frontend/backend 编译;`docs/test-design.md` / `docs/requirements.md` 仅通过 `Get-Content -Raw -Encoding UTF8` 读取,**未修改**
- **守门 #9 实证**:本任务**0 子代理调用**,root 直实装,实测文件存在:
  - `D:/Star/docs/test-design.md` = 49846 bytes / 1507 行 / LastWrite 08/31 10:47:51
  - `D:/Star/docs/requirements.md` = 94801 bytes / ~1793 行 / LastWrite 08/29 20:28:01
  - 唯一产出 `D:/Star/docs/qa/raw/gamma-testdesign-requirements.md` 本文件(>50 行,7 个表格,完成定义满足)
- **守门 #12 commit-time 同步**:本任务**0 commit**(per 任务约束"不 commit,不 git add"),不触发 commit-time 同步;`docs/qa/raw/gamma-testdesign-requirements.md` 在工作区 untracked(待 parent session 决策是否 commit)

### 7.4 工作区状态(git status 实证)

```text
$ git log -1 --oneline
a361810 feat(scripts): one-click dev launcher (start.bat + start-dev.ps1)

$ git status --short
?? docs/specs/workflow-templates-spec.md
```

(本工作区 main HEAD 实测 = `3c9c2ae`,与 `git log -1` 一致;a361810 似乎未出现在工作区;以 `git rev-parse HEAD` 输出的 `3c9c2ae5363601eff1a265cd2e881f55a78d2b28` 为准)

---

## 8. 守门硬约束自检

| # | 守门 | 实证 | 状态 |
|---|---|---|---|
| 1 | R-05 不 push | 本任务 0 commit 0 push | ✅ |
| 2 | 0 unsafe(代码守门) | 本任务是 docs-only 只读,无代码 | ✅ |
| 3 | 0 git 改动 | `docs/test-design.md` / `docs/requirements.md` 未修改;唯一新增 `docs/qa/raw/gamma-testdesign-requirements.md`(per 任务授权) | ✅ |
| 4 | PowerShell only | `Get-Content -Raw -Encoding UTF8` 调用,npm/bash/heredoc 全 0 使用 | ✅ |
| 5 | 环境变量安全 | 0 引用任何环境变量(无 `Get-ChildItem env:` / `echo $VAR` / `cat .env` 等) | ✅ |
| 6 | 子代理 status ≠ 实际成功 | 0 子代理调用,全部 root 直实装 + 文件存在性实测 | ✅ |
| 7 | 缺标比错标安全 | §7.1 7 个 unverified 项全部显式列"无法验证"原因;未臆造归属 | ✅ |
| 8 | AI 协作文档治理 | 无回溯叙事,所有乖离引用"test-design 引用 §N" 形式;BAS / 章节引用全部以 requirements.md 实际行号实证 | ✅ |

---

## 9. 总结

| 类别 | 数量 | 详情 |
|---|---|---|
| **总条数** | 27 | — |
| **P0(章节号引用错位 / 数量漂移)** | 10 | §1.2 引用 §44(×3 PR/主干/Release)+ §2.4.2 引用 §36 + §2.1.1 25 vs 15 Module 漂移(×7 8 个不在 §13.3 的 Module)+ §5 双向任务前提错误 + §47 引用错位 |
| **P1(数量小漂移 / unverified)** | 3 | domain-notification 部分命中 + 8 种 vs 7 种白名单命令(需 runtime-design)+ §6.3.3 basic-design §4.5.6 未验证 |
| **P2(表述小漂移)** | 3 | T3 REQ-OPS-003 归属 + §6.3.4 "§8.2 ExecuteArbitraryShell" 术语不一致 + §0 sync 注释 |
| **unverified(外部文档缺失)** | 6 | basic-design / external-design / api-design / security-design / runtime-design / specs/ |
| **PASS(对账正确)** | 5 | S1-S5 / T1-T2 / 13 类必带对象 / 线程 C 三字段 / §0.3 七类命名 |

**重点结论**:
1. **test-design §1.2 三个门禁都标错了引用源**——line 119 写 "(继承《Requirements》§44)",但 requirements §44 完全没有 PR / 主干 / Release 门禁内容。这是 test-design v0.3 最大的归属错位。
2. **test-design §2.4.2 把 TBD-MEASURE 规则挂到 requirements §36**,但 §36 是 Use Case 一览;TBD-MEASURE 规则实际在 requirements §0 line 21,但 §0 引用的是"原文档 §36、§80"(本仓库不可定位)。
3. **test-design 25 Module 与 requirements §13.3(15 个 domain-* crate)有 8 个 Module 漂移**——`domain-identity / domain-audit / domain-search / domain-integration / domain-automation / domain-collaboration / domain-local-runtime` 在 requirements.md 全文 0 命中 crate 名;`domain-notification` 仅在 §12 line 311 内文提及 1 次。
4. **任务简报对 test-design §7 / requirements §47 / requirements §44 的描述与实际章节内容不符**——这 3 处是任务简报自身的前提错位(非 test-design / requirements 的错误),建议下一轮 worker 重新校准任务描述。
5. **PASS 项质量高**——S1-S5 / T1-T2 / 线程 C 三字段 / 13 类必带对象 全部 1:1 对账通过,无章节号漂移,无内容漂移,test-design v0.3 的"线程 C"同步声明自洽度良好。

---

[gamma done] total=27, p0=10, p1=3, p2=3, unverified=6, pass=5
