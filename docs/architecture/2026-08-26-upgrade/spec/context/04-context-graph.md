# 19. Context Graph

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/context/01-context-api.md](01-context-api.md)

## 1. MVP 节点类型（4 类）

| 节点 | 字段 |
|---|---|
| Issue | id / title / status / labels |
| Repository | id / provider / url / name |
| Worktree | id / path / branch / head_commit |
| Commit | sha / author / message / files_changed |

## 2. MVP 关系类型（5 类）

| 关系 | 含义 |
|---|---|
| `implements` | Worktree → Issue |
| `modifies` | Commit → Worktree |
| `references` | Commit → Issue |
| `belongs_to` | Worktree → Repository |
| `derived_from` | Commit → Commit (parent) |

## 3. Phase 2+ 节点类型（10+ 类）

Symbol / File / MR / Test / Pipeline / Deployment / Incident / Agent / User / Document / Package / Vulnerability

## 4. Phase 2+ 关系类型（12+ 类）

depends_on / generated_by / reviewed_by / tested_by / deployed_by / caused_by / fixed_by / related_to / located_in / opened_in

## 5. 存储

- MVP: SQLite + 简单外键
- Phase 2: 考虑图数据库（per [Compatibility Matrix §6 已知缺口](../../ecosystem-survey/compatibility-matrix.md) — **不**自建图数据库）

## 6. 实施位置

- `crates/star-context/src/graph.rs` — 节点 + 关系
- `crates/star-context/migrations/` — SQLite schema

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
