# STAR-WBS-BACKEND-SUPPORT-001: 缺后端功能补全 (API 门面 + 后端联通 + 画布/主题/旧页欠账) v0.1

> **Status**: 🟢 Active v0.1 (2026-11-14 用户发令: "缺少后端支持的功能应该补全，可以制作 wbs，然后按照 wbs 推进")
> **触发**: dev server 日志实测 14 个前端 fetch 端点 404 + /api/v1/* proxy ECONNREFUSED (star-api-rest 未运行) + 本会话积压需求 (画布 Miro 工具集 / 主题全覆盖 / 时间线旧页统一)
> **范围**: 前端 API 门面层补全 + 本地后端联通 + 三项 UI 欠账。Rust 后端持久化路由实装不在本 WBS (走 star-api-rest 既有路线)。

---

## §0 现状 (2026-11-14 实测)

| 项 | 状态 |
|---|---|
| `/api/gemarc` (GemArc MCP 双通道代理) | ✅ route handler 已通 |
| `/api/magos` (MAGOS 只读代理) | ✅ route handler 已通 |
| `/api/v1/*` (next.config rewrites → 127.0.0.1:8081 star-api-rest) | ❌ ECONNREFUSED — 后端未运行 |
| `/api/notifications` `/api/cli-profiles` `/api/agents` `/api/comments` `/api/annotations` `/api/incidents` `/api/design-artifacts` `/api/validation/results` `/api/workspaces` `/api/api-keys` `/api/rbac/roles` `/api/arg/edges` | ❌ 404 — 仅 MSW browser mock (src/mocks/handlers/*)，无 route handler |
| `/api/tmo/spawn_subtask` | 🟡 route 在 /tasks/api/tmo/spawn_subtask (调用路径待核对) |
| `/api/v2/credentials` | 🟡 无实际调用点 (疑似废弃) |
| 数据源 | ✅ src/mocks/data/* 全套 (agents/annotations/cli/design-artifacts/five-domain/incidents/inbox/kanban/validation) |
| 画布 CanvasView | 🟡 7 工具 (select/pan/sticky/text/shape/frame-select/brush) + 调色盘 + 缩放 + 最小地图; 缺 Miro 级: 橡皮擦/直线箭头/形状库/撤销重做/注释/网格吸附 |
| 主题覆盖 | ❌ 296 处 `var(--cel-*, #硬值)` 非 cel 主题下不跟随 (CanvasView 26 / AppMatrixDrawer 31 / AgentCanvasView 30 / agents 页 36 / AppHeader 等) |
| kanban 时间线/日历 tab | ❌ 跳旧 /projects?tab=timeline (老版本 UI, 用户报"UI 变成老版本") |

## §1 W1 — API 门面层 (12 组 route handlers, 数据源 = mocks/data, 与 MSW 同形状)

设计原则: route handler = Next `/api/*` 门面, 直接返回 mocks/data 数据 (与 MSW browser mock 同一数据源, 形状一致); PATCH/POST/DELETE 返回内存行为 (与 MSW 相同语义)。真实持久化走 W2 后端联通后逐步替换, 每个 handler 头部预留 host 转发短路。

| # | 子任务 | 端点 | 依赖 | 状态 |
|---|---|---|---|---|
| **W1.1** | 通知域 | GET/PATCH /api/notifications | — | 🟡 |
| **W1.2** | CLI/密钥域 | GET/POST/PATCH/DELETE /api/cli-profiles + GET/POST/DELETE /api/api-keys | — | 🟡 |
| **W1.3** | Agent 域 | GET /api/agents | — | 🟡 |
| **W1.4** | 评论域 | GET/POST /api/comments | — | 🟡 |
| **W1.5** | 标注域 | GET/POST/PATCH /api/annotations | — | 🟡 |
| **W1.6** | 事件域 | GET/PATCH /api/incidents | — | 🟡 |
| **W1.7** | 工件域 | GET/POST/PATCH /api/design-artifacts | — | 🟡 |
| **W1.8** | 验证域 | GET/POST /api/validation/results | — | 🟡 |
| **W1.9** | 工作区域 | GET /api/workspaces | — | 🟡 |
| **W1.10** | RBAC 域 | GET/POST /api/rbac/roles | — | 🟡 |
| **W1.11** | ARG 域 | GET/POST/PATCH/DELETE /api/arg/edges (对齐 crates/api/src/arg/controller.rs 13 端点) | — | 🟡 |
| **W1.12** | 验证 | curl 全端点 2xx + tsc 0 + vitest 全绿 + cgg stamp | W1.1~11 | 🟡 |

## §2 W2 — 后端联通 (star-api-rest 本地运行 + proxy 链路)

| # | 子任务 | 依赖 | 状态 |
|---|---|---|---|
| **W2.1** | star-api-rest 编译 + 启动 (8081, DATABASE_URL + JWT_PUBLIC_KEY_PEM per star-local-run skill) | — | 🟡 |
| **W2.2** | next rewrites proxy 验证 (/api/v1/health 200, 401 语义正确) | W2.1 | 🟡 |
| **W2.3** | 前端 auth 优雅降级走查 (NETWORK_UNREACHABLE 路径) | W2.2 | 🟡 |

## §3 W3 — 画布 Miro 级工具集 (用户发令: 画笔等达到 Miro 标准)

| # | 子任务 | 依赖 | 状态 |
|---|---|---|---|
| **W3.1** | 工具集补全: 橡皮擦 / 直线+箭头连接器 / 形状库(矩形/圆/菱形/圆角矩形/三角形) | — | 🟡 |
| **W3.2** | 撤销/重做栈 (element/path 双栈) | W3.1 | 🟡 |
| **W3.3** | 网格吸附 + 框选拖拽增强 + 键盘快捷键 (V/H/P/S/T/N/E + Ctrl+Z/Shift+Z) | W3.1 | 🟡 |
| **W3.4** | 便签/文本双击编辑 + 注释工具 | W3.1 | 🟡 |
| **W3.5** | 主题贴合: SVG 舞台背景/网格 var(--cel-*) 硬值 → 主题 token (per 用户: 切换主题画布跟随) | — | 🟡 |

## §4 W4 — 主题全覆盖 (296 处 cel-* 硬值 token 化)

| # | 子任务 | 依赖 | 状态 |
|---|---|---|---|
| **W4.1** | CanvasView (26) + AgentCanvasView (30) + AppMatrixDrawer (31) + AppHeader | — | 🟡 |
| **W4.2** | agents 页 (36) + agent-view 页 (9) + AgentFilter (10) + sprint/SubNav/AppShell 等 | W4.1 | 🟡 |
| **W4.3** | 6 主题走查: 画布/顶栏/侧栏/抽屉全部跟随 | W4.2 | 🟡 |

## §5 W5 — 时间线/日历旧页统一

| # | 子任务 | 依赖 | 状态 |
|---|---|---|---|
| **W5.1** | kanban 时间线 tab 挂真实 GanttChart (去 /projects 旧页跳转) | — | 🟡 |
| **W5.2** | 日历 tab 挂 W3 Calendar 组件或统一占位 | W5.1 | 🟡 |

## §6 验收总闸

- tsc 0 / vitest 全绿 / cargo fmt --all --check / cgg_guard PASS
- 浏览器实测: 14 端点 2xx; 画布工具条 Miro 工具集可用; 6 主题切换画布+顶栏+侧栏跟随; kanban 时间线无旧页跳转
