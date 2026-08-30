# 14. STAR MCP Specification

> **状态**：🟡 草案 v0.2
> **依赖**：[Protocol Survey §1](../../../../ecosystem-survey/protocol-survey.md) · [ADR-0021 Zero Vendor Cooperation](../../adr/0021-zero-vendor-cooperation.md) · [spec/agent-api/01-schema.md §3.15 Error](../agent-api/01-schema.md)

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

### 2.1 命名约定（per F-08 / F-14 修复 2026-08-27）

显式声明 MCP tool 命名风格（per INTERFACE-REVIEW-A 🟡 #8 + 🟡 #14）：

- **查询（query）**：
  - `get_*` — 单对象查询（如 `get_issue`, `get_workspace`）
  - `search_*` — 多对象查询（如 `search_issues`, `search_code`, `find_references` 用 `find_*` 而非 `search_*`，因引用查找语义不同）
- **操作（action）**：
  - `create_*` — 创建（如 `create_worktree`, `create_merge_request`）
  - `update_*` — 更新（如未来 `update_issue`）
  - `delete_*` — 删除（如未来 `delete_branch`）
  - `request_*` — 触发动作（如 `request_review`，区别于 `create_*` 的"提交新对象"）
  - `submit` — Universal Submit 入口（per P1-F 修复，暴露 [spec/flows/05](../flows/05-universal-submit.md) 12 步流程）
- **缩写**：
  - MCP 用 `merge_request` 全名（machine 协议层，区别于 CLI `mr` 缩写 shell 习惯）
  - MCP 用 `validation`（STAR 内部测试+检查统一表达，区别于 CLI `test` 业务视角）
  - MCP 用 `code` 不用 `code_` 前缀（`search_code` 而非 `search_code_*`）
- **跨层命名差异**（per INTERFACE-REVIEW-A 🟡 #8）：CLI `list` 动词 → MCP `search_*`（`star issue list` ↔ `search_issues`）；CLI `status` 动词 → MCP `get_*`（`star worktree status` ↔ `get_worktree`）；CLI `current` 修饰 → MCP `get_*_current_*`（`star task current` ↔ `get_current_task`）。

### 2.2 Tool list 排序 + ttlMs 缓存要求（per F-18 修复 2026-08-27）

- **排序**：tool list 必须按 tool name 字典序升序（per [ecosystem-survey/protocol-survey.md §1 对 STAR 的推论](../../../../ecosystem-survey/protocol-survey.md)）—— 不排序会导致 client 端 hash 校验失败
- **ttlMs 缓存**：每个 tool 的 `metadata` 列包含 `ttlMs`（毫秒）+ `cacheScope`（`workspace` / `session` / `none` 三选一）；0 = 不缓存
- **cacheScope 语义**：
  - `workspace` — 同 workspace 内所有 agent 共享缓存（默认 30s / 60s）
  - `session` — 单 agent session 私有缓存（默认 5s）
  - `none` — 不缓存（如 `create_*` / `request_review` / `submit` 等写操作）

### 2.3 工具表（per F-18 修复 2026-08-27，已按 name 升序 + 加 metadata）

| Tool | 输入 | 输出 | metadata (ttlMs / cacheScope) |
|---|---|---|---|
| `create_merge_request` | `{title, description, base, head}` | `MR` (per agent-api/v1 §3.7) | 0 / none |
| `create_worktree` | `{issue_id, branch_name?}` | `Worktree` (per agent-api/v1 §3.2) | 0 / none |
| `find_references` | `{name, file?, line?}` | `ReferencesResult` | 60000 / workspace |
| `get_code_context` | `{file, range}` | `CodeContext` | 60000 / workspace |
| `get_context` | `{issue_id}` | `Context` (per agent-api/v1 §3.8) | 30000 / workspace |
| `get_current_task` | `{}` | `CurrentTask` (per agent-api/v1 §3.6) | 5000 / session |
| `get_issue` | `{issue_id}` | `Issue` (per agent-api/v1 §3.4) | 30000 / workspace |
| `get_pipeline_status` | `{pipeline_run_id}` | `PipelineStatus` | 5000 / session |
| `get_symbol` | `{name, file?}` | `SymbolResult` (per agent-api/v1 §3.10) | 60000 / workspace |
| `get_workspace` | `{workspace_id?}` | `WorkspaceSummary` (per P1-C 修复，agent-api/v1 §3.16) | 30000 / workspace |
| `get_worktree` | `{worktree_id?}` | `Worktree` (per agent-api/v1 §3.2) | 30000 / workspace |
| `request_review` | `{mr_id, reviewers?}` | `ReviewResult` | 0 / none |
| `run_validation` | `{worktree_id?}` | `ValidationResult` (TestResult, per agent-api/v1 §3.12) | 0 / none |
| `search_code` | `{query, limit?, paths?}` | `CodeSearchResult` (per agent-api/v1 §3.9) | 60000 / workspace |
| `search_issues` | `{query, filters?}` | `IssueList` (per agent-api/v1 §3.5) | 30000 / workspace |
| `submit` | `{worktree_id?, force?}` | `SubmitResult` (per agent-api/v1 §3.3) | 0 / none |

