# 29. Multi-Agent Coordination

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/flows/01-agent-task-lifecycle.md](01-agent-task-lifecycle.md)

## 1. 任务拆分（per §32 任务原文）

```
Issue
  ↓
Task Graph
├── Task A → Agent 1
├── Task B → Agent 2
├── Task C → Agent 3
└── Integration Task → Agent / Human
```

每个 Task 用独立 Worktree 或 Workspace。

## 2. 冲突类型

| 冲突类型 | 解决方式 |
|---|---|
| File Conflict | Git text conflict + AST-level diff |
| Semantic Conflict | Code Intelligence 检查（Phase 2） |
| API Conflict | Schema diff + OpenAPI 检查 |
| Schema Conflict | DDL diff + migration order |
| Dependency Conflict | Cargo.lock / package.json diff |
| Migration Conflict | Migration order check |
| Test Conflict | 跑全部测试 + 看 flake |
| Context Conflict | Context snapshot diff |
| Ownership Conflict | File ownership matrix |

**关键**：不能只依赖 Git Text Conflict。

## 3. MVP 范围

- 只做 File Conflict（Git text conflict）
- 其它冲突类型在 Issue 描述里 warning（不自动检测)

## 4. 实施位置

- `crates/star-agent/src/multi.rs` — Multi-agent coordinator
- `crates/star-agent/src/conflict.rs` — 冲突检测

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
