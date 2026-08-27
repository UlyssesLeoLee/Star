# 15. REST / OpenAPI Strategy

> **状态**：🟡 草案 v0.2
> **依赖**：[Protocol Survey §3](../../ecosystem-survey/protocol-survey.md) · [arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md) · [spec/agent-api/01-schema.md §3.14 Error](../agent-api/01-schema.md)

## 1. OpenAPI 版本

- **OpenAPI 3.1**（不是 3.0）— 完整对齐 JSON Schema 2020-12
- 不采用 OpenAPI 3.2（MVP 阶段，3.1 已稳）
- **OpenAPI 3.1 关键字段**（per F-10 修复 2026-08-27 — INTERFACE-REVIEW-A 🟡 #10）：
  - `info.summary` 允许（信息摘要，跟 `info.description` 区分）
  - `webhooks` 字段允许（per OpenAPI 3.1 加入）
  - `nullable: true` → `type: [string, "null"]`（**破坏性迁移**，OpenAPI 3.1 用 JSON Schema 2020-12 风格；v0.2 之前 spec 用 `nullable: true` 必须重写）
- **`info.version` vs `schema_version` 双版本字段**（per F-11 修复 2026-08-27 — INTERFACE-REVIEW-A 🟡 #11，跟 [spec/agent-api/01 §1](../agent-api/01-schema.md) 一致）：
  - `info.version`（OpenAPI 顶层）— OpenAPI 文档版本，SemVer
  - `schema_version`（schema 内嵌字段）— schema 业务版本，`agent-api/vN` / `ide-api/vN`
  - 同步演化：`v1.x` ↔ `info.version: "1.x.0"`；breaking 走 `v2` ↔ `info.version: "2.0.0"`

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟡 #10/#11 (F-10/F-11) — 强化 OpenAPI 3.1 关键字段；`info.version` vs `schema_version` 双版本字段（跟 agent-api §1 对齐）

## 2. Spec 文件位置

- `crates/star-rest/openapi/agent-api/v1.yaml` — Agent API（**斜杠风格**，per 子代理 A 🟡 #12 统一；schema identifier = `agent-api/v1`）
- `crates/star-rest/openapi/ide-api/v1.yaml` — IDE API（schema identifier = `ide-api/v1`）
- `crates/star-rest/openapi/git-provider/v1.yaml` — Git Provider API（per F-23 修复 2026-08-27：**无 endpoint**，仅作 ADR-0023 抽象边界声明 — 见 §8；GitGit 自身 axum HTTP 由 GitGit crate 单独维护，per ADR-0022 §2.2）

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #23 (F-23) — `git-provider/v1.yaml` 显式标"无 endpoint，仅 ADR-0023 引用"

## 3. 与 MCP 共享 Domain API

```text
                 STAR Domain API  (Application Service)
                        │
       ┌────────────────┼────────────────┐
       ↓                ↓                ↓
      CLI              MCP              REST
                        │
                        ↓
                   IDE Gateway
```

**关键约束**：CLI / MCP / REST 不得重复业务逻辑。所有 Adapter 调同一 Application Service。

## 4. 完整 endpoint 表（per F-09 修复 2026-08-27 — 23 个 endpoint）

> 扩 v0.2 的 12 endpoint → **23 endpoint**（3 agent discovery + 17 CLI 17 核心 + 3 IDE），全部**复数风格**（per F-17 修复 2026-08-27 — INTERFACE-REVIEW-A 🟡 #17）。

### 4.1 Agent 端点（20 个 = 3 discovery + 17 跟 CLI 17 核心命令一一对应）

