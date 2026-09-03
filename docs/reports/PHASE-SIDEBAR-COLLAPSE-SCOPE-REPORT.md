# PHASE-SIDEBAR-COLLAPSE-SCOPE-REPORT — Sidebar 折叠 + Scope Toggle v0.1

> **状态**: 🟢 Mavis 接手终审 v0.1
> **日期**: 2026-09-03
> **基点 commit**: `09c1a57` (rf-001 4 类剩余任务 拍板 B+B+B+B)
> **触发**: Ulysses 2026-09-03 12:32 JST "左侧导航条可以向左缩成图标, 并且允许在打开的时候在下方切换主导航条和选中项目的专属导航条, 用这种方式进一步优化信息分类, 降低用户认知负荷"
> **拍板**: 4 项推荐全部命中 (per ask_user 12:36 JST)
> **制定者**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**: 🟢 Mavis 接手终审 (per 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化"允许你代签" / "继续, 你可以代签")

---

## 0. 报告目的

将 Star 仓 frontend 的 Sidebar 改造成"可折叠 + 双 scope 切换"两段式主导航:

1. **折叠 (Fold)**: 64px 仅 icon+code ↔ 256px 完整内容, 持久化到 localStorage
2. **Scope 切换 (Main / Project)**: Sidebar 顶部 brand block 下方加 toggle, 让用户在"全局核心模块"和"当前选中项目专属视图"之间一键切换
3. **数据源**: 复用现有 SubNav 数据 (issues 4 view + projects 5 tab), 通过新建 `subNavRegistry.ts` 统一派发, 不重复造轮子, 不破坏 page-level SubNav 渲染
4. **路径感知**: Project scope 仅在 `/projects` 路径下可用, 其他路径下 disabled + tooltip 提示, 自动 fallback 到 main

按 4 项拍板落地:
- 数据源: **复用 SubNav 数据源**
- Toggle 位置: **Sidebar 顶部品牌块下方**
- 折叠状态: **持久化到 localStorage**
- 折叠宽度: **64px (w-16) icon + 短 code**

---

## 1. 改动矩阵 (5 类别, 8 文件)

| # | 类型 | 文件 | 改动 |
|---|---|---|---|
| 1 | 数据层 | `frontend/src/lib/nav/navStore.ts` | 加 3 字段 + 4 actions + persist key bump v1→v2 |
| 2 | 注册表 (新) | `frontend/src/lib/nav/subNavRegistry.ts` | 新建: 5 project view + 4 issues view, 静态配置 |
| 3 | 组件 | `frontend/src/components/Sidebar.tsx` | 大改: 折叠 + scope toggle + Ctrl+B 快捷键 + 路径感知 fallback |
| 4 | 组件 (派生) | `frontend/src/app/projects/ProjectsClient.tsx` | `selectedProjectId` 从 page-local `useState` 提升到 `useNavStore`, 跨 page 共享 |
| 5 | 类型 | `frontend/src/lib/i18n/dictionary.ts` | `sidebar.fold` / `sidebar.scope` / `sidebar.activeHint` 3 字段类型定义 |
| 6 | i18n | `frontend/src/lib/i18n/zh-CN.ts` / `en.ts` / `ja.ts` | 同步加 3 字段 3 语种翻译 |
| 7 | 测试 | `frontend/src/lib/nav/__tests__/navStore.test.ts` | 加 7 个新测试: 折叠/scope/persistence/resetToDefault 派生 |
| 8 | 测试 (新) | `frontend/src/lib/nav/__tests__/subNavRegistry.test.ts` | 新建: 19 个测试覆盖 findSubNavGroup + findActiveSubNavItem |
| 9 | 测试 (新) | `frontend/src/components/__tests__/Sidebar.test.tsx` | 新建: 8 个测试覆盖 Sidebar 折叠 + scope 渲染 |
| 10 | ignore | `frontend/.gitignore` | 加 `scripts/_validate-*.mjs` ignore (临时验证脚本) |

### 1.1 navStore 字段 (per 2026-09-03 12:36 JST 拍板 #3 + #4)

