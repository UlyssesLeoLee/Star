# 28. Agent Resume Protocol

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/flows/02-agent-lease-heartbeat.md](02-agent-lease-heartbeat.md)

## 1. 场景

Agent A 执行一半崩溃。Agent B / IDE Session B 必须能继续。

## 2. 协议

```bash
star task resume STAR-1024
star workspace resume STAR-1024
```

返回：

```json
{
  "current_state": "Implementing",
  "workspace": {...},
  "worktree": {
    "path": "...",
    "branch": "feature/STAR-1024",
    "head_commit": "...",
    "dirty": true,
    "modified_files": ["src/auth.rs", "src/session.rs"]
  },
  "previous_plan": [
    "1. Add timeout config",
    "2. Update login flow",
    "3. Add test"
  ],
  "modified_files": [...],
  "open_diagnostics": [...],
  "test_results": {"passed": 5, "failed": 1},
  "failed_attempts": [
    {"step": "Add timeout config", "error": "compile error: ..."}
  ],
  "relevant_context": {...},
  "remaining_work": [
    "Fix compile error in src/auth.rs",
    "Re-run tests"
  ]
}
```

## 3. 关键约束

- 实现真正的 **Vendor-independent Agent Handoff**
- 不同厂商的 Agent 都能 Resume 同一任务
- 必须包含"前一个 Agent 为什么失败"

## 4. 实施位置

- `crates/star-agent/src/resume.rs` — Resume 协议
- `crates/star-agent/src/snapshot.rs` — Workspace / Context snapshot

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
