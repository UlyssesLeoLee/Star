# AUDIT-003 — 详细设计文档全面审核

> **触发**: 2026-09-08 用户要求"全面审核详细设计文档"
> **前置**: 紧接 `docs/reports/2026-09-08-audit-002-requirements-basicdesign-followup.md`(母版对审核),本轮是 V-model 下一层。
> **范围声明**: 详细设计层包含两个文档家族,均纳入本轮审核范围(不做隐性排除):
> 1. 后端 10 份核心详设:`api-design.md`(3134)/`data-design.md`(5620)/`security-design.md`(2187)/`runtime-design.md`(1885)/`integration-design.md`(1686)/`ai-agent-design.md`(1692)/`external-design.md`(1524)/`internal-design.md`(1677)/`test-design.md`(2672)/`operation-design.md`(1781)
> 2. 前端详设家族:`frontend-design.md`(885)+`frontend-internal-01~04-*.md`(2867)+`frontend-canvas-design.md`(615)+**`frontend-design-feedback.md`(200,已有审核记录)**;以及同层的 `automation-design.md`(838)/`uat-design.md`(455)/`uat-runbook.md`(442)/`ubiquitous-language.md`(300)。
>
> **方法**(不做 24k+ 行逐字通读,采用与代码现实交叉核对为主的策略,原因见方法论说明):
> 1. 先读完此前唯一未读过的既有审核产物 `frontend-design-feedback.md`(200 行,FD-01~FD-15,16 条 Finding),核实其当前修复状态——这是与 AUDIT-002 同样的"验证优先"方法论。
> 2. 复核 `basic-design-feedback.md` D-01/D-05(此前只验证了 D-02/D-03/D-04)。
> 3. 用**外部权威**做机械交叉核对,而非读更多文字:`data-design.md`/`api-design.md` vs 实际代码(`crates/api`、`crates/domain-scm` 等);`test-design.md` 122 个正式测试用例编号 vs `crates/*/src/**` 与 `frontend/e2e/*.spec.ts` 实际测试代码。
> 4. AUDIT-002 §2.2 的"domain-llm/mcp/task/tool 零文档"模式,扩大核查到全部 10 份详设文档,确认是否为详设层的系统性延伸。
> 5. 对没有外部权威可比对的纯 prose 文档(`internal-design.md`/`external-design.md`/`operation-design.md`)沿用上一轮的结构性抽查(TODO/FIXME 密度),不重复逐章通读。

---

## 0. 结论先行

**这一轮审核的核心发现和 AUDIT-002 的调性不同**:AUDIT-002 里"报告说修复了,回去查文档确实修复了"的比例很高;这一轮发现**至少一份关键审核报告(`frontend-design-feedback.md`)里的 Blocker 级问题至今几乎没有任何一项被处理**(9 项抽查里 8 项原样未动),以及两个此前完全没人发现的**新的系统性/局部缺口**——`test-design.md` 定义的正式测试用例编号体系(161 个唯一编号)在实际代码里 0 个可确认真实引用;`basic-design.md §7.6` 状态机总览表除已知的 PullRequest 外,ValidationResult 也存在文档-代码状态数不一致(6 vs 5,`ERRORED` 状态未实现)。

| 类别 | 数量 | 说明 |
|---|---|---|
| 既有 Finding 复核 → 确认已修复 | 2 项 | D-01(ARCHIVED 归属)、D-05(聚合根注解),见 §1(注:F-01~F-08 未独立复核,不计入"闭环") |
| 既有 Finding 复核 → **抽查 9/16 项,8 项仍完全未修复(含 2 个 Blocker + 6 个 Major)** | frontend-design-feedback.md FD-01/FD-01B/FD-02/FD-03/FD-04/FD-07/FD-08/FD-11/FD-15 | 见 §2 |
| 既有 Finding 复核 → 确认已修复(抽查项) | frontend-design-feedback.md FD-13 | 见 §2.6 |
| 本轮新发现,Major | 3 项 | test-design.md 测试编号可追溯性缺失(§3.1,161 个编号 0 个确认命中)、`basic-design.md §7.6` ValidationResult 6 vs 5 状态矛盾(§3.2)、domain-llm/mcp/task/tool 零文档扩大到全部详设层+9 份前端/UAT 文档(§3.3) |
| 背景说明,非独立 Finding | 1 项 | api-design.md 279 端点 vs `crates/api` 162 行骨架实现差距(§4,已知 Mock-first 阶段性状态) |

