# BD-STAR-CLI-004

> **STAR CLI 基本設計書 v1.0 (CL-4+CL-5) — recreate, per ULYS-124 v1.0 缺口补齐**
>
> - 状态: Basic Design Baseline (v1.0 — recreate after 9/21 + 9/23 reviewer 打回, per 用户 9/25 01:09 JST "确保缺口已补")
> - 目标阶段: 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0
> - 上游 BD 总冊: [`docs/design/BD-STAR-CLI-001.md`](./BD-STAR-CLI-001.md) v1.0 §3.2.3 (FR-29~FR-33) + §3.2.5 (FR-35) + §4.1 (commands/mr.rs / test.rs / pipeline.rs / skill_registry.rs / error.rs / commands/agent.rs)
> - 实装参考: `crates/star-cli/src/error.rs` + `crates/star-cli/src/skill_registry.rs` + `crates/star-cli/src/commands/{mr,test,pipeline,agent}.rs`
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-25 JST

> **回溯记录（per ULYS-124 reviewer 9/21 + 9/23 打回与 9/25 01:09 用户缺口补齐指令）**：v0.1 BD-STAR-CLI-004 (`3b791824`) 在 `git ls-remote origin` 不可达。本 v1.0 按 (a) cli-spec v0.2 canonical (b) 当前 main `b0dddf7e` 实装 (c) SRS-STAR-CLI-001 v1.0 + BD-STAR-CLI-001 v1.0 总冊 (同期落档) 创作。

---

## §0 目的

本文档基于 [`SRS-STAR-CLI-001` §3.1.3](../requirements/SRS-STAR-CLI-001.md) Skill Registry 4 子命令 + §5.3 Capability Discovery + §8 错误模型, 定义 **CL-4 (Error + Capability Discovery + Skill Registry + agent meta)** + **CL-5 (MR + Test + Pipeline)** 子能力的 STAR CLI 基本設計.

**职责定位 (per 总冊 BD-001 §1.1)**: 本 BD 覆盖 4+7 = 11 命令 + 1 错误 schema + 1 capability array + 4 skill 子命令 + 8 agent/ide meta 子命令 (`commands/agent.rs` 待 [M] 子项).

**MVP 范围** (per SRS §3): FR-14/15/16/20/21/22/23 (CL-5 7 命令) + FR-29/30/31/32 (Skill 4 子命令) + FR-33 (Capability Discovery) + FR-34 (Error 6 字段) + 8 agent/ide meta 子命令 (待 [M]).

---

## §1 適用範囲

### 1.1 In-Scope: CL-4 + CL-5 (4 + 7 = 11 命令 + 1 schema + 1 array + 8 agent/ide meta)

