# 20. Workspace Protocol

> **状态**：🟡 草案 v0.1
> **依赖**：[ADR-0022 IDE Placement](../../adr/0022-ide-placement.md)

## 1. Workspace 归 STAR

Workspace 是研发工作空间，包含：
- Project / Repository / Worktree / Task / Agent / IDE Session / Environment / Context / Permission / Validation / MR

**Workspace 必须放 STAR（per §24 任务原文）**。

## 2. Workspace 生命周期

```text
Issue
   ↓ (star workspace create STAR-1024)
Workspace (active)
   ↓ (star worktree create STAR-1024)
Worktree (linked to workspace)
   ↓ (Agent / IDE 进入)
Modifications
   ↓ (star test affected)
Validation
   ↓ (star submit)
MR created
   ↓ (merge)
Workspace (closed) | (retained for rebase)
```

## 3. Workspace Schema（MVP）

```json
{
  "id": "ws-STAR-1024",
  "name": "STAR-1024-workspace",
  "issue_id": "STAR-1024",
  "user_id": "u-1",
  "agent_session_id": "agent-abc",
  "ide_session_id": "ide-xyz",
  "repository": {"id": "repo-1", "provider": "gitgit"},
  "worktree_id": "wt-STAR-1024",
  "permission_level": "L4_CREATE_MR",
  "context": {"depth": "minimal", "snapshot_id": "ctx-..."},
  "created_at": "...",
  "expires_at": "..."
}
```

## 4. CLI / MCP

```bash
star workspace create STAR-1024 --permission L4_CREATE_MR
star workspace list
star workspace current
star workspace close
```

```
MCP tool: get_workspace
MCP tool: create_workspace
```

## 5. 实施位置

- `crates/star-workspace/` — Workspace service
- `crates/star-workspace/src/lifecycle.rs` — create/close/expire
- `crates/star-workspace/src/permission.rs` — Permission Level 校验

## 6. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
