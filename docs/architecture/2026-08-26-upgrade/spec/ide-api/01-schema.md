# 13. IDE CLI JSON Schema

> **状态**：🟡 草案 v0.2
> **依赖**：[arch/04 STAR IDE Gateway](../../arch/04-star-ide-gateway-arch.md) · [spec/agent-api/01-schema.md](../agent-api/01-schema.md)

## 1. Versioning

- Schema version: `ide-api/v1`
- 跟 `agent-api/v1` 平行，独立演进
- **IDE 视角边界**（per P1-C 修复 2026-08-27 + ADR-0024 "IDE Session 独立"）：ide-api/v1 **只**描述 IDE 内部状态（open_files / diagnostics / active_symbol / ide_client / ide_version），**不**承载 Agent 业务字段；Agent 视角的逻辑抽象走 [agent-api/v1#WorkspaceSummary](../agent-api/01-schema.md#315-workspacesummary)
- **`info.version` vs `schema_version` 双版本字段**（per F-12 修复 2026-08-27，跟 [spec/agent-api/01 §1](../agent-api/01-schema.md) 一致）：
  - `info.version`（OpenAPI 顶层）— OpenAPI 文档版本，SemVer 字符串
  - `schema_version`（Capabilities 等 schema 内嵌字段）— schema 业务版本，`ide-api/vN` 字符串
  - 二者同步演化：`ide-api/v1.x` ↔ `info.version: "1.x.0"`；breaking 走 `v2` ↔ `info.version: "2.0.0"`

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟡 #11 (F-12) — 加 `info.version` vs `schema_version` 双版本字段关系（跟 agent-api §1 对齐）

## 2. 核心 Schemas

### 2.1 WorkspaceState（per F-05 修复 2026-08-27，原 `Workspace` 重命名）

> **IDE 视角的状态视图**（per F-05 修复 2026-08-27 — INTERFACE-REVIEW-A 🔴 #5）：**含** IDE 内部状态（`open_files` / `diagnostics` / `active_symbol` / `ide_client` / `ide_version`）。**守 ADR-0024 边界** — 不含 Agent 业务字段。
>
> Agent 命令 `star workspace current` 引用 **不**是本 schema，而是 [agent-api/v1#WorkspaceSummary §3.15](../agent-api/01-schema.md#315-workspacesummary)（per F-05 修复）；IDE 命令 `ide workspace state` 引用本 schema。

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

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🔴 #5 (F-05) — 强化 WorkspaceState = IDE 视角（含 open_files/diagnostics），跟 agent-api WorkspaceSummary 严格区分

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

### 2.4 IDE 视角下的 Worktree 引用

> **F-24 跟 agent-api §3.2 严格对齐**（per F-24 修复 2026-08-27）：ide-api 引用 Worktree 时，**禁止**同时含 `agent_session_id` + `ide_session_id`；IDE 视角的 Worktree 引用只能含 `ide_session_id`（通过 worktree_binding 数组的 `ide_sessions` 子数组）。

- `worktree_id` (string — 引用 [agent-api/v1#Worktree §3.2](../agent-api/01-schema.md#32-worktree))
- `ide_session_id` (string — 当前 IDE session)
- `view_state` (enum: `active` | `background` | `closed`)

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #24 (F-24) — ide-api 引用 Worktree 时禁同时含 agent_session_id

## 3. 落盘位置

> **落盘路径迁移**（per F-29 修复 2026-08-27 — INTERFACE-REVIEW-A 🟢 #29）：从 `crates/star-cli/src/schemas/ide-api-v1/`（v0.2）迁移到 **`crates/star-ide-gateway/src/schemas/ide-api-v1/`**（v0.2 fix），per [arch/04 STAR IDE Gateway](../../arch/04-star-ide-gateway-arch.md) IDE Gateway crate 边界。

`crates/star-ide-gateway/src/schemas/ide-api-v1/`：
- `WorkspaceState.json` (§2.1 — IDE 视角，含 open_files / diagnostics / ide_client)
- `WorkspaceSwitch.json` (§2.2)
- `CodeNavigation.json` (§2.3)
- `WorktreeView.json` (§2.4 — IDE 视角 Worktree 引用，禁同时含 agent_session_id)
- `Symbol.json` (子结构)
- `Diagnostic.json` (子结构)
- `Reference.json` (子结构)
- `ContextView.json` (子结构)
- ...

> §2 与 §3 交叉引用：§2.X 是已定义 schema 的字段说明；§3 是落盘文件清单 + 路径。两者一一对应。

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #29 (F-24/F-29) — 落盘路径 star-cli → star-ide-gateway；加 §2.4 Worktree 引用规则

## 4. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：§2.1 Workspace + §2.2 WorkspaceSwitch + §2.3 CodeNavigation | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-C：§2.1 `Workspace` 重命名为 `WorkspaceState`（明确 IDE 视角状态视图，避免 Agent 命令误引）+ §1 加 IDE 视角边界声明 | 8 子代理 INTERFACE-REVIEW-A 🔴 #5 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.2 fix | 2026-08-27 | Mavis（接手 agent per DEC-008 — 子代理 fix-api-spec-blockers）| F-05：§2.1 WorkspaceState 强化"含 open_files/diagnostics"声明 · F-12：§1 加 `info.version` vs `schema_version` 双版本字段关系（跟 agent-api §1 对齐）· F-24：§2.4 新增"IDE 视角 Worktree 引用"规则（禁同时含 agent_session_id） · F-29：§3 落盘路径 `crates/star-cli/` → `crates/star-ide-gateway/src/schemas/ide-api-v1/` | INTERFACE-REVIEW-A 🔴 #5 + 🟡 #11 + 🟢 #24/#29 |
