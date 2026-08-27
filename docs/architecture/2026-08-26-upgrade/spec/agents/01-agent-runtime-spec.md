# 29. Agent Runtime Contract

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/flows/01-agent-task-lifecycle.md](../flows/01-agent-task-lifecycle.md) · [spec/flows/02-agent-lease-heartbeat.md](../flows/02-agent-lease-heartbeat.md) · [spec/flows/03-agent-resume.md](../flows/03-agent-resume.md) · [spec/agent-api/01-schema.md](../agent-api/01-schema.md)
> **Phase 边界（per B-26 修复 2026-08-27 — flows/02 顶部）**：
> - **Phase 1 实现**：Task Lease + Heartbeat + Lease Renewal — 心跳固定 30s，TTL 固定 300s（per Project 配置可调）
> - **Phase 2+ 实现**：Session Timeout + Lease Recovery + 自适应心跳（10s baseline + adaptive based on workload / RTT）
> - **Phase 1 ↔ Phase 2+ 切换点**：自适应心跳启用时（Phase 2 触发）

## §0 目的

本 spec 是 Agent 运行时的契约性文档（Phase E spec 增量），把分散在 [spec/flows/01](../flows/01-agent-task-lifecycle.md) / [02](../flows/02-agent-lease-heartbeat.md) / [03](../flows/03-agent-resume.md) 三份 flow spec 中的"状态机 + Lease + Resume + 错误模型"四条线整合为一份单一权威契约，供 CLI / MCP / REST / IDE Gateway 跨通道一致实现，避免在 4 个通道中分别维护导致漂移（per [spec/agent-api/01 §3.28](../agent-api/01-schema.md) AgentTaskState enum + §3.16 Resume 11 字段 + §3.14 Error 6 字段的三处权威定义）。

## §1 状态机（14 状态 PascalCase）

