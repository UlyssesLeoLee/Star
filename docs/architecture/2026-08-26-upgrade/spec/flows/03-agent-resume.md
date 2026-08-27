# 28. Agent Resume Protocol

> **状态**：🟡 草案 v0.2
> **依赖**：[spec/flows/02-agent-lease-heartbeat.md](02-agent-lease-heartbeat.md) · [spec/agent-api/01-schema.md §3.17 Resume](../agent-api/01-schema.md) · [spec/flows/01-agent-task-lifecycle.md](01-agent-task-lifecycle.md)

## 1. 场景

Agent A 执行一半崩溃。Agent B / IDE Session B 必须能继续。

## 2. 协议

```bash
star task resume STAR-1024
star workspace resume STAR-1024
```

返回（schema = [`agent-api/v1#Resume`](../agent-api/01-schema.md#317-resumeper-p1-o-修复-2026-08-27)，11 字段 per P1-O 修复 2026-08-27）：

```json
{
  "current_state": "Implementing",
  "workspace": {"id": "ws-abc", "name": "main-workspace", "...": "..."},
  "worktree": {
    "id": "wt-STAR-1024",
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
  "modified_files": ["src/auth.rs", "src/session.rs"],
  "open_diagnostics": [
    {"file": "src/auth.rs", "line": 42, "severity": "warning", "message": "..."}
  ],
  "test_results": {"passed": 5, "failed": 1, "skipped": 0, "failed_tests": []},
  "failed_attempts": [
    {"step": "Add timeout config", "error": "compile error: ..."}
  ],
  "relevant_context": {"issue_id": "STAR-1024", "related_code": [...], "...": "..."},
  "remaining_work": [
    "Fix compile error in src/auth.rs",
    "Re-run tests"
  ],
  "last_modified": "2026-08-26T19:30:00+09:00"
}
```

> 11 字段权威定义见 [agent-api/v1 §3.17 Resume](../agent-api/01-schema.md)：
> 1. `current_state` (string, PascalCase — per P1-M 修复 2026-08-27，例如 `"Implementing"`，**与 [spec/flows/01 §1](01-agent-task-lifecycle.md) Rust enum 命名一致**)
> 2. `workspace` (WorkspaceSummary)
> 3. `worktree` (Worktree + `modified_files: string[]`)
> 4. `previous_plan` (string[])
> 5. `modified_files` (string[])
> 6. `open_diagnostics` (Diagnostic[])
> 7. `test_results` (TestResult)
> 8. `failed_attempts` (FailedAttempt[])
> 9. `relevant_context` (Context)
> 10. `remaining_work` (string[])
> 11. `last_modified` (timestamp)

> v0.3 fix: 2026-08-27 per B-19 (Resume 11 字段 cross-ref §3.17) / B-23 (state 1:1 对齐 flows/01)

## 3. 关键约束

- 实现真正的 **Vendor-independent Agent Handoff**
- 不同厂商的 Agent 都能 Resume 同一任务
- 必须包含"前一个 Agent 为什么失败"
- 状态字符串统一 PascalCase（与 [spec/flows/01 §1](01-agent-task-lifecycle.md) Rust enum 命名一致，per P1-M 修复 2026-08-27）

> v0.2 fix: 2026-08-27 per B-01 (PascalCase 引用) / B-19 (Resume 11 字段 §3.17 cross-ref)

## 4. 实施位置

- `crates/star-agent/src/resume.rs` — Resume 协议
- `crates/star-agent/src/snapshot.rs` — Workspace / Context snapshot

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：11 字段 Resume JSON 协议 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-M：§2/§3 状态字符串统一 PascalCase 引用（`"Implementing"`，与 flows/01 §1 Rust enum 一致） · P1-O：§2 完整引用 [`agent-api/v1#Resume`](../agent-api/01-schema.md) §3.17（11 字段权威定义），加 `last_modified` 字段 + 补全 `open_diagnostics` / `test_results` / `relevant_context` 等之前未明确的字段 | 8 子代理 INTERFACE-REVIEW-B 🔴 B-01/B-19 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.3 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent（per 2026-08-27 07:16 JST 代签规则反转）| 🔴 B-19 再次校准：明确 §2 11 字段权威定义位置 [`agent-api/v1` §3.17 Resume](../agent-api/01-schema.md#317-resumeper-p1-o-修复-2026-08-27)（W4 子代理定义），与 §3.1 Task.status enum 联动 · 🟡 B-23：§3 补 PascalCase 与 [flows/01 §1](01-agent-task-lifecycle.md) Rust enum 命名 1:1 对齐声明 | worker 子代理修 INTERFACE-REVIEW-B 8 子代理报告 follow-up |

> v0.3 fix: 2026-08-27 per B-19 / B-23

> **已知缺口（缺标比错标安全）**：B-19 / B-23 任务摘要列的 11 字段（`id, agent_id, state, last_heartbeat_at, lease_expires_at, current_state, current_step, retry_count, artifacts, checkpoint, recovery_hint`）与 [`agent-api/v1` §3.17 Resume](../agent-api/01-schema.md#317-resumeper-p1-o-修复-2026-08-27) 实际 11 字段（`current_state, workspace, worktree, previous_plan, modified_files, open_diagnostics, test_results, failed_attempts, relevant_context, remaining_work, last_modified`）**不对齐**。本 spec 以 `agent-api/v1` §3.17 为准（权威 schema 来自 W4 子代理 §3.17 定义，非任务摘要）。任务摘要与 schema 的差异需 Ulysses 终审时确认是调整 spec 还是调整 schema。
