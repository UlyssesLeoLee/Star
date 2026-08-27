# 29. Multi-Agent Coordination

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/flows/01-agent-task-lifecycle.md](01-agent-task-lifecycle.md) · [spec/context/02-code-intelligence-arch.md §3-4](../context/02-code-intelligence-arch.md)

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

| 冲突类型 | 解决方式 | 能力依赖（per B-15 修复 2026-08-27） |
|---|---|---|
| File Conflict | Git text conflict + AST-level diff | **Phase 1**（Git + 基础 AST diff per [context/02 §3](../context/02-code-intelligence-arch.md#3-mvp-范围)）|
| Semantic Conflict | Code Intelligence 检查 | **Phase 2+**（per [context/02 §4](../context/02-code-intelligence-arch.md#4-phase-2-范围) AST / Reference / Semantic）|
| API Conflict | Schema diff + OpenAPI 检查 | **Phase 2+**（per [context/02 §4](../context/02-code-intelligence-arch.md#4-phase-2-范围) Type / Reference）|
| Schema Conflict | DDL diff + migration order | **Phase 2+**（per [context/02 §4](../context/02-code-intelligence-arch.md#4-phase-2-范围) Reference）|
| Dependency Conflict | Cargo.lock / package.json diff | **Phase 2+**（per [context/02 §4](../context/02-code-intelligence-arch.md#4-phase-2-范围) Dependency）|
| Migration Conflict | Migration order check | **Phase 2+**（per [context/02 §4](../context/02-code-intelligence-arch.md#4-phase-2-范围) Reference）|
| Test Conflict | 跑全部测试 + 看 flake | **Phase 1**（`cargo test` / `npm test`，per [context/02 §3](../context/02-code-intelligence-arch.md#3-mvp-范围)）|
| Context Conflict | Context snapshot diff | **Phase 2+**（per [context/02 §4](../context/02-code-intelligence-arch.md#4-phase-2-范围) Semantic）|
| Ownership Conflict | File ownership matrix | **Phase 2+**（per [context/02 §4](../context/02-code-intelligence-arch.md#4-phase-2-范围) Reference）|

**关键**：不能只依赖 Git Text Conflict。**MVP（Phase 1）只能可靠解决 2 类**（File + Test），其余 7 类依赖 Code Intelligence Phase 2+ 能力（per [context/02 §3-4](../context/02-code-intelligence-arch.md) 能力边界 cross-ref per B-15 修复 2026-08-27）。

> v0.2 fix: 2026-08-27 per B-15 (9 类冲突能力依赖 cross-ref context/02 §3-4)

## 3. MVP 范围

- 只做 File Conflict（Git text conflict）
- 其它冲突类型在 Issue 描述里 warning（不自动检测）

> **Warning 字段定义（per B-22 修复 2026-08-27）**：MVP Phase 1 不能自动检测的 7 类冲突（除 File/Test 外）在 Issue 描述顶部加 Markdown banner，banner 字段：
> - `severity` (enum: `info` | `warning` | `error`) — 默认 `warning`
> - `conflict_id` (string, 唯一标识 e.g. `"semantic-conflict-STAR-1024-001"`)
> - `conflict_type` (enum: `Semantic` | `API` | `Schema` | `Dependency` | `Migration` | `Context` | `Ownership`)
> - `affected_paths` (string[] — 受影响文件/模块路径)
> - `resolution_hint` (string — 建议如何处理，e.g. `"请人工 review src/auth.rs vs src/auth_v2.rs 的 API 兼容性"`)
> - `detection_phase` (enum: `Phase1-MVP` | `Phase2-Enhanced` | `Phase3-Full` — 标识该冲突类型何时可自动检测)
>
> **示例 banner**：
>
> ```markdown
> > ⚠️ **Conflict Warning** (severity=warning, id=semantic-conflict-STAR-1024-001)
> > - **Type**: Semantic
> > - **Affected**: `src/auth.rs`, `src/session.rs`
> > - **Hint**: Semantic conflict requires Phase 2 Code Intelligence; please manual review
> > - **Auto-detection**: Phase 2+ (per [context/02 §4](../context/02-code-intelligence-arch.md#4-phase-2-范围))
> ```
>
> Agent 解析 banner 后选择是否进入该 worktree（per B-22 修复 2026-08-27）。

> v0.2 fix: 2026-08-27 per B-22 (Warning 6 字段定义)

## 4. 实施位置

- `crates/star-agent/src/multi.rs` — Multi-agent coordinator
- `crates/star-agent/src/conflict.rs` — 冲突检测
- `crates/star-agent/src/conflict_warning.rs` — Warning banner 渲染（per B-22 修复 2026-08-27）

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：9 类冲突清单 + MVP File-only | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转）| 🟡 B-15：§2 9 类冲突加"能力依赖"列，cross-ref 到 [context/02 §3-4](../context/02-code-intelligence-arch.md)（Phase 1 = File + Test；Phase 2+ = 7 类）· 🟡 B-22：§3 显式列 Conflict Warning 6 字段（`severity` / `conflict_id` / `conflict_type` / `affected_paths` / `resolution_hint` / `detection_phase`）+ Markdown banner 示例 + §4 增 `conflict_warning.rs` 实施位置 | worker 子代理修 INTERFACE-REVIEW-B 8 子代理报告 follow-up |

> v0.2 fix: 2026-08-27 per B-15 / B-22
