# 26. Agent Task Lifecycle

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/resources/03-agent-identity.md](../resources/03-agent-identity.md)

## 1. 状态机（per §29 任务原文）

```
CREATED
  ↓
CLAIMED
  ↓
CONTEXT_LOADING
  ↓
PLANNING
  ↓
IMPLEMENTING
  ↓
VALIDATING
  ↓
REVIEW_READY
  ↓
SUBMITTED
  ↓
COMPLETED
```

## 2. 异常状态

```
BLOCKED
CONFLICT
FAILED
CANCELLED
HUMAN_REQUIRED
```

## 3. 关键约束

- STAR 只关心这些状态
- 完全不关心执行任务的是 Claude / Codex / Gemini / Local LLM / Cursor Agent / JetBrains Agent
- Provider metadata 不影响状态转换

## 4. 状态转换实现

```rust
// crates/star-agent/src/lifecycle.rs
pub enum AgentTaskState {
    Created, Claimed, ContextLoading, Planning, Implementing,
    Validating, ReviewReady, Submitted, Completed,
    Blocked, Conflict, Failed, Cancelled, HumanRequired,
}

pub fn can_transition(from: AgentTaskState, to: AgentTaskState) -> bool {
    // 状态机
}
```

## 5. 实施位置

- `crates/star-agent/src/lifecycle.rs`

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
