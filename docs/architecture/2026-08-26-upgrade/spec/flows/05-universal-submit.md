# 30. Universal Submit Protocol

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/cli/01-cli-spec.md](../cli/01-cli-spec.md) · [spec/flows/01-agent-task-lifecycle.md](01-agent-task-lifecycle.md)

## 1. 目标

Agent / IDE 不需要知道 STAR 内部几十个流程。`star submit` 自动完成所有检查。

## 2. 11 步流程（per §33 任务原文）

```
star submit
  ↓
1. 检查 Task
  ↓
2. 检查 Workspace
  ↓
3. 检查 Worktree
  ↓
4. 检查 Diff
  ↓
5. 执行 Required Validation
  ↓
6. 检查 Policy
  ↓
7. Commit / 确认 Commit
  ↓
8. Push
  ↓
9. 创建 / 更新 MR
  ↓
10. 关联 Issue
  ↓
11. 回写 Agent 状态
  ↓
    12. 回写 IDE Session 状态
```

## 3. 错误恢复

如果失败，返回 Machine-readable Recovery Action（per [arch/06 §1.2 错误模型](../../arch/06-threat-model-nfr.md)）：

```json
{
  "error": "VALIDATION_FAILED",
  "recoverable": true,
  "suggested_actions": ["star test run", "fix failing tests", "star submit"],
  "details": {
    "failed_tests": ["test_login_with_expired_token"],
    "test_output": "..."
  }
}
```

## 4. 实施位置

- `crates/star-cli/src/commands/submit.rs` — submit 子命令
- `crates/star-application/src/submit.rs` — Application service

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
