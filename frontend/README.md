# Star Frontend

Next.js 14 (App Router) + TypeScript + Tailwind CSS — Control Plane UI for the Star Vibe Coding Work Management platform.

> 25 个 domain module 全部可视化,5 个核心状态机(worktree / agent / feedback / PR / changeset)可交互触发迁移。
> 数据来源:`D:\Star\docs\specs\*` + `docs/api-design.md` + `docs/basic-design.md §7`

## 启动

```bash
cd D:\Star\frontend
npm install
npm run dev
# → http://localhost:3000
```

## 模块对应表 (25 domain → 路由)

| Track | Route             | Module       | 关键 INV / 状态机                                |
|-------|-------------------|--------------|------------------------------------------------|
| —     | `/`               | Dashboard    | 5 状态机分布汇总                                 |
| B     | `/worktree`       | Worktree     | **17 状态机** + INV-WT-01~04                    |
| B     | `/agent`          | Agent        | **14 状态机** + INV-AGT-N01~N14                 |
| B     | `/feedback`       | Feedback     | **6 状态机** + INV-FB-01~02                     |
| B     | `/context`        | Context      | INV-CT-01~10 + 3-state Decision                 |
| B     | `/validation`     | Validation   | 7 实体 + 5 状态机 + 覆盖率                       |
| C     | `/scm`            | SCM          | **7 状态机 PR** + Webhook Idempotency           |
| C     | `/integration`    | Integration  | Loop 防护 (lp-* keys)                           |
| B     | `/notification`   | Notification | **INV-N-07 抑制策略**                           |
| B     | `/search`         | Search       | INV-SR-01/02 (Projection + tenant 隔离)         |
| D     | `/tenant`         | Tenant       | 13 类 tenant_id 必带对象根                      |
| D     | `/project`        | Project      | key 前缀 (PHYSIS-)                              |
| D     | `/identity`       | Identity     | 6 provider + MFA                                |
| D     | `/work-item`      | Work Item    | **6 状态机** + INV-PM-01~05                     |
| D     | `/comment`        | Comment      | 4 target kinds                                  |
| D     | `/permission`     | Permission   | Rules-based RBAC + CEL condition                |
| D     | `/workflow`       | Workflow     | State + Transition + guard                      |
| D     | `/development`    | Development  | **ChangeSet 5 状态机** + INV-DEV-01~05          |
| E     | `/planning`       | Planning     | Sprint + Milestone + Burndown chart             |
| E     | `/board`          | Board        | Kanban + WIP limit                              |
| E     | `/collaboration`  | Collaboration| Presence cursor + Whiteboard                    |
| E     | `/local-runtime`  | Local Runtime| device/tenant/user 三重绑定                     |
| E     | `/relation`       | Relation     | Graph + 5-layer BFS                             |
| E     | `/audit`          | Audit        | Append-only + 9 AI questions + cross-tenant    |
| E     | `/automation`     | Automation   | Rule + Trigger + Condition + Action + 6 INV    |
| E     | `/workspace`      | Workspace    | Member + branch policy                          |

## 文件结构

```
frontend/
├── package.json
├── tsconfig.json
├── tailwind.config.ts
├── next.config.js
├── postcss.config.js
└── src/
    ├── app/
    │   ├── layout.tsx        # 根 layout (Sidebar + Topbar)
    │   ├── globals.css       # Tailwind + dark theme
    │   ├── page.tsx          # Dashboard
    │   ├── worktree/page.tsx
    │   ├── agent/page.tsx
    │   └── ...  (25 个 route)
    ├── components/
    │   ├── Sidebar.tsx       # 25 模块导航 (按 Track 分组)
    │   ├── Topbar.tsx        # Tenant/Project switcher + ⌘K
    │   ├── PageHeader.tsx    # 标题 + Stat 卡片
    │   ├── StatusPill.tsx    # 状态/类别色码 pill
    │   └── StateMachineDiagram.tsx  # SVG 状态机可视化
    ├── types/
    │   └── ids.ts            # 25 domain TS type + 6 状态机定义
    └── lib/
        ├── seed.ts           # 全量 mock data
        ├── store.ts          # Zustand store + 状态机 transition mutations
        └── page-builders.tsx # ListPage / StatsPage 通用 builder
```

## 交互式状态机

任何状态机页面(worktree / agent / feedback / PR / changeset / work-item)都可:

1. 点击列表中的任一实例
2. 右侧详情面板显示 "allowed transitions" 按钮(由 5 状态机 transitions 表推导)
3. 点击按钮触发 zustand store mutation,实时更新所有视图
4. SVG 状态机图随当前选中实例高亮 in/out 边

## 配色方案

- Dark theme + GitHub Primer 配色
- 状态色:ok 绿 / warn 黄 / err 红 / info 蓝
- Track 标识:Track B/C/D/E (在 Sidebar 各模块右侧)

## 数据流(当前 mock,接 backend 时换实现)

```
seed.ts (immutable data)
    ↓
store.ts (Zustand,read + transition mutators)
    ↓
page.tsx (useStore 订阅,渲染)
```

切到真 backend 时,把 `store.ts` 里的 `seed.*` 替换成 `fetch('/api/...')` + SWR / React Query 即可,UI 层不动。

## 下一步

- 接 OpenAPI 生成的 client (待 `crates/api` 实现后)
- 加 WebSocket Realtime 推送 (presence / agent status / worktree events)
- 加 ⌘K 全局命令面板
- 加 dark/light theme switch
- 加权限视图(基于 PermissionScheme 隐藏/显示按钮)
