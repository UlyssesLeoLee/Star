# 16. Context API

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md)

## 1. 目标

不让 Agent 自己在 Repository 中盲目找上下文。提供 `star context get` 一站式拉取当前任务所有相关信息。

## 2. 端点 / 命令

```bash
star context get STAR-1024 --json
star context current --json
star context get STAR-1024 --depth=full --json
```

## 3. 响应 Schema（per §20 任务原文）

```json
{
  "issue": {
    "id": "STAR-1024",
    "title": "...",
    "description": "...",
    "acceptance_criteria": [...]
  },
  "requirement": {...},
  "acceptance_criteria": [...],
  "related_issues": [...],
  "related_mr": {...},
  "architecture_decisions": [...],
  "relevant_documents": [...],
  "relevant_files": [...],
  "relevant_symbols": [...],
  "relevant_tests": [...],
  "relevant_dependencies": [...],
  "historical_changes": [...],
  "schema_version": "context-api/v1"
}
```

## 4. Context Graph（per §21）

MVP 阶段只支持 4 类节点 + 5 类关系：

### 4.1 节点类型

| 节点 | 字段 |
|---|---|
| Issue | id / title / status / labels |
| Repository | id / provider / url |
| Worktree | id / path / branch / head_commit |
| Commit | sha / author / message / files_changed |

### 4.2 关系类型（MVP 5 类）

| 关系 | 含义 |
|---|---|
| `implements` | Worktree implements Issue |
| `modifies` | Commit modifies Worktree |
| `references` | Commit references Issue |
| `belongs_to` | Worktree belongs_to Repository |
| `derived_from` | Commit derived_from Commit (parent) |

### 4.3 留待 Phase 2+

- Symbol / File / MR / Test / Pipeline / Deployment / Incident / Agent / User / Document / Package / Vulnerability 节点
- 完整关系：depends_on / generated_by / reviewed_by / tested_by / deployed_by / caused_by / fixed_by / related_to / located_in / opened_in

## 5. 检索流程（per §21）

```
Graph Narrowing
   ↓
Semantic Retrieval
   ↓
Code Retrieval
   ↓
Symbol Retrieval
   ↓
LLM Context
```

避免把整个 Repository 塞给模型。

## 6. Token Budget

- `depth=minimal` (默认): < 5K tokens
- `depth=normal`: < 20K tokens
- `depth=full`: 无上限（必须显式指定）

## 7. 实施位置

- `crates/star-context/` — Context service
- `crates/star-context/src/graph.rs` — 简化版 context graph (4 节点 + 5 关系)
- `crates/star-context/src/retrieval.rs` — 4 段检索 pipeline

## 8. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