---

## 1. basic-design-feedback.md D-01 / D-05 复核(此前仅验证 D-02/D-03/D-04)

- **D-01**(api-design.md 引入未定义的 `ARCHIVED` WorkItem 终态,与 data-design.md 9 状态枚举矛盾):`api-design.md:649` 当前文本为"默认三态:`TODO → IN_PROGRESS → DONE`(无终态;`ARCHIVED` 是 Worktree/AgentSession 的状态,不属于 WorkItem;basic-design §4.9.3 / §7.2,**D-01 修复**)"。**确认关闭。**
- **D-05**(data-design.md 缺失 `development_execution`/`change_set`/`decision` 三张表的"核心聚合根"标注):`data-design.md:2795` `development_execution` 表(核心聚合根)、`:2842` `change_set` 表(核心聚合根)、`:3898` `decision` 表(核心聚合根)均已补齐标注。**确认关闭。**

**注意范围**:本轮独立核实的只有 D-01/D-05(本节)+ D-02/D-03/D-04(AUDIT-002 §1.5,含 F-05 残留问题)。F-01/F-02/F-03/F-04/F-06/F-07/F-08 这 7 项**从未被独立复核过**,其"已修复"状态目前仅依据 `basic-design-feedback.md` 文档自身的"已修复 in commit 81778d9"标注——而 §2.5(FD-15)恰好证明了这类自述不可尽信:同一份文档里 F-08 正确抓到了 AgentSession 13→14 的修复,却漏掉了同一张表里 PullRequest 7→8 的对应问题。**因此不能称 `basic-design-feedback.md` 已"完全闭环"**,只能说 D-01~D-05 五项经本轮及 AUDIT-002 独立验证为已修复,F-01~F-08 七项状态未知,建议下一轮抽查。

---

## 2. frontend-design-feedback.md 复核(此前从未被任何审核读过)—— 核心发现:多数 Blocker/Major 未修复

`frontend-design-feedback.md`(2026 年某次会话产出,针对 `frontend-design.md` v0.1)记录 16 条 Finding(2 Blocker + 8 Major + 6 Minor),结论为"有条件通过,但 FD-01/FD-01B/FD-15/FD-04/FD-03 必须在 V1 切真后端前修复"。本轮对其中 5 项(2 个 Blocker + 3 个高优先级 Major)做了当前状态核实,**结果是负面的**:

### 2.1 FD-01 [Blocker] 6 个状态机状态名向壁虚构 —— **仍未修复**

抽查 `frontend/src/types/ids.ts` 当前内容(约行 894-1015):
- `FEEDBACK_SM.states` 仍为 `["open", "acknowledged", "in_progress", "resolved", "wontfix", "reopened"]`——与 FD-01 报告时完全一致的错误状态名,和 `crates/domain-feedback/src/value_object.rs` 实际的 `Open/Acknowledged/Applied/Verified/Rejected/Superseded` 依旧不同构。
- `WORKITEM_SM.states` 仍为 `["todo", "in_progress", "review", "blocked", "done", "wontfix"]`,`CHANGESET_SM.states` 仍为原文件报告的旧值——均未回校 `crates/domain-work-item`/`domain-development` 实际枚举。
- **确认:FD-01 从提出到现在,`ids.ts` 的 6 个 `StateMachine` 常量未被回写,问题原样保留。**

### 2.2 FD-01B [Blocker] Decision 第 7 个状态机被遗漏 —— **仍未修复**

`ids.ts` 中未见 `DECISION_SM` 或等价常量定义(grep `Decision.*SM\|DCSM` 无结果),`frontend-design.md` §4 仍标题"6 状态机"。**未修复。**

### 2.3 FD-02 [Major] ADR-FE-003 引用不存在的 crate 名 `domain-api` —— **仍未修复**