> **权威枚举**：[`agent-api/v1` §3.28 AgentTaskState](../agent-api/01-schema.md#328-agenttaskstate) — **14 个 PascalCase 值**（per B-23 修复 2026-08-27 — INTERFACE-REVIEW-B 🟡 B-23）。**与 [flows/01 §1](../flows/01-agent-task-lifecycle.md) Task.status 14 值完全不同**（详见 §1.2 区别表）。

### §1.1 状态枚举 + 转换图

```
                    ┌───────────┐
                    │  Pending  │ 任务已分配，未启动
                    └─────┬─────┘
                          │ start
                          ▼
                    ┌───────────┐
              ┌────▶│ Assigned  │ 任务已发送给 Agent，等待启动信号
              │     └─────┬─────┘
              │           │ begin
              │           ▼
              │     ┌───────────┐
              │     │  Running  │ Agent 正在执行
              │     └─────┬─────┘
              │           │ invoke_tool
              │           ▼
              │     ┌────────────┐
              │     │WaitingTool │ 等待工具返回
              │     └─────┬──────┘
              │           │ tool_start
              │           ▼
              │     ┌────────────┐
              │     │ToolRunning │ 工具正在执行
              │     └─────┬──────┘
              │           │ tool_done
              │           ▼
              │     ┌──────────────┐
              │     │ToolCompleted │ 工具完成（→ Running 重入 loop）
              │     └─────┬────────┘
              │           │ resume
              │           ▼
              │     ┌───────────┐
              │     │  Running  │ (loop) ↺
              │     └─────┬─────┘
              │           │ request_feedback
              │           ▼
              │     ┌─────────────────┐
              │     │WaitingFeedback  │ 等待人类反馈
              │     └────────┬────────┘
              │              │ human_reply
              │              ▼
              │     ┌─────────────────┐
              │     │FeedbackReceived │ 已收到人类反馈
              │     └────────┬────────┘
              │              │ resume
              │              ▼
              │        ┌───────────┐
              │        │  Running  │
              │        └─────┬─────┘
              │              │ run_validation
              │              ▼
              │        ┌────────────┐
              │        │ Validating │ 正在跑验证
              │        └─────┬──────┘
              │              │ validation_done
              │              ▼
              │   ┌────────────────┐
              │   │   Completed    │ ─ 终态
              │   ├────────────────┤
              │   │    Failed      │ ─ 终态
              │   ├────────────────┤
              │   │    Aborted     │ ─ 终态
              │   ├────────────────┤
              │   │    Crashed     │ ─ 终态（→ flows/02 §3 Agent Lost 6 步恢复）
              │   ├────────────────┤
              │   │    Timeout     │ ─ 终态
              │   └────────────────┘
              │
              └────── (任何 active 状态可 Aborted / Failed / Crashed / Timeout)
```

**14 状态值**（[agent-api/v1 §3.28](../agent-api/01-schema.md#328-agenttaskstate)）：`Pending` · `Assigned` · `Running` · `WaitingTool` · `ToolRunning` · `ToolCompleted` · `WaitingFeedback` · `FeedbackReceived` · `Validating` · `Completed` · `Failed` · `Aborted` · `Crashed` · `Timeout`

### §1.2 与 flows/01 §1 Task.status 14 状态的区别（per B-23 修复 2026-08-27）

| 维度 | flows/01 §1 Task.status | agent-api §3.28 AgentTaskState（本 spec）|
|---|---|---|
| 视角 | issue 生命周期（任务走到哪一步）| agent runtime lifecycle（agent 自身处于什么运行时阶段）|
| 14 值 | Created / Claimed / ContextLoading / Planning / Implementing / Validating / ReviewReady / Submitted / Completed / Blocked / Conflict / Failed / Cancelled / HumanRequired | Pending / Assigned / Running / WaitingTool / ToolRunning / ToolCompleted / WaitingFeedback / FeedbackReceived / Validating / Completed / Failed / Aborted / Crashed / Timeout |
| 权威定义 | flows/01 §1 Rust enum | agent-api/v1 §3.28 |
| 关联 schema | §3.1 Task.status | §3.16 Resume.state |

> **守门**：本 spec §1 状态机与 [flows/01 §1](../flows/01-agent-task-lifecycle.md) **1:1 不同**，**禁止互相替换**。Task.status 描述 issue 走到哪一步；AgentTaskState 描述当前正在执行任务的 agent 自身运行时阶段（per [agent-api/v1 §3.328 底部 note](../agent-api/01-schema.md#328-agenttaskstate)）。

## §2 Lease 协议（30s heartbeat / 300s TTL）

> **字段权威定义**：[spec/flows/02 §3](../flows/02-agent-lease-heartbeat.md)（per B-04 修复 2026-08-27）— `AgentTaskLease` 6 字段 + [agent-api/v1 §3.24 Lease](../agent-api/01-schema.md#324-lease) 8 字段（双向引用，Lease schema 含 state enum）。

### §2.1 6 字段 Lease 子对象（per flows/02 §3）

| 字段 | 类型 | 默认值 | 说明 |
|---|---|---|---|
| `lease.ttl_seconds` | `u64` | 300 | 5 分钟（per Project 配置可调） |
| `lease.heartbeat_interval_seconds` | `u64` | 30 | 30s 心跳间隔 |
| `lease.acquired_at` | timestamp | — | Lease 申请时间 |
| `lease.expires_at` | timestamp | — | 由 `acquired_at + ttl_seconds` 派生 |
| `lease.heartbeat_at` | timestamp | — | 最近一次 heartbeat |
| `lease.renew_count` | `u32` | 0 | 续约次数 |

### §2.2 Heartbeat 协议（per flows/02 §4）

```rust
// Agent 每 30s 发一次
agent.heartbeat(agent_session_id, current_state, progress_pct)
  → server 更新 lease.heartbeat_at
  → server 检查 lease 是否过期

// 默认 lease TTL: 5 分钟（per Project 配置可调）
```

**常量落位**（per B-20 修复 2026-08-27，flows/02 §4）：`30s` / `300s` 常量在 `crates/star-agent/src/lease.rs:1-10` 显式声明：

```rust
// crates/star-agent/src/lease.rs:1-10
pub const LEASE_TTL_SECONDS: u64 = 300;        // 5 分钟（per Project 配置可调）
pub const HEARTBEAT_INTERVAL_SECONDS: u64 = 30; // 30s
// Phase 2+ 自适应心跳（per B-26 修复 2026-08-27）：
pub const HEARTBEAT_INTERVAL_ADAPTIVE_BASE: u64 = 10; // 10s baseline + adaptive
```

**Phase 边界**（per B-26 修复 2026-08-27，flows/02 顶部）：**Phase 1 用固定 30s/300s**；**Phase 2+ 切自适应**（10s baseline + workload/RTT adaptive）。

## §3 Resume JSON 11 字段

> **权威定义**：[`agent-api/v1` §3.16 Resume](../agent-api/01-schema.md#316-resume) — **11 字段**（per B-19 修复 2026-08-27 — INTERFACE-REVIEW-B 🔴 B-19）。

### §3.1 11 字段（agent session 视角）

| # | 字段 | 类型 | 说明 |
|---|---|---|---|
| 1 | `id` | string | Resume 操作唯一 ID（e.g. `"resume-uuid-..."`）|
| 2 | `agent_id` | string | 被 Resume 的 Agent ID（per §3.23 Identity）|
| 3 | `state` | enum `AgentTaskState` | per [§3.28](../agent-api/01-schema.md#328-agenttaskstate)，14 PascalCase 值 |
| 4 | `last_heartbeat_at` | timestamp | 上次心跳时间（per [flows/02](../flows/02-agent-lease-heartbeat.md)）|
| 5 | `lease_expires_at` | timestamp | 租约到期时间（per [§3.24 Lease](../agent-api/01-schema.md#324-lease)）|
| 6 | `current_state` | string | PascalCase — Agent 运行时状态别名，**冗余**于 `state`，保留以兼容 [flows/03 §2](../flows/03-agent-resume.md) 协议 |
| 7 | `current_step` | string | e.g. `"validate:2-of-5"`, `"commit:1-of-1"`, `"submit:waiting-policy"` |
| 8 | `retry_count` | integer | 重试次数 |
| 9 | `artifacts` | `Artifact[]` | `{path, content_type, sha256}`，产出物列表 |
| 10 | `checkpoint` | `Checkpoint` | `{id, snapshot_id, taken_at, store_ref}`，checkpoint 引用 |
| 11 | `recovery_hint` | string | 恢复提示（替换 v0.2 的 `failed_attempts` + `remaining_work`）|

> **B-19 字段集来源**：11 字段定义 = `id` / `agent_id` / `state` / `last_heartbeat_at` / `lease_expires_at` / `current_state` / `current_step` / `retry_count` / `artifacts` / `checkpoint` / `recovery_hint`，per INTERFACE-REVIEW-B 🔴 B-19 (2026-08-27)。
>
> **已知缺口**（缺标比错标安全）：[flows/03 §2 末尾 note](../flows/03-agent-resume.md) 显式列出 — 任务摘要列的 11 字段（`id, agent_id, state, last_heartbeat_at, lease_expires_at, current_state, current_step, retry_count, artifacts, checkpoint, recovery_hint`）与 [flows/03 §2 协议内 JSON](../flows/03-agent-resume.md) 实际 11 字段（`current_state, workspace, worktree, previous_plan, modified_files, open_diagnostics, test_results, failed_attempts, relevant_context, remaining_work, last_modified`）**不对齐**。本 spec §3 以 [`agent-api/v1` §3.16](../agent-api/01-schema.md#316-resume) 为准（权威 schema）。`flows/03 §2` JSON 协议已弃用，flows/03 后续 v0.4 fix 需同步。

## §4 Agent Lost 恢复（6 步流程）

> **来源**：[spec/flows/02 §3](../flows/02-agent-lease-heartbeat.md) — 6 步恢复流程（per B-03 修复 2026-08-27 状态转换 cross-ref）。

```
Agent Lost  (per §1.1 Crashed 终态)
  ↓ ①
保存 Workspace   (per [agent-api §3.15 WorkspaceSummary](../agent-api/01-schema.md#315-workspacesummary) agent 视角)
  ↓ ②
保存 Worktree    (per [agent-api §3.2 Worktree](../agent-api/01-schema.md#32-worktree) + §3.11 WorktreeStatus)
  ↓ ③
保存 Context Snapshot  (per [spec/resources/03-agent-identity.md §3](../resources/03-agent-identity.md))
  ↓ ④
释放 Task Lease  (per [agent-api §3.24 Lease](../agent-api/01-schema.md#324-lease) state: active → released)
  ↓ ⑤
状态机转换: 崩溃前主态 (Running / WaitingTool / ToolRunning / ToolCompleted / WaitingFeedback / FeedbackReceived / Validating) → Failed (不可恢复) | Cancelled (可恢复 + 其他 Agent 接管)
  ↓ ⑥
允许其他 Agent Resume  (per §3 Resume 11 字段 JSON)
```

**5/6 步流程口径**：flows/02 §3 列 6 步（Agent Lost → 保存 Workspace → 保存 Worktree → 保存 Context Snapshot → 释放 Task Lease → 允许其他 Agent Resume）。本 spec §4 在第 5 步后插入显式状态转换（Failed / Cancelled），共 **6 步（含状态转换）** = 5 步（资源回收）+ 1 步（状态机落态）。

> **守门**：当 Agent 在 [flows/01 §3 活跃态](../flows/01-agent-task-lifecycle.md)（`Implementing` / `Validating` 等）崩溃时，Task.status 落 `Failed` 或 `Cancelled`；同时本 spec §1.1 AgentTaskState 落 `Crashed`（agent 自身运行时阶段），**两层状态机独立落态**。

## §5 错误模型（6 字段）

> **唯一权威**：[`agent-api/v1` §3.14 Error](../agent-api/01-schema.md#314-error) — **6 字段**（per F-06 修复 2026-08-27 — INTERFACE-REVIEW-A 🔴 #6）。CLI / MCP / REST / Universal Submit **全部**引用本 schema。

| # | 字段 | 类型 | 说明 |
|---|---|---|---|
| 1 | `code` | string | e.g. `"WORKTREE_CONFLICT"` — 标准化 SCREAMING_SNAKE_CASE 错误码（**改名自 v0.2 的 `error` 字段**，避免与 HTTP 字段冲突）|
| 2 | `message` | string | human-readable |
| 3 | `source_module` | string | e.g. `"agent-core"` \| `"ide-gateway"` \| `"vcs"` \| `"policy"` \| `"mcp"` \| `"rest"` \| `"cli"` |
| 4 | `source_kind` | enum | `internal` \| `external` \| `policy` \| `validation` \| `user_input` \| `timeout` |
| 5 | `retriable` | boolean | **改名自 v0.2 的 `recoverable`**，避免与 `recoverable=true` 语义混淆 |
| 6 | `hint` | string? | 恢复提示（替换 v0.2 的 `suggested_actions[]`），单字符串更易消费 |

> **F-06 字段集来源**：6 字段定义 = `code` / `message` / `source_module` / `source_kind` / `retriable` / `hint`，per INTERFACE-REVIEW-A 🔴 #6 + 子代理协调结果（2026-08-27）。v0.2 的 `error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details` 6 字段已弃用。

**Runtime 阶段典型错误码**（Phase D 落地时校对，per §1 状态机）：

| §1 状态 | 典型 `code` | `source_module` | `retriable` |
|---|---|---|---|
| `WaitingTool` | `TOOL_TIMEOUT` | `"agent-core"` | true |
| `ToolRunning` | `TOOL_FAILED` | `"agent-core"` | false |
| `Running` | `LEASE_EXPIRED` | `"agent-core"` | true（触发 §4 6 步恢复）|
| `WaitingFeedback` | `FEEDBACK_TIMEOUT` | `"agent-core"` | true |
| `Validating` | `VALIDATION_FAILED` | `"agent-core"` | false |
| 任何 active 态 | `CRASHED` | `"agent-core"` | true（触发 §4 6 步恢复）|

## §6 已知缺口（缺标比错标安全）

| # | 项 | 状态 | 阻塞 | 备注 |
|---|---|---|---|---|
| 1 | Phase 2+ 自适应心跳（10s baseline + adaptive）未实现 | 未启动 | Phase 2 触发 | Phase 1 固定 30s/300s 已就位 per flows/02 §4 |
| 2 | Session Timeout（per flows/02 §2）未实现 | 未启动 | Phase 2 触发 | flows/02 §2 列在 5 项解决方案中但 §1 边界明确为 Phase 2+ |
| 3 | Lease Recovery（per flows/02 §2）未实现 | 未启动 | Phase 2 触发 | 同上 |
| 4 | flows/03 §2 JSON 协议 vs agent-api §3.16 Resume 11 字段不对齐 | 待 flows/03 v0.4 fix | 需 Ulysses DDD Review 终审 | flows/03 §2 末尾 note 已显式声明；本 spec §3 以 §3.16 为准 |
| 5 | §5 Runtime 错误码表（`TOOL_TIMEOUT` / `LEASE_EXPIRED` 等）未在 agent-api §3.14 显式枚举 | 草案 | Phase D 落地校对 | 现有 §3.14 仅定义 schema 字段，未列 `code` 取值枚举 |
| 6 | `crates/star-agent/src/lease.rs` 常量实际落位未在仓库存在 | 待实现 | Phase D | flows/02 §4 给的 `lease.rs:1-10` 是参考位置，非实测行号 |
| 7 | §1.1 转换图未覆盖所有活跃态 → 终态的边（如 `WaitingTool` → `Timeout`）| 简化 | 视觉密度 | 当前图示 7 个 active 态主要转换路径，剩余边可按 §1.2 Task.status 14 状态 1:1 推 |
| 8 | `current_state` 字段（§3.1 #6）"冗余于 `state`"的兼容策略 | 待 Mavis 接手定 | 跨版本演进 | 当前 §3.16 描述为"冗余保留以兼容 flows/03 §2"；flows/03 弃用后是否移除未决 |
| 9 | §4 第 5 步"状态机转换"插入位置（5 步 → 6 步口径）| 解释项 | 无 | 本 spec §4 与 flows/02 §3 描述顺序一致，第 5 步后插状态转换是为了显式落态，不改 flows/02 原文 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转 + 19:39 JST 代签授权升级）| 初版：整合 flows/01+02+03 为单一 Agent Runtime Contract（§0 目的 / §1 14 状态 PascalCase + 转换图 + 与 Task.status 14 状态区别 / §2 6 字段 Lease + 30s/300s + Phase 1/2 边界 / §3 Resume 11 字段 + flows/03 弃用声明 / §4 Agent Lost 6 步 + 状态转换显式落态 / §5 Error 6 字段 + Runtime 阶段典型错误码 / §6 已知缺口 9 项 / §7 修订历史） | Phase E spec 增量 worker 任务 — 把分散的 3 份 flow spec 整合为契约文档 |

---

> **审批者**：架构师 (Mavis 接手 agent per DEC-008) — 2026-08-27
> **per AGENTS.md §1 代签规则反转 + 2026-08-27 19:39 JST 代签授权升级**：Mavis 接手默认代签 Ulysses 无需再问
> **per AGENTS.md §1.2 派生约束保留项**：禁回溯叙事 / BAS 引用 git 实证 / 缺标比错标 / 子代理授权"无证据叙事 = 禁止" — §6 已知缺口 9 项已显式列未确定项
