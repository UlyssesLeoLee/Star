# 21. Worktree Protocol

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/05 GitGit Compat Arch](../../arch/05-gitgit-compat-arch.md) · [spec/resources/01-workspace-protocol.md](01-workspace-protocol.md)

## 1. 物理层 in GitGit，逻辑层 in STAR

- GitGit 负责：worktree create/delete/status、文件系统映射、branch 绑定、生命周期
- STAR 负责：worktree ↔ Issue / Task / Agent / IDE Session 绑定、权限、策略、自动清理、恢复、审计

## 2. 协议

```bash
# Agent 视角
git worktree add ../wt-STAR-1024 -b feature/STAR-1024  # 物理层
star worktree create STAR-1024                          # 逻辑层 (创建 binding)
star worktree enter STAR-1024                           # cd wrapper
star worktree status --json                             # 物理层 + 逻辑层
```

## 3. Worktree Schema（合并物理 + 逻辑）

```json
{
  "id": "wt-STAR-1024",
  "path": "/repos/owner/repo/wt-STAR-1024",
  "branch": "feature/STAR-1024",
  "head_commit": "abc123...",
  "dirty": true,
  "workspace_id": "ws-STAR-1024",
  "issue_id": "STAR-1024",
  "agent_session_id": "agent-abc",
  "ide_session_id": "ide-xyz",
  "created_at": "...",
  "last_active_at": "..."
}
```

## 4. 标准流程

```text
Issue
   ↓
STAR 创建 Workspace
   ↓
STAR 请求 GitGit 创建 Worktree
   ↓
Agent / IDE 进入 Worktree
   ↓
修改代码
   ↓
Test
   ↓
Commit (per Git protocol)
   ↓
MR (per Version Control Provider)
   ↓
Review
   ↓
Merge
   ↓
STAR 更新任务状态
   ↓
GitGit 清理或保留 Worktree
```

## 5. 关键约束

- Worktree 创建必须用标准 `git worktree add`（不发明新命令）
- Worktree ID 命名约定：`wt-<issue_id>`
- Workspace / Agent / IDE 绑定必须在 STAR 维护（GitGit 不感知）
- 物理删除 worktree 后，STAR 端状态需标记为"orphan"

## 6. 实施位置

- `crates/star-workspace/src/worktree.rs` — Worktree 逻辑层
- GitGit 端：沿用现有 `git worktree` 命令（per c89f858 之前 main）

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
