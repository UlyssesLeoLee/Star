# 42. Phase 2

> **状态**：🟡 草案 v0.2
> **依赖**：[spec/acceptance/04-mvp.md](04-mvp.md) · [arch/03 §2.4 REST + OpenAPI 3.1](../../arch/03-star-ai-compat-arch.md) · [arch/03 §5 5 接入通道](../../arch/03-star-ai-compat-arch.md)

## 1. 范围（在 MVP 之上）

### 1.1 跨期约束（per P2-10 修复 2026-08-27 重申）

> **Web UI 不属于 Agent API 通道**（per [arch/03 §2.4 REST + OpenAPI 3.1](../../arch/03-star-ai-compat-arch.md) + 5 接入通道 = Git / CLI / MCP / REST / AGENTS.md）。Web UI 是 Human Interface（Phase 2 起逐步交付），**不**作为 Agent / AI Coding Agent 的 API 入口；Agent 仍通过 5 接入通道（Git / CLI / MCP / REST / AGENTS.md）接入 STAR。Phase D 实施 Web UI 时**严禁**误把 Web UI 当 Agent API 通道（per 子代理 C P2-10 弱信号）。

### 1.2 范围列表

- Symbol Index（用 tree-sitter + rust-analyzer LSP）
- AST 解析（多语言）
- Find References（LSP）
- Document Symbols（LSP）
- Call Hierarchy
- Workspace Symbol
- Decision Memory
- Agent Handoff 完整
- Acceptance Coverage UI
- Saved Worktree Views
- Development Heatmap Phase 1
- Agent Policy Templates
- Remote Runner
- Context Cost Analysis
- PR Review Feedback Import
- **Web UI（Human Interface，但**不**作为 Agent API — per §39 + §1.1 跨期约束）**

> Web UI 走 REST API 但**不**等同 Agent API：Web UI 调 REST 是为人类交互优化（人类 UI 流程 ≠ 机器可读 API）。Agent API 仍是 5 接入通道（Git / CLI / MCP / REST / AGENTS.md）的 REST 端（per [arch/03 §2.4](../../arch/03-star-ai-compat-arch.md)）。

## 2. 不在 Phase 2

- 完整 RAG（Phase 3）
- Code Embedding（Phase 3）
- 完整多 Agent 编排（Phase 3）
- Agent API 通道扩展（5 通道在 Phase 1 已固化，Phase 2/3 不加新通道；如需扩展走 ADR 流程）

## 3. 实施位置

- `crates/star-code-intelligence/src/lsp.rs` (Phase 2)
- `crates/star-web-ui/` (Phase 2 Human Interface，**不**作为 Agent API 通道)
- 其它新增 crates 由 Phase 2 plan 决定

## 4. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：16 项 Phase 2 新增 + 3 项 Phase 3 排除 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P2-10：§1 加跨期约束重申"Web UI 不属 Agent API 通道"（per arch/03 §2.4 + §5）+ §3 实施位置加 `crates/star-web-ui/` Human Interface 标注 · §2 禁项加"Agent API 通道扩展（5 通道 Phase 1 固化）" | 8 子代理 INTERFACE-REVIEW-C P2-10 + P1-BLOCKERS-SUMMARY v0.2 |

> v0.2 fix: 2026-08-27 per C-5 (P2-10)
