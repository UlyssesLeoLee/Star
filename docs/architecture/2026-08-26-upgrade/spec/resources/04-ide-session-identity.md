# 23. IDE Session Identity

> **状态**：🟡 草案 v0.1
> **依赖**：[ADR-0024 IDE Session Identity](../../adr/0024-ide-session-identity.md) · [spec/resources/03-agent-identity.md](03-agent-identity.md)

## 1. IDE Session 必为 STAR 一等对象

```text
IDE Session
├── IDE Identity
├── User
├── Workspace
├── Repository
├── Worktree
├── Open Files
├── Active Symbols
├── Diagnostics
├── Selection
├── Terminal
├── Agent Sessions (link)
└── Audit
```

## 2. IDE Session Schema

```json
{
  "id": "ide-xyz",
  "kind": "vscode",                  // vscode | cursor | jetbrains | vim | web | unknown
  "version": "1.95.0",
  "user_id": "u-1",
  "workspace_id": "ws-STAR-1024",
  "repository_id": "repo-1",
  "worktree_id": "wt-STAR-1024",
  "open_files": [...],
  "active_symbol": {...},
  "diagnostics": [...],
  "selection": {...},
  "terminal_id": "term-1",
  "agent_sessions": ["agent-abc"],
  "audit_id": "audit-..."
}
```

## 3. GitGit 不感知 IDE Session

GitGit 只看到：

```json
{
  "id": "repo-1",
  "path": "...",
  "worktree_path": "...",
  "branch": "...",
  "head_commit": "...",
  "dirty": false
}
```

## 4. 实施位置

- `crates/star-ide/src/session.rs` — IDE Session 模型
- `crates/star-ide/src/file.rs` — OpenFile 模型
- LSP proxy（Phase 2+） — 见 [arch/04 STAR IDE Gateway](../../arch/04-star-ide-gateway-arch.md) §6

## 5. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