`frontend-design.md:709` 原文依旧是"backend 25 module 仅 `domain-api` crate 是骨架 Port trait,无真实 handler"。`crates/` 目录下依然不存在 `domain-api`,真实骨架 crate 是 `crates/api`(见 §4)。**未修复。**

### 2.4 FD-04 [Major] NATS Subject 缺失 tenant_id 隔离段(安全相关) —— **仍未修复,且已扩散进已实现的前端代码**

`frontend/src/app/worktree/page.tsx:136` 实际渲染文案"状态切换会触发 NATS event `star.worktree.*`"——与 `api-design.md §5.2` 定义的稳定格式 `star.events.{tenant_id}.{domain}.{aggregate}.{action}` 依旧不符,缺 `events.` 段与 `{tenant_id}` 段。**这是本轮唯一从"设计文档层面的错误"发展为"已落地到前端代码里"的一项**——FD-04 提出时问题还只在 `frontend-design.md` 的表格里,现在这个错误字符串已经写进了实际渲染给用户看的 UI 文案。**未修复,且劣化。**

### 2.5 FD-15 [Major] `basic-design.md §7.6` PullRequest 状态数"7"仍未修复,已三层传播 —— **仍未修复**

- `basic-design.md:2737` §7.6 状态机总览表仍为 `| PullRequest | 7 | 2 (User / Webhook) | A.6 |`,与同文档 §7.5 正文 / 附录 A.6 的 8 状态(`DRAFT/OPEN/REVIEWING/CHANGES_REQUESTED/APPROVED/MERGEABLE/MERGED/CLOSED`)依旧矛盾。
- `crates/domain-scm/src/lib.rs:25,233` 文档注释依旧写"7 状态机",但紧接着定义的 `PullRequestState` 枚举(`:237-254`)实测为 **8 个变体**(`Draft/Open/Reviewing/ChangesRequested/Approved/Mergeable/Merged/Closed`)——代码注释与代码实现本身依旧自相矛盾。
- `frontend/src/types/ids.ts` 的 `PR_SM`(约行 973-990)状态列表仍为 7 项(`draft/open/ci_failed/review_required/approved/merged/closed`),`ci_failed`/`review_required` 两个名字依旧是前端自创、backend 不存在的状态名,真实的 `Reviewing`/`Mergeable` 依旧完全没出现。
- **三处(母版表格、后端源码注释、前端状态机定义)全部原样保留,无一处修复。** 这是本次审核中修复优先级最高的一项:源头是 `basic-design.md:2737` 一个字符("7"→"8"),但因为三处都没人回去改,持续对新读者造成误导。

### 2.6 FD-13 [Minor] ⌘K SearchPanel 未实现 —— **已修复**

`frontend/src/components/CommandBar.tsx:48` 现有 `isCmdK = (e.metaKey || e.ctrlKey) && e.key.toLowerCase() === "k"` 真实按键监听逻辑,`Sidebar.tsx`/`ThemeSwitcher.tsx` 也有类似的真实快捷键实现。**确认关闭**——这是本轮抽查中唯一一项已修复的 Finding,说明前端团队后续确实做过迭代,只是没有回头处理 FD 系列里更基础的状态机/契约类问题。

### 2.7 追加核实:FD-03/FD-07/FD-08/FD-11 —— **同样仍未修复**

直接对 `frontend-design.md` 文档正文(而非其对应的前端代码)做关键词核对:

- **FD-03**(错误码字典虚构):`frontend-design.md:665-670` 原表格逐行仍在,`SEC-001`(误用为跨租户,真实码是 `SEC-007`)、`WF-403`、`WF-409`、`API-429`、`API-500`、`SC-001` 六个码字面未变,均不在 `api-design.md` 真实错误码字典中。**未修复。**
- **FD-07**(Local Runtime 字段虚构):`frontend-design.md:545` `mount_root` 仍在("`mount_root` 是否在 policy.allowlist"),`:572` LocalRuntime 行仍列 `status / mount_root / policy_violations`——真实 `LocalRuntime` 结构体字段是 `id/tenant_id/user_id/device_id/status/version/capabilities/last_heartbeat/registered_at/metadata`,不存在 `mount_root`。**未修复。**
- **FD-08**(字段名不匹配):`frontend-design.md:563` `lock_version` 仍在(真实字段是 `version`,见 `crates/domain-worktree/src/lib.rs:222`);`:570` `suppression_reason` 仍在(Notification 被抑制的事件从不落库,该字段不存在)。**未修复。**
- **FD-11**("三选一"标签数量不符):`frontend-design.md:320` "页面模式三选一"实际列出 4 个选项(Dashboard/ListPage/DetailPage/StatsPage),`:321` "主组件三选一"实际列出 6 个选项(Table/Kanban/SmView/FlowChart/Canvas/List)。**未修复。**

