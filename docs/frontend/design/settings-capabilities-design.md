# Settings 能力管理面板 — 设计文档

> per WBS §23.33-B (2026-11-13 用户发令)
> 用户原话: "设置里要有 MCP / Skill / Plugin 和类似 Claude Code 的 MCP 相关设置, 并且支持这些功能"

## 1. 目标

在 `/settings/advanced` 下把 Skills / MCP / Plugins 三个 tab 从占位符升级为可用的管理面板,
模型对齐 Claude Code 的 MCP 管理心智:

| Claude Code 概念 | Star 对应 |
|---|---|
| `claude mcp add` / `remove` / `list` | MCP 面板 添加/移除/列表 |
| 连接类型 `stdio` / `sse` / `http` | 同上三枚举 (transport) |
| 作用域 `user` (~/.claude.json) / `project` (.mcp.json) | scope: `user` / `project` |
| `claude mcp get <name>` 状态 | 健康状态徽章 (unknown/connecting/healthy/error) + last_error |
| plugin marketplace / 启用禁用 | Plugins 面板 (权限清单 + 启停) |
| skills 目录 (~/.claude/skills) | Skills 面板 (版本/作用域/启停) |

## 2. 路由与组件

- 路由不变: `/settings/advanced/[tab]`, `tab ∈ {hooks, mcp, skills, plugins}` (layout.tsx 已有 4 tab 骨架)
- `page.tsx` 分派: hooks → HookPolicyPage (已实装), mcp → McpSettingsPanel, skills → SkillsSettingsPanel, plugins → PluginsSettingsPanel

## 3. 数据层 (capabilityRegistry)

`frontend/src/lib/settings/capabilityRegistry.ts`

- 类型: `McpServerConfig` (transport/command/args/env/scope/enabled/health/last_error),
  `SkillEntry` (name/version/scope/enabled/description),
  `PluginEntry` (name/version/scope/enabled/permissions/description)
- `useSyncExternalStore` hook + localStorage 持久化 (key `star-capability-registry:v1`)
- `CapabilityRegistryApi`: add/remove/toggle × 3 类
- 接口形状对齐未来后端 REST CRUD (见 api-design-mcp-capabilities.md); web 无后端时降级 localStorage seed
- seed: filesystem (stdio/user), github (http/project), 2 skills, 1 plugin

## 4. 交互

- MCP 面板: 添加表单 (名称/连接类型/命令或URL/参数/作用域), 名称与命令必填校验;
  行内 启用/停用 (Power), 移除 (Trash2), 健康徽章 (healthy/error/connecting/unknown)
- Skills 面板: 名称/版本/描述/作用域 + 启停 + 移除
- Plugins 面板: 名称/版本/权限(逗号空格分隔)/描述/作用域 + 启停 + 移除; 权限 chip 列表 (warn 色 + ShieldCheck)

## 5. i18n

`settingsCapabilities.{mcp,skills,plugins}` 三语 (zh-CN/en/ja), key 一致性由
`dictionaryConsistency.test.ts` 硬门禁保证。

## 6. 与现有系统的关系

- HookPolicyPage 不动 (hooks tab 已实装, 独立测试 13 条)
- 弹层实色 (`popover-solid`, per WBS §23.33-A) 是本面板的视觉前置依赖:
  面板内所有浮层不得透底

## 7. 验收标准

- [x] 三个 tab 从占位符变为可用面板
- [x] MCP 添加/移除/启停 + 连接类型/作用域/健康状态
- [x] localStorage 持久化 (refresh 不丢)
- [x] 三语 i18n + key 一致性测试
- [x] 组件测试 (CapabilityPanels.test.tsx, 7 条)
