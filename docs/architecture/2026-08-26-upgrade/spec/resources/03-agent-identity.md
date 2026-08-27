# 22. Agent Identity

> **状态**：🟡 草案 v0.1
> **依赖**：[ADR-0021 Zero Vendor Cooperation](../../adr/0021-zero-vendor-cooperation.md)

## 1. AI Agent 不得只是 User API Token

建立独立 Agent Identity（per §25 任务原文）：

```text
Agent
├── Identity
├── Provider Metadata  (核心决策不依赖)
├── Session
├── IDE Session (link)
├── Permissions
├── Workspace
├── Task
├── Actions (audit)
├── Cost (per session)
└── Audit
```

## 2. 关键约束

- **Provider Metadata 不得进入核心业务决策**（per ADR-0021 + §25 任务原文）
- 决策路径必查：`if provider == "claude" { ... }` 应**不存在**于 Core
- Provider metadata 仅用于 audit / billing / UX（如展示"由 Claude Sonnet 提供"）

## 3. Agent Identity Schema

```json
{
  "id": "agent-abc",
  "kind": "coding_agent",
  "provider": "claude-code",        // 仅 audit/UX 用
  "model": "claude-sonnet-4-5",     // 仅 audit/UX 用
  "session_id": "agent-session-xyz",
  "user_id": "u-1",
  "ide_session_id": "ide-xyz",
  "workspace_id": "ws-STAR-1024",
  "task_id": "STAR-1024",
  "permissions": ["read_repo", "write_worktree", "create_mr"],
  "lease": {
    "acquired_at": "...",
    "expires_at": "...",
    "heartbeat_at": "...",
    "renew_count": 3
  },
  "cost": {"input_tokens": 12345, "output_tokens": 6789, "estimated_usd": 0.42}
}
```

## 4. 与 IDE Session 的关系

- Agent Session 可独立于 IDE Session（如 background agent）
- 但通常与 IDE Session 联动（用户在 IDE 内启动 agent）
- 1 个 Agent Session 可跨多个 IDE Session（handoff 时切换）

## 5. 实施位置

- `crates/star-agent/src/identity.rs` — Agent Identity 模型
- `crates/star-agent/src/lease.rs` — Lease 管理

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
