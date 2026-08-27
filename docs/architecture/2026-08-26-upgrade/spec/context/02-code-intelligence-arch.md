# 17. Code Intelligence Architecture

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md) · [ADR-0022 IDE Placement](../../adr/0022-ide-placement.md)

## 1. 责任归属

**Code Intelligence 全部归 STAR（per §22 任务原文）**。GitGit 只提供 Repository Snapshot / File Content / Diff / Change Events。

## 2. STAR Code Intelligence 子系统

```
GitGit
  │
  ├── Repository Snapshot
  ├── File Content
  ├── Diff
  └── Change Events
          │
          ↓
STAR Code Intelligence
  ├── Parser
  ├── AST Index
  ├── Symbol Index
  ├── Reference Index
  ├── Dependency Graph
  ├── Semantic Index
  └── Context Graph
          │
          ↓
IDE / AI Agent / Review / RAG
```

## 3. MVP 范围（Phase D）

- **只做** Symbol Index + 基础文本搜索 + 简单 Diff
- **不做** AST / Type / Call Graph / Dependency Graph / Semantic Search / RAG
- Phase 2 引入 tree-sitter / rust-analyzer LSP server

## 4. Phase 2+ 范围

- AST 解析（tree-sitter，11+ 语言）
- Symbol Index（跨文件）
- Reference Index
- Type Info（通过 LSP）
- Call Hierarchy
- Dependency Graph
- Semantic Search
- Code Embedding（Phase 3）
- RAG（Phase 3）

## 5. 实施位置

- `crates/star-code-intelligence/` — Code Intelligence service
- `crates/star-code-intelligence/src/indexer.rs` — 索引器
- `crates/star-code-intelligence/src/symbol.rs` — 符号存储
- `crates/star-code-intelligence/src/grep.rs` — 兜底 ripgrep 调用

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