> **F-18 排序验证（per 2026-08-27）**：上表 16 行已按 name 字典序升序（`create_*` → `find_*` → `get_*` → `request_*` → `run_*` → `search_*` → `submit`），可直接复制为 MCP `tools/list` 响应的数组。
>
> **F-18 ttlMs 验证（per 2026-08-27）**：16 个 tool 全部带 `metadata.ttlMs` + `metadata.cacheScope`，0 = 写操作不缓存。
>
> **数字基线（per P1-F 修复）**：实际 **16** 个 tools = 15 原工具 + 1 新增 `submit`。`submit` 暴露 Universal Submit 12 步流程（per [spec/flows/05 §2](../flows/05-universal-submit.md)），让 MCP 用户不必手动拼 `create_merge_request` + `request_review` + `run_validation` 三件套。

## 3. 禁止直接暴露 / 错误模型

### 3.1 禁止直接暴露

| ❌ 禁止 | 替代 |
|---|---|
| `update_issue_table` | `update_issue` (领域操作) |
| `insert_worktree_row` | `create_worktree` (领域操作) |
| `delete_branch_record` | `delete_branch` (领域操作) |
| `update_symbol_index_table` | `invalidate_code_intel_cache` (领域操作) |

### 3.2 错误模型（per P1-G / F-06 修复 2026-08-27）

> MCP server 错误响应**全部**引用 `agent-api/v1#Error`（per [spec/agent-api/01-schema.md §3.15 Error](../agent-api/01-schema.md)，W4 子代理定义 per P1-G 修复 2026-08-27），与 CLI / REST / Universal Submit 统一。
>
> **F-06 引用约定（per 2026-08-27）**：本 spec 引用 `agent-api/01-schema.md §3.15 Error`（**不**重新定义 6 字段）。任务原始描述 "§3.14" 是 W4 子代理修复时的初稿编号，正式落盘后 `Capabilities` 占 §3.14，`Error` 落 §3.15。统一以落盘节号为准。
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
> `data` 字段 = 完整 `agent-api/v1#Error` 6 字段对象（`error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details`，per F-06 修复 2026-08-27）。CLI / MCP / REST / Submit 4 处共用同一 schema，per P1-G 修复。

## 4. Resources（per MCP 2026-07-28）

**MVP 范围（Level 1-2）声明（per F-28 修复 2026-08-27）**：MCP Resources **MVP 不实现**，Phase 2 评估。

per [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md) Capability Level 定义：
- **Level 1** (Basic Discovery): tools + capabilities + permissions + instructions — 已实现
- **Level 2** (Submit): tools + submit tool — 已实现
- **Level 3+** (Full MCP 2026-07-28): tools + submit + resources + prompts — **MVP 不实现**

MVP 阶段本 spec 不实现 Resources；Phase 2 评估时再展开。**本节列的 `repo://` / `issue://` / `agent-session://` 仅为 Phase 2 候选，不进 MVP 退出条件**。

> **修复说明（per F-28）**：原 v0.1 / v0.2 用 "可选 / 不强制" 措辞模糊，arch/03 §2.3 没说 "可选"。F-28 明确 MVP 不实现（不是 "可选"），与 arch/03 §2.3 Level 1-2 范围对齐。

## 5. Prompts（per MCP 2026-07-28）

**MVP 范围（Level 1-2）声明（per F-28 修复 2026-08-27）**：MCP Prompts **MVP 不实现**，Phase 2 评估。

per [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md) Capability Level 定义（与 §4 同步）：
- **Level 1-2** MVP 范围 = tools + submit，**不含** Prompts
- **Level 3+** 才展开 Prompts

MVP 阶段本 spec 不实现 Prompts；Phase 2 评估时再展开。**本节列的 `submit-pr` / `code-review` 仅为 Phase 2 候选，不进 MVP 退出条件**。

> **修复说明（per F-28）**：与 §4 同步，明确 MVP 不实现 Prompts。原 v0.1 / v0.2 用 "可选 / 不强制" 措辞模糊，F-28 统一改为 "MVP 不实现，Phase 2 评估"。

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

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：15 tools + 3 节（禁止 / Resources / Prompts） | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-E：§1.1 加 2026-07-28 6 项关键变更符合度表 · §2 工具表加 metadata 列（ttlMs / cacheScope） · P1-F：§2 加 `submit` tool（16 tools 总计）+ §6/§7 同步 16 · P1-G：§3.2 错误模型引用 `agent-api/v1#Error` 6 字段 | 8 子代理 INTERFACE-REVIEW-A 🔴 #2/#3/#6 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.2 fix | 2026-08-27 | Mavis（接手 agent per DEC-008）| **F-08 / F-14**：§2.1 显式声明命名约定（query=`get_*` / `search_*`；action=`create_*` / `update_*` / `request_*` / `submit`） · **F-18**：§2.2 显式声明 tool list 字典序升序 + ttlMs 缓存要求；§2.3 工具表重排为升序（`create_*` → `get_*` → `search_*` → `submit`），每行加 `agent-api/v1 §3.x` schema 节号引用 · **F-06**：§3.2 错误模型明确引用 `agent-api/01 §3.15 Error`（注：W4 初稿编号 §3.14，落盘后 Error 在 §3.15） · **F-28**：§4/§5 Resources / Prompts 明确 "MVP 不实现（Level 1-2 不含），Phase 2 评估" —— 改原"可选 / 不强制"模糊措辞 | 8 子代理 INTERFACE-REVIEW-A 🟡 #8/#14/#18 + 🟢 #28 + INTERFACE-REVIEW-C P1-F |
