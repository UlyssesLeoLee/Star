# 33. Event Model

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/flows/07-audit-model.md](07-audit-model.md)

## 1. 所有 Agent / IDE / 代码工作区操作产生标准 Domain Event

（per §34 任务原文）

### 1.1 STAR Domain Events

```
AgentTaskClaimed
ContextRequested
WorkspaceCreated
WorktreeCreated
IDESessionStarted
CodeNavigationRequested
CodeModified
ValidationStarted
ValidationFailed
ValidationSucceeded
MergeRequestCreated
HumanReviewRequested
AgentTaskCompleted
```

### 1.2 GitGit 原生事件

```
RepositoryCreated
CommitCreated
BranchCreated
RefUpdated
WorktreeCreated
WorktreeRemoved
ObjectsReceived
ObjectsFetched
MergeCompleted
ConflictDetected
```

> **重名事件边界澄清（per B-17 修复 2026-08-27）**：`WorktreeCreated` 在 §1.1 (STAR Domain Events) 和 §1.2 (GitGit 原生事件) **双名**。命名表：
>
> | 事件名（命名空间前缀） | 层 | 含义 | 触发 |
> |---|---|---|---|
> | `WorktreeCreated.star` | 逻辑层（STAR Domain Event） | Workspace/Worktree 绑定完成（含 binding metadata） | `gitgit_bridge.rs` 收到 GitGit `WorktreeCreated` 后，在 STAR 业务层**重新发射**（per [arch/05 §3](../../arch/05-gitgit-compat-arch.md) bridge 设计）|
> | `WorktreeCreated.gitgit` | 物理层（GitGit 原生事件） | `git worktree` 实际创建（git CLI 退出 0） | `git worktree add` 命令执行成功 |
> | 命名空间前缀 | 字段位置：`event.source` 字段值（`"star"` vs `"gitgit"`），避免与 event.type 混淆（per B-27 修复 2026-08-27 Event schema 7 字段）|
> | 触发顺序 | `WorktreeCreated.gitgit` (T0) → `WorktreeCreated.star` (T0 + bridge latency) | | |
>
> **关键约束**：
> - 两个事件**都是真事件**，不是 alias；订阅方按 `event.source` 区分
> - `WorktreeCreated.star` **由 STAR 业务层重新发射**，不直接转发 GitGit 原始事件（per [arch/05 §3](../../arch/05-gitgit-compat-arch.md) bridge 不透明原则）
> - 其他 12 个 STAR Domain Events 命名唯一，无重名风险

> v0.2 fix: 2026-08-27 per B-17 (WorktreeCreated 双名命名表)

## 2. 关键约束

- GitGit 事件必须与 AI Vendor / IDE Vendor 无关
- STAR 在上层把 GitGit 事件转译为软件工程领域事件

## 3. Event Schema（per B-27 修复 2026-08-27）

> **7 字段权威定义**（`DomainEvent` schema）：
>
> - `event_id` (string, UUID v7 时间排序 — e.g. `"event-2026-08-27-uuid-v7-..."`)
> - `type` (string, PascalCase — e.g. `"WorktreeCreated"` / `"AgentTaskClaimed"` / `"ValidationFailed"`)
> - `source` (string, 命名空间前缀 — e.g. `"star"` / `"gitgit"` / `"ide-session"`，per B-17 修复 2026-08-27 命名表)
> - `timestamp` (timestamp, ISO 8601 with timezone)
> - `trace_id` (string, 关联 [agent-api/v1 §3.15 Error.trace_id](../agent-api/01-schema.md#315-error))
> - `payload` (object, 领域特定 payload — e.g. `{task_id: "STAR-1024", from_state: "Implementing", to_state: "Failed"}` / `{worktree_id: "wt-...", branch: "feature/STAR-1024"}`)
> - `schema_version` (string, e.g. `"star-event/v1"` — 允许 schema 演进)
>
> **权威来源（per B-27 修复 2026-08-27）**：本 spec 定义 7 字段；权威 schema 由 W4 子代理落 [agent-api/v1 §3.X DomainEvent](../agent-api/01-schema.md)（预计 §3.19+ — W4 起草中，本 spec §3 字段定义需与 §3.X 1:1 对齐）。CLI / MCP / REST / Event 消费方 **全部**统一引用本 schema，per Error 模型 1 源原则。
>
> **source 字段命名规范（per B-17 修复 2026-08-27）**：
> - `source: "star"` — STAR 业务层发射（§1.1 13 个）
> - `source: "gitgit"` — GitGit 原生事件（§1.2 11 个）
> - `source: "ide-session"` — IDE Session 发射（Phase 2+）
> - `source: "automation"` — Automation 触发（per [flows/07 §3](07-audit-model.md) ActorType）
>
> **与 AuditEvent 关系**：本 spec `DomainEvent` 是 Event Bus 上传输的事件；[flows/07 §6](07-audit-model.md) `AuditEntry` 是审计落盘事件。**两份独立**（DomainEvent = 触发 / AuditEntry = 审计），但 `trace_id` 共享用于联动追踪。

> v0.2 fix: 2026-08-27 per B-27 (Event 7 字段 schema)

## 4. 实施位置

- `crates/star-event/` — Event bus
- `crates/star-event/src/star_events.rs` — STAR domain events
- `crates/star-event/src/gitgit_bridge.rs` — GitGit → STAR 事件转译（per B-17 修复 2026-08-27 bridge 重新发射 WorktreeCreated.star）
- `crates/star-event/src/domain_event.rs` — DomainEvent schema 落地（per B-27 修复 2026-08-27）

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：13 STAR + 11 GitGit 事件 + bridge 设计 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转）| 🟡 B-17：§1.1+§1.2 WorktreeCreated 双名命名表（`WorktreeCreated.star` 逻辑层 / `WorktreeCreated.gitgit` 物理层 + 触发顺序 + bridge 重新发射）· 🟡 B-27：§3 增 DomainEvent 7 字段 schema（`event_id` / `type` / `source` / `timestamp` / `trace_id` / `payload` / `schema_version`），W4 子代理落 [agent-api/v1 §3.X DomainEvent](../agent-api/01-schema.md) 后 1:1 对齐 + §4 增 `domain_event.rs` 实施位置 | worker 子代理修 INTERFACE-REVIEW-B 8 子代理报告 follow-up |

> v0.2 fix: 2026-08-27 per B-17 / B-27
