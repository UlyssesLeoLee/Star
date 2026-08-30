# ADR-0032: MCP Transport 选型（stdio）

> **状态**：🟡 Draft v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签（per §6 签字栏）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)（待归档）
> **依赖**：[ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) · [ADR-0026 STAR AI Compat](0026-star-ai-compat.md) · [Protocol Survey §1](../../../ecosystem-survey/protocol-survey.md)
> **关联**：[spec/mcp/01 MCP Spec](../spec/mcp/01-mcp-spec.md) · [spec/agent-api/01-schema.md §3.15 Error](../spec/agent-api/01-schema.md)

---

## 1. 背景与问题

MCP（Model Context Protocol）2026-07-28 规范提供两种 transport：

1. **stdio** — 标准输入 / 标准输出双向流（JSON-RPC 2.0 帧）
2. **Streamable HTTP** — HTTP POST + SSE（Server-Sent Events）双向流

2026-08-26 调研时（per Protocol Survey §1）发现：
- Rust SDK 仍在 beta，**Streamable HTTP 实现存在 API 不稳定风险**
- stdio transport 在所有主流 Agent 客户端（Codex / Claude Code / Gemini CLI / Cursor / Junie / Kiro CLI / VS Code）中 100% 可用
- 6/7 主流 Coding Agent 客户端把 stdio 当作唯一 transport（per AI Compatibility Matrix）
- IDE 端（VS Code / Cursor / Junie / JetBrains via Junie）也都先实现 stdio，Streamable HTTP 是 Phase 2+ 增量

需要明确 STAR MCP server 在 MVP 阶段选用哪个 transport。

## 2. 决策

**MVP 阶段 STAR MCP server 选 stdio transport。Streamable HTTP 推迟到 Phase 2+。**

### 2.1 stdio transport 关键约束

- stdin / stdout 双向流，JSON-RPC 2.0 帧
- 一行一帧（LF 分隔），server 主动写 stdout 不能混入 stderr / log
- Process 由 Agent 客户端 fork-and-exec（`star-mcp` binary）
- OAuth 2.1 + RFC 9207 issuer validation 在 MCP server 入口校验（per 2026-07-28 关键变更 ⑤）
- Header-based routing（`Mcp-Method` / `Mcp-Name`）通过 JSON envelope 携带（per 2026-07-28 关键变更 ③）
- Stateless core：server 不持有 agent session 状态；所有上下文由 tool input 传入（per 2026-07-28 关键变更 ①）
- 必须兼容旧 spec 至少 12 个月（per MCP 官方 12 个月 deprecation 窗口）

### 2.2 MVP 16 tools 子集边界（per spec/mcp/01 §2 + P1-F 修复 2026-08-27）

实际 16 tools = 13 MVP 必实现 + 1 新增 `submit`（P1-F）+ 2 Phase 2+ 扩展 = 完整 16：

| MVP 必实现 | Phase 2+ 扩展 |
|---|---|
| `get_issue` / `search_issues` / `get_current_task` | `get_workspace`（P1-C 修复后独立） |
| `get_worktree` / `create_worktree` | `request_review`（Phase 2+ 协作） |
| `search_code` / `get_symbol` / `find_references` |  |
| `get_code_context` / `get_context` |  |
| `create_merge_request` |  |
| `run_validation` / `get_pipeline_status` |  |
| `submit`（per P1-F 修复 2026-08-27） |  |

### 2.3 错误模型（per spec/mcp/01 §3.2 + P1-G 修复 2026-08-27）

MCP server 错误响应**全部**引用 `agent-api/v1#Error`（6 字段：error / recoverable / suggested_actions / message / trace_id / details），与 CLI / REST / Universal Submit 统一。

JSON-RPC 2.0 error envelope 映射：

```json
{
  "jsonrpc": "2.0",
  "id": 1,
  "error": {
    "code": -32000,
    "message": "Worktree STAR-1024 has uncommitted changes conflicting with main",
    "data": {
      "error": "WORKTREE_CONFLICT",
      "recoverable": true,
      "suggested_actions": ["inspect_conflict", "request_rebase"],
      "message": "Worktree STAR-1024 has uncommitted changes conflicting with main",
      "trace_id": "...",
      "details": {"worktree_id": "wt-STAR-1024", "conflicting_files": ["src/auth.rs"]}
    }
  }
}
```

### 2.4 2026-07-28 关键变更符合度（per spec/mcp/01 §1.1 + P1-E 修复 2026-08-27）