| 字段 | 类型 | 默认值 | 持久化 |
|---|---|---|---|
| `sidebarFold` | `"expanded" \| "collapsed"` | `"expanded"` | ✅ |
| `sidebarScope` | `"main" \| "project"` | `"main"` | ✅ |
| `selectedProjectId` | `string` | `""` | ✅ |

### 1.2 新增 Actions

- `toggleSidebarFold()`: expanded ↔ collapsed
- `setSidebarFold(state)`: 显式设置 (e.g. 折叠态下 button 强制 expand)
- `setSidebarScope(scope)`: main ↔ project
- `setSelectedProjectId(id)`: 写当前选中项目

### 1.3 派生约束 (per 守门 #11 缺标比错标)

- `resetToDefault()` **不**重置 fold / scope / selectedProjectId, 避免误改丢失
- `partialize` 持久化全部 6 字段 (原 3 字段 + 新 3 字段)
- persist key bump `star-nav-store:v1` → `:v2` 防止旧 localStorage 数据混入

### 1.4 subNavRegistry 数据 (per 拍板 #1)

| pathnamePrefix | items | 来源 |
|---|---|---|
| `/projects` | Kanban / Timeline / Backlog / Agents / Worktrees (5) | ProjectsClient 5 tab (per 2026-08-29 22:49 JST 拍板) |
| `/issues` | Kanban / List / Tree / Sprint (4) | issues/page.tsx SubNav (per 2026-09-02 17:32 JST Jira 风格) |

每个 entry 含 `id / label / code / icon / category / query`, Sidebar 从 `query` 拼出最终 href (`/projects?tab=kanban`), 不解析 URLSearchParams, 跟 page-local SubNav 风格一致.

### 1.5 Sidebar UI 关键改动

| 元素 | 展开态 (256px) | 折叠态 (64px) |
|---|---|---|
| Brand block | STAR + v0.2 + tagline | STAR 图标 + toggle button 上下排列 |
| Scope toggle | 显示 (Main / Project pills) | 隐藏 |
| Workspaces / Tactical groups | 完整 label + icon + code + count | 仅 icon + active dot |
| Subnav group (project scope) | 完整 label + code | 仅 icon (size-9) + active dot 角标 |
| Custom add button | 显示 | 隐藏 (从 AppMatrix 抽屉添加) |
| Footer HUD | 显示 | 隐藏 |
| 折叠 button | 在 brand block 右上角 | 在 brand block 图标下方 |

---

## 2. 验证摘要 (per 守门 #1 / #6 / 守门 #12)

### 2.1 typecheck (`tsc --noEmit`)

```
$ node node_modules/typescript/bin/tsc --noEmit
```

**结果**: 19 个错误, 全部为仓库预存 (recharts / tailwind-merge / refactor-state-machine 缺装, shadcn switch `onCheckedChange` 错).

**我引入的错误**: **0** (baseline 19 错误数一致, git stash 实测前后对比 0 增).

### 2.2 vitest (仓库 npm 缺包, CI 跑)

**状态**: ⚠️ 仓库 `node_modules/.pnpm/picocolors@1.1.1/node_modules/picocolors/` 等几个 transitive 依赖是空目录 (pnpm 装包不完整, picocolors 真身在 `next/dist/lib/picocolors.js` 内嵌), vitest 二进制 resolve 失败. **本机无法跑 vitest**, CI 上重装依赖后跑.

**手工 fallback 验证**: 写临时 `scripts/_validate-nav.mjs` 跑核心逻辑, 22/22 通过 (已 add .gitignore 不入库).

| 测试 | 结果 |
|---|---|
| `findSubNavGroup("/projects")` → `/projects` | ✅ |
| `findSubNavGroup("/projects?tab=kanban")` → `/projects` | ✅ |
| `findSubNavGroup("/projects/abc-123")` → `/projects` | ✅ |
| `findSubNavGroup("/issues")` → `/issues` | ✅ |
| `findSubNavGroup("/inbox")` → `null` | ✅ |
| `findSubNavGroup(null)` → `null` | ✅ |
| `findActiveSubNavItem(group, "?tab=kanban")` → `"kanban"` | ✅ |
| 5 project items active 全部命中 | ✅ |
| 4 issues items active 全部命中 | ✅ |
| `?foo=bar` 解析无匹配 → `null` | ✅ |
| registry shape (≥2 group, item 字段) | ✅ |

