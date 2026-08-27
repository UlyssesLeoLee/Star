# 30. Universal Submit Protocol

> **状态**：🟡 草案 v0.2
> **依赖**：[spec/cli/01-cli-spec.md](../cli/01-cli-spec.md) · [spec/flows/01-agent-task-lifecycle.md](01-agent-task-lifecycle.md) · [spec/agent-api/01-schema.md §3.15 Error](../agent-api/01-schema.md)

## 1. 目标

Agent / IDE 不需要知道 STAR 内部几十个流程。`star submit` 自动完成所有检查。

## 2. 12 步流程（per P1-B 修复 2026-08-27 统一 12 步）

> 原 §2 文字写"11 步流程"但代码块列了 12 个步骤（含"12. 回写 IDE Session 状态"作为 comment），任务摘要和 arch/03 + acceptance/04 引"11 步"，**spec 内部自相矛盾**。修法：文字 + 列表统一到 **12 步**（保留 IDE Session 回写步作为正式步骤，因为 IDE Session 状态回写是业务完整闭环，per P1-B 修复 2026-08-27）。对应 9+5 状态机（per [spec/flows/01 §1](01-agent-task-lifecycle.md) PascalCase 修复）最后两步：第 11 步"回写 Agent 状态" → `Completed`；第 12 步"回写 IDE Session 状态" → IDE Session state machine transition（per [spec/resources/04](../resources/04-ide-session-identity.md)）。

> **步骤数显式区分（per B-18 修复 2026-08-27）**：**11 + 1 comment step = 12 total**。
> - 步骤 1-11 = 业务主流程（per [agent-api/v1 §3.3 SubmitResult](../agent-api/01-schema.md#33-submitresult) 触发顺序）
> - 步骤 12 = comment 形式提示（IDE Session 状态回写，作为正式步骤保留 — 业务完整闭环）
> - 任务摘要写"11 步"实际是步骤 1-11；arch/03 + acceptance/04 引"11 步"也是同一口径
> - spec §2 文字 + 列表统一到 12 步（per P1-B 修复）

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

> 步骤 4 / 6 / 7 / 8 / 10 对应 5 个新加 CLI 命令（`star diff` / `star policy check` / `star commit` / `star push` / `star mr link`，per P1-H 修复 2026-08-27 详见 [spec/cli/01-cli-spec.md §2.2](../cli/01-cli-spec.md)）。这些命令在 `star submit` 内部自动调用，**也**作为独立命令对外暴露。

## 3. 错误恢复（per P1-G 修复 2026-08-27）

如果失败，返回 Machine-readable Recovery Action（schema = [`agent-api/v1#Error`](../agent-api/01-schema.md) §3.15 6 字段，CLI / MCP / REST / Submit **全部**统一引用）：

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

> 字段：`error` / `recoverable` / `suggested_actions` / **`message`** / **`trace_id`** / `details`（6 字段）— 原 §3 4 字段基础上 + `message` + `trace_id` 与 [agent-api/v1#Error §3.15](../agent-api/01-schema.md) 完全对齐。CLI / MCP / REST / Submit 4 处共用同一 schema，per P1-G 修复 2026-08-27。

## 4. 实施位置

- `crates/star-cli/src/commands/submit.rs` — submit 子命令
- `crates/star-application/src/submit.rs` — Application service
- `crates/star-cli/src/commands/diff.rs` / `policy.rs` / `commit.rs` / `push.rs` / `mr_link.rs` — 5 个新加独立命令（per P1-H 修复 2026-08-27）

> **建议内部模块拆解（per B-21 修复 2026-08-27）**：12 步共用一个 `submit.rs` 是合理设计（一个 submit 主流程串 12 步），但**单文件过长**会牺牲可读性。建议按业务域拆为 `handlers/` 子模块（Phase D 实现时校对）：
>
> | 步骤 | 业务域 | 建议模块 |
> |---|---|---|
> | 1. 检查 Task | task | `crates/star-application/src/submit/handlers/task_check.rs` |
> | 2. 检查 Workspace | workspace | `crates/star-application/src/submit/handlers/workspace_check.rs` |
> | 3. 检查 Worktree | worktree | `crates/star-application/src/submit/handlers/worktree_check.rs` |
> | 4. 检查 Diff | diff | `crates/star-application/src/submit/handlers/diff_check.rs` |
> | 5. 执行 Required Validation | validation | `crates/star-application/src/submit/handlers/validation.rs` |
> | 6. 检查 Policy | policy | `crates/star-application/src/submit/handlers/policy.rs` |
> | 7. Commit / 确认 Commit | commit | `crates/star-application/src/submit/handlers/commit.rs` |
> | 8. Push | push | `crates/star-application/src/submit/handlers/push.rs` |
> | 9. 创建 / 更新 MR | mr | `crates/star-application/src/submit/handlers/mr.rs` |
> | 10. 关联 Issue | issue | `crates/star-application/src/submit/handlers/issue_link.rs` |
> | 11. 回写 Agent 状态 | agent | `crates/star-application/src/submit/handlers/agent_state.rs` |
> | 12. 回写 IDE Session 状态 | ide | `crates/star-application/src/submit/handlers/ide_state.rs` |
>
> **CLI 入口**仍是单一 `crates/star-cli/src/commands/submit.rs`，通过 `submit/handlers/*.rs` 调度（per B-21 修复 2026-08-27）。

> v0.2 fix: 2026-08-27 per B-21 (12 步 handlers/ 拆解建议)

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：文字"11 步"+ 列表 12 步（自相矛盾）+ 4 字段错误模型 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-B：§2 文字 + 列表统一到 **12 步**（含"12. 回写 IDE Session 状态"作为正式步骤），附 5 步内部命令也独立暴露标记 · P1-G：§3 错误模型 6 字段（+ `message` + `trace_id`），统一引用 [`agent-api/v1#Error`](../agent-api/01-schema.md) §3.15 | 8 子代理 INTERFACE-REVIEW-A 🔴 #7 + INTERFACE-REVIEW-C P1-6 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.3 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转）| 🟡 B-18：§2 加注 "11 + 1 comment step = 12 total" 显式区分（步骤 1-11 主流程 + 步骤 12 IDE Session 回写 comment 形式）· 🟡 B-21：§4 加 12 步 `handlers/` 子模块拆解建议表（每步业务域 + 建议模块路径） | worker 子代理修 INTERFACE-REVIEW-B 8 子代理报告 follow-up |

> v0.3 fix: 2026-08-27 per B-18 / B-21
