# 14. STAR MCP Specification

> **状态**：🟡 草案 v0.1
> **依赖**：[Protocol Survey §1](../../ecosystem-survey/protocol-survey.md) · [ADR-0021 Zero Vendor Cooperation](../../adr/0021-zero-vendor-cooperation.md)

## 1. 规范版本

- MCP **2026-07-28**（per 2026-08-26 调研）
- Transport: **stdio**（Rust SDK 仍在 beta，Streamable HTTP 风险规避；per Protocol Survey §1）
- 必须兼容旧 spec 至少 12 个月（per MCP 官方 12 个月 deprecation 窗口）

## 2. Tools 列表（13 个领域语义 tools, per §17）

| Tool | 输入 | 输出 |
|---|---|---|
| `get_issue` | `{issue_id}` | `Issue` (per agent-api/v1) |
| `search_issues` | `{query, filters?}` | `IssueList` |
| `get_current_task` | `{}` | `Task` |
| `get_workspace` | `{workspace_id?}` | `Workspace` |
| `get_worktree` | `{worktree_id?}` | `Worktree` |
| `create_worktree` | `{issue_id, branch_name?}` | `Worktree` |
| `search_code` | `{query, limit?, paths?}` | `CodeSearchResult` |
| `get_symbol` | `{name, file?}` | `Symbol` |
| `find_references` | `{name, file?, line?}` | `References` |
| `get_code_context` | `{file, range}` | `CodeContext` |
| `get_context` | `{issue_id}` | `Context` |
| `create_merge_request` | `{title, description, base, head}` | `MR` |
| `request_review` | `{mr_id, reviewers?}` | `Review` |
| `run_validation` | `{worktree_id?}` | `ValidationResult` |
| `get_pipeline_status` | `{pipeline_run_id}` | `PipelineStatus` |

> 注：实际 15 个 tools，比 §17 任务原文多 2 个（get_workspace + request_review 是 17 任务原文中未列但常需要）。

## 3. 禁止直接暴露

| ❌ 禁止 | 替代 |
|---|---|
| `update_issue_table` | `update_issue` (领域操作) |
| `insert_worktree_row` | `create_worktree` (领域操作) |
| `delete_branch_record` | `delete_branch` (领域操作) |
| `update_symbol_index_table` | `invalidate_code_intel_cache` (领域操作) |

## 4. Resources（per MCP 2026-07-28）

可选 Resources（不强制）：
- `repo://{owner}/{name}` — Repository 描述
- `issue://{id}` — Issue 当前状态
- `agent-session://{id}` — Agent session 状态

## 5. Prompts（per MCP 2026-07-28）

可选 Prompts（不强制）：
- `submit-pr` — 引导 agent 完成 PR 流程
- `code-review` — 引导 agent 跑 review

## 6. 实施位置

- `crates/star-mcp/` — MCP server crate
- `crates/star-mcp/src/tools/` — 13 个 tool 实现
- `crates/star-mcp/src/main.rs` — stdio transport entry

## 7. 验证

```bash
# 跑官方 MCP Inspector
npx @modelcontextprotocol/inspector star-mcp

# 必须列出 13+ tools
# 必须能 invoke get_issue
# 必须能 invoke get_current_task
# 必须能 invoke create_worktree
# 必须能 invoke star submit
```

## 8. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
