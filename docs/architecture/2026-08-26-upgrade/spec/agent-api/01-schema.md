# 12. Agent CLI JSON Schema

> **状态**：🟡 草案 v0.2
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md) · [spec/cli/01-cli-spec.md](../cli/01-cli-spec.md)

## 1. Versioning

- Schema version: `agent-api/v1`（per F-12 修复 2026-08-27：见下方"双版本字段"说明）
- Breaking change 必须升 v2
- 任何 field 重命名 / 移除 / 类型变更都算 breaking
- New field 是 additive（minor）
- **`info.version` vs `schema_version` 双版本字段**（per F-12 修复 2026-08-27 — INTERFACE-REVIEW-A 🟡 #11）：
  - `info.version`（OpenAPI 3.1 顶层 `info` 字段）— **OpenAPI 文档版本**，用 SemVer 字符串（`"1.0.0"`），面向 OpenAPI 工具链（Redocly / openapi-generator）
  - `schema_version`（Capabilities / Error 等 schema 内嵌字段，见 §3.17 / §3.14）— **schema 业务版本**，用 `agent-api/vN` 字符串，面向 runtime consumer（CLI / MCP / Agent）
  - 二者**同步演化**：schema `v1.x` ↔ `info.version: "1.x.0"`；schema `v2`（breaking）↔ `info.version: "2.0.0"`
  - Patch（bug fix 字段语义）走 `metadata` 字段标注，**两个版本字段都不动**
  - Example：`agent-api/v1.0` ↔ `info.version: "1.0.0"`；`agent-api/v1.1`（additive）↔ `info.version: "1.1.0"`；`agent-api/v2`（breaking）↔ `info.version: "2.0.0"`
- 完整 OpenAPI 3.1 规范：webhooks 字段允许 / `nullable: true` 替换为 `type: [string, "null"]` / `info.summary` 允许（per 子代理 A 🟡 #10）

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟡 #11 (F-12) — 明确 `info.version` (OpenAPI SemVer) vs `schema_version` (agent-api/vN 业务) 关系

## 2. 核心 Schema（顶层）

```yaml
openapi: 3.1.0
info:
  title: STAR Agent API
  version: "1.0.0"
  description: |
    Machine-readable schema for any Coding Agent / AI Agent
    to interact with STAR.
  license:
    identifier: Apache-2.0
  summary: STAR Agent API v1.0 (P1-D 2026-08-27 versioning 规则首版)
```

## 3. 核心 Schemas

> §3.1-§3.13 = 13 个 issue/worktree/code 领域核心 schema；§3.14 = **Error**（per F-06，唯一权威，6 字段）；§3.15 = **WorkspaceSummary**（per F-05，agent 视角）；§3.16 = **Resume**（per B-19，agent session 视角，11 字段）；§3.17 = Capabilities；§3.18-§3.27 = 10 个新增 schema（Validation / Audit / Event / Decision / Permission / Identity / Lease / Integration / Notification / Search，per F-04 扩展）；§3.28 = **AgentTaskState** enum（per B-23，14 PascalCase 值）。共 **28 schema**。

### 3.1 Task

- `id` (string, e.g. `"STAR-1024"`)
- `title` (string)
- `status` (enum: `Created` | `Claimed` | `ContextLoading` | `Planning` | `Implementing` | `Validating` | `ReviewReady` | `Submitted` | `Completed` | `Blocked` | `Conflict` | `Failed` | `Cancelled` | `HumanRequired` — PascalCase, 跟 [spec/flows/01 §1](../flows/01-agent-task-lifecycle.md) Rust enum 命名 + [spec/flows/03 §2 Resume `current_state`](../flows/03-agent-resume.md) 字段值一致, per P1-M 修复 2026-08-27)
- `assigned_to` (string?)
- `context_refs` (string[])
- `acceptance_criteria` (string[])
- `labels` (string[])
- `updated_at` (timestamp ISO 8601)

### 3.2 Worktree

