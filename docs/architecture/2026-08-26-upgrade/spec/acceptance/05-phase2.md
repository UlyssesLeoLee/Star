# 42. Phase 2

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/acceptance/04-mvp.md](04-mvp.md)

## 1. 范围（在 MVP 之上）

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
- Web UI（Human Interface，但**不**作为 Agent API — per §39）

## 2. 不在 Phase 2

- 完整 RAG（Phase 3）
- Code Embedding（Phase 3）
- 完整多 Agent 编排（Phase 3）

## 3. 实施位置

- `crates/star-code-intelligence/src/lsp.rs` (Phase 2)
- 其它新增 crates 由 Phase 2 plan 决定

## 4. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
