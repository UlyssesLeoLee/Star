# 27. Agent Lease / Heartbeat

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/flows/01-agent-task-lifecycle.md](01-agent-task-lifecycle.md)
> **Phase 边界（per B-26 修复 2026-08-27）**：
> - **Phase 1 实现**：Task Lease + Heartbeat + Lease Renewal — 心跳固定 30s，TTL 固定 300s（per Project 配置可调）
> - **Phase 2+ 实现**：Session Timeout + Lease Recovery + 自适应心跳（10s baseline + adaptive based on workload / RTT）
> - **Phase 1 ↔ Phase 2+ 切换点**：自适应心跳启用时（Phase 2 触发）；Phase 1 全部用固定常量，Phase 2 切到 dynamic

> v0.2 fix: 2026-08-27 per B-26 (顶部 Phase 1/2 边界)

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

> **状态转换（per B-03 修复 2026-08-27）**：6 步恢复流程中第 5 步"释放 Task Lease"完成后，Agent Task 状态从崩溃前主态（`Implementing` / `Validating` / ...）落 `Failed`（不可恢复）或 `Cancelled`（可恢复 + 其他 Agent 接管）。具体状态机转换见 [spec/flows/01 §3](01-agent-task-lifecycle.md)。
>
> **Lease 字段（per B-04 修复 2026-08-27）**：`AgentTaskLease` 显式字段定义（落 [spec/resources/03 §3](../resources/03-agent-identity.md) `lease` 子对象）：
> - `lease.ttl_seconds: u64` — 默认 300（5 分钟，per Project 配置可调）
> - `lease.heartbeat_interval_seconds: u64` — 默认 30（30s）
> - `lease.acquired_at: timestamp` — Lease 申请时间
> - `lease.expires_at: timestamp` — 由 `acquired_at + ttl_seconds` 派生
> - `lease.heartbeat_at: timestamp` — 最近一次 heartbeat
> - `lease.renew_count: u32` — 续约次数

> v0.2 fix: 2026-08-27 per B-03 (状态转换 cross-ref) / B-04 (Lease 6 字段)

## 4. Heartbeat 协议

```rust
// Agent 每 30s 发一次
agent.heartbeat(agent_session_id, current_state, progress_pct)
  → server 更新 lease.heartbeat_at
  → server 检查 lease 是否过期

// 默认 lease TTL: 5 分钟（per Project 配置可调）
```

> **常量落位（per B-20 修复 2026-08-27）**：`30s` / `300s` 常量在 `crates/star-agent/src/lease.rs:1-10` 显式声明（参考值，Phase D 落地时校对）：
>
> ```rust
> // crates/star-agent/src/lease.rs:1-10
> pub const LEASE_TTL_SECONDS: u64 = 300;        // 5 分钟（per Project 配置可调）
> pub const HEARTBEAT_INTERVAL_SECONDS: u64 = 30; // 30s
> // Phase 2+ 自适应心跳（per B-26 修复 2026-08-27）：
> pub const HEARTBEAT_INTERVAL_ADAPTIVE_BASE: u64 = 10; // 10s baseline + adaptive
> ```
>
> **Phase 1 用固定 30s/300s**（per B-26 修复）；**Phase 2+ 切自适应**（10s baseline + workload/RTT adaptive）。

> v0.2 fix: 2026-08-27 per B-20 (常量落位 lease.rs:1-10) / B-26 (Phase 边界)

## 5. 实施位置

- `crates/star-agent/src/lease.rs` — Lease 管理（30s/300s 常量落位：line 1-10 per B-20）
- `crates/star-agent/src/heartbeat.rs` — Heartbeat 协议
- `crates/star-agent/src/recovery.rs` — Agent Lost 恢复流程

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：6 步恢复 + 30s/300s heartbeat 协议 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转）| 🟡 B-03：§3 加状态转换 cross-ref 到 [flows/01 §3](01-agent-task-lifecycle.md) · 🟡 B-04：§3 显式列 `AgentTaskLease` 6 字段（`ttl_seconds` / `heartbeat_interval_seconds` 等）· 🟡 B-20：§4 30s/300s 常量落位 `crates/star-agent/src/lease.rs:1-10` · 🟡 B-26：顶部 + §4 加 Phase 1（30s 固定）vs Phase 2+（10s baseline + adaptive）边界 | worker 子代理修 INTERFACE-REVIEW-B 8 子代理报告 follow-up |

> v0.2 fix: 2026-08-27 per B-03 / B-04 / B-20 / B-26
