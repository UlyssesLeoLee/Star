# ADR-0030: Agent 租约 / 心跳 / 恢复

> **状态**：🟡 Draft v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签（per §6 签字栏）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)（待归档）
> **依赖**：[ADR-0026 STAR AI Compat](0026-star-ai-compat.md) · [spec/flows/01-agent-task-lifecycle.md](../spec/flows/01-agent-task-lifecycle.md) · [spec/agent-api/01-schema.md §3.17 Resume](../spec/agent-api/01-schema.md)
> **关联**：[flows/02 Agent Lease/Heartbeat](../spec/flows/02-agent-lease-heartbeat.md) · [flows/03 Agent Resume](../spec/flows/03-agent-resume.md)

---

## 1. 背景与问题

AI Coding Agent 跟人类开发者有本质不同：**会中途崩溃**（OOM / 上下文超限 / vendor 服务断流 / 临时网络分区 / Agent SDK panic）。

如果 Agent 崩溃时 Issue / Task / Worktree 仍被占着：

- 1) 永久占用 — 没人能 claim，Issue 永远卡住
- 2) 状态丢失 — 崩溃前的 worktree 改动 / context snapshot / test result 全丢
- 3) 跨厂商不可能 resume — Claude Code 崩溃后 Codex 无法接续工作

需要一套 **Lease + Heartbeat + Resume** 三段机制，让任何 Agent 崩溃都可被任意其他 Agent / IDE Session 接续。

## 2. 决策

**采用 "Task Lease + Agent Heartbeat + 5 元素 Snapshot + 11 字段 Resume JSON" 的 Lease/Heartbeat/Resume 协议。**

### 2.1 Lease + Heartbeat（per flows/02）

```rust
// Agent 每 30s 发一次
agent.heartbeat(agent_session_id, current_state, progress_pct)
  → server 更新 lease.heartbeat_at
  → server 检查 lease 是否过期

// 默认 lease TTL: 5 分钟（per Project 配置可调）
```

**Lease 5 元素**（per flows/02 §2）：
- Task Lease
- Agent Heartbeat
- Session Timeout
- Lease Renewal
- Lease Recovery

### 2.2 Agent Lost 后 5 步（per flows/02 §3）

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

### 2.3 Resume 协议 11 字段（per flows/03 + P1-O 修复 2026-08-27）

```bash
star task resume STAR-1024
star workspace resume STAR-1024
```

返回 schema = `agent-api/v1#Resume` 11 字段（per P1-O 修复 2026-08-27）：

1. `current_state` (string, PascalCase — per P1-M 修复 2026-08-27，例如 `"Implementing"`)
2. `workspace` (WorkspaceSummary)
3. `worktree` (Worktree + `modified_files: string[]`)
4. `previous_plan` (string[])
5. `modified_files` (string[])
6. `open_diagnostics` (Diagnostic[])
7. `test_results` (TestResult)
8. `failed_attempts` (FailedAttempt[])
9. `relevant_context` (Context)
10. `remaining_work` (string[])
11. `last_modified` (timestamp)

### 2.4 关键架构约束

- **真正的 Vendor-Independent Agent Handoff**（per flows/03 §3 强约束）
- 不同厂商的 Agent 都能 Resume 同一任务
- Resume payload **必须**包含"前一个 Agent 为什么失败"（`failed_attempts` 字段）
- 状态字符串统一 PascalCase（与 spec/flows/01 §1 Rust enum 命名一致，per P1-M 修复 2026-08-27）
- Lease TTL 默认 5 分钟，per Project 可调
- Heartbeat 间隔 30s，timeout 后 5 步释放 lease
- Context Snapshot 必须包含 Workspace / Worktree / Context / Test / Diagnostics（5 元素）

### 2.5 实施位置（per flows/02 §5 + flows/03 §4）

- `crates/star-agent/src/lease.rs` — Lease 管理
- `crates/star-agent/src/heartbeat.rs` — Heartbeat 协议
- `crates/star-agent/src/recovery.rs` — Agent Lost 恢复流程
- `crates/star-agent/src/resume.rs` — Resume 协议
- `crates/star-agent/src/snapshot.rs` — Workspace / Context snapshot

## 3. 备选方案与拒绝理由

### 备选 A：Agent 不持有 Lease（崩溃后人工重新 claim）
- 拒绝理由：违背"AI 全自动协作"目标；单 Agent 崩溃即整个流水线停滞

### 备选 B：Lease = 永久占用（直到 Issue 完成）
- 拒绝理由：崩溃 Agent 永久卡 Issue

### 备选 C：Resume 协议不包含失败原因
- 拒绝理由：下一个 Agent 必须能从前一个 Agent 失败中学到东西；否则会重复犯错

### 备选 D：状态字符串用 snake_case
- 拒绝理由：与 spec/flows/01 §1 Rust enum 命名（PascalCase）不一致；per P1-M 修复 2026-08-27 统一 PascalCase

## 4. 后果与影响

### 4.1 正面

- 任意 Agent 崩溃都可被任意其他 Agent 接续
- 跨厂商 Handoff（Claude Code 崩溃 → Codex 接手）
- 失败原因持久化，跨 Agent 经验累积
- Lease TTL 可调，Project 级配置

### 4.2 负面 / 成本

- 11 字段 Resume JSON schema 需稳定化
- Heartbeat 30s 间隔 = 网络往返
- Context Snapshot 大小受限于 `agent-api/v1` schema
- 跨 Agent 状态共享需要 Audit 完整记录

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| Lease 过期但 Agent 实际未死 | 中 | 中 | Agent 退出前主动释放 lease；IDE Session 状态辅助判断 |
| Resume payload 过大 | 中 | 中 | `agent-api/v1#Resume` 11 字段 + `depth=normal` < 20K tokens |
| 不同 Agent 对 PascalCase 状态解释不一致 | 中 | 中 | 状态机集中在 spec/flows/01 单一来源 |

## 5. 与其他 ADR 的关系

- **依赖**：[ADR-0026 STAR AI Compat](0026-star-ai-compat.md) — 5 通道都需要 Heartbeat / Resume
- **依赖**：[ADR-0027 STAR IDE Gateway](0027-star-ide-gateway.md) — IDE Session 状态辅助 Lease 决策
- **依赖**：[ADR-0029 Universal Submit](0029-universal-submit.md) — Resume 后 Submit 可接着上次进度
- **被依赖**：[ADR-0031 Context Graph](0031-context-graph.md) — Resume 协议从 Context Graph 拉 `relevant_context`

## 6. 签字栏

| # | 角色 | 姓名 | 签字日 | 结论/条件 |
|---|---|---|---|---|
| 1 | 架构负责人 | Mavis（per DEC-008） | 2026-08-26 | ⏳ 待 Ulysses 拍板 |
| 2 | SRE Lead | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 3 | Platform Engineer | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 4 | 评审主持人 | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |
| 5 | 项目负责人（PM） | ⏳ 待签 | ⏳ 待签 | ⏳ 待签 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版：Lease + Heartbeat + 5 步 Agent Lost + 11 字段 Resume（per P1-O）+ PascalCase 状态（per P1-M） | Phase B 起草（per 2026-08-26 升级 Plan） |
