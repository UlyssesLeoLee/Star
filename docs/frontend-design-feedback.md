# Frontend Design 詳細設計書 Review Feedback（基于后端功能审核）

> **审核对象**: `docs/frontend-design.md` v0.1（2026-08-26，884 行）
> **审核基线（按优先级）**: `crates/domain-*/src/**/*.rs`（后端实际功能，最高优先级）> `docs/specs/domain-*.md` > `docs/api-design.md` v0.2 / `docs/basic-design.md` v0.1（frontend-design.md 自行声明的上游）> `frontend/src/`（前端实施现状）
> **审核方法**: 逐条核对 frontend-design.md 中一切"与 backend 严格一致 / 1:1 对应"的断言（状态机状态数与状态名、错误码、NATS Subject、字段投影、组件目录、MVP 承诺）是否在对应 `crates/domain-*` 源码中可验证；backend 功能 → 是否有对应前端需求覆盖（反向覆盖检查）；以及文档内部自相矛盾。审核不改写被审文档，仅产出 Finding。
> **审核者**: Claude（本会话），非原作者。

---

## 使用说明

- 每条 Finding：`ID | Severity | 位置 | 依据 | 影响 | 期望修正`。
- Severity：**Blocker**（前端设计对 backend 功能的核心断言被代码证伪，且该断言是 §4/§9 设计决策的直接前提）/ **Major**（字段级/契约级错误，会导致对接返工，但不否定整体架构）/ **Minor**（内部矛盾、措辞、局部遗漏）/ **Question**（需要作者澄清）。
- 位置格式：`frontend-design.md:行号(§章节)`，backend 证据格式：`crates/xxx/src/lib.rs:行号`。

---

## Findings — Blocking（backend 功能断言被证伪）

### FD-01 [Blocker] 附录 B"6 SM 与 backend 严格一致"——6 个状态机中至少 5 个的**状态名/迁移完全是前端自创词汇**，与 backend 实际枚举无一对应；PR 状态机连**数量**都错

- **位置**: `frontend-design.md:844-855`（附录 B，"总和：6 × 9.5 = 62 transitions，**与 backend 严格一致**"）+ `frontend-design.md:356-367`（§4.1 状态机清单表）+ `frontend/src/types/ids.ts:631-758`（`WORKTREE_SM`/`AGENT_SM`/`FEEDBACK_SM`/`PR_SM`/`WORKITEM_SM`/`CHANGESET_SM` 六个常量的实际状态/迁移定义）
- **依据（逐一比对状态名）**:
  | SM | 前端 (`ids.ts`) 状态 | Backend 实际枚举 | 状态数 |
  |---|---|---|---|
  | Worktree | `initializing/cloning/syncing/active/dirty/behind/diverged/conflict/committing/pushing/ci_running/review_requested/merged/closed/abandoned/archived/reverted` | `crates/domain-worktree/src/lib.rs:58-93` `Created/Initializing/Ready/Assigned/AgentRunning/Committing/Completed/ReadyForReview/Reviewing/ChangesRequested/Fixing/Merged/Archived/Abandoned/Blocked/Conflicted/Stale` | 17=17（数量凑巧一致，仅 `committing/merged/archived/abandoned/initializing` 5 个名字字面重合） |
  | Agent | `queued/spawning/initializing/compiling_context/planning/executing/awaiting_feedback/awaiting_human/awaiting_tool/validating/paused/completed/failed/cancelled` | `crates/domain-agent/src/lib.rs:66-95` `Created/Starting/Running/WaitingTool/ToolRunning/ToolCompleted/WaitingFeedback/FeedbackReceived/Validating/Completed/Failed/Aborted/Crashed/Timeout` | 14=14（仅 `validating/completed/failed` 重合） |
  | Feedback | `open/acknowledged/in_progress/resolved/wontfix/reopened` | `crates/domain-feedback/src/value_object.rs:65-78` `Open/Acknowledged/Applied/Verified/Rejected/Superseded` | 6=6（仅 `open/acknowledged` 重合；`in_progress/resolved/wontfix/reopened` backend 均不存在，backend 的 `Applied/Verified/Rejected/Superseded` 前端均未出现） |
  | PR | `draft/open/ci_failed/review_required/approved/merged/closed`（**7 个**） | `crates/domain-scm/src/lib.rs:232-253` `Draft/Open/Reviewing/ChangesRequested/Approved/Mergeable/Merged/Closed`（**8 个**），且与上游 `basic-design.md:3395-3412`（附录 A.6，同样 8 状态、同样命名）逐字一致 | **7≠8**，但见下方"归因说明"——这个"7"并非前端向壁虚造，而是原样继承自一处上游既有缺陷（详见 FD-15）；`ci_failed`/`review_required` 这两个名字才是前端真正自创、在 backend/`basic-design.md`/`ids.ts` 上游任何地方都不存在的部分，backend 真实的 `Reviewing`/`Mergeable` 两个状态在前端完全没有出现 |
  | WorkItem | `todo/in_progress/review/blocked/done/wontfix` | `crates/domain-work-item/src/value_object.rs:154-167` `TODO/IN_PROGRESS/DONE/IN_REVIEW/BLOCKED/CANCELLED` | 6=6（`review`≈`IN_REVIEW`、`blocked`=`BLOCKED` 可对应，但 `wontfix` 与 backend 唯一剩余状态 `CANCELLED` 语义不同——backend 命名强调"取消"而非"不予处理"） |
  | ChangeSet | `draft/applied/merged/abandoned/reverted` | `crates/domain-development/src/lib.rs:150-161` `Draft/ReadyForReview/Approved/Rejected/Merged` | 5=5（仅 `draft/merged` 重合；backend 的 `ReadyForReview/Approved/Rejected` 与前端的 `applied/abandoned/reverted` 完全是两套不同的业务语义——backend 走"草稿→评审→批准→合并/拒绝"，前端却假想了一条"草稿→应用→合并/放弃/回退"的流程） |