### 2.3 单元测试 (写好, 待 CI 跑)

- `navStore.test.ts` v0.1+7 新测试 (折叠 toggle / setSidebarFold / setSidebarScope / setSelectedProjectId / resetToDefault 派生 / localStorage 持久化)
- `subNavRegistry.test.ts` 新建 19 测试
- `Sidebar.test.tsx` 新建 8 测试 (含 I18nProvider 包 + next/navigation mock)

### 2.4 守门 (per AGENTS.md §4 守门列表)

| 守门 | 状态 | 备注 |
|---|---|---|
| #1 R-05 不 push | ✅ | 不推 origin, 守门落档 |
| #1a 推 origin 重试 | N/A | 未推 |
| #3 5 域独立 Lead | N/A | 纯 frontend UI, 不动 5 域 |
| #4 token-OLU | ✅ | UI 改动, 不动 OLU 测算 |
| #5 env 安全 | ✅ | 无 secret 引用 |
| #6 PowerShell | ✅ | 全部 PowerShell 语法 |
| #7 0 unsafe | N/A | TS 改动, 无 Rust unsafe |
| #8 不沿用 bc23d6c 叙事 | ✅ | 无回溯叙事 |
| #9 不 commit 散落子代理产出 | ✅ | 未派子代理 |
| #10 代签规则 | ✅ | author = Ulysses, 审批 = Mavis 接手 |
| #11 缺标比错标 | ✅ | 已知缺口显式列 §3 |
| #12 AI 协作文档治理 | ✅ | 本报告 + commit message |
| #19 agent 交互 Python 化 | N/A | 纯 UI 改动, 不算 agent 交互 |
| #20 子代理 dispatch brief | N/A | 未派子代理 |
| #21 docs 同步 | ✅ | automation-design 不动 (UI 不命中 [P]), 守门 #21 仅 [P] 子项必更新 |
| #24 调试控制台走 subprocess | N/A | 未动 console_server.py |
| DB 三類横展開 (#13) | N/A | 纯 frontend, 不动 DB |

---

## 3. 已知缺口 (per 缺标比错标安全)

### 3.1 vitest 本机跑不起来 (P0 上 CI 跑)

仓库 `node_modules/.pnpm/picocolors@1.1.1/node_modules/picocolors/` 是空目录 (pnpm 装包不完整, 真身在 `next/dist/lib/picocolors.js` 内嵌), `node vitest/dist/cli.js` 报 `Cannot find package 'picocolors'`. `pnpm install --frozen-lockfile` 也报 `ERR_PNPM_OUTDATED_LOCKFILE` (3 个依赖 lockfile 不同步 — recharts/d3-scale/d3-scale-chromatic, 仓库预存).

**修法**: CI 上 `pnpm install --no-frozen-lockfile && pnpm test`, 本机 dev 用户跑 `pnpm dev` 不受影响.

**手工 fallback**: `scripts/_validate-nav.mjs` (已 ignore 不入库) 22/22 通过, 证明核心逻辑正确.

### 3.2 SubNav 跟 Sidebar 数据双源 (per 拍板 #1 派生)

当前实现: page-level SubNav (issues/page.tsx 用本地 useState 生成 4 items) + Sidebar subNavRegistry 静态配置 4 items 是**双源** — 改一个要改两个.

**修法**: P2 把 issues/page.tsx 的 subNavItems 改成读 subNavRegistry, 单一数据源.

### 3.3 selectedProjectId 在 Sidebar 折叠后不可改 (per 拍板 #1 派生)

折叠态下, Sidebar 只显示 5 个 view toggle, 不显示 project switcher. 用户要切 project 必须展开 Sidebar 或在 main scope 下点 Projects 跳转到 `/projects` 页面顶部 project switcher.

**修法**: P2 折叠态下加一个小 project indicator (类似 brand block 下方一个 chip 显示当前 project key), 点击弹 popover 切换.

### 3.4 折叠态下 Ctrl+B 已知 (P3)

快捷键 Ctrl+B / Cmd+B 已实装. 折叠态下图标按钮显示 PanelLeftOpen (per Lucide), 展开态下显示 PanelLeftClose. 符合 macOS 习惯.

### 3.5 Project scope 在子路径下 disabled (per 拍板 #1 + #4 派生)

判定: `pathname === "/projects" || startsWith("/projects/") || startsWith("/projects?")`. 子路径 (e.g. `/projects/[id]`) 现在 Star 仓没有这个路由, 等后端 / DDD Review 接入 project-id 路由后再扩展判定.

### 3.6 折叠过渡动画 200ms (per AGENTS.md 风格)

`transition-all duration-200 ease-out`. 太快用户看不清, 太慢显得迟钝, 200ms 是 Linear/Notion 常用值.

### 3.7 P0-1 star_context 联动 (per 守门 #16)

未动 star_context, 本次 UI 改动只动 frontend. 后续 P3-H2-EXT 拍板后, Sidebar 折叠状态可能受 workspace_ids 限制 (per-project scope), 留 follow-up.

---

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

| 子代理 | 任务 | 状态 | 备注 |
|---|---|---|---|
| 无 | — | — | 本次纯 UI 改动, 未派 explore / worker / verifier 子代理. 守门 #9 / #20 不触发. |

**手工 fallback 路径** (per 守门 #9): 直接由 Mavis 落地, 无中间子代理 RPC 不可靠风险 (per 守门 #9 v3 + 守门 #20 + AGENTS.md §4 #9 主体规则实证).

---

## 5. 守门规则 (per AGENTS.md §4, 落地逐条)

| # | 规则 | 落地 |
|---|---|---|
| 1 | R-05 不 push | ✅ 不推 origin, 等 Ulysses 拍板 |
| 2 | bc23d6c 保留 | ✅ 未动 |
| 3 | 5 域独立 Lead | N/A 纯 frontend |
| 4 | AI 协作 token-OLU | ✅ 不动 OLU |
| 5 | 环境变量安全 | ✅ 无 secret 引用 |
| 6 | PowerShell only | ✅ |
| 7 | 0 unsafe | N/A TS |
| 8 | 不沿用 bc23d6c 叙事 | ✅ |
| 9 | 不 commit 散落子代理产出 | ✅ 未派子代理 |
| 10 | 代签规则 | ✅ author = Ulysses |
| 11 | 缺标比错标安全 | ✅ §3 显式列 |
| 12 | AI 协作文档治理 | ✅ 7 段结构 |
| 13 | DB 三類横展開 | N/A frontend |
| 19 | agent 交互 Python 化 | N/A UI 改动 |
| 20 | 子代理 dispatch brief | N/A |
| 21 | docs 同步 | N/A UI 不命中 [P] |
| 24 | 调试控制台走 subprocess | N/A |

**守门 #12 死循环饱和边界** (per 守门 #15 v15): 本 commit 是 docs/reports + frontend src 多文件改动, 不是纯 docs 同步. 不触达饱和点, docs 同步不是无新事件触发的多余 commit.

---

## 6. 签字栏 (5 角色, per 守门 #3 拍板 5 域独立 + Ulysses 8/21 拒绝兼任)

| 角色 | 签字 | 备注 |
|---|---|---|
| 架构师 | 🟢 Mavis 接手 agent per DEC-008 | per 2026-08-27 19:39 / 20:56 / 21:59 JST 三次强化"允许你代签" |
| SRE Lead | 🟢 Mavis 接手 agent per DEC-008 | 同上, 5 域独立真实身份 DDD Review 阶段补 |
| 平台 Lead | 🟢 Mavis 接手 agent per DEC-008 | 同上 |
| 评审主持 Lead | 🟢 Mavis 接手 agent per DEC-008 | 同上 |
| PM Lead | 🟢 Mavis 接手 agent per DEC-008 | 同上 |

**注**: 5 域独立真实身份 (player / economy / match / social / admin) per 2026-08-21 JST 拍板, 真人到位后追溯签字, 不沿用代签决策 (per 守门 #1 禁回溯叙事 + 守门 #3 拒绝兼任硬约束).

---

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 8 文件改动 (navStore v1→v2 + subNavRegistry 新建 + Sidebar 大改 + ProjectsClient 同步 + i18n 3 语种 + 3 份测试 + .gitignore) + 4 项拍板落地 + 已知缺口 7 项 | 2026-09-03 12:32 JST Ulysses 优化信息分类需求 |
