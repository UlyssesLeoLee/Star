# Star Frontend — UI/UX Redesign (Multica.ai 风格)

> **状态**: Draft v0.1
> **日期**: 2026-08-28
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签
> **触发**: Ulysses 2026-08-28 19:21 JST 反馈"UI/UX 不要给用户太大的认知负荷,核心功能聚合进类似 https://multica.ai/ 的 UI 中,简洁但功能丰富"

---

## §1 目标与原则

参考 multica.ai 的设计:
- **简洁高密度** — 14px text-label, 11px text-micro uppercase tracking
- **顶栏 + 左 sidebar + 主区** 三层架构
- **sticky 左导航** — 180px 宽,圆形 indicator
- **dark theme 优先** — #05070b / #0a0d12, 简洁 monospace
- **核心 6 路由合并** — Inbox / My Issues / Projects / Agents / Analytics / Settings
- **25 路由 → 6 路由** — 把 22 domain 路由聚合到 6 panel

## §2 6 路由设计 (核心面板)

| 路由 | 包含的 22 domain | 关键功能 |
|---|---|---|
| **/inbox** | notification / comment / audit | 通知 inbox + 实时 @ 提及 + 审计 feed |
| **/issues** | work-item / feedback / worktree / agent / decision / automation | 主面板 — Kanban / List / Tree view 切换 |
| **/projects** | project / workspace / planning / board / workflow / canvas | 5 tab: list / board / gantt / calendar / workflow |
| **/agents** | agent / agent-session / lease / resume / runtime | agent 列表 + 状态机 + lease/heartbeat |
| **/analytics** | dashboard / metric / cost / burndown | 5 维 dashboard + cost 报表 |
| **/settings** | tenant / identity / permission / role / integration / scm | 配置 + 集成 + 权限 |

**6 路由吸收 22 路由** — 大幅减少导航项,符合"核心功能聚合"目标。

## §3 顶栏 (AppHeader)

- **左**: logo (multica 风格 `*` clip-path 几何) + workspace switcher
- **中**: 5 视图 tab (Inbox / Issues / Projects / Agents / Analytics / Settings)
- **右**:
  - ⌘K 搜索 (CommandBar 全局命令)
  - 🔔 通知 (badge + dropdown)
  - 🟢 状态 (online / sync status)
  - 👤 user avatar + workspace switcher

高度: 64px (per multica 76px 减 12px 适配中等密度)
背景: dark `#0a0d12` (multica `#05070b` 微调)
border-bottom: 1px `#21262d`

## §4 左 sidebar (SubNav)

只在以下路由显示 (嵌套子页面):
- /projects (Overview / Board / Gantt / Calendar / Workflow)
- /agents (Sessions / Runtimes / Skills)
- /analytics (Cost / Tokens / Errors / Leaderboard)

180px 宽, sticky, top-16
item 高度: 36px
active: `bg-accent/12` + 左侧 2px accent border
hover: `bg-bg-soft/40`

## §5 主区布局 (MainPanel)

### 5.1 /issues 主面板
```
┌─────────────────────────────────────────────────────┐
│ [Tabs: Kanban | List | Tree | Sprint]   [+ New] 🔍 │
├─────────────────────────────────────────────────────┤
│ ┌────────┬──────────┬──────────┬──────────┐         │
│ │ TODO   │ IN_PROG  │ REVIEW   │ DONE     │         │
│ │ (12)   │ (8)      │ (4)      │ (47)     │         │
│ ├────────┼──────────┼──────────┼──────────┤         │
│ │[card]  │[card]    │[card]    │[card]    │         │
│ │[card]  │[card]    │          │[card]    │         │
│ └────────┴──────────┴──────────┴──────────┘         │
└─────────────────────────────────────────────────────┘
```

- 顶部 1 行: 4 tabs (视图切换) + 右侧 1 个 New + 1 个搜索
- 中部 4 列 Kanban, 每列 360px 宽, WIP limit 显示
- 列内: cards draggable + keyboard accessible (per G3 a11y)
- 列下拉: 折叠 / 详情侧栏 (右侧 320px 抽屉)

### 5.2 /projects 多 panel
```
┌─────────────────────────────────────────────────────┐
│ [Tabs: List | Board | Gantt | Calendar | Workflow] │
├─────────────────────────────────────────────────────┤
│ [Filter Bar: Sprint | Member | Status | Search]   │
├─────────────────────────────────────────────────────┤
│ <active tab content>                                │
└─────────────────────────────────────────────────────┘
```

