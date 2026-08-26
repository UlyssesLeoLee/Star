# 15. REST / OpenAPI Strategy

> **状态**：🟡 草案 v0.1
> **依赖**：[Protocol Survey §3](../../ecosystem-survey/protocol-survey.md) · [arch/03 STAR AI Compat Arch](../../arch/03-star-ai-compat-arch.md)

## 1. OpenAPI 版本

- **OpenAPI 3.1**（不是 3.0）— 完整对齐 JSON Schema 2020-12
- 不采用 OpenAPI 3.2（MVP 阶段，3.1 已稳）

## 2. Spec 文件位置

- `crates/star-rest/openapi/agent-api-v1.yaml` — Agent API
- `crates/star-rest/openapi/ide-api-v1.yaml` — IDE API
- `crates/star-rest/openapi/git-provider-v1.yaml` — Git Provider API

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

## 4. 关键 endpoint 草案

| Endpoint | 用途 |
|---|---|
| `GET /api/v1/agent/capabilities` | Capability Discovery |
| `GET /api/v1/agent/permissions` | Permission Discovery |
| `GET /api/v1/agent/instructions` | Agent Instructions |
| `GET /api/v1/tasks/current` | Current Task |
| `GET /api/v1/workspaces/current` | Current Workspace |
| `GET /api/v1/worktrees` | List Worktrees |
| `POST /api/v1/worktrees` | Create Worktree |
| `GET /api/v1/code/search?q=...` | Code Search |
| `GET /api/v1/code/symbols/{name}` | Symbol Lookup |
| `POST /api/v1/mr` | Create MR |
| `POST /api/v1/submit` | Universal Submit |
| `GET /api/v1/context/{issue_id}` | Context |

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
