# 14. STAR MCP Specification

> **状态**：🟡 草案 v0.2
> **依赖**：[Protocol Survey §1](../../ecosystem-survey/protocol-survey.md) · [ADR-0021 Zero Vendor Cooperation](../../adr/0021-zero-vendor-cooperation.md) · [spec/agent-api/01-schema.md §3.15 Error](../agent-api/01-schema.md)

## 1. 规范版本

- MCP **2026-07-28**（per 2026-08-26 调研）
- Transport: **stdio**（Rust SDK 仍在 beta，Streamable HTTP 风险规避；per Protocol Survey §1）
- 必须兼容旧 spec 至少 12 个月（per MCP 官方 12 个月 deprecation 窗口）

### 1.1 2026-07-28 关键变更符合度（per P1-E 修复 2026-08-27）

| 关键变更 | 符合度 | 说明 |
|---|---|---|
| ① Stateless core（无 session） | ✅ 必遵 | server 不持有 agent session 状态；所有上下文由 tool input 传入 |
| ② Multi Round-Trip Requests (MRTR) | 🟡 暂不实现 | Phase 2 再评估，MVP 工具都是单回合 |
| ③ Header-based routing（`Mcp-Method` / `Mcp-Name`） | ✅ 必遵 | stdio transport 通过 JSON envelope 携带 method/name |
| ④ 可缓存 list 结果（`ttlMs` / `cacheScope`） | ✅ 必遵 | tool list metadata 包含 `ttlMs=30000` + `cacheScope=workspace`（per §2 metadata 列） |
| ⑤ Authorization hardening（RFC 9207 issuer validation） | ✅ 必遵 | OAuth 2.1 + issuer validation 在 MCP server 入口校验 |
| ⑥ 正式 Feature Lifecycle（Active / Deprecated / Removed） | ✅ 必遵 | 本 spec 列的 tools 全部 Active；12 个月内不弃用 |

### 1.2 兼容承诺

- 必须兼容旧 spec 至少 12 个月（per MCP 官方 12 个月 deprecation 窗口）
- tool list 按 name 字典序排序（deterministic order）
- tool metadata 必含 `ttlMs` + `cacheScope` 字段

## 2. Tools 列表（16 个领域语义 tools, per §17 + P1-F submit）

| Tool | 输入 | 输出 | metadata (ttlMs / cacheScope) |
|---|---|---|---|
| `get_issue` | `{issue_id}` | `Issue` (per agent-api/v1) | 30000 / workspace |
| `search_issues` | `{query, filters?}` | `IssueList` | 30000 / workspace |
| `get_current_task` | `{}` | `Task` | 5000 / session |
| `get_workspace` | `{workspace_id?}` | `WorkspaceSummary` (per P1-C 修复，agent-api/v1 §3.16) | 30000 / workspace |
| `get_worktree` | `{worktree_id?}` | `Worktree` | 30000 / workspace |
| `create_worktree` | `{issue_id, branch_name?}` | `Worktree` | 0 / none |
| `search_code` | `{query, limit?, paths?}` | `CodeSearchResult` | 60000 / workspace |
| `get_symbol` | `{name, file?}` | `SymbolResult` | 60000 / workspace |
| `find_references` | `{name, file?, line?}` | `ReferencesResult` | 60000 / workspace |
| `get_code_context` | `{file, range}` | `CodeContext` | 60000 / workspace |
| `get_context` | `{issue_id}` | `Context` | 30000 / workspace |
| `create_merge_request` | `{title, description, base, head}` | `MR` | 0 / none |
| `request_review` | `{mr_id, reviewers?}` | `ReviewResult` | 0 / none |
| `run_validation` | `{worktree_id?}` | `ValidationResult` (TestResult) | 0 / none |
| `get_pipeline_status` | `{pipeline_run_id}` | `PipelineStatus` | 5000 / session |
| `submit` | `{worktree_id?, force?}` | `SubmitResult` | 0 / none |

> 注：实际 **16** 个 tools = 15 原工具 + 1 新增 `submit`（per P1-F 修复 2026-08-27）。`submit` 暴露 Universal Submit 12 步流程（per [spec/flows/05 §2](../flows/05-universal-submit.md)），让 MCP 用户不必手动拼 `create_merge_request` + `request_review` + `run_validation` 三件套。
>
> 命名风格约定：query = `get_*` / `search_*`；action = `create_*` / `update_*` / `request_*` / `submit`（per 子代理 A 🟡 #14）。
>
> tool list 排序：按 name 字典序升序（per §1.2 deterministic order 约束）。

## 3. 禁止直接暴露 / 错误模型

### 3.1 禁止直接暴露

| ❌ 禁止 | 替代 |
|---|---|
| `update_issue_table` | `update_issue` (领域操作) |
| `insert_worktree_row` | `create_worktree` (领域操作) |
| `delete_branch_record` | `delete_branch` (领域操作) |
| `update_symbol_index_table` | `invalidate_code_intel_cache` (领域操作) |

### 3.2 错误模型（per P1-G 修复 2026-08-27）

> MCP server 错误响应**全部**引用 `agent-api/v1#Error`（per [spec/agent-api/01-schema.md §3.15](../agent-api/01-schema.md)），与 CLI / REST / Universal Submit 统一。
>
> JSON-RPC 2.0 error envelope 映射：
>
> ```json
> {
>   "jsonrpc": "2.0",
>   "id": 1,
>   "error": {
>     "code": -32000,
>     "message": "Worktree STAR-1024 has uncommitted changes conflicting with main",
>     "data": {
>       "error": "WORKTREE_CONFLICT",
>       "recoverable": true,
>       "suggested_actions": ["inspect_conflict", "request_rebase"],
>       "message": "Worktree STAR-1024 has uncommitted changes conflicting with main",
>       "trace_id": "...",
>       "details": {"worktree_id": "wt-STAR-1024", "conflicting_files": ["src/auth.rs"]}
>     }
>   }
> }
> ```
>
> `data` 字段 = 完整 `agent-api/v1#Error` 6 字段对象。CLI / MCP / REST / Submit 4 处共用同一 schema，per P1-G 修复。

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
- `crates/star-mcp/src/tools/` — 16 个 tool 实现（含 submit, per P1-F 修复 2026-08-27）
- `crates/star-mcp/src/main.rs` — stdio transport entry

## 7. 验证

```bash
# 跑官方 MCP Inspector
npx @modelcontextprotocol/inspector star-mcp

# 必须列出 16 tools（per P1-F 修复 2026-08-27：含 submit）
# 必须能 invoke get_issue
# 必须能 invoke get_current_task
# 必须能 invoke create_worktree
# 必须能 invoke submit（per P1-F：原"star submit"验证项替换为"submit" tool 验证，per agent-api/v1#SubmitResult 验证响应）
```

## 8. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：15 tools + 3 节（禁止 / Resources / Prompts） | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-E：§1.1 加 2026-07-28 6 项关键变更符合度表 · §2 工具表加 metadata 列（ttlMs / cacheScope） · P1-F：§2 加 `submit` tool（16 tools 总计）+ §6/§7 同步 16 · P1-G：§3.2 错误模型引用 `agent-api/v1#Error` 6 字段 | 8 子代理 INTERFACE-REVIEW-A 🔴 #2/#3/#6 + P1-BLOCKERS-SUMMARY v0.2 |
