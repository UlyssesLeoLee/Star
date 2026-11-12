# Settings 能力管理面板 — 测试设计

> per WBS §23.33-C (2026-11-13)

## 1. 测试分层

| 层 | 文件 | 缝 (seam) | 覆盖 |
|---|---|---|---|
| 数据层 | (vitest, capabilityRegistry 直接导入) | CapabilityRegistryApi 公共方法 | CRUD + 持久化 + 幂等 |
| 组件层 | `CapabilityPanels.test.tsx` | 面板 DOM (data-testid) | 渲染 seed / 表单校验 / 启停 / 移除 / 权限解析 |
| 字典层 | `dictionaryConsistency.test.ts` (已有) | 三语 key 集合 | 新增 settingsCapabilities 段三语同构 |
| 路由层 | `layout.test.tsx` (已有) | tab 导航 | 4 tab 可达性 (回归) |

## 2. 已实现用例清单 (CapabilityPanels.test.tsx, 7 条)

1. McpSettingsPanel 渲染 seed 服务器 + stdio/http 徽章
2. 表单添加新服务器 (transport=sse, scope=project, enabled=true)
3. 空名称拒绝 + 错误提示 (不产生脏数据)
4. 启停 + 移除服务器
5. Skills 列表/添加(版本)/移除
6. Plugins 权限解析 ("tenant:read, audit:write" → ["tenant:read","audit:write"]) + seed 权限 chip 渲染
7. 注册表持久化到 localStorage (key star-capability-registry:v1)

## 3. 关键设计决策

- **缝 = 面板 DOM + API 公共方法**, 不测内部状态 (React 内部 state 不暴露)
- **resetCapabilityRegistryForTest()** 每个用例前重置注册表, 保证测试独立性 (localStorage 污染隔离)
- **seed 数据即期望值来源**: 测试断言 seed 结构 (filesystem/github/2 skills/1 plugin),
  期望值来自 DEFAULT_CAPABILITY_REGISTRY 常量而非测试内联, 避免同源重算 (tautological)
- **持久化断言**: 检查 localStorage 原文含新条目, 不 mock storage 实现

## 4. 后续 (v0.2 后端就绪后)

- API 层契约测试: CapabilityRegistryApi fetch 实现 vs api-design-mcp-capabilities.md 端点 (走 axum oneshot, per guard #25)
- 健康探测: POST /check 的 202/504 分支 (timeout 注入)
- E2E: settings → mcp 面板添加服务器 → refresh 持久 → 状态徽章变化

## 5. 门禁

- `npx tsc --noEmit` 0 errors
- `npx vitest run "src/app/(app)/settings" src/lib/i18n` 全绿 (44/44 实测)
- CypherGraph Guardian --stamp PASS