- **PR 迁移数同样算错**: `basic-design.md:3399-3410` 附录 A.6 mermaid 图实际有 10 条边（`DRAFT→OPEN, OPEN→REVIEWING, REVIEWING→CHANGES_REQUESTED, CHANGES_REQUESTED→OPEN, REVIEWING→APPROVED, APPROVED→MERGEABLE, MERGEABLE→MERGED, OPEN→CLOSED, REVIEWING→CLOSED, CHANGES_REQUESTED→CLOSED`），frontend-design.md:363 与附录 B 均记为 "8"，`ids.ts:718-727` 的 `PR_SM.transitions` 也只写了 8 条且与图中 10 条不同构。
- **影响**: 这是全文档最关键的断言——§4"状态机可视化规范"、ADR-FE-002（"6 SM 行为完全一致，UI 复用率达 100%"）、§9 下游交接清单第 2/3 条（"状态机可视化为后续算法…6 SM 完全复用"）、ADR-FE-003 的"切真后端时只需重写 `lib/store.ts`，UI 不动"全部建立在"前端状态机与 backend 状态机同构，只是数据源从 mock 换成 fetch"这一假设上。但实测除 PR 外 5 个 SM 的状态**名称**均为前端向壁虚造（更像是从一个通用"Git 分支生命周期"模板臆想出来的，而非阅读 `crates/domain-*` 源码得出），PR 连**数量**都不对。这意味着 V1 切真后端时，不仅要重写 `store.ts` 的数据获取方式，还必须重写 `types/ids.ts` 全部 6 个 `StateMachine` 常量、`StatusPill.tsx` 的颜色映射表、以及 `StateMachineDiagram` 依赖的 `highlightState` 字符串匹配逻辑——这是 UI 层的改动，直接推翻 ADR-FE-003"UI 不动"的结论。
- **期望修正**: (a) 重新从 `crates/domain-{worktree,agent,feedback,scm,work-item,development}/src` 六个枚举逐一抄录真实状态名/迁移，重写 `ids.ts` 六个 `StateMachine` 常量；(b) 附录 B 表格的 states/transitions 数字与迁移图按 §7.5（`basic-design.md`）与各 crate 的 `can_transition`/`is_terminal` 逻辑重新统计；(c) 去掉"与 backend 严格一致"这类未经验证的断言，或改为"V0.1 为占位命名，V1 切真后端前必须逐一核对"。

### FD-01B [Blocker] §4 标题自称"继承 §7"，但上游 `basic-design.md §7` 实际有 **7 个**状态机，frontend-design.md 只列了 6 个——遗漏的 Decision SM 已有对应 backend 实现与前端路由，却从未被当作状态机处理

