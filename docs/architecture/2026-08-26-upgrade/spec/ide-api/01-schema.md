# 13. IDE CLI JSON Schema

> **状态**：🟡 草案 v0.1
> **依赖**：[arch/04 STAR IDE Gateway](../../arch/04-star-ide-gateway-arch.md) · [spec/agent-api/01-schema.md](../agent-api/01-schema.md)

## 1. Versioning

- Schema version: `ide-api/v1`
- 跟 `agent-api/v1` 平行，独立演进

## 2. 核心 Schemas

### 2.1 Workspace

```json
{
  "id": "ws-abc",
  "name": "main-workspace",
  "repository": {"id": "repo-1", "provider": "gitgit", "url": "..."},
  "worktree_id": "wt-STAR-1024",
  "open_files": [
    {"path": "src/auth.rs", "cursor": {"line": 42, "col": 10}, "dirty": false}
  ],
  "active_symbol": {"kind": "function", "name": "login", "file": "src/auth.rs"},
  "diagnostics": [
    {"file": "src/auth.rs", "line": 42, "severity": "warning", "message": "..."}
  ],
  "ide_client": "vscode",
  "ide_version": "1.95.0"
}
```

### 2.2 WorkspaceSwitch

```json
{
  "workspace_id": "ws-abc",
  "worktree_path": "/repos/owner/repo/wt-STAR-1024",
  "shell_command": "cd /repos/owner/repo/wt-STAR-1024"
}
```

### 2.3 CodeNavigation

```json
{
  "query": "AuthService::login",
  "results": [
    {
      "kind": "function",
      "name": "AuthService::login",
      "file": "src/auth/service.rs",
      "line": 42,
      "snippet": "pub async fn login(...) -> Result<...> { ... }"
    }
  ]
}
```

## 3. 落盘位置

`crates/star-cli/src/schemas/ide-api-v1/`：
- `Workspace.json`
- `WorkspaceSwitch.json`
- `CodeNavigation.json`
- `Symbol.json`
- `Diagnostic.json`
- `Reference.json`
- `ContextView.json`
- ...

## 4. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。