至此累计核实 9/16 项(2 Blocker + 6 Major + 1 Minor),**9 项里 8 项原样未动,仅 FD-13 已修复**。

### 剩余 6 项(FD-05/FD-06/FD-09/FD-10/FD-12/FD-14)未在本轮核实

因时间预算限制未覆盖:FD-05/FD-06(SCM/Automation 反向覆盖缺口,需要逐一核对 7 个 domain-scm 实体和 automation dry-run 能力在前端的落地情况,工作量较大)、FD-09/FD-10/FD-12/FD-14(文档内部矛盾类,RSC vs Client-Component / 复用率表格 / 组件目录树 / INV-SCM 范围表述)。**状态未知,不能假设已修复**。鉴于已核实的 9 项里 8 项原样未动(89% 未修复率),**先验估计其余 6 项大概率也维持原状**,建议下一轮优先补齐。

---

## 3. 本轮新发现

### 3.1 【新发现,Major】test-design.md 的 122 个正式测试用例编号与实际测试代码几乎零关联,文档自身的可追溯性机制未落地

- **核实方法**: 提取 `test-design.md` 全文出现的所有形如 `T-XX...`/`T-E2E-N-N` 的测试用例编号,逐一在 `crates/*/src/**/*.rs`、`frontend/src/**/*.ts(x)`、`frontend/e2e/*.spec.ts` 全仓做字符串检索。**注**:首次提取用的正则 `T-[A-Z]+[0-9-]+` 有缺陷,会把 `T-E2E-1-1` 这类编号截断成 `T-E2`(`[A-Z]+` 在第一个数字前停止,漏掉后续的 `E2E`),导致 31 个 `T-E2E-*` 编号全部坍缩成同一个错误 token,原始去重计数(122)不可复现。改用 `T-[A-Z0-9]+(-[A-Z0-9]+)*` 重新提取,去重后共 **161 个**唯一编号。
- **结果**: 161 个编号里字符串检索命中 **14 个**(`T-001`/`T-002`/`T-10`/`T-11`/`T-12`/`T-A1`/`T-CROSS-1~4`/`T-M`/`T-WORKER`/`T-WORKER-02`/`T-WORKER-03`)。逐一查看这 14 处命中的上下文后,**全部确认为误报**,且可归为两类:(a) 子串巧合——`T-A1` 命中的实际是 `OPT-A1-code-todo-scan`("PT-A1" 恰好包含 "T-A1")、`T-WORKER-02/03` 命中的是 `OPT-WORKER-02/03`(brief 文档编号)、`T-001`/`T-002`/`T-10`/`T-M` 分别命中 `LRT-001`/`REQ-TST-002`/`INV-CT-10`/"W/T/M 分类"里的巧合子串;(b) 平行但不同源的编号体系——`T-CROSS-1~4`/`T-11`/`T-12` 命中的其实是 `crates/star-mcp/tests/it_actor_context_integration.rs` 与 `crates/star-context/tests/it_actor_context.rs` 里的 `IT-CROSS-N`/`IT-N` 集成测试编号(前缀 `IT-` 不是 `T-`,只是子串包含关系导致命中),这是一套独立的、命名相似但与 `test-design.md` 无关的集成测试编号约定。**修正后的结论:161 个正式测试用例编号里,0 个能确认与实际测试代码存在真实的引用关系**,原始"7/122 命中,大概率巧合"的表述在数字上不可复现,但方向是对的且证据更强——真实情况是 0/161 精确命中,14 个原始候选经上下文核查全部排除。
- **交叉验证**(排除"实际测试用其它编号体系,如 INV- 不变量 ID"这一可能性):抽查 `crates/domain-validation/src/lib.rs` 的实际 `#[test]` 函数(如 `:125/136/151`),函数名均为英文语义化命名(如 `make_test_actor`),`#[test]` 标注前后未见任何 `T-` 或 `INV-` 编号注释。`frontend/e2e/*.spec.ts`(16 个真实 Playwright 文件,如 `canvas-share-export.spec.ts`/`cross-domain-5b.spec.ts`/`mcp-16-tool-coverage.spec.ts`)命名体系与 `test-design.md` 的 `T-E2E-N-N` 编号体系完全不同源,零文件包含 `T-E2E` 字样。
- **性质**: 这是与 D-04(VAL-001 四重门测试覆盖缺口,已修复)相邻但更根本的问题——D-04 问的是"test-design.md 文档内部,这个编号体系覆盖是否完整",本项问的是"这个编号体系是否真的对应到了任何一行实际执行的测试代码"。当前答案是:**test-design.md 是一份自洽的测试计划文档,但与代码仓库里实际存在的测试几乎是两个平行世界**——文档定义了"应该测什么"(122 个用例),代码写了"实际测了什么"(真实 `#[test]`/`.spec.ts` 数量未知但明显是另一套命名),两者之间没有任何显式的双向可追溯锚点。这会导致:(a) 无法从 test-design.md 判断某个用例是否已被实现验证;(b) 无法从一个失败的真实测试反查它对应文档里的哪条验收标准;(c) `docs/reports/2026-09-03-blue-team-26-items-review.md` 提出的 GAP-BLUE-7 教训("核实需 grep/git log 实证,不能只信文档自述")在测试这一环完全适用且尚未被应用过。
- **建议**: 不要求一次性把 122 个用例全部串上代码(工作量过大);但建议至少对 P0 级别的用例(如 VAL-001 四重门相关的 T-VL 系列、跨租户安全相关的 T-CROSS 系列)在对应测试函数上补充 `// per test-design.md T-VL4` 这类锚点注释,并在 `test-design.md` 里补一列"实现状态/对应测试文件路径",否则这份 122 条目的文档目前的实际作用更接近"测试意图记录",而非可验证的"测试覆盖矩阵"。

