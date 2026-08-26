# 11. STAR CLI Specification

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md)

## 1. 设计原则

- 表达 Software Engineering Domain Semantics（不只 REST API 映射）
- Machine-readable 优先于 human-readable
- --json 必须稳定（versioned as `agent-api/v1`）
- Fallback 兼容 shell 解析

## 2. 核心命令（per 任务原文 §9）

| 命令 | 用途 | 输出 schema |
|---|---|---|
| `star project list` | 列出项目 | `agent-api/v1#ProjectList` |
| `star issue list` | 列出 issue | `agent-api/v1#IssueList` |
| `star issue show <id>` | 显示 issue 详情 | `agent-api/v1#Issue` |
| `star issue claim <id>` | 认领 issue | `agent-api/v1#ClaimResult` |
| `star task current` | 当前任务 | `agent-api/v1#CurrentTask` |
| `star context get <id>` | 获取 context | `agent-api/v1#Context` |
| `star context current` | 当前 context | `agent-api/v1#Context` |
| `star code search <q>` | 搜索代码 | `agent-api/v1#CodeSearchResult` |
| `star code symbol <name>` | 符号定位 | `agent-api/v1#SymbolResult` |
| `star code references <name>` | 引用查找 | `agent-api/v1#ReferencesResult` |
| `star workspace list` | 列出 workspace | `agent-api/v1#WorkspaceList` |
| `star workspace current` | 当前 workspace | `agent-api/v1#Workspace` |
| `star worktree create <id>` | 创建 worktree | `agent-api/v1#Worktree` |
| `star worktree enter <id>` | 进入 worktree | n/a (cd) |
| `star worktree status` | worktree 状态 | `agent-api/v1#WorktreeStatus` |
| `star mr create` | 创建 MR | `agent-api/v1#MR` |
| `star mr show <id>` | MR 详情 | `agent-api/v1#MR` |
| `star mr review <id>` | Review MR | `agent-api/v1#ReviewResult` |
| `star test affected` | 跑受影响测试 | `agent-api/v1#TestResult` |
| `star test run` | 跑全部测试 | `agent-api/v1#TestResult` |
| `star pipeline run` | 跑 pipeline | `agent-api/v1#PipelineRun` |
| `star pipeline status` | pipeline 状态 | `agent-api/v1#PipelineStatus` |
| `star submit` | Universal Submit | `agent-api/v1#SubmitResult` |

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

## 5. 错误模型（per §11）

```json
{
  "error": "WORKTREE_CONFLICT",
  "recoverable": true,
  "suggested_actions": ["inspect_conflict", "request_rebase"],
  "message": "Worktree STAR-1024 has uncommitted changes conflicting with main",
  "trace_id": "..."
}
```

## 6. 实施位置

- `crates/star-cli/` — 主 binary
- `crates/star-cli/src/commands/` — 子命令模块
- `crates/star-cli/src/output.rs` — JSON schema 输出

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
