# 13. IDE CLI JSON Schema

> **状态**：🟡 草案 v0.2
> **依赖**：[arch/04 STAR IDE Gateway](../../arch/04-star-ide-gateway-arch.md) · [spec/agent-api/01-schema.md](../agent-api/01-schema.md)

## 1. Versioning

- Schema version: `ide-api/v1`
- 跟 `agent-api/v1` 平行，独立演进
- **IDE 视角边界**（per P1-C 修复 2026-08-27 + ADR-0024 "IDE Session 独立"）：ide-api/v1 **只**描述 IDE 内部状态（open_files / diagnostics / active_symbol / ide_client / ide_version），**不**承载 Agent 业务字段；Agent 视角的逻辑抽象走 [agent-api/v1#WorkspaceSummary](../agent-api/01-schema.md#316-workspacesummary-per-p1-c-修复-2026-08-27)

## 2. 核心 Schemas

### 2.1 WorkspaceState（per P1-C 修复 2026-08-27，原 `Workspace` 重命名）

> IDE 视角的**状态视图**：含 IDE 内部状态（open_files / diagnostics / active_symbol / ide_client / ide_version）。**守 ADR-0024 边界** — 不含 Agent 业务字段。
>
> Agent 命令 `star workspace current` 引用 **不**是本 schema，而是 `agent-api/v1#WorkspaceSummary`（per P1-C 修复）；IDE 命令 `ide workspace state` 引用本 schema。

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

> 字段（与原 `Workspace` 完全一致，仅重命名）：
> - `id` / `name` / `repository` / `worktree_id` — 基础标识
> - `open_files` — IDE 视角的打开文件（含 cursor + dirty）
> - `active_symbol` — 当前光标位置的符号
> - `diagnostics` — IDE 报错 / 警告
> - `ide_client` / `ide_version` — IDE 客户端身份
>
> 改名动机：原 `Workspace` 名字让 CLI / Agent 误以为是 Agent 视角的实体；`WorkspaceState` 明确是"IDE 视角的状态快照"，避免跨层数据泄漏。

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

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：§2.1 Workspace + §2.2 WorkspaceSwitch + §2.3 CodeNavigation | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-C：§2.1 `Workspace` 重命名为 `WorkspaceState`（明确 IDE 视角状态视图，避免 Agent 命令误引）+ §1 加 IDE 视角边界声明 | 8 子代理 INTERFACE-REVIEW-A 🔴 #5 + P1-BLOCKERS-SUMMARY v0.2 |