### 3.2 【新发现,Major】`basic-design.md §7.6` 状态机总览表:ValidationResult 声称 6 状态(含 ERRORED),实际代码只实现 5 状态——PullRequest 之外的第二处行数矛盾

FD-15 原文建议"同步核对 §7.6 表里其它 5 行是否与各自附录一致,本次未逐行复核"——本轮补上这一步,逐行核对 `basic-design.md:2731-2737` 七行 vs 各自附录(A.1~A.7)vs 代码实际枚举:

| 实体 | §7.6 表声称 | 附录内容 | 代码实际枚举(crate) | 结论 |
|---|---|---|---|---|
| Worktree | 17 | (未逐条比对,附录为 mermaid 图) | `WorktreeStatus` 17 个变体(`crates/domain-worktree/src/lib.rs:56-90`,注释编号 0~16) | 一致 |
| WorkItem(默认+扩展) | 3+扩展 | A.2 | 已由 D-01 澄清默认三态 | 一致(D-01 已修复) |
| Feedback | 6 | A.3:OPEN/ACKNOWLEDGED/APPLIED/VERIFIED/REJECTED/SUPERSEDED(6) | `crates/domain-feedback` 实际 6 态(Open/Acknowledged/Applied/Verified/Rejected/Superseded) | 一致 |
| AgentSession | 14 | A.4 | `AgentSessionStatus` 14 个变体(`crates/domain-agent/src/lib.rs:64-96`,注释编号 0~13) | 一致 |
| **ValidationResult** | **6** | **A.5:PENDING/RUNNING/PASSED/FAILED/ERRORED/SKIPPED(6,含 ERRORED)** | **`ValidationStatus`(`crates/domain-validation/src/value_object.rs:98-111`)仅 5 个变体:Pending/Running/Passed/Failed/Skipped——`ERRORED` 不存在,且该枚举自带文档注释明确写"SOW 要求 5 状态"** | **矛盾** |
| PullRequest | 7 | A.6:8 态(含 MERGEABLE) | `PullRequestState` 8 个变体 | 矛盾(FD-15,已知) |
| Decision | 3 | A.7:ACTIVE/SUPERSEDED/INVALIDATED(3) | `DecisionStatus`(`crates/domain-context/src/lib.rs:313-320`)3 个变体 | 一致 |