| 关键变更 | 符合度 | 说明 |
|---|---|---|
| ① Stateless core | ✅ 必遵 | server 不持有 agent session 状态 |
| ② Multi Round-Trip Requests (MRTR) | 🟡 暂不实现 | Phase 2 再评估 |
| ③ Header-based routing | ✅ 必遵 | stdio transport 通过 JSON envelope 携带 method/name |
| ④ 可缓存 list 结果 | ✅ 必遵 | tool list metadata 包含 `ttlMs=30000` + `cacheScope=workspace` |
| ⑤ Authorization hardening | ✅ 必遵 | OAuth 2.1 + issuer validation 在 MCP server 入口校验 |
| ⑥ Feature Lifecycle | ✅ 必遵 | 本 spec 列的 tools 全部 Active；12 个月内不弃用 |

### 2.5 关键架构约束

- tool list 排序按 name 字典序 + metadata 含 `ttlMs` / `cacheScope`（per MCP §1.2）
- server 进程不能输出到 stdout 任何 log / debug 信息（per stdio transport 强约束）— log 走 stderr 或独立文件
- server 必须能 `npx @modelcontextprotocol/inspector star-mcp` 验证
- tool list 16 个（per P1-F 修复 2026-08-27：含 submit）
- 命名风格约定：query = `get_*` / `search_*`；action = `create_*` / `update_*` / `request_*` / `submit`
- 错误模型 6 字段（per P1-G 修复 2026-08-27）

### 2.6 实施位置（per spec/mcp/01 §6）

- `crates/star-mcp/` — MCP server crate
- `crates/star-mcp/src/tools/` — 16 个 tool 实现（含 submit）
- `crates/star-mcp/src/main.rs` — stdio transport entry

## 3. 备选方案与拒绝理由

### 备选 A：MVP 阶段同时实现 stdio + Streamable HTTP
- 拒绝理由：Rust SDK 仍在 beta；Streamable HTTP API 不稳定；2 套 transport 调试成本翻倍

### 备选 B：MVP 阶段只实现 Streamable HTTP
- 拒绝理由：6/7 主流 Agent 客户端只先实现 stdio；MVP 0 客户端覆盖

### 备选 C：自研 MCP 兼容协议（不走 stdio / Streamable HTTP）
- 拒绝理由：违反 ADR-0021 Zero Vendor Cooperation；Vendor 不会改 SDK 适配 STAR 私有协议

### 备选 D：tool 输出走 stdout 但包含 ANSI 颜色
- 拒绝理由：违反 stdio transport 帧约束（ANSI 颜色 = 非法 JSON-RPC 帧）

## 4. 后果与影响

### 4.1 正面

- 6/7 主流 Agent 客户端 100% 可用（per AI Compatibility Matrix）
- stdio transport 实现简单、调试容易（本地 fork-and-exec）
- 兼容旧 spec 至少 12 个月（MCP 官方 deprecation 窗口承诺）
- Stateless core 让 server 横向扩展容易
- 错误模型 6 字段统一（per P1-G），CLI / MCP / REST / Submit 4 处共用

### 4.2 负面 / 成本

- MVP 阶段不支持 Streamable HTTP（Phase 2+ 再加）
- 进程模型限制：单 Agent client 拉起一个 server 进程
- 多 Agent 共享 server 需要 client 端协调（不是 STAR 责任）
- stdout 不能写 log，调试成本略高

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| Streamable HTTP 2027 成为主流 transport | 中 | 中 | Phase 2+ 加 transport；老 spec 12 个月兼容 |
| Rust SDK API 破坏性变更 | 中 | 中 | 锁版本 + 自有 wrapper 层 |
| 某 Agent client 私自改 stdio 帧格式 | 极低 | 中 | MCP Inspector conformance test + 多客户端实测 |

## 5. 与其他 ADR 的关系

- **依赖**：[ADR-0021 Zero Vendor Cooperation](0021-zero-vendor-cooperation.md) — 最高原则
- **依赖**：[ADR-0026 STAR AI Compat](0026-star-ai-compat.md) — 5 通道中 MCP server 由 stdio transport 承载
- **被依赖**：[ADR-0029 Universal Submit](0029-universal-submit.md) — MCP `submit` tool 走 stdio
- **被依赖**：[ADR-0031 Context Graph](0031-context-graph.md) — MCP `get_context` tool 走 stdio

## 6. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | Platform Engineer | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版：stdio transport + 16 tools（per P1-F 含 submit）+ 6 字段错误模型（per P1-G）+ 6 项关键变更符合度（per P1-E） | Phase B 起草（per 2026-08-26 升级 Plan） |
