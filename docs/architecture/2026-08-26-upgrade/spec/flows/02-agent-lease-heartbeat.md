# 27. Agent Lease / Heartbeat

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/flows/01-agent-task-lifecycle.md](01-agent-task-lifecycle.md)

## 1. 问题

AI 中途崩溃导致 Issue 永久占用。

## 2. 解决方案（per §30 任务原文）

- Task Lease
- Agent Heartbeat
- Session Timeout
- Lease Renewal
- Lease Recovery

## 3. Agent 消失后

```
Agent Lost
  ↓
保存 Workspace
  ↓
保存 Worktree
  ↓
保存 Context Snapshot
  ↓
释放 Task Lease
  ↓
允许其他 Agent Resume
```

## 4. Heartbeat 协议

```rust
// Agent 每 30s 发一次
agent.heartbeat(agent_session_id, current_state, progress_pct)
  → server 更新 lease.heartbeat_at
  → server 检查 lease 是否过期

// 默认 lease TTL: 5 分钟（per Project 配置可调）
```

## 5. 实施位置

- `crates/star-agent/src/lease.rs` — Lease 管理
- `crates/star-agent/src/heartbeat.rs` — Heartbeat 协议
- `crates/star-agent/src/recovery.rs` — Agent Lost 恢复流程

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