- **性质**:与 FD-15(PullRequest)同类但独立的第二处矛盾,而且方向相反——PullRequest 是"母版文档数字过时,代码是对的(8)";ValidationResult 是"母版文档(表格+附录 A.5 mermaid 图完全自洽)都要求 6 态含 `ERRORED`,但代码实现只做了 5 态,`ERRORED`(异常终止,如编译失败/网络中断)这个业务上有意义的终态从未被实现",代码自身的文档注释("SOW 要求 5 状态")甚至没有承认与母版设计文档的差异,读起来像是把"5 态"当成了独立权威来源,而不是"故意简化了设计里的 6 态"。
- **影响**:如果 `ERRORED`(编译失败/网络中断等基础设施异常)在实际系统里发生,目前的 `ValidationStatus` 没有对应状态可以承载它——很可能会被错误地归入 `Failed`(业务断言失败)或导致状态转移逻辑处理这类异常时没有专门路径。这比 PullRequest 的"字段计数误标"更接近一个功能缺口。
- **建议**:与 FD-15 一并处理时明确这是两个独立的根因(不要在同一次修复里把 ValidationResult 也简单改成"文档数字对齐代码"——需要先确认业务上是否真的要支持 `ERRORED`,再决定是给代码补状态还是把文档的 6 改成 5)。

### 3.3 【延伸自 AUDIT-002,Major】domain-llm/mcp/task/tool 零文档问题延伸至全部 10 份详细设计文档,是详设层的系统性缺口,而非母版文档独有

- **核实结果**: 对 `security-design.md`/`runtime-design.md`/`integration-design.md`/`ai-agent-design.md`/`external-design.md`/`internal-design.md`/`operation-design.md`/`test-design.md`/`data-design.md`/`api-design.md` 共 10 份详设逐一 grep `domain-llm`/`domain-mcp`/`domain-task`/`domain-tool`,**全部零命中**。
- **性质**: 这不是一个新的独立 Finding,而是把 AUDIT-002 §2.2("这 4 个 crate 在母版 basic-design.md 里零文档")的核实范围扩大到了整个详设层,确认结果是一致的——**这 4 个 crate 目前在项目的整条 V-model 文档链(基本设计 + 全部 10 份详设)里完全不存在**,唯一的设计线索来自游离在 V-model 主链之外的 `docs/architecture/2026-09-03-agent-runtime/02-basic-design.md` 等专题文档。这加重(而非新增)了 AUDIT-002 §2.2 的严重程度判断:不是"母版表格没更新"这么局部的问题,而是这 4 个 crate 事实上从未被纳入过正式详设评审流程。
- **建议**: 与 AUDIT-002 §2.2 合并处理,不单独立项。

---

## 4. 背景说明(非独立 Finding):api-design.md 设计端点数 vs 实现进度

`api-design.md` 定义 279 行 HTTP 端点(`GET`/`POST`/`PUT`/`DELETE`/`PATCH` 表格行),而承载真实 REST handler 的 `crates/api/src/lib.rs` 目前仅 162 行。这与 `frontend-design-feedback.md` FD-02 的判断一致(骨架 crate 是 `api`,不是文档里误写的 `domain-api`),是 Mock-first 阶段性策略下的已知状态,**不构成新缺陷**,仅作为理解 §2.3(FD-02 未修复)现实影响的背景数据补充在此。

---

## 5. 建议处理顺序

