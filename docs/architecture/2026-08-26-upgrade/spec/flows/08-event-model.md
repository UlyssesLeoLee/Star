# 33. Event Model

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/flows/07-audit-model.md](07-audit-model.md)

## 1. 所有 Agent / IDE / 代码工作区操作产生标准 Domain Event

（per §34 任务原文）

### 1.1 STAR Domain Events

```
AgentTaskClaimed
ContextRequested
WorkspaceCreated
WorktreeCreated
IDESessionStarted
CodeNavigationRequested
CodeModified
ValidationStarted
ValidationFailed
ValidationSucceeded
MergeRequestCreated
HumanReviewRequested
AgentTaskCompleted
```

### 1.2 GitGit 原生事件

```
RepositoryCreated
CommitCreated
BranchCreated
RefUpdated
WorktreeCreated
WorktreeRemoved
ObjectsReceived
ObjectsFetched
MergeCompleted
ConflictDetected
```

## 2. 关键约束

- GitGit 事件必须与 AI Vendor / IDE Vendor 无关
- STAR 在上层把 GitGit 事件转译为软件工程领域事件

## 3. 实施位置

- `crates/star-event/` — Event bus
- `crates/star-event/src/star_events.rs` — STAR domain events
- `crates/star-event/src/gitgit_bridge.rs` — GitGit → STAR 事件转译

## 4. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