- **位置**: `frontend-design.md:356`（§4 标题："6 状态机可视化规范（继承 §7）"）+ `frontend-design.md:339`（§3 第 14 行："domain-context | /context | StatsPage | Table + DecisionCard | priority 颜色 + decision pending 红点"）
- **依据**: `basic-design.md:2469-2479`（§7.6"状态机总览表"）本身列了 **7** 行：Worktree/WorkItem/Feedback/AgentSession/ValidationResult/PullRequest/**Decision**（附录 A.7）；`crates/domain-context/src/lib.rs:242-246` 确认 `DecisionStatus` 已实现为 `Active/Superseded/Invalidated` 三态枚举，且 `INV-CT-05`（`:22`）明确要求"Decision 3 状态"必须被强制校验。frontend-design.md §4 标题写"继承 §7"却只抄了 6 行，§3 路由表虽然已经给 `/context` 规划了 `DecisionCard`（说明作者知道这个聚合根存在），但从未把它算作第 7 个"状态机"、没有 SmView、没有出现在 §4.1 清单表或附录 B 的任何统计里。
- **影响**: 这让附录 B"6 SM 与 backend 严格一致"的断言在 FD-01 揭示的"状态名不同构"之外，又多了一层更基础的错误——**backend 需要可视化状态迁移的聚合根本身就有 7 个，不是 6 个**。`/context` 页面目前只打算用"红点"表示"有 pending decision"，但 `Active→Superseded`/`Active→Invalidated` 是有明确触发者（`§A.7`："User / System"）和不可逆语义的真实状态机迁移，与其他 6 个 SM 的可视化需求（谁触发、能否回退、终态是什么）性质相同，理应享受同等的 SmView 规范，而不是被简化成一个布尔红点。
- **期望修正**: 在 §4.1 清单表补上第 7 行"Decision (DCSM) | 3 | 2 | INV-CT-05 | `active`/`superseded`/`invalidated`"，为 `/context` route 增加一个复用现有 `StateMachineDiagram` 组件的 Decision SmView（哪怕是最简单的 3 节点图），并把文档标题/附录 B 的"6 SM"统一改为"7 SM"（或明确注明"本设计范围内只做 6/7，Decision 列为 V1 候选"并给出理由）。

### FD-02 [Major] ADR-FE-003"backend 25 module 仅 `domain-api` crate 是骨架 Port trait，无真实 handler"的前提已不成立（且引用的 crate 名不存在）

- **位置**: `frontend-design.md:705-709`（ADR-FE-003，理由栏原文引用的 crate 名是"`domain-api`"）
- **依据（先说命名本身）**: `crates/` 目录下不存在名为 `domain-api` 的 crate——真实的 3 个 supporting crate 分别叫 `api`、`application`、`infrastructure`（均无 `domain-` 前缀，因为它们本来就不是 domain crate）。其次，对 25 个 `domain-*` crate 做行数与 `#[test]` 数量统计（见下表节选，命令 `wc -l`/`grep -c '#\[test\]'`），代表性样本：
  | crate | 源码行数 | `#[test]` 数 |
  |---|---|---|
  | domain-work-item | 3331 | 5 |
  | domain-feedback | 2634 | 3 |
  | domain-validation | 2642 | 3 |
  | domain-tenant | 2468 | 6 |
  | domain-integration | 2842 | 3 |
  | domain-scm | 1390 | 1（且 commit `c591c9a`/`6645350` 明确记录"v0.2 单文件重写，8/8 测试通过"） |
  | domain-automation | 1439 | 1（commit `f7587a3` 记录"v0.2，9/9 测试通过"） |
  | domain-worktree | 1424 | 8 |
  | domain-agent | 1791 | 14 |

  与之对照，真正接近"骨架"的只有 3 个**非** domain-* 的 supporting crate：`crates/api/src/lib.rs`（169 行）、`crates/application/src/lib.rs`（292 行）、`crates/infrastructure/src/lib.rs`（180 行）——这恰恰是 frontend-design.md `§1.3` 末尾自己指出"前端不感知这 3 个 crate"的那 3 个。
- **影响**: ADR-FE-003 把"25 module 均为骨架"作为 Mock-first 优先于 OpenAPI client 的**唯一理由**，但这句话赖以成立的前提——"没有真实 handler"——本身可能仍然成立（真正叫 `api` 的那个 crate 确实只有 169 行），只是 ADR 把这个理由错误地安在一个不存在的 crate 名（`domain-api`）和一个错误的范围（"25 module"）上：25 个 domain crate 已经有状态机、不变量校验、`InMemoryXxxService` 完整实现与单元测试，真正的骨架只在没有被前端引用的 3 个 supporting crate（`api`/`application`/`infrastructure`，即 REST handler 层）。这个措辞偏差会误导下游判断"何时能切真后端"：如果只等"domain 有真实逻辑"，现在就该切；如果要等"REST handler 落地"，那本 ADR 应该点名 `crates/api`，而不是引用一个不存在的 crate 名笼统地说"25 module"。之所以定为 Major 而非 Blocker：Mock-first 这个**决策本身**（先用 Zustand 模拟，不等 REST 落地）在事实修正后依然成立，被推翻的只是"理由"栏的措辞和归因，不是决策结论。
- **期望修正**: 把 ADR-FE-003"理由"栏改为"`crates/api`（REST handler 层）与 `crates/application`（编排层）仍是骨架，尚无可调用的 HTTP 端点；25 个 `domain-*` crate 本身已有状态机与单元测试（见 commit 历史），但未经 `api` 暴露，前端无法直连"，并相应重新评估 FD-01 揭示的"状态机词汇不同构"问题——这才是 V1 切换成本被低估的真正原因。

### FD-03 [Major] §8.2 错误反馈规范 6 个错误码全部与 `api-design.md` §8.3 真实错误码字典不符

- **位置**: `frontend-design.md:660-670`（§8.2 错误码 → UI 表现对照表）
- **依据（逐一核对 `docs/api-design.md`）**:
  | 前端引用 | 前端声称含义 | 真实情况 |
  |---|---|---|
  | `SEC-001`（跨 tenant） | "跨 tenant 访问" | `api-design.md:143` `SEC-001` = 401 未认证（JWT 缺失/失效）；跨 tenant 的真实错误码是 `SEC-007`（`api-design.md:1990`："403 Cross-Tenant Access Forbidden"） |
  | `WF-403`（effect=deny） | "无权限" | 全文档 grep 零匹配；真实的 Role 权限拒绝码是 `SEC-004`（`api-design.md:1987`："403 Role Permission Denied"） |
  | `WF-409`（InvalidTransition） | "状态机非法迁移" | 全文档 grep 零匹配；真实做法是**按 Module 独立编码**，如 `WT-003`/`AGT-003`/`FBK-003`/`WI-002`（`api-design.md:1904/1919/1934/2054`），并非一个跨模块通用的 `WF-409` |
  | `API-429`（rate limit） | "限流" | 全文档 grep 零匹配；真实码是 `RATE-001`/`RATE-002`（`api-design.md:2004-2005`） |
  | `API-500` | 服务端错误 | 全文档 grep 零匹配；真实码是 `SRV-001`（`api-design.md:2030`） |
  | `SC-001`（lock_version 不一致） | "乐观并发冲突" | 全文档 grep 零匹配；真实码是 `VAL-002`（412 Validation Version Conflict，基于 `If-Match` ETag，`api-design.md:1955`），且 backend 字段名是 `version` 不是 `lock_version`（见 FD-08） |
- **影响**: 这 6 行是 §8.2 的**全部内容**，无一能在 `api-design.md §8.3`（本设计自称的"上游契约"）中找到对应项；命名规则也系统性错误——真实错误码是 `{2 位 Module Code}{3 位数字}`（如 `WT-003`），前端却发明了一种 `{HTTP 状态码前缀}-{3 位数字}` 的命名法（`API-429`/`API-500`）与真实规则完全不同构。按此设计实现 toast/banner 逻辑，前端会去匹配一个后端永远不会返回的错误码字符串，导致 `WF-409`/`SEC-001` 等分支永远走不到，用户看到的会是兜底的通用错误提示而非本节设计的精确文案。
- **期望修正**: 重新对照 `api-design.md §8.2`（命名规则）与 §8.3（字典）逐行改写，尤其注意"跨 tenant"必须映射到 `SEC-007` 而非 `SEC-001`，"InvalidTransition"必须按前端当前正在展示的 Module 动态选码（`WT-003`/`AGT-003`/`FBK-003`/`WI-002`/…），不能用一个通用 `WF-409` 覆盖 6 个状态机。

### FD-04 [Major] §7.2 NATS Subject 映射表与 `api-design.md §5.2` 稳定承诺的真实格式不符，且缺失 tenant 隔离段（安全相关）

- **位置**: `frontend-design.md:624-641`（§7.2 25 Module ↔ Subject 映射表，如 `star.worktree.*`/`star.agent.*`/`star.workitem.*`）
- **依据**: `api-design.md:1312-1323`（§5.2，"本设计稳定承诺"）明确定义业务事件 Subject 格式为 `star.events.{tenant_id}.{domain}.{aggregate}.{action}`（强制 tenant_id 段），并在 §5.3 事件清单中给出 20 个具体示例，如 `star.events.{tenant_id}.worktree.worktree.created.v1`、`star.events.{tenant_id}.agent.agent_session.started.v1`；前端表格中的 `star.worktree.*`/`star.agent.*` 既缺少 `events.` 段，也缺少 `{tenant_id}` 段。命名风格也不一致：前端用 `star.workitem.*`（无连字符），而 25 个 domain crate 命名是 `domain-work-item`（有连字符），真实事件格式用的是 `worktree`/`agent_session` 这类 aggregate 名而非 crate 名。
- **影响**: 这不只是命名不一致——`{tenant_id}` 是 §6.1"13 类必带 tenant_id 对象"隔离机制在事件总线上的落地方式。若前端 WebSocket 客户端按本节设计去订阅 `star.worktree.*` 这种**不带 tenant_id 段**的 subject，在真实 NATS 多租户部署下要么无法匹配任何真实 subject（因为真实 subject 一定带 tenant_id 段），要么如果后端为兼容前端而放宽为可无 tenant_id 订阅，会造成跨 tenant 事件泄漏，与 §6.1"任何 missing X-Tenant-Id → 401"的强制隔离精神相悖。这是本节唯一一处"文档错误可能直接演变为安全缺陷"的地方。
- **期望修正**: 按 `api-design.md §5.2/§5.3` 的真实格式重写 §7.2 表格，明确前端 WS 客户端订阅时必须把当前登录 tenant_id 拼入 subject（如 `star.events.${tenantId}.worktree.>`），而不是订阅一个全局通配符。

### FD-05 [Major] 反向覆盖缺口：`domain-scm` 后端 7 个实体，前端 `/scm` route 只覆盖 1.5 个

- **位置**: `frontend-design.md:341`（§3 路由表第 16 行："domain-scm | /scm | DetailPage | Table + SmView (7 PR SM) + Repository"）
- **依据**: `crates/domain-scm/src/lib.rs` 实际定义 7 个一等公民实体（`pub struct`）：`Repository:389`、`Branch:421`、`Commit:441`、`PullRequest:464`、`Review:521`、`Pipeline:539`、`WebhookEvent:559`，commit `c591c9a`/`6645350` 的说明本身也写"SCM 7 entities"。前端 §3/§6.3/附录 A 对 `/scm` route 的描述只提到"Table + SmView(PR) + Repository"，`Branch`/`Commit`/`Review`/`Pipeline`/`WebhookEvent` 五个实体在 25-route 表、§6.3 数据流契约表、组件目录中均无任何字段投影或 UI 元素提及。
- **影响**: `PullRequest.state` 迁移强依赖 `Review`（review 状态变更触发 PR 迁移，`INV-SCM-07`）与 `Pipeline`（CI 结果触发 `Approved→Mergeable`）两个实体的数据，但前端设计里这两者完全不可见——用户在 PR 详情页看到状态从 `REVIEWING` 变成 `APPROVED`，却没有任何 UI 展示"是哪个 Review 触发的"；`WebhookEvent`（`INV-SCM-08`"Webhook 入站 100% 写 Audit"）也没有对应的前端可观测入口，与 §6.6"Loop 防护"想要展示的 webhook 幂等信息（见 FD-08）互相矛盾——前端一边说要展示 webhook 相关信息，一边完全没规划 `WebhookEvent` 实体的路由/组件。
- **期望修正**: 在 `/scm` route 的组件契约中补充 `Branch`/`Commit`/`Review`/`Pipeline`/`WebhookEvent` 至少的只读列表视图（哪怕作为 PR 详情页的子 Tab），并在 §6.3 表格中补充这 5 个 Frontend type 行。

### FD-06 [Major] 反向覆盖缺口：`domain-automation` 后端已有执行历史与 dry-run 能力，前端只字未提

- **位置**: `frontend-design.md:348`（§3 第 23 行："domain-automation | /automation | ListPage | RuleCard | enabled toggle(占位)/24h 计数"）、`frontend-design.md:765`（FE-OI-08"Automation rule 24h 计数实时刷新?"）
- **依据**: `crates/domain-automation/src/lib.rs` 除 `AutomationRule` 外还定义了 `AutomationExecution:404`（执行记录实体）、`ExecutionResult:432`（枚举）、`RuleExecuted:567`（领域事件）、`TestRuleCommand:643`（显式的 dry-run 命令）与 `RuleExecutor` trait（`:696`，供 Worker 调用）。
- **影响**: 前端"已知缺口"FE-OI-08 只问"24h 计数是否需要实时刷新"，完全没有意识到后端已经建模了**执行历史列表**（`AutomationExecution`）和**规则试跑**（`TestRuleCommand`）这两个更大的能力面——这不是"性能/准确性"层面的小缺口（FE-OI-08 归为 P2），而是整整一块被遗漏的功能：用户无法在 UI 上看到"这条规则最近 24h 到底执行了哪几次、结果是什么"，也无法在编辑规则前"试跑一下看看会触发什么 Action"。
- **期望修正**: 在 `/automation` route 增加 DetailPage 模式（当前是纯 ListPage），展示某条 Rule 的 `AutomationExecution` 历史列表 + `ExecutionResult`；为 RuleCard 增加"Test Rule"按钮对应 `TestRuleCommand`；相应更新 §6.3 数据流契约表，补充 `AutomationExecution`/`ExecutionResult` 两个 Frontend type。

### FD-07 [Major] §6.2 Local Runtime"三重绑定"字段与 `domain-local-runtime` 实际结构体不符

- **位置**: `frontend-design.md:538-546`（§6.2，要求详情页显示 `device_id`/`tenant_id`/`user_id`/`mount_root`，任何 mismatch → `status=compromised` + `audit.policy_violation` 红色高亮）
- **依据**: `crates/domain-local-runtime/src/lib.rs:149-167`（`LocalRuntime` 结构体）实际字段为 `id/tenant_id/user_id/device_id/status/version/capabilities/last_heartbeat/registered_at/metadata`——**没有 `mount_root` 字段**；`RuntimeStatus` 枚举（`:96-105`）只有 `Online/Offline/Degraded/Maintenance` 四个变体，**没有 `compromised` 状态**；全 crate grep `policy_violation` 零匹配。`docs/specs/domain-local-runtime-spec.md:76-85` 定义的不变量编号是 `INV-LR-01~10`（10 条），前端 §6.3 写的是 "INV-LR-01~05"，范围也对不上。
- **影响**: §6.2 是安全相关小节（Local Runtime 三重绑定防伪装），但其描述的具体字段/状态在当前后端实现里根本不存在——按此设计做的详情页会尝试渲染一个不存在的 `mount_root` 字段和一个不存在的 `compromised` 状态分支。这更像是复制自更早期或规划阶段的设计草稿，未与当前 `domain-local-runtime` 代码回校。
- **期望修正**: 按实际字段重写 §6.2/§6.3：三重绑定改为核对 `device_id`/`tenant_id`/`user_id`（无 `mount_root`）；violation 高亮改为基于 `RuntimeStatus::Degraded`/心跳超时（`last_heartbeat` 与 `INV-RT-04`）而非不存在的 `compromised`/`policy_violations`；INV 范围改为 `INV-LR-01~10` 并注明 spec 与代码注释目前均未见 `INV-LR` 字样出现在 `domain-local-runtime` 源码里（这是 spec/代码本身的既有缺口，超出本次前端审核范围，但前端不应凭空引用一个代码里查不到的编号）。

### FD-08 [Major] §6.3 数据流契约表 3 处字段名与后端实际字段不符

- **位置**: `frontend-design.md:562,567,569`（§6.3 表格：Worktree 行"lock_version"、Repository/PullRequest 行"webhook_idempotency_key"、Notification 行"suppression_reason"）
- **依据**:
  - Worktree：`crates/domain-worktree/src/lib.rs:222` 乐观并发字段名为 `pub version: u32`，全 crate 无 `lock_version` 字样。
  - SCM：`crates/domain-scm/src/lib.rs:571` 字段名为 `pub idempotency_key: String`，无 `webhook_` 前缀。
  - Notification：`crates/domain-notification/src/lib.rs:210-224`（`Notification` 结构体）字段为 `id/tenant_id/user_id/event_type/resource_type/resource_id/channel_id/subject/body/status/created_at/sent_at/read_at/retry_count`，**没有 `suppression_reason` 字段**——INV-N-07 的抑制逻辑是在写入前通过 `NotificationEventType::is_suppressed()`（`:116`）拦截，被抑制的事件根本不会产生 `Notification` 记录，因此不存在"字段"意义上的抑制原因可展示。
- **影响**: 三处均为"前端字段名与后端字段名不同但语义相近"的低成本对接返工点；其中 Notification 的问题更严重——§3 路由表（`frontend-design.md:343`）与 §7.1 图（`:616-621`）都要求"InboxList + SuppressIndicator"、"INV-N-07 抑制标记"作为 UI 元素，但既然后端根本不落库被抑制的事件，前端就无法从 `GET /v1/notifications` 拿到"哪些通知被抑制了"的列表——这个 UI 需求在当前后端数据模型下**无法实现**，不是字段改名能解决的，需要后端补一张抑制日志表或前端放弃这个需求。
- **期望修正**: `lock_version`→`version`，`webhook_idempotency_key`→`idempotency_key`；`suppression_reason` 一行需要与后端确认是否要新增"被抑制事件审计表"，否则应从 §6.3/§3/§7.1 中移除"SuppressIndicator"这个 UI 需求或改为"V1 候选，待 backend 补充抑制日志"。

---

## Findings — Doc Hygiene（内部矛盾 / 遗漏，不涉及 backend 事实错误）

### FD-09 [Minor] P-FE-5"Server-render first"与"Page 100% = Client Component"自相矛盾

- **位置**: `frontend-design.md:204`（P-FE-5"Server-render first：列表/详情走 RSC；只有交互式组件（sm/canvas）走 client"）对照 `frontend-design.md:150`（"关键不变量：Page 100% = Client Component（`"use client"`）"）与 `frontend-design.md:711-715`（ADR-FE-004"25 route 的 page.tsx 全部 `use client`"）
- **依据**: 三处对同一层面（Page 级别是否为 RSC）给出互斥的结论：P-FE-5 说"列表/详情走 RSC"，另外两处说"Page 100% 是 Client Component"。
- **影响**: 下游实施工程师如果先读 §1.4 设计原则会认为列表页应实现为 Server Component，与实际落地的 ADR-FE-004（且 `frontend/src/app/*/page.tsx` 实际也确认全部标了 `"use client"`）冲突，容易在 V1 重构时误判"哪些 Page 本来就该是 RSC"。
- **期望修正**: P-FE-5 补充一句"MVP 阶段本原则**未生效**（见 ADR-FE-004，因 Zustand mock-first 需要 client component），仅作为 V1 切真后端后的目标状态"。

### FD-10 [Minor] §5.2 复用率矩阵的具体百分比与"100% 复用"的反复宣称相互矛盾

- **位置**: `frontend-design.md:447-460`（§5.2 表格：`StatusPill` 92%、`Stat` 19%、`SectionTitle` 42%、`StateMachineDiagram` 23%、`ListPage` 38%）对照同页"核心目标：`StatusPill` 100% 复用…`StateMachineDiagram` 100% 复用"及 `frontend-design.md:152,406`（"6 状态机都有独立 SmView 组件，UI 复用率达 100%"）
- **依据**: 表格自己给出的分子分母（`StatusPill` 24/26、`StateMachineDiagram` 6/26）与紧接着的文字断言"100%"矛盾——两处其实在说不同的分母（"在用到状态机的 6 个 route 里 100% 用同一组件" vs "在全部 26 个 route 里的占比"），但文档没有做这个区分，字面上是两个相反的数字。
- **影响**: 影响小，纯属措辞歧义，但"100%复用"被多次用作 ADR-FE-002 的核心论据，容易被下游当作可验证的 KPI 误读。
- **期望修正**: 把"100%复用"统一改为"在其适用的 6/24 个 route 内 100% 复用同一组件"，避免与全局占比数字（23%/92%）产生字面冲突。

### FD-11 [Minor] §3 表头"三选一"与实际选项数不符

- **位置**: `frontend-design.md:319`（"页面模式三选一：Dashboard(聚合)/ListPage(列表+筛选)/DetailPage(列表+详情+状态机)/StatsPage(统计+图表)"——列了 4 个；"主组件三选一：Table/Kanban/SmView/FlowChart/Canvas/List"——列了 6 个）
- **依据**: 数字面版本明显笔误。
- **期望修正**: "三选一"改为"四选一"/"六选一"，或去掉具体数字改为"以下几种之一"。

### FD-12 [Minor] §5.1 组件目录树与 `frontend/src/components` 实际落地结构不符

- **位置**: `frontend-design.md:419-445`（§5.1，声称 `components/` 下分 `atoms/molecules/organisms/layout` 四个子目录，`molecules/` 下含 `StatusPill.tsx/PageHeader.tsx/Stat.tsx/SectionTitle.tsx/Row.tsx` 5 个独立文件）对照 `frontend/src/components/` 实际只有 5 个平铺文件：`StatusPill.tsx/PageHeader.tsx/StateMachineDiagram.tsx/Sidebar.tsx/Topbar.tsx`（无子目录，且 `Stat.tsx`/`SectionTitle.tsx`/`Row.tsx` 三个声称存在的文件不存在，`frontend/src/lib/page-builders.tsx` 也只导出 `ListPage`/`StatsPage` 两个函数，未见 `Stat`/`SectionTitle`/`Row` 的独立实现）
- **影响**: 影响面窄，但 `frontend-design.md:881`"下游交接清单"第 1 条明确写"前端实施：`D:\Star\frontend\` **已按本设计落地**"——这个断言不完全成立，组件粒度上有真实落差（并非仅仅是文件位置差异，是 3 个声称的 molecule 组件实际未被拆分为独立文件）。
- **期望修正**: 更新 §5.1 目录树以反映当前扁平结构，或在"下游交接清单"中改为"已部分落地，组件粒度细化（atoms/molecules 拆分、Stat/SectionTitle/Row 独立组件化）留待 V1"。

### FD-13 [Minor] MVP 唯一承诺的交互（⌘K SearchPanel）实际未实现

- **位置**: `frontend-design.md:312`（§2.4"MVP 实现：只做 ⌘K（SearchPanel 抽屉），其他 V1 候选"）、`frontend-design.md:653`（§8.1 同一承诺，标注"✅ V0.1"）
- **依据**: `frontend/src/components/Topbar.tsx:24` 只在搜索框 `placeholder` 文案里写死了字符串 `"...⌘K)"`，全组件 grep `metaKey`/`ctrlKey`/`onKeyDown`/`SearchPanel` 均零匹配——没有任何键盘事件监听器，也没有 SearchPanel 抽屉组件。
- **影响**: §8.1 明确用"✅ V0.1"标注这是已完成项，与 `frontend/` 当前代码状态不符；这是"下游交接清单"整体可信度的一个具体反例。
- **期望修正**: 将 §2.4/§8.1 中 ⌘K 的状态改为"⚠️ 占位（仅 UI 文案，未绑定交互）"，或在下一次前端实施迭代中补齐 `useEffect` 键盘监听 + SearchPanel 组件。

### FD-14 [Minor/Question] PRSM 引用的 `INV-SCM-05~08` 范围不准确，只有 `INV-SCM-07` 真正描述状态机迁移

- **位置**: `frontend-design.md:363`（附录 B："PRSM | 7 | 8 | INV-SCM-05~08 | …"）、`frontend-design.md:341,717`（§3/`ids.ts` 同样引用这个范围作为 PR 状态机的不变量依据）
- **依据**: `crates/domain-scm/src/lib.rs:14-23` 逐条列出 `INV-SCM-01~08` 的实际含义：`05`=Credential Broker 不存明文、`06`=PR Content Object Storage Key 必带 tenant 前缀、`07`=`PullRequest.state 状态机严格按 §7.5 迁移`、`08`=Webhook 入站 100% 写 Audit——四条里只有 `07` 是状态机迁移本身的不变量，`05`/`06`/`08` 分别是凭据管理、存储隔离、审计相关，与"PR 状态机"的关联是间接的（Credential/Content/Webhook 是 PR 生命周期里出现的旁路概念，不是迁移规则本身）。
- **影响**: 影响小，但如果下游据此认为"改任意一条 INV-SCM-05/06/08 都要联动测试 PR 状态机迁移逻辑"，会误扩测试范围；反过来，如果真正相关的 `INV-SCM-07` 被淹没在一个 4 条一起引用的范围里，也容易在写状态机单测时被忽略成"这只是一堆不太相关的编号之一"。
- **期望修正**: 若目的是"标注这 4 条与 PR 领域相关的不变量"，措辞改为"关联 INV-SCM-05~08（其中迁移规则本身见 INV-SCM-07）"，避免暗示四条同等地都是迁移规则。

---

## 衍生发现（超出 `frontend-design.md` 自身范围，但直接影响 FD-01/FD-01B 的归因）

### FD-15 [Major] PR SM"7 状态"这个数字的真正源头是 `basic-design.md §7.6` 自相矛盾，并已传播进 `domain-scm` 源码注释——frontend-design.md 只是抄错的最后一环

- **位置**: `basic-design.md:2478`（§7.6"状态机总览表"："PullRequest | 7 | 2 (User / Webhook) | A.6"）对照同一份文档的 `basic-design.md:2444-2456`（§7.5 正文，8 个状态：`DRAFT/OPEN/REVIEWING/CHANGES_REQUESTED/APPROVED/MERGEABLE/MERGED/CLOSED`）与 `basic-design.md:3395-3412`（附录 A.6，同样 8 状态）
- **依据**: `basic-design.md` 自己的 §7.6 汇总表把 PullRequest 状态数记成 "7"，但同一文档紧邻的 §7.5 正文流程图与附录 A.6 都明确画出 8 个状态——这是 `basic-design.md` 内部的自相矛盾，且**没有被 `docs/basic-design-feedback.md` 捕获**（该审核文档的 F-08 精确抓到了同一类错误——AgentSession 总览表记 13、实际 14——但没有对 PullRequest 这一行做同样的核对）。这个"7"进一步传播进了代码：`crates/domain-scm/src/lib.rs:228` 的文档注释写着 `/// **PR 状态**(7 状态机,§7.5)`，但紧接着定义的 `PullRequestState` 枚举（`:232-249`）有 8 个变体——即代码本身的注释与代码本身的实现都对不上，说明这个"7"是原样从 §7.6 抄过去的，没有人回头数过 §7.5 正文或枚举变体数。frontend-design.md（§4.1、附录 B、`ids.ts`）里的"7"与这两处完全一致，是同一条传播链的第三环，而不是独立发明的错误。
- **影响**: 这改变了 FD-01 中 PR 行"7≠8"的责任归属——前端不是这条错误信息的源头，只是忠实继承者；真正需要先修的是 `basic-design.md §7.6` 这一行数字，否则即使按本报告的建议重写 `ids.ts`，下一次有人回去对照"官方总览表"时仍会看到"7"这个错误数字，可能把前端刚改对的"8"又改回"7"。这也说明 `docs/basic-design-feedback.md` 当初的"上游一致性传播检查"（其方法论原文就是要核对"状态机状态数等关键数字的传播链路"）在 PullRequest 这一项上有遗漏，属于该文档自身的一处残留缺陷。
- **期望修正**: 优先修 `basic-design.md:2478` 的"7"→"8"（并同步核对 §7.6 表里其它 5 行是否与各自附录一致，本次未逐行复核）；再修 `crates/domain-scm/src/lib.rs:228` 的文档注释"7 状态机"→"8 状态机"；最后按 FD-01 的建议重写 `ids.ts` 的 `PR_SM`。三处应作为同一个 PR/commit 一起修，避免再次出现"改了代码没改文档，或改了下游没改上游"的传播断裂。

