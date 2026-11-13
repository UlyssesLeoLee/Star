# MCP / Skills / Plugins 能力管理 API 设计

> per WBS §23.33-C (2026-11-13)
> 状态: v0.1 前端已实装 (localStorage 降级), 本文档定义未来后端 REST 形状;
> 前端 `CapabilityRegistryApi` 的方法签名即为本文档端点的客户端映射。

## 1. 资源模型

```
McpServerConfig {
  id: uuid
  name: string (unique per scope)
  transport: "stdio" | "sse" | "http"
  command: string          // stdio: 可执行命令; sse/http: URL
  args: string[]
  env: map<string,string>  // stdio 子进程环境
  scope: "user" | "project"
  enabled: bool
  health: "unknown" | "connecting" | "healthy" | "error"
  last_error: string | null
  updated_at_ms: int64
}
SkillEntry  { id, name, version, scope, enabled, description }
PluginEntry { id, name, version, scope, enabled, permissions: string[], description }
```

## 2. 端点 (对齐 RestResponse/RestError 6 字段模型, per api-design.md §8)

### MCP servers
```
GET    /api/v1/capabilities/mcp-servers?scope=user|project   → list
POST   /api/v1/capabilities/mcp-servers                      → create (409 name dup per scope)
PATCH  /api/v1/capabilities/mcp-servers/{id}                 → partial update (enabled/command/args/env)
DELETE /api/v1/capabilities/mcp-servers/{id}                 → remove (204)
POST   /api/v1/capabilities/mcp-servers/{id}/check           → health probe (202, async result → health/last_error)
```

### Skills
```
GET    /api/v1/capabilities/skills?scope=user|project
POST   /api/v1/capabilities/skills
PATCH  /api/v1/capabilities/skills/{id}          (enabled/version)
DELETE /api/v1/capabilities/skills/{id}
```

### Plugins
```
GET    /api/v1/capabilities/plugins?scope=user|project
POST   /api/v1/capabilities/plugins
PATCH  /api/v1/capabilities/plugins/{id}         (enabled/permissions)
DELETE /api/v1/capabilities/plugins/{id}
```

## 3. 权限

- scope=user 的变更要求 `tenant_admin` 或本人 owner
- scope=project 的变更要求该 project 的 `project_admin`
- 读取要求登录即可 (user scope 过滤到本人)

## 4. 错误码

| 场景 | source_kind | code |
|---|---|---|
| name 重复 (同 scope) | validation | 409 CONFLICT |
| id 不存在 | validation | 404 NOT_FOUND |
| 无权改 project scope | policy | 403 FORBIDDEN |
| health probe 超时 | timeout | 504 UPSTREAM_TIMEOUT |

## 5. 后端落地路径 (v0.2+)

- `crates/domain-mcp` 现有 `McpServer` trait 扩展 `list/add/remove/health_check`
- `crates/star-api-rest` 新 `routes/capabilities.rs` 挂入 `build_router`
- stdio 启动 = 子进程 spawn (对齐 workspace_files 的 canonical_directory 防逃逸先例),
  sse/http = 出站 HTTP 客户端 (SSRF 白名单, 对齐 AGENTS.md 出站规则)

## 6. 与前端降级模式的关系

web 部署无此 API 时, `capabilityRegistry.ts` 自动走 localStorage (面板功能完整可用);
API 就绪后替换 `CapabilityRegistryApi` 内部实现为 fetch 即可, UI 零改动。