- `id` (string, e.g. `"wt-STAR-1024"`)
- `path` (string, absolute filesystem path)
- `branch` (string, e.g. `"feature/STAR-1024"`)
- `head_commit` (string, SHA-1)
- `dirty` (boolean)
- `worktree_binding` (`WorktreeBinding` — per F-24 修复 2026-08-27 / 子代理 A 🟢 #24, 拆出 agent_sessions / ide_sessions 数组，**Worktree 顶层 schema 不再含 `agent_session_id` 或 `ide_session_id` 字段**，避免 agent 视角 schema 同时绑两类 session，跨层数据泄漏）
- `created_at` (timestamp)

> **F-24 硬约束**：Worktree schema 顶层**禁止**同时含 `agent_session_id` 和 `ide_session_id` —— 跨层数据泄漏（per INTERFACE-REVIEW-A 🟢 #24）。如需 session 绑定，引用 `worktree_binding` 数组，按 session 类型归类。CLI `star worktree status` 引用本 schema（per [spec/cli/01 §2.1](../cli/01-cli-spec.md)）。

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #24 (F-24) — 显式禁止 Worktree 顶层同时含 `agent_session_id` + `ide_session_id`

### 3.3 SubmitResult

- `status` (enum: `OK` | `VALIDATION_FAILED` | `POLICY_DENIED` | `CONFLICT` | `ERROR`)
- `commit_sha` (string?)
- `mr_id` (string?)
- `pipeline_run_id` (string?)
- `validation_passed` (boolean)
- `policy_checked` (boolean)

### 3.4 Issue

- `id` (string, e.g. `"STAR-1024"`)
- `title` (string)
- `status` (enum: `OPEN` | `IN_PROGRESS` | `REVIEW` | `CLOSED`)
- `priority` (enum: `P0` | `P1` | `P2` | `P3`)
- `labels` (string[])
- `assignee` (string?)
- `created_at` / `updated_at` (timestamp)

### 3.5 IssueList

- `items` (Issue[])
- `total` (integer)
- `cursor` (string?)

### 3.6 CurrentTask

> 继承 Task（§3.1）全部字段，**额外必含** claim 生命周期字段：

- `claimed_at` (timestamp)
- `claim_expires_at` (timestamp)
- `claim_renew_count` (integer, default 0)

### 3.7 MR (MergeRequest)

- `id` (string, e.g. `"MR-789"`)
- `title` (string)
- `status` (enum: `OPEN` | `MERGED` | `CLOSED`)
- `source_branch` / `target_branch` (string)
- `url` (string?)
- `linked_issue_ids` (string[])

### 3.8 Context

- `issue_id` (string)
- `related_code` (`CodeRef[]` — `{file, range, snippet}`)
- `related_docs` (`DocRef[]` — `{path, excerpt}`)
- `related_mrs` (`MRRef[]` — `{mr_id, relevance}`)

### 3.9 CodeSearchResult

- `query` (string)
- `matches` (`CodeMatch[]` — `{file, line, snippet, score}`)
- `total` (integer)

### 3.10 SymbolResult

- `name` (string)
- `kind` (enum: `function` | `struct` | `enum` | `trait` | `const` | `module`)
- `file` (string, relative path)
- `line` (integer)
- `signature` (string?, e.g. `"pub async fn login(...) -> Result<...>"`)

### 3.11 WorktreeStatus

- `worktree` (Worktree — per §3.2)
- `last_commit` (`Commit` — `{sha, author, message, files_changed}`)
- `uncommitted_files` (integer)
- `modified_files` (string[])

### 3.12 TestResult

- `passed` (integer)
- `failed` (integer)
- `skipped` (integer)
- `failed_tests` (`TestCase[]` — `{name, error, stack_trace}`)

### 3.13 PipelineRun

- `id` (string, e.g. `"pipe-123"`)
- `status` (enum: `QUEUED` | `RUNNING` | `SUCCESS` | `FAILED` | `CANCELLED`)
- `url` (string?)
- `started_at` / `finished_at` (timestamp?)

### 3.14 Error

> **唯一权威 Error schema**（per F-06 修复 2026-08-27）— CLI / MCP / REST / Universal Submit **全部**引用本 schema。**6 字段统一**（per F-06 重定义 — INTERFACE-REVIEW-A 🔴 #6）：

- `code` (string, e.g. `"WORKTREE_CONFLICT"` — 标准化 SCREAMING_SNAKE_CASE 错误码；**改名自 v0.2 的 `error` 字段**，避免与 HTTP 字段冲突)
- `message` (string, human-readable)
- `source_module` (string, e.g. `"agent-core"` | `"ide-gateway"` | `"vcs"` | `"policy"` | `"mcp"` | `"rest"` | `"cli"`)
- `source_kind` (enum: `internal` | `external` | `policy` | `validation` | `user_input` | `timeout` — 错误来源分类)
- `retriable` (boolean — 改名自 v0.2 的 `recoverable`，避免与 `recoverable=true` 语义混淆)
- `hint` (string?, 恢复提示 — 替换 v0.2 的 `suggested_actions[]`，单字符串更易消费)

> **F-06 字段集来源**：6 字段定义 = `code` / `message` / `source_module` / `source_kind` / `retriable` / `hint`，per INTERFACE-REVIEW-A 🔴 #6 + 子代理协调结果（2026-08-27）。**v0.2 的 `error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details` 6 字段已弃用**——其他 spec (cli/01 §5, mcp/01 §3, rest/01 §4, flows/05 §3) 引用本 schema 时需同步更新。

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🔴 #6 (F-06) — Error schema 6 字段重定义为 code/message/source_module/source_kind/retriable/hint

### 3.15 WorkspaceSummary

> **Agent 视角的逻辑抽象**（per F-05 修复 2026-08-27 — INTERFACE-REVIEW-A 🔴 #5）— **不含** IDE 内部状态（无 `open_files` / `diagnostics` / `ide_client` / `ide_version`）。CLI `star workspace current` 引用本 schema 而非 ide-api/v1#WorkspaceState。守 ADR-0024 "IDE Session 独立" 边界。

- `id` (string, e.g. `"ws-abc"`)
- `name` (string)
- `repository` (`RepositoryRef` — `{id, provider, url}`，**不**带 `worktree_id`)
- `worktree_id` (string?)
- `agent_session_id` (string — **仅 agent session**，不含 ide_session_id)
- `created_at` / `updated_at` (timestamp)

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🔴 #5 (F-05) — 显式声明"Agent 视角、不含 IDE 内部状态"

### 3.16 Resume

> Agent Session Resume schema（per B-19 修复 2026-08-27 — INTERFACE-REVIEW-B 🔴 B-19）。**11 字段**定义如下（与 v0.2 的 11 字段**不同**——v0.2 是 Task Resume 协议字段集，本节是 Agent Session Resume 字段集）：

- `id` (string, e.g. `"resume-uuid-..."` — Resume 操作唯一 ID)
- `agent_id` (string — 被 Resume 的 Agent ID)
- `state` (enum: `AgentTaskState` — per §3.28，PascalCase)
- `last_heartbeat_at` (timestamp — 上次心跳时间，per [spec/flows/02-agent-lease-heartbeat.md](../flows/02-agent-lease-heartbeat.md))
- `lease_expires_at` (timestamp — 租约到期时间)
- `current_state` (string, PascalCase — Agent 运行时状态别名，**冗余**于 `state`，保留以兼容 flows/03 §2 协议)
- `current_step` (string, e.g. `"validate:2-of-5"`, `"commit:1-of-1"`, `"submit:waiting-policy"`)
- `retry_count` (integer — 重试次数)
- `artifacts` (`Artifact[]` — `{path, content_type, sha256}`，产出物列表)
- `checkpoint` (`Checkpoint` — `{id, snapshot_id, taken_at, store_ref}`，checkpoint 引用)
- `recovery_hint` (string, 恢复提示 — 替换 v0.2 的 `failed_attempts` + `remaining_work`)

> **B-19 字段集来源**：11 字段定义 = `id` / `agent_id` / `state` / `last_heartbeat_at` / `lease_expires_at` / `current_state` / `current_step` / `retry_count` / `artifacts` / `checkpoint` / `recovery_hint`，per INTERFACE-REVIEW-B 🔴 B-19 (2026-08-27)。**v0.2 的 11 字段（`current_state` / `workspace` / `worktree` / `previous_plan` / `modified_files` / `open_diagnostics` / `test_results` / `failed_attempts` / `relevant_context` / `remaining_work` / `last_modified`）已弃用**——flows/03 §2 协议需同步更新到本节字段集。

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-B 🔴 B-19 — Resume schema 11 字段重定义为 agent session 视角 (id/agent_id/state/last_heartbeat_at/lease_expires_at/current_state/current_step/retry_count/artifacts/checkpoint/recovery_hint)

### 3.17 Capabilities

> **能力发现 schema**（per F-04 扩展 2026-08-27 — 从原 §3.14 移到本节）。`star agent capabilities` / `GET /api/v1/agent/capabilities` 引用本 schema。

- `schema_version` (string, e.g. `"agent-api/v1"` — 见 §1 双版本字段说明)
- `agent` (`{commands: Command[]}`)
- `ide` (`{commands: Command[]}`)
- `capabilities` (string[] — per [arch/03 §4](../../arch/03-star-ai-compat-arch.md) capability 数组)

### 3.18 Validation

> 提交前验证结果（per F-04 扩展 2026-08-27 — `agent-api/v1#Validation`，CLI `star submit` 前置验证 / REST `POST /api/v1/validate` 引用）。

- `passed` (boolean)
- `failed_tests` (string[] — 失败的测试 ID 列表)
- `policy_violations` (`PolicyViolation[]` — `{rule, severity, file, line, message}`)
- `warnings` (`Warning[]` — `{code, message, file?, line?}`)
- `ran_at` (timestamp)

### 3.19 Audit

> 审计日志条目（per F-04 扩展 2026-08-27 + [spec/flows/07-audit-model.md](../flows/07-audit-model.md)）。所有 STAR 写操作必须产生 audit entry。

- `id` (string — audit entry UUID)
- `event_type` (string, e.g. `"submit.committed"`, `"worktree.created"`, `"policy.denied"`)
- `timestamp` (timestamp ISO 8601)
- `actor` (`Actor` — `{type: "agent" | "human" | "system", id}`)
- `action` (string, e.g. `"git.commit"`, `"mr.create"`, `"policy.check"`)
- `resource` (`Resource` — `{type, id, path?}`)
- `trace_id` (string?, 关联 request lifecycle)
- `payload` (object?, action-specific payload)

### 3.20 Event

> 事件流条目（per F-04 扩展 2026-08-27 + [spec/flows/08-event-model.md](../flows/08-event-model.md)）。IDE hooks / Webhooks / MCP event subscription 引用。

- `id` (string — event UUID)
- `event_type` (string, e.g. `"task.state_changed"`, `"worktree.dirty"`, `"mr.review_requested"`)
- `source` (string, e.g. `"agent-core"`, `"ide-gateway"`, `"vcs"`)
- `payload` (object — event-specific payload)
- `emitted_at` (timestamp)
- `correlation_id` (string?, 关联因果链)

### 3.21 Decision

> 人类决策记录（per F-04 扩展 2026-08-27）。Agent 在 `state = WaitingFeedback` 时挂起，等待人类决策。

- `decision_id` (string)
- `question` (string — 决策问题)
- `options` (`Option[]` — `{id, label, description, preview?}`)
- `selected_option_id` (string?, 已选 option)
- `decided_by` (`Actor` — per §3.19)
- `decided_at` (timestamp?)
- `state` (enum: `pending` | `decided` | `timeout` | `cancelled`)

### 3.22 Permission

> 权限授予（per F-04 扩展 2026-08-27 + [spec/resources/05-agent-permission-model.md](../resources/05-agent-permission-model.md)）。

- `permission` (string, e.g. `"git.push"`, `"git.force_push"`, `"agent.fork"`, `"mcp.write"`)
- `granted` (boolean)
- `scope` (enum: `workspace` | `repository` | `worktree` | `global`)
- `scope_ref` (string?, e.g. `worktree_id`)
- `granted_by` (`Actor` — per §3.19)
- `granted_at` (timestamp)
- `expires_at` (timestamp?)

### 3.23 Identity

> Agent 身份（per F-04 扩展 2026-08-27 + [spec/resources/03-agent-identity.md](../resources/03-agent-identity.md)）。

- `agent_id` (string — Agent 唯一 ID)
- `type` (enum: `claude-code` | `codex` | `gemini-cli` | `cursor-agent` | `jetbrains-agent` | `local-llm` | `custom`)
- `vendor` (string, e.g. `"anthropic"`, `"openai"`, `"google"`, `"local"`)
- `version` (string, 客户端版本)
- `session_token` (string?, 当前 session 令牌)
- `capabilities_hash` (string — 客户端能力摘要，per §3.17)

### 3.24 Lease

> 任务租约（per F-04 扩展 2026-08-27 + [spec/flows/02-agent-lease-heartbeat.md](../flows/02-agent-lease-heartbeat.md)）。

- `lease_id` (string)
- `agent_id` (string — per §3.23)
- `worktree_id` (string? — per §3.2)
- `acquired_at` (timestamp)
- `expires_at` (timestamp)
- `renew_count` (integer, default 0)
- `last_heartbeat_at` (timestamp?)
- `state` (enum: `active` | `expired` | `released` | `revoked`)

### 3.25 Integration

> 外部集成（per F-04 扩展 2026-08-27）。MCP server / Git provider / IDE client 等外部依赖注册。

- `integration_id` (string)
- `type` (enum: `mcp-server` | `git-provider` | `ide-client` | `ci-pipeline` | `notification`)
- `vendor` (string, e.g. `"gitgit"`, `"github"`, `"gitlab"`, `"vscode"`)
- `config_ref` (string?, 配置引用 e.g. `~/.config/star/integrations/<id>.toml`)
- `status` (enum: `active` | `disabled` | `error` | `pending_auth`)
- `last_sync_at` (timestamp?)

### 3.26 Notification

> 通知（per F-04 扩展 2026-08-27）。CLI 输出 / IDE toast / Email / Slack 等通道通知统一抽象。

- `notification_id` (string)
- `channel` (enum: `cli` | `ide-toast` | `email` | `slack` | `webhook`)
- `severity` (enum: `info` | `success` | `warning` | `error`)
- `title` (string)
- `message` (string)
- `related_resource` (`Resource` — per §3.19, optional)
- `sent_at` (timestamp)
- `read_at` (timestamp?)

### 3.27 Search

> 搜索请求/结果统一 schema（per F-04 扩展 2026-08-27 — 覆盖 `code_search` / `code_symbol` / `code_references` 三种查询）。`star code search` / `star code symbol` / `star code references` 全部引用本 schema。

- `query` (string)
- `query_type` (enum: `text` | `symbol` | `references` | `regex` | `ast`)
- `scope` (`Scope` — `{repo_id, worktree_id?, path_glob?}`)
- `filters` (object?, query-specific filters)
- `results` (`SearchHit[]` — `{file, line, snippet, score, kind?}`)
- `total` (integer)
- `cursor` (string?, 分页)

### 3.28 AgentTaskState

> **Agent 运行时状态枚举**（per B-23 修复 2026-08-27 — INTERFACE-REVIEW-B 🟡 B-23）。**14 个 PascalCase 状态值**（per Rust enum 命名约定 [spec/flows/01 §1](../flows/01-agent-task-lifecycle.md)）：

- `Pending` — 任务已分配，未启动
- `Assigned` — 任务已发送给 Agent，等待启动信号
- `Running` — Agent 正在执行
- `WaitingTool` — 等待工具返回
- `ToolRunning` — 工具正在执行
- `ToolCompleted` — 工具完成
- `WaitingFeedback` — 等待人类反馈
- `FeedbackReceived` — 已收到人类反馈
- `Validating` — 正在跑验证
- `Completed` — 任务完成
- `Failed` — 任务失败
- `Aborted` — 用户中止
- `Crashed` — Agent 崩溃
- `Timeout` — 任务超时

> **B-23 vs flows/01 §1 区别**：
> - flows/01 §1 = **Task.status 状态机**（issue 生命周期，14 值：Created / Claimed / ContextLoading / Planning / Implementing / Validating / ReviewReady / Submitted / Completed / Blocked / Conflict / Failed / Cancelled / HumanRequired）
> - **AgentTaskState（本节）= Agent 运行时状态**（agent runtime lifecycle，14 值：上面 14 个）
> - 两者**完全不同**：Task.status 描述 issue 走到哪一步；AgentTaskState 描述当前正在执行任务的 agent 自身处于什么运行时阶段
> - Resume schema §3.16 `state` 字段引用本 enum
> - 关联使用场景：审计日志（§3.19 event_type）、决策请求（§3.21 state=pending）、通知（§3.26 severity）

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-B 🟡 B-23 — 新增 AgentTaskState enum (14 PascalCase 值)

## 4. 全部 schema 落盘位置

> **落盘路径迁移**（per F-29 修复 2026-08-27 — INTERFACE-REVIEW-A 🟢 #29）：从 `crates/star-cli/src/schemas/agent-api-v1/`（v0.2）迁移到 **`crates/star-ide-gateway/src/schemas/agent-api-v1/`**（v0.2 fix），per [arch/04 STAR IDE Gateway](../../arch/04-star-ide-gateway-arch.md) 的 crate 边界（agent-api / ide-api 共享 star-ide-gateway crate，与 star-cli 解耦）。

`crates/star-ide-gateway/src/schemas/agent-api-v1/`：
- `Task.json` (§3.1)
- `Worktree.json` (§3.2 — 含 `worktree_binding: WorktreeBinding`)
- `SubmitResult.json` (§3.3)
- `Issue.json` (§3.4)
- `IssueList.json` (§3.5)
- `CurrentTask.json` (§3.6)
- `MR.json` (§3.7)
- `Context.json` (§3.8)
- `CodeSearchResult.json` (§3.9)
- `SymbolResult.json` (§3.10)
- `WorktreeStatus.json` (§3.11)
- `TestResult.json` (§3.12)
- `PipelineRun.json` (§3.13)
- **`Error.json` (§3.14 — 唯一权威，6 字段：code/message/source_module/source_kind/retriable/hint)**
- **`WorkspaceSummary.json` (§3.15 — agent 视角，不含 IDE 内部状态)**
- **`Resume.json` (§3.16 — 11 字段：id/agent_id/state/last_heartbeat_at/lease_expires_at/current_state/current_step/retry_count/artifacts/checkpoint/recovery_hint)**
- `Capabilities.json` (§3.17 — 含 `schema_version` 字段，per §1 双版本字段)
- `Validation.json` (§3.18)
- `Audit.json` (§3.19)
- `Event.json` (§3.20)
- `Decision.json` (§3.21)
- `Permission.json` (§3.22)
- `Identity.json` (§3.23)
- `Lease.json` (§3.24)
- `Integration.json` (§3.25)
- `Notification.json` (§3.26)
- `Search.json` (§3.27)
- `AgentTaskState.json` (§3.28 — 14 PascalCase enum)
- `WorktreeBinding.json`（§3.2 子结构）
- `RepositoryRef.json`（§3.15 子结构）
- `ProjectList.json` / `ClaimResult.json` / `PipelineStatus.json` / `WorkspaceList.json` / `Permissions.json` / `DiffResult.json` / `PolicyCheckResult.json` / `CommitResult.json` / `PushResult.json` / `MRLinkResult.json` / `ReferencesResult.json` / `ReviewResult.json`（CLI 扩展命令的 output schema，per [spec/cli/01 §2.2](../cli/01-cli-spec.md)）

> §3 与 §4 交叉引用（per F-21 修复 2026-08-27 — INTERFACE-REVIEW-A 🟢 #21）：§3.X 是已定义 schema 的字段说明；§4 是落盘文件清单 + 落盘路径。两者一一对应，**新增 schema 必须同时在 §3 和 §4 出现**。

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #21/#29 (F-21/F-29) — 落盘路径 star-cli → star-ide-gateway；§3-§4 交叉引用强化

## 5. `--schema-version` 默认值

> **新增节**（per F-27 修复 2026-08-27 — INTERFACE-REVIEW-A 🟢 #27）：

- `--schema-version <v>` flag 默认值 = **`agent-api/v1`**（当前实现 schema version）
- 默认值必须与 `star agent capabilities` 输出的 `schema_version` 字段一致（per §3.17）
- 用户可显式指定 `--schema-version agent-api/v1.1` 强制 additive 兼容性
- 用户不可降级到 `agent-api/v0.x`（v0 弃用）— 显式指定 v0.x 触发 `Error.code = SCHEMA_VERSION_UNSUPPORTED`（per §3.14）
- 实现层：`crates/star-cli/src/commands/mod.rs::default_schema_version() -> "agent-api/v1"`

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #27 (F-27) — `--schema-version` 默认值 = `agent-api/v1`

## 6. 验证

```bash
# schema 必须合法
npx ajv validate -s crates/star-ide-gateway/src/schemas/agent-api-v1/SubmitResult.json \
                   -d test-data/submit-result.json

# CLI 输出必须符合 schema
star submit --json | python3 -c "import json,sys; print(json.dumps(json.load(sys.stdin), indent=2))"  # 不抛错

# Error schema 统一性校验（per F-06 6 字段）
for cmd in cli mcp rest submit; do
  npx ajv validate -s crates/star-ide-gateway/src/schemas/agent-api-v1/Error.json -d test-data/$cmd-error.json
done

# Resume schema 校验（per B-19 11 字段）
npx ajv validate -s crates/star-ide-gateway/src/schemas/agent-api-v1/Resume.json \
                   -d test-data/resume-agent-session.json
```

> 验证命令中 `crates/star-cli/...` → `crates/star-ide-gateway/...`（per F-29 路径迁移）

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：§3 展开 3 个 schema（Task / Worktree / SubmitResult） | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-C：§3.16 WorkspaceSummary（agent 视角） · P1-D：§3 扩展为 3.1-3.15（15 核心 schema 各 3-5 字段定义）+ §1 加 OpenAPI info.version 同步演化规则 · P1-G：§3.15 Error 6 字段统一权威 schema · P1-O：§3.17 Resume 11 字段 + flows/03 §2 引用 | 8 子代理 INTERFACE-REVIEW-A 🔴 #4/#5/#6 + INTERFACE-REVIEW-B 🔴 B-19 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.2 fix | 2026-08-27 | Mavis（接手 agent per DEC-008 — 子代理 fix-api-spec-blockers）| F-04: §3 扩到 28 schema（新增 §3.18 Validation / §3.19 Audit / §3.20 Event / §3.21 Decision / §3.22 Permission / §3.23 Identity / §3.24 Lease / §3.25 Integration / §3.26 Notification / §3.27 Search） · F-05: §3.15 WorkspaceSummary 显式"agent 视角、不含 IDE 内部状态" · F-06: §3.14 Error 6 字段重定义（code/message/source_module/source_kind/retriable/hint，弃用 v0.2 的 error/recoverable/suggested_actions/message/trace_id/details） · F-12: §1 加 info.version (OpenAPI SemVer) vs schema_version (agent-api/vN) 双版本字段关系 · F-21: §3 / §4 交叉引用强化 · F-24: §3.2 Worktree 顶层显式禁同时含 agent_session_id + ide_session_id（拆 worktree_binding 数组） · F-27: §5 新增 `--schema-version` 默认值 = `agent-api/v1` · F-29: §4 落盘路径 `crates/star-cli/` → `crates/star-ide-gateway/src/schemas/agent-api-v1/` · B-19: §3.16 Resume schema 11 字段重定义为 agent session 视角（id/agent_id/state/last_heartbeat_at/lease_expires_at/current_state/current_step/retry_count/artifacts/checkpoint/recovery_hint，弃用 v0.2 的 11 字段） · B-23: §3.28 新增 AgentTaskState enum（14 PascalCase 值）| INTERFACE-REVIEW-A 🔴 #4/#5/#6 + 🟡 #11 + 🟢 #21/#24/#27/#29 + INTERFACE-REVIEW-B 🔴 B-19 + 🟡 B-23 |
