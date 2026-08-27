# 26. Agent Task Lifecycle

> **状态**：🟡 草案 v0.2
> **依赖**：[spec/resources/03-agent-identity.md](../resources/03-agent-identity.md) · [spec/agent-api/01-schema.md §3.1 Task](../agent-api/01-schema.md) · [spec/flows/03-agent-resume.md](03-agent-resume.md)
> **状态数校准（per B-02 / P1-N 修复 2026-08-27）**：9 task states + 5 session states = **14 total**。任务摘要曾写"9+4"误去 CONFLICT，spec 实际是 9+5，CONFLICT 在 [flows/04 §2](04-multi-agent.md) 9 类冲突语境下业务语义完整保留。
> **状态枚举（per B-23 修复 2026-08-27）**：`AgentTaskState` 权威定义在 [agent-api/v1 §3.1 Task.status](../agent-api/01-schema.md#31-task)（W4 子代理定义）；本 spec §4 Rust enum 是其在 Rust 端的实现，需与 §3.1 严格 1:1 对齐。

> v0.3 fix: 2026-08-27 per B-02 (顶部 note) / B-23 (AgentTaskState cross-ref)

## 1. 状态机（per §29 任务原文）

> **状态字符串以 Rust enum 命名为准**（per P1-M 修复 2026-08-27）。所有 spec / JSON / CLI / MCP / REST 引用统一 PascalCase（`Implementing` / `Validating` / ...），全大写（SHOUTING_CASE）形式仅作 ASCII 兼容别名，不作为权威。

```
Created
  ↓
Claimed
  ↓
ContextLoading
  ↓
Planning
  ↓
Implementing
  ↓
Validating
  ↓
ReviewReady
  ↓
Submitted
  ↓
Completed
```

## 2. 异常状态

> **5 异常状态保留**（per P1-N 修复 2026-08-27）— 业务语义完整：BLOCKED / CONFLICT / FAILED / CANCELLED / HUMAN_REQUIRED。任务摘要曾写"9+4"误去 CONFLICT，但 CONFLICT 在 flows/04 §2 9 类冲突语境下有意义。spec 实际是 **9 + 5 = 14 状态**。

```
Blocked
Conflict
Failed
Cancelled
HumanRequired
```

## 3. 关键约束

- STAR 只关心这些状态
- 完全不关心执行任务的是 Claude / Codex / Gemini / Local LLM / Cursor Agent / JetBrains Agent
- Provider metadata 不影响状态转换
- 状态机对外行为（per B-03 修复 2026-08-27）：当 Agent 在 `Implementing` / `Validating` 等活跃态崩溃时，状态最终落 `Failed` 或 `Cancelled`，完整 6 步恢复流程见 [spec/flows/02 §3](02-agent-lease-heartbeat.md)（Agent Lost → 保存 Workspace → 保存 Worktree → 保存 Context Snapshot → 释放 Task Lease → 允许其他 Agent Resume）

> v0.3 fix: 2026-08-27 per B-03 (cross-ref 到 flows/02 §3)

## 4. 状态转换实现

> **权威枚举（per B-23 修复 2026-08-27）**：`AgentTaskState` 与 [agent-api/v1 §3.1 Task.status](../agent-api/01-schema.md#31-task) **1:1 对齐**（W4 子代理定义在 agent-api/v1）；本 spec 只放 Rust 实现，权威 schema 以 agent-api/v1 为准。

```rust
// crates/star-agent/src/lifecycle.rs
pub enum AgentTaskState {
    // 9 task states (main flow)
    Created, Claimed, ContextLoading, Planning, Implementing,
    Validating, ReviewReady, Submitted, Completed,
    // 5 session states (exception flow)
    Blocked, Conflict, Failed, Cancelled, HumanRequired,
}

pub fn can_transition(from: AgentTaskState, to: AgentTaskState) -> bool {
    // 状态机
}
```

> v0.3 fix: 2026-08-27 per B-23 (§4 加 1:1 对齐声明)

## 5. 实施位置

- `crates/star-agent/src/lifecycle.rs`

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：9 主态 + 5 异常（SHOUTING_CASE 全大写） | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-M：§1/§2 状态机改 PascalCase（`Implementing` 而非 `IMPLEMENTING`），加注"以 Rust enum 命名为准" · P1-N：§2 保留 5 异常状态（业务语义完整），修订行显式说明 spec 9+5 vs 任务摘要 9+4 的来源差异 | 8 子代理 INTERFACE-REVIEW-B 🔴 B-01/B-02 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.3 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转）| 🔴 B-01 再次校准：明确 §1/§2 14 状态 PascalCase（与 agent-api §3.1 Task.status enum 一致）· 🟡 B-02：顶部加 "9 task states + 5 session states = 14 total" 校准 note · 🟡 B-03：§3 加 cross-ref 到 [flows/02 §3](02-agent-lease-heartbeat.md) Agent Lost 6 步恢复 · 🟡 B-23：§4 加 AgentTaskState 与 [agent-api/v1 §3.1 Task.status](../agent-api/01-schema.md#31-task) 1:1 对齐声明 | worker 子代理修 INTERFACE-REVIEW-B 8 子代理报告 follow-up |

> v0.3 fix: 2026-08-27 per B-01 / B-02 / B-03 / B-23