| # | Method + Path | 用途 | CLI 对应 | 4xx / 5xx 响应（per F-19 修复 2026-08-27）|
|---|---|---|---|---|
| 1 | `GET /api/v1/agent/capabilities` | Agent Capability Discovery（v0.2 已有） | `star agent capabilities` | 500 → `Error` (INTERNAL) |
| 2 | `GET /api/v1/agent/permissions` | Agent Permission Discovery（v0.2 已有） | `star agent permissions` | 403 → `Error` (PERMISSION_DENIED) |
| 3 | `GET /api/v1/agent/instructions` | Agent Instructions（v0.2 已有） | `star agent instructions` | 500 → `Error` (INTERNAL) |
| 4 | `GET /api/v1/projects` | 列出项目 | `star project list` | 500 → `Error` (INTERNAL) |
| 5 | `GET /api/v1/issues` | 列出 issue | `star issue list` | 500 → `Error` (INTERNAL) |
| 6 | `GET /api/v1/issues/{id}` | 显示 issue 详情 | `star issue show <id>` | 404 → `Error` (ISSUE_NOT_FOUND) |
| 7 | `POST /api/v1/issues/{id}/claim` | 认领 issue | `star issue claim <id>` | 409 → `Error` (ISSUE_ALREADY_CLAIMED) |
| 8 | `GET /api/v1/tasks/current` | 当前任务 | `star task current` | 404 → `Error` (NO_CURRENT_TASK) |
| 9 | `GET /api/v1/contexts/{id}` | 获取 context | `star context get <id>` | 404 → `Error` (CONTEXT_NOT_FOUND) |
| 10 | `GET /api/v1/code/search?q=...` | 搜索代码 | `star code search <q>` | 400 → `Error` (VALIDATION_FAILED) |
| 11 | `GET /api/v1/code/symbols/{name}` | 符号定位 | `star code symbol <name>` | 404 → `Error` (SYMBOL_NOT_FOUND) |
| 12 | `GET /api/v1/workspaces` | 列出 workspace | `star workspace list` | 500 → `Error` (INTERNAL) |
| 13 | `GET /api/v1/workspaces/current` | 当前 workspace（`WorkspaceSummary`） | `star workspace current` | 404 → `Error` (NO_CURRENT_WORKSPACE) |
| 14 | `POST /api/v1/worktrees` | 创建 worktree | `star worktree create <id>` | 400 → `Error` (VALIDATION_FAILED) / 409 → `Error` (WORKTREE_CONFLICT) |
| 15 | `POST /api/v1/worktrees/{id}/enter` | 进入 worktree | `star worktree enter <id>` | 404 → `Error` (WORKTREE_NOT_FOUND) |
| 16 | `GET /api/v1/worktrees/{id}/status` | worktree 状态 | `star worktree status` | 404 → `Error` (WORKTREE_NOT_FOUND) |
| 17 | `POST /api/v1/merge-requests` | 创建 MR（**复数**，per F-17） | `star mr create` | 400 → `Error` (VALIDATION_FAILED) / 409 → `Error` (MR_CONFLICT) |
| 18 | `GET /api/v1/merge-requests/{id}` | MR 详情（**复数**） | `star mr show <id>` | 404 → `Error` (MR_NOT_FOUND) |
| 19 | `POST /api/v1/tests/affected` | 跑受影响测试 | `star test affected` | 500 → `Error` (TEST_RUN_FAILED) |
| 20 | `POST /api/v1/submits` | Universal Submit（**复数**） | `star submit` | 422 → `Error` (VALIDATION_FAILED) / 403 → `Error` (POLICY_DENIED) |

> **CLI 17 核心 ↔ REST 17 endpoint 一一对应**（per [spec/cli/01 §2.1](../cli/01-cli-spec.md) 17 命令表 = §4.1 rows 4-20）。`star agent capabilities` / `permissions` / `instructions` 3 个子命令是 v0.2 已有的 discovery 端点（§4.1 rows 1-3），跟 [arch/03 §2.5](../../arch/03-star-ai-compat-arch.md) AGENTS.md bootstrap 的 3 个最小命令对齐。

### 4.2 IDE 端点（3 个，per F-16 修复 2026-08-27 — INTERFACE-REVIEW-A 🟡 #16）