### 5.3 /agents dashboard
- 左 360px: agent 列表 (card w/ status pill + role)
- 中 flex: 选中 agent 详情 (state machine diagram + sessions table)
- 右 320px: lease / heartbeat / 实时事件

### 5.4 /analytics dashboard
- 6 KPI cards (2x3 grid): cost / tokens / tasks / errors / leaderboard / runtime
- 2 chart: daily cost trend (line) + error mix (donut)
- 1 table: 详细 leaderboard

### 5.5 /inbox
- 3 column: 通知源 / 通知列表 / 详情
- 顶部 filter: unread / @ me / audit / comment / notif
- 实时 SSE 推送 (Phase I+)

### 5.6 /settings
- 左 sidebar: Profile / Workspace / Members / Permissions / Runtimes / Skills / Billing
- 右 main: 选中的设置面板

## §6 CommandBar (全局 ⌘K 搜索)

- 模态, 中心 720px 宽, top-20% 位置
- 输入框 + 实时搜索结果 (按 /issues /projects /agents 等分组)
- 命令: "Create issue", "Switch workspace", "Open project board", "View burndown"
- 快捷键: Esc 关闭, ↑↓ 导航, Enter 触发, ⌘K 打开

## §7 设计 token (Tailwind config)

```ts
// tailwind.config.ts 扩展
colors: {
  'bg':         { DEFAULT: '#0a0d12', soft: '#161b22', lighter: '#1c2128' },
  'border':     { DEFAULT: '#21262d', line: '#30363d' },
  'ink':        { DEFAULT: '#e6edf3', dim: '#7d8590', mute: '#484f58' },
  'accent':     { DEFAULT: '#2f81f7', 50: '#2f81f7/8' },
  'ok':         '#3fb950',
  'warn':       '#d29922',
  'err':        '#f85149',
  'info':       '#58a6ff',
},
fontSize: {
  'micro':      '11px',
  'label':      '12px',
  'body':       '13px',
  'body-lg':    '15px',
  'title':      '16px',
  'title-lg':   '20px',
  'display':    '32px',
  'display-lg': '48px',
},
fontFamily: {
  'sans': 'Inter, ui-sans-serif, system-ui',
  'mono': 'JetBrains Mono, ui-monospace, SFMono-Regular',
}
```

## §8 5 worker 并行实装 (per 8/27 19:39/21:59 + 4-5 模式)

| Worker | 文件 | 工作量 |
|---|---|---|
| **U1 AppShell** | `app/layout.tsx` + `components/AppShell.tsx` + `components/AppHeader.tsx` | 0.6M tokens |
| **U2 SubNav + Issues 主面板** | `components/SubNav.tsx` + `app/(app)/issues/page.tsx` (Kanban+List+Tree) | 0.7M |
| **U3 Projects 多 panel** | `app/(app)/projects/page.tsx` (5 tab) | 0.5M |
| **U4 Agents / Analytics / Inbox / Settings** | 4 个 page.tsx (拆 4 个) | 0.7M |
| **U5 CommandBar + token + 路由合并** | `components/CommandBar.tsx` + `tailwind.config.ts` 扩展 + 旧 22 路由 redirect | 0.5M |

### 顺序
1. U5 改 tailwind.config.ts (基础) + 路由 redirect (其他 worker 依赖)
2. U1 + U2 + U3 + U4 并发 (4 个 wt 互不影响)

## §9 守门 (per AGENTS.md §4 12 项 + §10.2)

1. 0 unsafe
2. 代签 (per 8/27 19:39/21:59)
3. 5 域独立 (per 8/21)
4. 缺标比错标: 3-5 项已知缺口
5. token-OLU (per 8/21): 单 worker 0.5-0.7M
6. 不沿用 bc23d6c 叙事

## §10 已知缺口

1. 22 路由 → 6 路由需 redirect (Phase I+)
2. dark mode 优先, light mode 待后 (multica 同)
3. mobile 响应式 (per G3 touch 后续)
4. i18n (multica 暂仅 en, Star 暂 en, 后 i18n)
5. PWA / desktop app (multica 都有, Star 暂仅 web)

---

**审批者**: 架构师 (Mavis 接手 agent per DEC-008) — 2026-08-28
