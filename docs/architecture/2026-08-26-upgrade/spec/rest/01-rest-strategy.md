# 15. REST / OpenAPI Strategy

> **状态**：🟡 草案 v0.2
> **依赖**：[Protocol Survey §3](../../ecosystem-survey/protocol-survey.md) · [arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md) · [spec/agent-api/01-schema.md §3.15 Error](../agent-api/01-schema.md)

## 1. OpenAPI 版本

- **OpenAPI 3.1**（不是 3.0）— 完整对齐 JSON Schema 2020-12
- 不采用 OpenAPI 3.2（MVP 阶段，3.1 已稳）
- **OpenAPI 3.1 关键字段**（per 子代理 A 🟡 #10）：
  - `info.summary` 允许（信息摘要，跟 `info.description` 区分）
  - `webhooks` 字段允许（per OpenAPI 3.1 加入）
  - `nullable: true` → `type: [string, "null"]`（破坏性迁移，OpenAPI 3.1 用 JSON Schema 2020-12 风格）

## 2. Spec 文件位置

- `crates/star-rest/openapi/agent-api/v1.yaml` — Agent API（**斜杠风格**，per 子代理 A 🟡 #12 统一）
- `crates/star-rest/openapi/ide-api/v1.yaml` — IDE API
- `crates/star-rest/openapi/git-provider/v1.yaml` — Git Provider API（per ADR-0023 抽象；GitGit 自身 axum HTTP 由 GitGit crate 单独维护）

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

## 4. 关键 endpoint 草案（per P1-G 修复 2026-08-27 加 error response）

| Endpoint | 用途 | 4xx / 5xx 响应 |
|---|---|---|
| `GET /api/v1/agent/capabilities` | Capability Discovery | 500 → `Error` (INTERNAL) |
| `GET /api/v1/agent/permissions` | Permission Discovery | 403 → `Error` (PERMISSION_DENIED) |
| `GET /api/v1/agent/instructions` | Agent Instructions | 500 → `Error` (INTERNAL) |
| `GET /api/v1/tasks/current` | Current Task | 404 → `Error` (NO_CURRENT_TASK) |
| `GET /api/v1/workspaces/current` | Current Workspace | 404 → `Error` (NO_CURRENT_WORKSPACE) |
| `GET /api/v1/worktrees` | List Worktrees | 500 → `Error` (INTERNAL) |
| `POST /api/v1/worktrees` | Create Worktree | 400 → `Error` (VALIDATION_FAILED) / 409 → `Error` (WORKTREE_CONFLICT) |
| `GET /api/v1/code/search?q=...` | Code Search | 400 → `Error` (VALIDATION_FAILED) |
| `GET /api/v1/code/symbols/{name}` | Symbol Lookup | 404 → `Error` (SYMBOL_NOT_FOUND) |
| `POST /api/v1/merge-requests` | Create MR（per 子代理 A 🟡 #17 改复数） | 400 → `Error` (VALIDATION_FAILED) / 409 → `Error` (MR_CONFLICT) |
| `POST /api/v1/submit` | Universal Submit | 422 → `Error` (VALIDATION_FAILED) / 403 → `Error` (POLICY_DENIED) |
| `GET /api/v1/context/{issue_id}` | Context | 404 → `Error` (ISSUE_NOT_FOUND) |

> **所有 4xx / 5xx 响应** body 统一引用 [`agent-api/v1#Error`](../agent-api/01-schema.md) §3.15（6 字段：`error` / `recoverable` / `suggested_actions` / `message` / `trace_id` / `details`），与 CLI / MCP / Universal Submit 共用同一 schema，per P1-G 修复 2026-08-27。OpenAPI 3.1 `responses` 块必须为每个端点显式列 4xx/5xx 引用 `$ref: '#/components/schemas/Error'`。

## 5. 验证

```bash
# spec 必须合法
npx @redocly/cli lint crates/star-rest/openapi/agent-api-v1.yaml

# 自动生成 client
npx openapi-generator-cli generate \
  -i crates/star-rest/openapi/agent-api-v1.yaml \
  -g typescript-fetch \
  -o clients/ts/

# 跑 conformance 测试
pnpm test --filter=@star/rest-conformance
```

## 6. 实施位置

- `crates/star-rest/` — REST server crate (axum)
- `crates/star-rest/openapi/` — OpenAPI 3.1 spec
- `crates/star-rest/tests/conformance.rs` — spec ↔ 实现一致性测试

## 7. 签字栏 / 修订历史

per [arch/01](../../arch/01-current-architecture-analysis.md) 模板。Mavis 代签 2026-08-26。

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-08-26 | Mavis（per DEC-008）| 初版：12 endpoint 草案 | Phase C 54 份 spec 草案 |
| v0.2 | 2026-08-27 | Ulysses（一人公司 12 角色 per DEC-008）| P1-G：§4 每个 endpoint 加 4xx/5xx 响应（统一引用 `agent-api/v1#Error`） · §1 加 OpenAPI 3.1 关键字段（`info.summary` / `webhooks` / `nullable: true` → `type: [string, "null"]`） · §2 spec 文件路径改斜杠风格（per 子代理 A 🟡 #12） | 8 子代理 INTERFACE-REVIEW-A 🟡 #10/#12/#17 + INTERFACE-REVIEW-A 🔴 #6 + P1-BLOCKERS-SUMMARY v0.2 |