1. **FD-15 源头(`basic-design.md:2737` 的"7"→"8")**:一个字符改动,但因为已经三层传播(母版表格 + `domain-scm` 源码注释 ×2 + 前端 `ids.ts`),建议同一个 PR 里把四处一起改,避免修复方向反了(参考 `frontend-design-feedback.md` FD-15 原文的告诫)。
2. **FD-01/FD-01B(6→7 状态机状态名重写)**:是前端 V1 切真后端前的阻塞项,拖得越久重写成本越高(mock 数据、组件、路由都会依赖这些错误状态名继续生长)。
3. **FD-04(NATS Subject 缺 tenant_id 隔离段)**:已从文档问题发展为真实前端代码里的用户可见文案,安全相关,建议提前于常规详设修订单独处理。
4. **FD-03/FD-07/FD-08/FD-11**:本轮已确认字面未修复(§2.7),修复成本低(错误码表格、字段名对齐),建议与 FD-01/FD-15 合并进同一批前端文档修订。
5. **ValidationResult ERRORED 状态矛盾(§3.2)**:需先做业务判断(是否要支持基础设施异常终态),再决定改代码还是改文档,不要机械套用 FD-15 的"以代码为准"结论。
6. **test-design.md 可追溯性缺口(§3.1)**:建议从 P0 用例(VAL-001/跨租户)开始补锚点,不要求一次性覆盖全部 161 项。
7. **FD-05/FD-06/FD-09/FD-10/FD-12/FD-14(§2.7 剩余未核实项)**:下一轮审核优先级最高的遗留工作——鉴于已核实 9 项里 8 项未动,先验估计这 6 项大概率也维持原状,建议专项复核而非再次搁置。
8. **domain-llm/mcp/task/tool(§3.3)**:与 AUDIT-002 §2.2 合并处理。
9. **frontend-internal-01~04/frontend-canvas-design.md/automation-design.md/uat-design.md/uat-runbook.md/ubiquitous-language.md 逐章通读**:本轮仅完成结构性抽查(§6),尚未做语义级复核,建议安排下一轮专项覆盖。

---

## 6. 审核范围外声明

- `internal-design.md`/`external-design.md`/`operation-design.md`/`security-design.md`/`runtime-design.md`/`integration-design.md` 六份文档本轮**未逐章通读**,仅做了 domain-llm/mcp/task/tool 关键词排查与 VAL-001/白名单数字传播抽查,沿用 AUDIT-002 已有的结构性抽查结论(TODO/FIXME 密度低),未发现新增异常但也未做语义级复核。
- `frontend-internal-01-architecture.md`(532)/`-02-components.md`(568)/`-03-dataflow.md`(1047)/`-04-interaction.md`(720)、`frontend-canvas-design.md`(615)、`automation-design.md`(838)/`uat-design.md`(455)/`uat-runbook.md`(442)/`ubiquitous-language.md`(300)本轮**未逐章通读**,但补做了一次结构性抽查(未在最初草稿里执行,现已补齐):TODO/TBD/待定/待补充/草稿 关键词密度,以及 `domain-llm`/`domain-mcp`/`domain-task`/`domain-tool` 命中检索。结果:9 份文档 `domain-llm/mcp/task/tool` **全部零命中**(与 §3.3 的结论一致,进一步扩大了该缺口的确认范围);TODO 类密度普遍很低(`frontend-internal-01/frontend-canvas-design.md`/`uat-runbook.md` 为 0,其余 1~4 处,仅 `uat-design.md` 略高为 10 处/455 行)。**这只是结构性抽查,不等于语义级复核**——9 份文档的实际内容(例如 `uat-design.md`/`uat-runbook.md` 是否与 `test-design.md` 的可追溯性问题(§3.1)相关联、`frontend-internal-03-dataflow.md` 1047 行的状态管理设计是否与 FD-01 的状态机错误存在联动)仍是本报告最大的一块已知盲区,下一轮应优先安排逐章通读。
- `frontend-design-feedback.md` 剩余 10 项 Finding(FD-03/05-12/14)未核实当前状态(见 §2.7)。
- `test-design.md` 122 个用例中,除本轮做整体命中率统计外,未逐条判断"未命中"是否等同于"未测试"(有可能存在测试代码用完全不同措辞覆盖了同等场景,只是无法通过编号字符串检索发现)——命中率数字是一个强信号但非精确证明。
