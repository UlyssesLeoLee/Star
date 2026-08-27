# ADR-0029: Universal Submit 协议

> **状态**：🟡 Draft v0.1
> **日期**：2026-08-26
> **制定者**：架构师（Mavis 接手 agent per DEC-008）— per 2026-08-26 08:40 JST 代签新规则
> **签批**：⏳ 待签（per §6 签字栏）
> **父文档**：[STAR × GitGit AI/IDE 零厂商适配架构升级 Plan](../../docs/plan/2026-08-26-upgrade-plan.md)（待归档）
> **依赖**：[ADR-0026 STAR AI Compat](0026-star-ai-compat.md) · [spec/flows/01-agent-task-lifecycle.md](../architecture/2026-08-26-upgrade/spec/flows/01-agent-task-lifecycle.md) · [spec/agent-api/01-schema.md §3.15 Error](../architecture/2026-08-26-upgrade/spec/agent-api/01-schema.md)
> **关联**：[flows/05 Universal Submit](../architecture/2026-08-26-upgrade/spec/flows/05-universal-submit.md)

---

## 1. 背景与问题

Agent / IDE 在 STAR 上完成一个 Issue 需要手动跑十几步：

- 检查 task / workspace / worktree 状态
- 跑 diff / validation / policy check
- 跑 commit / push
- 创建 / 更新 MR
- 关联 Issue
- 回写 Agent 状态

**没有 Agent / IDE 应该被要求知道这 12 步细节**（per spec/flows/05 §1 目标）。

需要一条 **Universal Submit 协议** 屏蔽全部细节，让 Agent / IDE 只调一次 `star submit`（或 MCP `submit` tool）即可。

## 2. 决策

**采用 12 步 Universal Submit 协议 + 统一错误模型 `agent-api/v1#Error`。**

### 2.1 12 步流程（per flows/05 §2 + P1-B 修复 2026-08-27）

```
star submit
  ↓
1. 检查 Task
  ↓
2. 检查 Workspace
  ↓
3. 检查 Worktree
  ↓
4. 检查 Diff                # 也可独立调用 star diff
  ↓
5. 执行 Required Validation
  ↓
6. 检查 Policy               # 也可独立调用 star policy check
  ↓
7. Commit / 确认 Commit      # 也可独立调用 star commit
  ↓
8. Push                      # 也可独立调用 star push
  ↓
9. 创建 / 更新 MR
  ↓
10. 关联 Issue               # 也可独立调用 star mr link
  ↓
11. 回写 Agent 状态
  ↓
12. 回写 IDE Session 状态
```

> 步骤 4 / 6 / 7 / 8 / 10 对应 5 个新加 CLI 命令（`star diff` / `star policy check` / `star commit` / `star push` / `star mr link`，per P1-H 修复 2026-08-27）。这些命令在 `star submit` 内部自动调用，**也**作为独立命令对外暴露（per flows/05 §2）。

### 2.2 统一错误模型（per flows/05 §3 + P1-G 修复 2026-08-27）

失败时返回 Machine-readable Recovery Action（schema = `agent-api/v1#Error` §3.15 6 字段，CLI / MCP / REST / Submit **全部**统一引用）：

```json
{
  "error": "VALIDATION_FAILED",
  "recoverable": true,
  "suggested_actions": ["star test run", "fix failing tests", "star submit"],
  "message": "Required validation failed: 1 test failed",
  "trace_id": "...",
  "details": {
    "failed_tests": ["test_login_with_expired_token"],
    "test_output": "..."
  }
}
```

6 字段：`error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details`。

### 2.3 关键架构约束

- 12 步必须按顺序执行；任一步失败立即停止 + 返回 `agent-api/v1#Error`
- 步骤 4 / 6 / 7 / 8 / 10 同时是独立 CLI 命令（per P1-H），Agent 可单独调用
- 步骤 12（回写 IDE Session 状态）必须执行（per P1-B 修复 2026-08-27 统一 12 步）
- Submit 必须可重入：失败修复后再次 `star submit` 接着上次进度
- Submit 12 步内部调用 `star commit` / `star push` / `star mr link` 等命令，**不**绕过 Policy / Audit
- MCP server 的 `submit` tool 暴露同 12 步（per P1-F 修复 2026-08-27，per spec/mcp/01 §2）

### 2.4 实施位置

- `crates/star-cli/src/commands/submit.rs` — submit 子命令
- `crates/star-application/src/submit.rs` — Application service
- `crates/star-cli/src/commands/diff.rs` / `policy.rs` / `commit.rs` / `push.rs` / `mr_link.rs` — 5 个新加独立命令（per P1-H 修复 2026-08-27）

## 3. 备选方案与拒绝理由

### 备选 A：要求 Agent / IDE 手动跑 12 步
- 拒绝理由：违反"零学习成本"原则；不同 Agent 需要各自实现一遍

### 备选 B：自定义 Submit 二进制协议（不走 CLI / MCP）
- 拒绝理由：违反 ADR-0021 Zero Vendor Cooperation；Vendor 不能接受 STAR 私有协议

### 备选 C：11 步（不含"回写 IDE Session 状态"）
- 拒绝理由：IDE Session 状态回写是业务完整闭环，per P1-B 修复 2026-08-27 统一到 12 步

## 4. 后果与影响

### 4.1 正面

- Agent / IDE 一次 `star submit`（或 MCP `submit` tool）完成全部流程
- 5 个内部步骤同时是独立 CLI 命令（per P1-H），支持 Agent 细粒度控制
- 统一错误模型 6 字段（per P1-G），CLI / MCP / REST / Submit 4 处共用同一 schema
- Submit 失败可重入（接着上次进度）

### 4.2 负面 / 成本

- 12 步必须按顺序执行；事务一致性需精细处理
- 5 个独立命令的 schema 需同步稳定化
- 回写 IDE Session 状态是 12 步最后一步，IDE 端需配合

### 4.3 风险

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 12 步任一步的非原子性 | 中 | 中 | Submit 可重入 + 状态机显式持久化 |
| 错误模型 6 字段破坏向后兼容 | 低 | 高 | `agent-api/v1` 至少 12 个月稳定 |
| MCP `submit` tool 跟 CLI `submit` 行为不一致 | 中 | 高 | 共用同一 Application service |

## 5. 与其他 ADR 的关系

- **依赖**：[ADR-0026 STAR AI Compat](0026-star-ai-compat.md) — 5 通道都需要暴露 `submit`
- **依赖**：[ADR-0027 STAR IDE Gateway](0027-star-ide-gateway.md) — 步骤 12 回写 IDE Session 状态
- **依赖**：[ADR-0028 GitGit Compat](0028-gitgit-compat.md) — 步骤 8 Push 走 GitGit
- **被依赖**：[ADR-0030 Agent Lease/Heartbeat/Resume](0030-agent-lease-heartbeat-resume.md) — Resume 协议需要 Submit 状态信息

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
| v0.1 | 2026-08-26 | 架构师（Mavis 接手 agent per DEC-008） | 初版：12 步流程（per P1-B 统一）+ 6 字段错误模型（per P1-G 统一） | Phase B 起草（per 2026-08-26 升级 Plan） |
