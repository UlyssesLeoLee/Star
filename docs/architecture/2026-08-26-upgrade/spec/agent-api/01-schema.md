# 12. Agent CLI JSON Schema

> **状态**：🟡 草案 v0.2
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md) · [spec/cli/01-cli-spec.md](../cli/01-cli-spec.md)

## 1. Versioning

- Schema version: `agent-api/v1`
- Breaking change 必须升 v2
- 任何 field 重命名 / 移除 / 类型变更都算 breaking
- New field 是 additive（minor）
- **OpenAPI `info.version` 演化规则**（per P1-D 硬约束 9 + 子代理 A 🟡 #11，2026-08-27）：
  - `info.version` 与 schema major.minor **同步演化**（schema `v1.x` → OpenAPI `1.x.0`）
  - Patch（bug fix 字段语义）走 `metadata` 字段标注，不动 `info.version`
  - Example：`agent-api/v1.0` → `info.version: "1.0.0"`；`agent-api/v1.1`（additive）→ `info.version: "1.1.0"`；`agent-api/v2`（breaking）→ `info.version: "2.0.0"`
- 完整 OpenAPI 3.1 规范：webhooks 字段允许 / `nullable: true` 替换为 `type: [string, "null"]` / `info.summary` 允许（per 子代理 A 🟡 #10）

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

> §3.1-3.15 = 15 个核心 schema（per P1-D 修复 2026-08-27，每 schema 3-5 字段定义）；§3.16-3.17 = WorkspaceSummary（per P1-C）+ Resume（per P1-O）。

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
- `worktree_binding` (`WorktreeBinding` — per 子代理 A 🟢 #24, 拆出 agent_sessions / ide_sessions 数组)
- `created_at` (timestamp)

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

### 3.14 Capabilities

- `schema_version` (string, e.g. `"agent-api/v1"`)
- `agent` (`{commands: Command[]}`)
- `ide` (`{commands: Command[]}`)
- `capabilities` (string[] — per [arch/03 §4](../../arch/03-star-ai-compat-arch.md) capability 数组)

### 3.15 Error

> **唯一权威 Error schema**（per P1-G 修复 2026-08-27）— CLI / MCP / REST / Universal Submit **全部**引用本 schema，5 字段原版 + `details` = **6 字段**。

- `error` (string, e.g. `"WORKTREE_CONFLICT"`)
- `recoverable` (boolean)
- `suggested_actions` (string[])
- `message` (string, human-readable)
- `trace_id` (string, 关联 audit log)
- `details` (object?, 领域特定 payload — e.g. `{"failed_tests": [...]}`, `{"worktree_id": "..."}`)

### 3.16 WorkspaceSummary（per P1-C 修复 2026-08-27）

> **Agent 视角的逻辑抽象**，不含 IDE 内部状态。CLI `star workspace current` 引用本 schema 而非 §3.x 或 ide-api/v1#WorkspaceState。

- `id` (string, e.g. `"ws-abc"`)
- `name` (string)
- `repository` (`RepositoryRef` — `{id, provider, url}`，**不**带 `worktree_id`)
- `worktree_id` (string?)
- `agent_session_id` (string)
- `created_at` / `updated_at` (timestamp)

### 3.17 Resume（per P1-O 修复 2026-08-27）

> Resume JSON 字段定义（per [spec/flows/03-agent-resume.md §2](../flows/03-agent-resume.md) 协议）。11 字段：

- `current_state` (string, PascalCase — per P1-M 修复，例如 `"Implementing"`)
- `workspace` (WorkspaceSummary)
- `worktree` (Worktree + `modified_files: string[]`)
- `previous_plan` (string[] — Agent A 的 TODO 列表)
- `modified_files` (string[])
- `open_diagnostics` (Diagnostic[] — per [spec/ide-api/01-schema.md](../ide-api/01-schema.md))
- `test_results` (TestResult — per §3.12)
- `failed_attempts` (FailedAttempt[] — `{step, error}`)
- `relevant_context` (Context — per §3.8)
- `remaining_work` (string[])
- `last_modified` (timestamp, 何时 snapshot)

## 4. 全部 schema 落盘位置

`crates/star-cli/src/schemas/agent-api-v1/`：
- `Task.json` (§3.1)
- `Worktree.json` (§3.2)
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
- `Capabilities.json` (§3.14)
- `Error.json` (§3.15 — 唯一权威)
- `WorkspaceSummary.json` (§3.16)
- `Resume.json` (§3.17)
- `ProjectList.json` / `ClaimResult.json` / `PipelineStatus.json` / `WorkspaceList.json` / `Permissions.json` / `DiffResult.json` / `PolicyCheckResult.json` / `CommitResult.json` / `PushResult.json` / `MRLinkResult.json` / `ReferencesResult.json` / `ReviewResult.json` / `WorktreeBinding.json`（扩展命令 / 子结构）

> §3 与 §4 交叉引用：§3 列出已定义 schema，§4 列出待落盘 schema 全部文件名（per 子代理 A 🟢 #21）。

## 5. 验证

```bash
# schema 必须合法
npx ajv validate -s crates/star-cli/src/schemas/agent-api-v1/SubmitResult.json \
                   -d test-data/submit-result.json

# CLI 输出必须符合 schema
star submit --json | python3 -c "import json,sys; print(json.dumps(json.load(sys.stdin), indent=2))"  # 不抛错

# Error schema 统一性校验（per P1-G）
for cmd in cli mcp rest submit; do
  npx ajv validate -s crates/star-cli/src/schemas/agent-api-v1/Error.json -d test-data/$cmd-error.json
done
```

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：§3 展开 3 个 schema（Task / Worktree / SubmitResult） | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-C：§3.16 WorkspaceSummary（agent 视角） · P1-D：§3 扩展为 3.1-3.15（15 核心 schema 各 3-5 字段定义）+ §1 加 OpenAPI info.version 同步演化规则 · P1-G：§3.15 Error 6 字段统一权威 schema · P1-O：§3.17 Resume 11 字段 + flows/03 §2 引用 | 8 子代理 INTERFACE-REVIEW-A 🔴 #4/#5/#6 + INTERFACE-REVIEW-B 🔴 B-19 + P1-BLOCKERS-SUMMARY v0.2 |