---

## 未审核 / 超出本次范围

- 除 SCM/Automation 外，其余 23 个 domain 的"反向覆盖"（backend 功能 → 前端是否遗漏 UI/字段）未逐一排查，仅验证了高可疑样本（SCM 7 实体、Automation 执行历史）；建议后续对 `domain-relation`（前端标"占位：Graph viz"）、`domain-workflow`（FlowChart 是否支持 guard CEL 表达式展示）、`domain-collaboration`（Presence 之外是否有 Whiteboard 后端支撑）做同等深度的反向核对。
- `frontend/src/lib/seed.ts` 的具体 mock 数据内容（是否与 FD-01 指出的错误状态名一致地贯穿全部 mock 记录）未逐行核对，只核对了 `types/ids.ts` 的状态机定义源头。
- api-design.md 之外的其余 8 份详细设计文档（`data-design.md`/`security-design.md`/`runtime-design.md`/`integration-design.md`/`ai-agent-design.md`/`external-design.md`/`internal-design.md`/`operation-design.md`）与 frontend-design.md 的交叉一致性未检查（仅用 `api-design.md`/`basic-design.md` 作为 frontend-design.md 自称的直接上游）。
- 前端可访问性、i18n、性能（BurndownChart/Kanban/Audit 虚拟滚动）等 §10.2 已列出的 Open Issues 本身的优先级判断未复核，只指出 FD-06 揭示的"Automation 执行历史"是一个 FE-OI 列表里完全遗漏的新缺口。

