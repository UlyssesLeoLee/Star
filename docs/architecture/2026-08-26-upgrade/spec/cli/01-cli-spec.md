# 11. STAR CLI Specification

> **状态**：🟡 草案 v0.2
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md)

## 1. 设计原则

- 表达 Software Engineering Domain Semantics（不只 REST API 映射）
- Machine-readable 优先于 human-readable
- --json 必须稳定（versioned as `agent-api/v1`）
- Fallback 兼容 shell 解析
- **`star` 是 `git` 的 superset**（per P1-H 修复 2026-08-27） — `star` 提供 `git` 不具备的领域操作（issue / mr / workspace / submit），并包装 `git` 子命令（diff / commit / push）以注入 Policy / Audit / Worktree 上下文；`star` **不**替代 `git`，所有 Git 协议能力继续由 GitGit 提供（per [arch/02 §2 IDE Capability Boundary](../../arch/02-ide-capability-boundary.md)）

## 2. 核心命令

### 2.1 MVP 17 核心命令（per 任务原文 §9，MVP 子集边界）

| 命令 | 用途 | 输出 schema |
|---|---|---|
| `star project list` | 列出项目 | `agent-api/v1#ProjectList` |
| `star issue list` | 列出 issue | `agent-api/v1#IssueList` |
| `star issue show <id>` | 显示 issue 详情 | `agent-api/v1#Issue` |
| `star issue claim <id>` | 认领 issue | `agent-api/v1#ClaimResult` |
| `star task current` | 当前任务 | `agent-api/v1#CurrentTask` |
| `star context get <id>` | 获取 context | `agent-api/v1#Context` |
| `star code search <q>` | 搜索代码 | `agent-api/v1#CodeSearchResult` |
| `star code symbol <name>` | 符号定位 | `agent-api/v1#SymbolResult` |
| `star workspace list` | 列出 workspace | `agent-api/v1#WorkspaceList` |
| `star workspace current` | 当前 workspace | `agent-api/v1#WorkspaceSummary` |
| `star worktree create <id>` | 创建 worktree | `agent-api/v1#Worktree` |
| `star worktree enter <id>` | 进入 worktree | n/a (cd) |
| `star worktree status` | worktree 状态 | `agent-api/v1#WorktreeStatus` |
| `star mr create` | 创建 MR | `agent-api/v1#MR` |
| `star mr show <id>` | MR 详情 | `agent-api/v1#MR` |
| `star test affected` | 跑受影响测试 | `agent-api/v1#TestResult` |
| `star submit` | Universal Submit | `agent-api/v1#SubmitResult` |

> MVP 退出条件 acceptance/04 §3 第一条 = 这 17 命令全部可调用 + --json 稳定 schema。

### 2.2 扩展命令（非 MVP 子集，共 11 个）

| 命令 | 用途 | 输出 schema | 来源 |
|---|---|---|---|
| `star context current` | 当前 context | `agent-api/v1#Context` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star code references <name>` | 引用查找 | `agent-api/v1#ReferencesResult` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star mr review <id>` | Review MR | `agent-api/v1#ReviewResult` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star test run` | 跑全部测试 | `agent-api/v1#TestResult` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star pipeline run` | 跑 pipeline | `agent-api/v1#PipelineRun` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star pipeline status` | pipeline 状态 | `agent-api/v1#PipelineStatus` | 原 §2 表内已有，arch/03 §2.2 缺 |
| `star diff` | Diff 检查（Universal Submit 第 4 步暴露） | `agent-api/v1#DiffResult` | **P1-H 新增**（per 2026-08-27） |
| `star policy check` | Policy 检查（Universal Submit 第 6 步暴露） | `agent-api/v1#PolicyCheckResult` | **P1-H 新增**（per 2026-08-27） |
| `star commit` | Commit（注入 Policy / Audit / Worktree 上下文） | `agent-api/v1#CommitResult` | **P1-H 新增**（per 2026-08-27） |
| `star push` | Push（注入 Audit 上下文） | `agent-api/v1#PushResult` | **P1-H 新增**（per 2026-08-27） |
| `star mr link <id>` | 关联 Issue 到 MR（Universal Submit 第 10 步暴露） | `agent-api/v1#MRLinkResult` | **P1-H 新增**（per 2026-08-27） |

> 6 个原扩展命令（context current / code references / mr review / test run / pipeline run / pipeline status）业务语义完整但不在 MVP 退出条件 17 之列，作为 Phase 2+ 候选。5 个新加命令（diff / policy check / commit / push / mr link）对应 Universal Submit 12 步流程中原本**没有**独立 CLI 命令的 5 步（第 4 / 6 / 7 / 8 / 10 步），per P1-H 修复 2026-08-27。

## 3. 通用 flags

| flag | 说明 |
|---|---|
| `--json` | 强制 JSON 输出 |
| `--quiet` | 只输出 ID / 摘要 |
| `--fields k1,k2` | 限制输出字段 |
| `--limit N` | 限制行数 |
| `--cursor <c>` | 分页游标 |
| `--no-color` | 关闭 ANSI |
| `--schema-version <v>` | 显式 schema 版本 |

## 4. 子命令

```bash
star agent capabilities       # Capability Discovery
star agent describe <cmd>     # 单命令详细 schema
star agent instructions      # 当前环境的 AI 操作说明
star agent permissions       # 权限查询

star ide capabilities
star ide describe <cmd>
star ide instructions
star ide permissions
```

## 5. 错误模型（per §11，统一权威 schema）

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

> 字段：`error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details`（6 字段）— 5 字段原版基础上 + `details` 与 `agent-api/v1#Error`（per [spec/agent-api/01-schema.md §3.15](../agent-api/01-schema.md)）完全对齐。CLI / MCP / REST / Universal Submit **全部**引用同一份 `Error` schema，per P1-G 修复 2026-08-27。

## 6. 实施位置

- `crates/star-cli/` — 主 binary
- `crates/star-cli/src/commands/` — 子命令模块
- `crates/star-cli/src/output.rs` — JSON schema 输出

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：23 命令单表 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-A：§2 拆 17 核心 + 11 扩展（标 MVP 子集边界） · P1-C：`star workspace current` 引用改 `WorkspaceSummary` · P1-G：§5 错误模型 6 字段 + 引用 `agent-api/v1#Error` · P1-H：§1 加 "`star` 是 `git` superset" 原则 + §2.2 增 5 个新命令（diff / policy check / commit / push / mr link） | 8 子代理 INTERFACE-REVIEW-A 🔴 #1/#5/#6/#7 + P1-BLOCKERS-SUMMARY v0.2 |