> 补 v0.2 缺失的 3 个 `/api/v1/ide/*` 端点（agent 有，ide 无）：

| # | Method + Path | 用途 | CLI 对应 | 4xx / 5xx 响应 |
|---|---|---|---|---|
| 21 | `GET /api/v1/ide/capabilities` | IDE Capability Discovery | `star ide capabilities` | 500 → `Error` (INTERNAL) |
| 22 | `GET /api/v1/ide/permissions` | IDE Permission Discovery | `star ide permissions` | 403 → `Error` (PERMISSION_DENIED) |
| 23 | `GET /api/v1/ide/instructions` | IDE Instructions | `star ide instructions` | 500 → `Error` (INTERNAL) |

> **总计 23 endpoint**：20 agent (§4.1) + 3 ide (§4.2)。F-09 要求"17+ endpoint 跟 CLI 17 命令对应"已达成（§4.1 rows 4-20 共 17 endpoint 一一对应 CLI 17 核心）。F-16 要求 3 个 IDE 端点已补齐（§4.2 rows 21-23）。

### 4.3 错误响应（per F-19 修复 2026-08-27 — INTERFACE-REVIEW-A 🟡 #19）

> **所有 4xx / 5xx 响应** body 统一引用 [agent-api/v1#Error §3.14](../agent-api/01-schema.md#314-error)（6 字段：`code` / `message` / `source_module` / `source_kind` / `retriable` / `hint`，per F-06 重定义），与 CLI / MCP / Universal Submit 共用同一 schema。

OpenAPI 3.1 `responses` 块必须为每个端点显式列 4xx/5xx 引用 `$ref: '#/components/schemas/Error'`：

```yaml
responses:
  '200':
    description: 成功
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/Task'
  '400':
    description: 参数错误
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/Error'
  '404':
    description: 资源不存在
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/Error'
  '500':
    description: 内部错误
    content:
      application/json:
        schema:
          $ref: '#/components/schemas/Error'
```

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟡 #9/#16/#17/#19 + 🔴 #6 (F-09/F-16/F-17/F-19) — 端点表扩到 20+3=23；统一复数；每个端点显式列 4xx/5xx Error 响应

## 5. 验证

```bash
# spec 必须合法
npx @redocly/cli lint crates/star-rest/openapi/agent-api/v1.yaml

# 自动生成 client
npx openapi-generator-cli generate \
  -i crates/star-rest/openapi/agent-api/v1.yaml \
  -g typescript-fetch \
  -o clients/ts/

# 跑 conformance 测试
pnpm test --filter=@star/rest-conformance
```

> 验证命令中 `agent-api-v1.yaml` → `agent-api/v1.yaml`（per 子代理 A 🟡 #12 斜杠风格统一）

## 6. 实施位置

- `crates/star-rest/` — REST server crate (axum)
- `crates/star-rest/openapi/` — OpenAPI 3.1 spec
- `crates/star-rest/tests/conformance.rs` — spec ↔ 实现一致性测试

## 7. ADR-0022 IDE 归 STAR 边界（per F-30 修复 2026-08-27 — INTERFACE-REVIEW-A 🟢 #30）

> **本节明确 IDE 边界**（per F-30 修复 2026-08-27 — INTERFACE-REVIEW-A 🟢 #30），per [ADR-0022 IDE Placement](../../adr/0022-ide-placement.md)：

- **IDE 业务归 STAR，IDE 客户端不归 STAR**：VSCode / JetBrains / Cursor / 其他 IDE 客户端由各厂商维护，**不**在 STAR 仓库
- **STAR 仅提供 `ide-api/v1` schema + `star-ide-gateway` crate** 供 IDE 客户端集成
- **REST 23 endpoint 全部由 `star-rest` 实现**，不依赖任何 IDE 厂商 HTTP API
- **MCP transport：stdio (local) / Streamable HTTP (remote, Phase 2)**（per [arch/03 §2.3](../../arch/03-star-ai-compat-arch.md)）
- 跟 ADR-0022 §2.2 一致：STAR = IDE 业务控制面；IDE 客户端 = 厂商产品

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #30 (F-30) — §7 明确 IDE 归 STAR 边界（per ADR-0022）

## 8. git-provider-v1.yaml 范围（per F-23 修复 2026-08-27 — INTERFACE-REVIEW-A 🟢 #23）

> **本节明确 `git-provider/v1.yaml` 范围**（per F-23 修复 2026-08-27 — INTERFACE-REVIEW-A 🟢 #23）：

- **该 spec 列出但不实现 endpoint**（per ADR-0023 抽象边界）：
  - Git Provider Abstraction 是 STAR 的**类型契约**（per [arch/05 §3](../../arch/05-gitgit-compat-arch.md) + [ADR-0023 Version Control Provider](../../adr/0023-version-control-provider.md)）
  - 实际 Git 操作 HTTP API 由各 provider 自己实现：GitGit 暴露 `axum HTTP` API（由 GitGit crate 维护，**不**在 STAR 仓库）、GitHub 暴露 `https://api.github.com`、GitLab 暴露 `https://gitlab.com/api/v4`
  - STAR 仓库内的 `crates/star-rest/openapi/git-provider/v1.yaml` 仅作**类型契约引用**，**不**对应真实 endpoint
- **CLI `star mr create` 不会调 git-provider endpoint**：所有 Git Provider HTTP 调用由 GitGit / GitHub / GitLab 客户端 crate 发起，STAR 只通过这些客户端的 Rust API 间接使用
- **REST 23 endpoint（§4）无 git-provider endpoint**：所有 git 操作走 Agent API（agent-api/v1），由 `star-vcs` crate 内部路由到对应 provider 客户端

> v0.2 fix: 2026-08-27 per INTERFACE-REVIEW-A 🟢 #23 (F-23) — §8 明确 git-provider-v1.yaml 范围（无 endpoint，per ADR-0023）

## 9. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：12 endpoint 草案 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-G：§4 每个 endpoint 加 4xx/5xx 响应（统一引用 `agent-api/v1#Error`） · §1 加 OpenAPI 3.1 关键字段（`info.summary` / `webhooks` / `nullable: true` → `type: [string, "null"]`） · §2 spec 文件路径改斜杠风格（per 子代理 A 🟡 #12） | 8 子代理 INTERFACE-REVIEW-A 🟡 #10/#12/#17 + INTERFACE-REVIEW-A 🔴 #6 + P1-BLOCKERS-SUMMARY v0.2 |
| v0.2 fix | 2026-08-27 | Mavis（接手 agent per DEC-008 — 子代理 fix-api-spec-blockers）| F-09：§4 端点表扩到 20 agent + 3 ide = 23 endpoint（CLI 17 核心一一对应）· F-10：§1 强化 OpenAPI 3.1 关键字段（`info.summary` / `webhooks` / `nullable: true` 破坏性迁移）· F-11：§1 加 `info.version` vs `schema_version` 双版本字段关系（跟 agent-api §1 对齐）· F-16：§4.2 补 3 个 IDE 端点（capabilities / permissions / instructions）· F-17：§4 统一复数（`/merge-requests` / `/submits` / `/issues` / `/projects` / `/workspaces` / `/worktrees` / `/contexts` / `/tests`）· F-19：§4.3 强化每个端点显式 4xx/5xx Error 响应 `$ref` 块 · F-23：§8 新增 `git-provider/v1.yaml` 范围说明（无 endpoint，per ADR-0023）· F-30：§7 新增 ADR-0022 IDE 归 STAR 边界声明 | INTERFACE-REVIEW-A 🟢 #23/#30 + 🟡 #9/#10/#11/#16/#17/#19 + 🔴 #6 |