| 子能力 | 命令 | 行数近似 | 状态 | 详细 BD § |
|---|---|---|---|---|
| **CL-4.1** Skill Registry | `star skill add/list/show/remove` (FR-29/30/31/32) | ~526 行 (skill_registry.rs 已就绪) | ✅ (per ULYS-196 PR #81) | §4 (本 BD) |
| **CL-4.2** Error 模型 | 全部命令 stderr 输出 (FR-34) | 34 行 (error.rs) | ✅ | §2 (本 BD) |
| **CL-4.3** Capability Discovery | `star agent capabilities` (FR-33) | 待补 [M] | ⏳ [M] | §5 (本 BD) |
| **CL-4.4** agent/ide meta (8 子命令) | `star agent capabilities/describe/instructions/permissions` + `star ide capabilities/describe/instructions/permissions` | 326 行 agent.rs enum 已就绪, 4 子命令内 impl 待 [M] | ⏳ [M] | §5 (本 BD) |
| **CL-5.1** MR | `star mr create/show/review/link` (FR-14/15/20/28) | 124 行 (mr.rs) | ✅ | §3.1 (本 BD) |
| **CL-5.2** Test | `star test affected/run` (FR-16/21) | 待补 (test.rs) | ✅ | §3.2 (本 BD) |
| **CL-5.3** Pipeline | `star pipeline run/status` (FR-22/23) | 67 行 (pipeline.rs) | ✅ | §3.3 (本 BD) |

### 1.2 Out-of-Scope

| 不做 | 原因 | 替代 |
|---|---|---|
| Universal Submit 12 步内部 | CL-6 in BD-005 §2 | `star submit` 全 12 步 |
| MVP 17 核心命令 | CL-1+CL-3 in BD-002 | BD-002 §3 |
| 11 扩展命令 (非 Skill) | CL-2 in BD-003 | BD-003 §3 |
| 5 跨层缺口 (CLI↔MCP 命名) | per cli-spec §2.5 F-08 | Phase 2 MCP 补齐 |
| 真实 Star API client | Phase E | Phase D.2 mock + Phase E real |
| `--schema-version v2` | F-27 修复, 当前仅 `v1` | Phase 2+ |

---

## §2 CL-4.2 Error 模型 (per cli-spec §5 + per SRS-001 §8)

### 2.1 6 字段 Error schema 引用 (per cli-spec §5 F-06)

```json
{
  "error": "WORKTREE_CONFLICT",
  "recoverable": true,
  "suggested_actions": ["inspect_conflict", "request_rebase"],
  "message": "Worktree STAR-1024 has uncommitted changes conflicting with main",
  "trace_id": "...",
  "details": {"worktree_id": "wt-STAR-1024", "conflicting_files": ["src/auth.rs"]}
}
```

**字段来源**: `agent-api/v1#Error` §3.15 (per cli-spec §5 F-06 修复, W4 子代理初稿编号 §3.14 已废止).

**CLI 端引用, 不重定义** (per cli-spec §5 F-06):
- CLI 端 `error.rs` 仅实现 `StarError` enum → 序列化 `agent-api/v1#Error` 6 字段
- CLI 端不直接定义 6 字段 (避免双口径)
- `output.rs` 顶层始终含 `schema_version: "agent-api/v1"`

### 2.2 当前 `error.rs` 9 类 ToError stub (per `crates/star-cli/src/error.rs:34`)

```rust
#[derive(Debug, Error)]
pub(crate) enum StarError {
    #[error("json serialization failed: {0}")]
    Json(#[from] serde_json::Error),

    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),

    // Phase D.1 增量补齐 7 类 (per SRS §8 7 项缺口):
    // - GitError  (Command::new("git").status() != 0)
    // - WorkspaceError  (.star/ 文件读失败)
    // - NetworkError  (HTTP 失败)
    // - InvalidArgumentError  (clap derive 校验)
    // - SkillRegistryError  (skill.rs 解析失败)
    // - SubmitStepError  (12 步中某步失败)
    // - SchemaVersionError  (--schema-version v2 不支持)
}
```

> 当前 2 类 stub (Json/Io), 完整 9 类待 Phase D.1 增量补齐 (per `error.rs` line 1-5 注释).

### 2.3 9 类 ToError trait 完整设计 (per Phase D.1 详细 BD 待开)

| # | 类别 | 触发 | 序列化到 `agent-api/v1#Error` 6 字段 |
|---|---|---|---|
| 1 | Json | `serde_json::Error` | `error="JSON_SERIALIZE_FAILED"`, `message=...`, `details={...}` |
| 2 | Io | `std::io::Error` | `error="IO_FAILED"`, `message=...`, `details={}` |
| 3 | Git | `Command::new("git").status() != 0` | `error="GIT_FAILED"`, `suggested_actions=["retry","check_workdir"]`, `details={stderr}` |
| 4 | Workspace | `.star/workspace.json` 解析失败 | `error="WORKSPACE_NOT_FOUND"`, `recoverable=true`, `details={path}` |
| 5 | Network | HTTP client error | `error="NETWORK_FAILED"`, `recoverable=true`, `details={url,status}` |
| 6 | InvalidArgument | clap derive 校验失败 | `error="INVALID_ARGUMENT"`, `suggested_actions=["check_help"]`, `details={arg,value}` |
| 7 | SkillRegistry | `skill_registry.rs` 解析失败 | `error="SKILL_REGISTRY_FAILED"`, `details={name}` |
| 8 | SubmitStep | 12 步中某步失败 | `error="SUBMIT_STEP_FAILED"`, `details={step,reason}` |
| 9 | SchemaVersion | `--schema-version v2` 暂未支持 | `error="SCHEMA_VERSION_UNSUPPORTED"`, `details={requested_version}` |

### 2.4 Error 守门 (per 守门 #13)

- 全部错误必须含 `trace_id` (UUID v4)
- 全部错误必须含 `recoverable` (bool)
- 全部错误 `suggested_actions` 必须为非空数组 (per cli-spec §5 第 1 条 comment)
- 全部错误必须可被 `multica issue comment` 写入 (per [M] 子项待 Phase D.1)

---

## §3 CL-5 命令数据视图

### 3.1 MR (3 命令 — `commands/mr.rs:124`)

#### 3.1.1 `star mr create` (FR-14) → `agent-api/v1#MR` §3.7

```bash
star mr create --title "fix: auth" --description "..." --json
```

**输出 schema** (`agent-api/v1#MR` §3.7):

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `id` | int | ✅ | MR id (per `arch/03 §3.7 MR.id`) |
| `title` | string | ✅ | MR title |
| `description` | string | ⏳ | MR description (空可不写) |
| `source_branch` | string | ✅ | source branch (per 当前 worktree) |
| `target_branch` | string | ✅ | target branch (default `main`) |
| `state` | enum (`opened`/`closed`/`merged`) | ✅ | MR 状态 |
| `author_id` | string | ✅ | MR 作者 |
| `created_at` | ISO 8601 | ✅ | 创建时间 |
| `linked_issues` | array of string | ⏳ | 关联 issue id (per FR-28 `star mr link`) |
| `web_url` | URL | ✅ | MR web URL |

#### 3.1.2 `star mr show <id>` (FR-15) → 同 MR schema

**差异**: 入参 `<id>` (int), 输出 schema 同 §3.1.1 + `additions`/`deletions`/`commits_count` 字段 (per cli-spec §3.7).

#### 3.1.3 `star mr review <id>` (FR-20) → `agent-api/v1#ReviewResult`

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `id` | string | ✅ | review id |
| `mr_id` | int | ✅ | MR id |
| `reviewer_id` | string | ✅ | reviewer id |
| `action` | enum (`approve`/`request_changes`/`comment`) | ✅ | review action |
| `comment` | string | ⏳ | review comment |
| `submitted_at` | ISO 8601 | ✅ | review 时间 |

#### 3.1.4 `star mr link <id>` (FR-28) → `agent-api/v1#MRLinkResult`

**特殊**: 此命令是 Universal Submit 步骤 10 暴露 (per cli-spec §2.2 P1-H 新增).

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `mr_id` | int | ✅ | MR id |
| `issue_id` | string | ✅ | issue id (e.g. `STAR-1024`) |
| `linked_at` | ISO 8601 | ✅ | link 时间 |
| `status` | enum (`linked`/`failed`) | ✅ | link 状态 |

### 3.2 Test (2 命令 — `commands/test.rs` 待补)

#### 3.2.1 `star test affected` (FR-16, MVP) → `agent-api/v1#TestResult` §3.12

```bash
star test affected --json
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `scope` | enum (`affected`/`all`) | ✅ | test scope |
| `total` | int | ✅ | total tests |
| `passed` | int | ✅ | passed tests |
| `failed` | int | ✅ | failed tests |
| `skipped` | int | ⏳ | skipped tests |
| `failed_tests` | array of `FailedTest` | ⏳ | failed test details |
| `duration_ms` | int | ✅ | total duration |
| `trace_id` | string | ✅ | trace id (per 守门 #13) |

#### 3.2.2 `star test run` (FR-21, 扩展) → 同 `TestResult` schema

**差异**: `scope = "all"` (per cli-spec §2.2 P1-H 补全 F-20 修复).

### 3.3 Pipeline (2 命令 — `commands/pipeline.rs:67`)

#### 3.3.1 `star pipeline run` (FR-22) → `agent-api/v1#PipelineRun`

```bash
star pipeline run --json
```

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `pipeline_id` | string | ✅ | pipeline id |
| `status` | enum (`running`/`succeeded`/`failed`/`cancelled`) | ✅ | status |
| `stages` | array of `PipelineStage` | ✅ | pipeline stages |
| `created_at` | ISO 8601 | ✅ | 创建时间 |
| `trace_id` | string | ✅ | trace id |

#### 3.3.2 `star pipeline status` (FR-23) → `agent-api/v1#PipelineStatus`

**差异**: 单 field 查询 (`--pipeline-id <id>` 入参), 输出同 `PipelineRun` + `updated_at` 字段.

---

## §4 CL-4.1 Skill Registry (per ULYS-196 / FR-ORCA-034 §10.2)

### 4.1 4 子命令 (per `crates/star-cli/src/skill_registry.rs:526`)

#### 4.1.1 `star skill add` (FR-29)

```bash
star skill add <url> --skill <name>
```

**入参 (per `skill_registry.rs:248-274`)**:

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `url` | string (URL) | ✅ | 来源 URL (per `npx skills add <url>`) |
| `--skill <name>` | string | ✅ | skill 名称 |

**输出**: `agent-api/v1#Skill`:

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `name` | string | ✅ | skill 名称 |
| `source_url` | string (URL) | ✅ | 安装源 |
| `version` | string | ✅ | 版本 |
| `installed_at` | ISO 8601 | ✅ | 安装时间 |
| `description` | string | ⏳ | skill 描述 |

#### 4.1.2 `star skill list` (FR-30)

**输出**: `agent-api/v1#SkillList` = array of `Skill` + `total`.

#### 4.1.3 `star skill show <name>` (FR-31)

**输出**: `agent-api/v1#SkillDetail`:

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| (Skill fields 全部) | — | ✅ | 同上 |
| `instructions` | string | ✅ | skill usage instructions |
| `metadata` | object | ⏳ | skill metadata |

#### 4.1.4 `star skill remove <name>` (FR-32)

**输出**: `agent-api/v1#SkillRemoveResult`:

| 字段 | 类型 | 必填 | 说明 |
|---|---|---|---|
| `name` | string | ✅ | 删除的 skill 名称 |
| `removed_at` | ISO 8601 | ✅ | 删除时间 |
| `status` | enum (`removed`/`not_found`) | ✅ | 删除结果 |

### 4.2 Skill Registry 内部错误 (per `skill_registry.rs:94-110`)

```rust
#[derive(Debug, Error)]
pub(crate) enum SkillRegistryError {
    #[error("skill not found: {0}")]
    NotFound(String),

    #[error("skill already exists: {0}")]
    AlreadyExists(String),

    #[error("invalid skill source URL: {0}")]
    InvalidUrl(String),
}
```

> Phase D MVP 3 类 stub, Phase D+ 增量补齐.

---

## §5 CL-4.3 Capability Discovery + agent/ide meta (待 [M] 子项)

### 5.1 `star agent capabilities` (FR-33) — 15 capability 数组

**输入**: 无入参
**输出**:

```json
{
  "schema_version": "agent-api/v1",
  "capabilities": [
    "projects", "issues", "tasks", "workspaces", "worktrees",
    "repositories", "code_search", "code_navigation", "code_context",
    "merge_requests", "context", "tests", "pipelines", "reviews", "deployments"
  ]
}
```

| capability | CLI 命令 | 覆盖状态 (per cli-spec §2.3 F-15) |
|---|---|---|
| `projects` | `star project list` | ✅ MVP |
| `issues` | `star issue list/show/claim` | ✅ MVP |
| `tasks` | `star task current` | ✅ MVP |
| `workspaces` | `star workspace list/current` | ✅ MVP |
| `worktrees` | `star worktree create/enter/status` | ✅ MVP |
| `repositories` | （无独立命令） | ⚠️ CLI 间接覆盖 (隐式通过 `workspace.repository`) |
| `code_search` | `star code search` | ✅ MVP |
| `code_navigation` | `star code symbol/references` | ✅ MVP + 扩展 |
| `code_context` | `star context get/current` | ✅ MVP + 扩展 |
| `merge_requests` | `star mr create/show/review` | ✅ MVP + 扩展 |
| `context` | `star context get/current` | ✅ MVP + 扩展 (与 `code_context` 重叠) |
| `tests` | `star test affected/run` | ✅ MVP + 扩展 |
| `pipelines` | `star pipeline run/status` | ✅ 扩展 |
| `reviews` | `star mr review` | ✅ 扩展 |
| `deployments` | （无独立命令） | ⚠️ CLI 间接覆盖 (用 `star pipeline run` 派生) |
| `skills` (per ULYS-196) | `star skill add/list/show/remove` | ✅ Skill Registry |

> 数组**不删除** `repositories` / `deployments` (向后兼容 arch/03 §4), 仅在 CLI 端标注覆盖关系.

### 5.2 8 个 agent/ide meta 子命令 (待 [M] 子项)

| 命令 | 用途 | 输出 | 状态 |
|---|---|---|---|
| `star agent capabilities` | 15 capability 数组 (per §5.1) | `Capabilities` §3.17 | ⏳ [M] |
| `star agent describe <cmd>` | 单命令详细 schema | `CommandDescribe` §3.x | ⏳ [M] |
| `star agent instructions` | 当前环境 AI 操作说明 | `AgentInstructions` §3.x | ⏳ [M] |
| `star agent permissions` | 权限查询 | `Permissions` §3.x | ⏳ [M] |
| `star ide capabilities` | IDE 端 15 capability (per cli-spec §4) | `Capabilities` §3.17 | ⏳ [M] |
| `star ide describe <cmd>` | IDE 命令描述 | `CommandDescribe` §3.x | ⏳ [M] |
| `star ide instructions` | IDE 操作说明 | `IdeInstructions` §3.x | ⏳ [M] |
| `star ide permissions` | IDE 权限查询 | `Permissions` §3.x | ⏳ [M] |

**实装现状**: `crates/star-cli/src/commands/agent.rs:326` 含 `AgentCommand` enum + 4 子命令 enum 已定义, impl 待 [M].

> 待 5 域 Lead 真人到位后追溯签字 (per 守门 #3 + #14), 当前临时代签 Mavis 接手.

---

## §6 已知缺口 (10 项 per 守门 #11)

| # | 缺口 | 状态 |
|---|---|---|
| G-4.1 | `commands/agent.rs` 4 子命令 impl 待 [M] (FR-33 + 8 meta 子命令) | ⏳ [M] |
| G-4.2 | `commands/ide.rs` 不存在 (CLI 不重复 IDE 协议, per AGENTS.md §5) | 🟡 已知边界 |
| G-4.3 | 9 类 ToError trait 仅 2 stub (Json/Io), 完整 7 类待 Phase D.1 | ⏳ Phase D.1 |
| G-4.4 | Skill Registry schema (`Skill`/`SkillList`/`SkillDetail`/`SkillRemoveResult`) agent-api/v1 canonical 化 (per [S] 子项) | ⏳ [S] |
| G-4.5 | `repositories` + `deployments` 2 capability CLI 间接覆盖声明, 无独立命令 (per cli-spec §2.3 F-15) | 🟡 已知边界 |
| G-4.6 | `PipelineRun` / `PipelineStatus` schema agent-api/v1 canonical 化 (per [S] 子项) | ⏳ [S] |
| G-4.7 | `ReviewResult` / `MRLinkResult` schema agent-api/v1 canonical 化 (per [S] 子项) | ⏳ [S] |
| G-4.8 | MR/Test/Pipeline 真实 backend 接入待 [M] 子项 (Phase D.2 mock → Phase E real) | ⏳ [M] |
| G-4.9 | Skill Registry `remove` 命令 dry-run 模式未暴露 (per [S] 子项) | ⏳ [S] |
| G-4.10 | Error 6 字段 `trace_id` 与 `multica issue comment` 写入集成待 Phase D.1 | ⏳ Phase D.1 |

---

## §7 テスト (per FR-14/15/16/20/21/22/23/29/30/31/32/33/34)

| 测试 ID | 命令 | 测试内容 |
|---|---|---|
| TC-4.1 | `star mr create` | 接受 `--title` + `--description` + `--target-branch` 入参, 输出 MR schema |
| TC-4.2 | `star mr show <id>` | MR 含 `additions`/`deletions`/`commits_count` |
| TC-4.3 | `star mr review <id>` | `--approve` / `--request-changes` / `--comment` 3 mode |
| TC-4.4 | `star mr link <id> --issue STAR-NNN` | 输出 `MRLinkResult`, `status=linked` |
| TC-4.5 | `star test affected` | scope=affected 正确过滤, 输出 TestResult |
| TC-4.6 | `star test run` | scope=all, 输出 TestResult |
| TC-4.7 | `star pipeline run` | 启动 pipeline, 异步输出 PipelineRun.status |
| TC-4.8 | `star pipeline status --pipeline-id <id>` | 输出当前 PipelineStatus |
| TC-4.9 | `star skill add <url> --skill <name>` | 安装并输出 Skill schema |
| TC-4.10 | `star skill list` | 输出 SkillList |
| TC-4.11 | `star skill show <name>` | 输出 SkillDetail 含 instructions |
| TC-4.12 | `star skill remove <name>` | 输出 SkillRemoveResult |
| TC-4.13 | 任何命令触发错误 | Error 6 字段全字段输出 |
| TC-4.14 | `star agent capabilities` (待 [M]) | 输出 15 capability 数组 |

---

## §8 インタフェース設計

### 8.1 内部 trait: `ToAgentApiError` (CLI 端 StarError → agent-api/v1#Error 序列化)

```rust
pub(crate) trait ToAgentApiError {
    fn to_agent_api_error(&self) -> agent_api::v1::Error;
}

impl ToAgentApiError for StarError { /* 6 字段填充 */ }
```

> 9 类全部 impl ToAgentApiError trait, per cli-spec §5 F-06.

### 8.2 外部接口契约 (CLI 是 consumer, 不替代)

| 接口 | CLI 端职责 | 边界 |
|---|---|---|
| agent-api/v1 (HTTP) | CLI 是 consumer, 调 star-api-rest 端点 | per AGENTS.md §5 |
| Git 协议 | CLI 是 wrapper, 不实现 git 内部 | per cli-spec §1 第 5 条 |
| Skill 来源 URL | CLI 解析 URL 后 in-memory 存储 | per skill_registry.rs Phase D MVP |

---

## §9 関連ドキュメント (12 项)

| # | 文档 | 关系 |
|---|---|---|
| 1 | [`SRS-STAR-CLI-001.md`](../requirements/SRS-STAR-CLI-001.md) v1.0 | 上游 SRS |
| 2 | [`BD-STAR-CLI-001.md`](./BD-STAR-CLI-001.md) v1.0 | 上游总冊 |
| 3 | [`BD-STAR-CLI-002.md`](./BD-STAR-CLI-002.md) v1.0 | 平行专题 BD (CL-1+CL-3) |
| 4 | [`BD-STAR-CLI-003.md`](./BD-STAR-CLI-003.md) v1.0 | 平行专题 BD (CL-2) |
| 5 | [`BD-STAR-CLI-005.md`](./BD-STAR-CLI-005.md) v1.0 | 平行专题 BD (CL-6) |
| 6 | [`spec/cli/01-cli-spec.md`](../architecture/2026-08-26-upgrade/spec/cli/01-cli-spec.md) v0.2 | upstream canonical |
| 7 | [`spec/agent-api/01-schema.md`](../architecture/2026-08-26-upgrade/spec/agent-api/01-schema.md) | 15 + 1 schema |
| 8 | [`acceptance/04-mvp.md`](../architecture/2026-08-26-upgrade/spec/acceptance/04-mvp.md) | MVP 17 退出条件 |
| 9 | [`crates/star-cli/src/error.rs`](../crates/star-cli/src/error.rs) | current 9 类 stub |
| 10 | [`crates/star-cli/src/skill_registry.rs`](../crates/star-cli/src/skill_registry.rs) | current 526 行 Skill |
| 11 | [`crates/star-cli/src/commands/agent.rs`](../crates/star-cli/src/commands/agent.rs) | current 326 行 enum stub |
| 12 | ULYS-196 PR #81 (`3e510d2b feat(star-cli): ULYS-196 Skill Registry`) | Skill Registry 主实装 |

---

## §10 サインオフ + 改訂履歴

### 5 角色签字栏

| 角色 | 签字 |
|---|---|
| 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手终审 |
| SRE Lead (5 域 Lead 真人到位) | ⏳ 待追溯 |
| 平台 (per 守门 #14) | 🟢 Mavis 接手审核 |
| 评审主持 (per CLI spec 守门 v0.62) | ⏳ 待追溯 |
| PM (Ulysses 一人公司 12 角色 per DEC-008) | 🟢 Ulysses 派发, Mavis 接手 |

### 修订履历 (per 守门 #12 v21)

| 版本 | 日期 | commit | 内容 |
|---|---|---|---|
| v0.1 | 2026-09-20 | `3b791824` | 初版 CL-4+CL-5 BD — 后不可达 |
| **v1.0** | **2026-09-25** | **(本 turn TBD)** | **recreate: 9 类 ToError + Skill 4 子命令 schema + 15 capability 数组 + 8 agent/ide meta 子命令 + MR/Test/Pipeline 7 命令 schema, 全部基于当前 main `b0dddf7e` 实测 + cli-spec v0.2 canonical + SRS-001 v1.0 + BD-001 总冊 v1.0 (同期落档), 0 沿用 v0.1 git 历史叙述 (per AGENTS.md §1.2 #1)** |
| v1.0-fix | 2026-09-25 | (TBD) | F-06 修复: Error 引用 `agent-api/v1#Error` §3.15 (非 §3.14) |

---

**CL-4 + CL-5 落地状态**:
- 7 命令 (CL-5) + 4 Skill 子命令 (CL-4.1) + 1 错误 schema (CL-4.2) 全部 ✅ 已实装 or spec 已固化
- 1 Capability Discovery (CL-4.3) + 8 agent/ide meta (CL-4.4) 待 [M] 子项 (per 守门 #3 5 域 Lead 真人到位)

总行数: ~720 行 (per `wc -l docs/design/BD-STAR-CLI-004.md`).
