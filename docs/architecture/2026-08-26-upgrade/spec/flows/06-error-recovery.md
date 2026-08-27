# 31. Error / Recovery Model

> **状态**：🟡 草案 v0.1
> **依赖**：[spec/flows/05-universal-submit.md](05-universal-submit.md) · [arch/06 §1.2](../../arch/06-threat-model-nfr.md)

## 1. 错误必须可操作

不能只返回 `Error 500`。必须返回（per §11 任务原文）：

```json
{
  "error": "WORKTREE_CONFLICT",
  "recoverable": true,
  "suggested_actions": ["inspect_conflict", "request_rebase"],
  "message": "...",
  "trace_id": "..."
}
```

## 2. Agent / IDE 需要的能力

- 发生了什么（error code + message）
- 是否可恢复（recoverable）
- 是否允许重试
- 推荐下一步（suggested_actions）
- 是否需要 Human Intervention
- 是否需要刷新 Context
- 是否需要重新获取 Workspace
- 是否需要切换 Worktree

## 3. 错误码分类

| 类别 | 示例 |
|---|---|
| `VALIDATION_*` | `VALIDATION_FAILED` / `VALIDATION_TIMEOUT` |
| `WORKTREE_*` | `WORKTREE_CONFLICT` / `WORKTREE_NOT_FOUND` |
| `PERMISSION_*` | `PERMISSION_DENIED` / `PERMISSION_REQUIRED` |
| `CONTEXT_*` | `CONTEXT_TOO_LARGE` / `CONTEXT_NOT_FOUND` |
| `NETWORK_*` | `NETWORK_TIMEOUT` / `NETWORK_UNREACHABLE` |
| `PROVIDER_*` | `PROVIDER_AUTH_FAILED` / `PROVIDER_RATE_LIMITED` |
| `AGENT_*` | `AGENT_LOST` / `AGENT_LEASE_EXPIRED` |
| `POLICY_*` | `POLICY_DENIED` / `POLICY_VIOLATION` |
| `INTERNAL_*` | `INTERNAL_ERROR` / `INTERNAL_PANIC` |

## 4. 实施位置

- `crates/star-error/` — Error types + machine-readable
- `crates/star-cli/src/output.rs` — 错误输出

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