---

## 总体结论

**Finding 统计**：共 16 条（Blocker 2：FD-01/FD-01B；Major 8：FD-02/FD-03~FD-08/FD-15；Minor 6：FD-09~FD-14）。

**根因归类**：

1. **状态机词汇未回查源码，整体是向壁虚构，且清单本身少数了一个 SM**（FD-01/FD-01B，影响面最大且是 Blocker）——6 个已列 SM 里 5 个的状态名与 backend 完全不同构（仅数量巧合相同），PR 连数量都对不上（但见第 3 点，这一项的数字错误另有更根本的上游成因）；同时"继承 §7"的 §7 实际有 7 个 SM，Decision 被整体遗漏。这直接推翻了附录 B"与 backend 严格一致"、ADR-FE-002"UI 复用率 100%"、ADR-FE-003"V1 切真后端只需改 `store.ts`"三处核心断言的前提。
2. **对 backend 当前实现进度的判断已过时，且引用了一个不存在的 crate 名**（FD-02）——ADR-FE-003 写作时可能 backend 确实只有骨架，但截至本次审核，25 个 domain crate 已有状态机+单测，且理由栏引用的"`domain-api`"crate 根本不存在（真实名为 `api`）；Mock-first 决策本身可以保留，但"理由"部分需要重写，且更需要正视 FD-01 揭示的"即使切后端，UI 层改动也很大"这一被低估的成本。
3. **错误码/事件总线契约照抄了一套自创命名法，未逐行核对 `api-design.md` 字典**（FD-03/FD-04）——6 个错误码、全部 NATS Subject 均对不上，其中 Subject 缺失 tenant_id 隔离段还触及安全边界。
4. **反向覆盖检查缺失**（FD-05/FD-06）——backend 新落地的 SCM 7 实体、Automation 执行历史/dry-run 能力，前端设计阶段完全没有捕捉到，说明 frontend-design.md 编写时主要参照的是较早期/较抽象的 `basic-design.md`/`api-design.md`，而未对照 commit `c591c9a`/`f7587a3` 之后的实际 domain 代码。
5. **字段级抄录未回校当前代码**（FD-07/FD-08）——Local Runtime、Worktree、SCM、Notification 四处字段名/结构与当前代码不符，Notification 的 `suppression_reason` 更是在当前数据模型下无法实现的 UI 需求。
6. **上游文档自身数字矛盾并已传播进代码注释**（FD-15，超出 frontend-design.md 范围但是 FD-01 归因的关键依据）——`basic-design.md §7.6` 总览表把 PullRequest 记成 7 状态，与同文档 §7.5 正文/附录 A.6 的 8 状态矛盾，且这个错误已经传播进 `domain-scm` 源码的文档注释，`docs/basic-design-feedback.md` 当初的"传播链路核对"在这一项上有遗漏（对比它成功抓到的 AgentSession 13→14 一例，即 F-08）。
7. **文档自身局部矛盾与"已完成"断言与代码现状不符**（FD-09~FD-14）——多为写作时未回读自己之前章节或未回查 `frontend/` 现状导致。

**是否可以进入前端实施下一阶段（V1 / 真后端对接）**：**有条件通过（Conditional Pass）**。建议按以下优先级处理：

1. **必须在 V1 切真后端前修复**：FD-01/FD-01B（重写 6→7 个状态机的状态名/迁移，这是所有交互与可视化的基础）、FD-15（先修正 `basic-design.md §7.6` 与 `domain-scm` 注释里的"7"，再改前端，避免修复方向反了）、FD-04（NATS Subject 补齐 tenant_id 隔离段，否则有跨租户订阅风险）、FD-03（错误码改用真实字典，否则错误处理分支永远走不到）。
2. **建议在下一轮详细设计修订中一并处理**：FD-02（重写 ADR-FE-003 理由，改引用真实 crate 名）、FD-05/FD-06（补齐 SCM/Automation 的反向覆盖缺口）、FD-07/FD-08（字段名回校）。
3. **可与常规文档维护一并处理，不阻塞**：FD-09~FD-14。
