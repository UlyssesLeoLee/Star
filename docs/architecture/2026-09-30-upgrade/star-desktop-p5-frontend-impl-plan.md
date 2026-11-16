# Star Desktop — Tauri P5 前端复用 KanbanBoard 实施计划

> **date**: 2026-09-30 JST (首版); 状态核对 2026-11-16。
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent (commit 正文 Co-authored-by 记录)
> **trigger**: PR #244 follow-up of PR #243 (Tauri P3 docs) + PR-#241 (P2 adapters)
> **核对基线**: 工作区 dev (HEAD `80830732` + 未提交改动), 2026-11-16; Routing 行含 e3636462 (2026-11-13) 的路由迁移结果。
> **status**: 设计阶段已完成 (PR #244, e994e362); 实施已由后续 PR 部分落地: #246 React+Vite 骨架 (df099df1)、#247 类型与常量 (b8f0a18a)、#248 KanbanCard 移植 (508e7fee)、#249 KanbanBoard 移植 (630ec19c)、#250 Tauri IPC hooks (afbed082)、#251 WASM hooks (0a596d8e)、#252 BoardView 集成 (a1616b5a)。计划中的 loader.ts 未创建, .wasm 产物未打包, 集成测试未按计划文件名落地; 逐阶段实际状态见 §3.0。

## 1. 目标

把 Tauri 端 KanbanBoard 从根目录 frontend/src/components/board/KanbanBoard.tsx (525 行, W/T/M swimlane) 适配为 crates/star-desktop/frontend/src/components/KanbanBoard.tsx (243 行, 630ec19c 落地), 不是把根目录文件原样嵌入; 未移植根目录的列增删改名 (onAddColumn、onRemoveColumn、onRenameColumn), 未分类项的处理与根目录不同 (桌面端落入 W 泳道); WIP 限制提示已保留。

## 2. 现状分析

> 下表各行以 2026-11-16 的工作区代码为准; LOC 为 wc -l 结果。

### 2.1 frontend/src/components/board/ 现有资源

| 文件 | LOC | 用途 |
|---|---|---|
| KanbanBoard.tsx | 580 | Kanban 看板列容器 (默认 5 列, 来自 frontend/src/mocks/data/kanban.ts 的 KANBAN_COLUMNS: todo / in_progress / review / blocked / done; × W/T/M swimlane); 无授权数据时渲染 StatusNotice 诚实态, 不渲染本地 seed 任务卡 |
| KanbanCard.tsx | 234 | 卡片组件 (桌面端对应 crates/star-desktop/frontend/src/components/KanbanCard.tsx 159 行) |
| board 目录其他 | - | 共 4 个 React 组件 (ArchGraphModal / KanbanFilters / TaskCardCommand / WorkItemDetailDrawer)；桌面 components 目录共 3 个 (BoardView / KanbanBoard / KanbanCard) |

### 2.2 frontend/ vs crates/star-desktop/frontend/ 对比

| 维度 | frontend/ (Next.js) | crates/star-desktop/frontend/ (Tauri) |
|---|---|---|
| Framework | Next.js 16 (frontend/package.json 当前 next ^16.4.6) App Router | Vite 7 + React 19 + TypeScript (TSX)；入口 crates/star-desktop/frontend/src/main.tsx 使用 createBrowserRouter。遗留 main.ts 仅为 6 行注释占位。（P5 计划的 React 19 + Vite 骨架已由 df099df1 / #246 落地） |
| Routing | /board (`app/board/page.tsx`)、/kanban (`app/(app)/kanban/page.tsx`)、/canvas/worktree-graph (原 `(worktree-canvas)` 路由组已于 e3636462 迁移; 旧路径 /worktree-canvas 无 redirect 条目, 该路由的 not-found 页仍链接旧路径, 为死链, 待修) | React Router 7（react-router-dom，createBrowserRouter）路由：/、/worktree、/viz（crates/star-desktop/frontend/src/main.tsx） |
| State | Zustand store (全局状态, seed + productionApi); TanStack Query (useBoardSync 以 useQuery + refetchInterval 轮询 productionApi.boardSync) | React 19 (已集成: crates/star-desktop/frontend/package.json react ^19.0.0, 由 df099df1 (PR #246) 引入; 看板为桌面端内 243 行适配副本, 未直接引用根目录 KanbanBoard.tsx) |
| Data | 看板页面: zustand store 与 useBoardSync (TanStack Query, 2000 ms 轮询, 仅在宿主数据通道已接入且会话未失效时启用); 宿主未接入、会话未授权或无授权数据时显示诚实态 (未接入 / 未授权 / 暂无任务数据), 不渲染本地 seed 任务卡; lib/api.ts boardSync 同步读取 store, 无 fetch; 未接入 /api/v1; 部分 lib 模块 fetch /api/v1/*; 看板任务卡 CLI 提交时经 useTerminalStackWs 调用 localhost:8081 /api/v1/terminal/*; components/board/ArchGraphModal 调用 /api/graph/ensure-fresh 与 /api/graph/cypher | Tauri IPC `invoke('cmd')` (前端仅 3 处调用: useTauriWorkItems → list_work_items, 由 BoardView 使用, 失败即关闭; useTauriBoardInfo → get_board_info 与 useTauriKeyboardLayout → get_keyboard_layout, 两个 hook 当前无组件引用) |
| Components | KanbanBoard + 22 个组件（/kanban 路由闭包 23 个组件文件；/board 路由单独闭包为 9 个组件文件） | 3 个页面: HomePage (8 条 IPC 命令的静态清单)、WorktreePage (BoardView 经 Tauri IPC, 失败即关闭, 无 mock 兜底)、WorktreeVizPage (mock 节点 + useLayoutEngine 布局; WASM 未打包, 回退 JS) |

### 2.3 集成挑战

1. **Framework 迁移**: KanbanBoard.tsx 文件顶部带有 'use client' 指令 (第 12 行), 但本身不引入 Next 模块。Next 耦合实际来自两处: KanbanCard.tsx 使用 next/navigation 的 useRouter 做卡片跳转, 以及经 KanbanBoard 间接引入的 GasParticlesHint.tsx 所用的 next/dynamic (ssr: false)。Tauri 副本已移除这两处 (KanbanCard 改用 react-router-dom 的 useNavigate, KanbanBoard 不引入 GasParticlesHint), 迁移只需 React 19 + Vite + React Router。
2. **Data source 切换**: fetch → Tauri IPC (8 个命令已在 PR #239-#241 落地并注册于 generate_handler, 见 lib.rs:306-315; generate_handler 自 PR #240 (81b03503) 起存在); 前端实际仅 3 处 invoke: useTauriWorkItems → list_work_items (失败即关闭, 无 mock 兜底), useTauriBoardInfo → get_board_info, useTauriKeyboardLayout → get_keyboard_layout (后两者 hook 无组件引用); 其余 5 个命令无前端调用。
3. **CSS**: 根目录 KanbanBoard.tsx 所在 Next 应用使用 Tailwind 4 (frontend/package.json 声明 tailwindcss ^4.3.5)；Tauri 副本 (crates/star-desktop/frontend) 未声明 Tailwind 依赖, 样式入口为 App.module.css, 其 KanbanBoard 副本中的工具类名 (如 flex、gap-2、grid-cols-6) 的样式来源尚需确定 (CSS module 或引入 Tailwind)。Vite 配置位于 crates/star-desktop/frontend/vite.config.ts (PR #239 引入, port 1420 strictPort; PR #246 改为 React 插件配置)。
4. **WASM hooks**: KanbanBoard.tsx 本身不引入 WASM hook; WASM 集成已在上层接入: BoardView.tsx 调用 useLayoutEngine 与 useQueryEngine (PR #252), WorktreeVizPage 使用 useLayoutEngine (PR #270)。在 wasm-pack 产物放入 crates/star-desktop/frontend/public/wasm/ 之前, 这些 hook 不会加载 WASM 模块。

## 3. 实施方案

### 3.0 落地状态（2026-11-16 核对）

| 阶段 | 原计划 PR | 实际 PR / commit | 状态 | 未完成项 |
|---|---|---|---|---|
| 1 React + Vite skeleton | PR-245 | PR #246 df099df1 (PR #245 为 P4 DB Adapter, e8156b43) | 已落地 | 旧 PoC 入口 src/main.ts 仍保留在仓内, 未删除 |
| 2 KanbanBoard.tsx 复制 + 适配 | PR-246 | PR #247 b8f0a18a (类型)、PR #248 508e7fee (KanbanCard)、PR #249 630ec19c (KanbanBoard)、PR #251 0a596d8e (layout / query hooks) | 部分落地 | 桌面 KanbanBoard 为 243 行适配重写, 非逐行复制; 根目录 KanbanFilters、WorkItemDetailDrawer、TaskCardCommand、ArchGraphModal 未移植; 列增删改名未移植; 计划中的 lib/wasm/layout-engine.ts 与 query-engine.ts 未在桌面端创建 (由 hooks 替代) |
| 3 Tauri IPC 数据接入 | PR-247 | PR #250 afbed082 | 部分落地 | 实时更新 (tauri::Window listen) 未做 (前端无 listen, 后端无 emit); list_work_items 后端 fail closed, hook 无浏览器 mock 兜底 |
| 4 WASM lazy load 集成 | PR-248 | PR #251 0a596d8e (hooks)、PR #252 a1616b5a (BoardView) | 部分落地 | 无懒加载 (无 React.lazy 或动态 import); 计划中的 loader.ts 未创建 (加载逻辑内联于 hooks); layout / query 的 .wasm 未打包, 运行时回退 JS (ready 为 false) |
| 5 集成测试 | PR-249 | 测试分布于 df099df1 (tests/setup.ts)、630ec19c (KanbanBoard.test.tsx)、a1616b5a (BoardView.test.tsx) 及 IPC / WASM hook 测试 | 部分落地 | 无 tests/board.test.tsx; 无桌面 E2E; vitest 通过状态未在本次核查中运行 |

### 3.1 阶段 1（原计划 PR-245，实际 PR #246 df099df1）: React + Vite skeleton — 已落地

- crates/star-desktop/frontend/package.json: 阶段 1 (df099df1) 新增 react@^19.0.0、react-dom@^19.0.0、react-router-dom@^7.0.0、@types/react@^19.0.0、@types/react-dom@^19.0.0、clsx@^2.1.0、@testing-library/react@^16.0.0、jsdom@^25.0.0、vitest、@vitejs/plugin-react; vite 在 df099df1 时为 ^5.2.0, HEAD 为 ^7.3.5。
- crates/star-desktop/frontend/vite.config.ts: 加 @vitejs/plugin-react 与 vitest jsdom 测试配置 (df099df1); dev server 端口 1420 (strictPort) 为 PoC 既有配置 (3ad73ad7)。
- crates/star-desktop/frontend/index.html: 删除 PoC 页面结构, 新增 <div id="root"></div>; module script 由 /src/main.ts 改指 /src/main.tsx (df099df1)。
- crates/star-desktop/frontend/src/main.tsx: ReactDOM.createRoot + createBrowserRouter/RouterProvider。df099df1 同时改动 index.html、package.json、vite.config.ts、tsconfig.json、src/App.tsx、src/App.module.css、src/pages/HomePage.tsx、src/pages/WorktreePage.tsx、tests/setup.ts、tests/HomePage.test.tsx、tests/WorktreePage.test.tsx。
- 注 (main.tsx 头注释): 旧 PoC 入口 src/main.ts (6 行) 已不被 index.html 或任何 import 引用, 但仍保留在仓内, 尚未删除; 看板复用已由 PR #249 (630ec19c) 落地。
- 注: PR #245 (e8156b43) 为 P4 DB Adapter (trait 与 MockDb), 不属于本阶段。

估: 1 天（规划估算，仓内无实际工时记录）

### 3.2 阶段 2（原计划 PR-246，实际 PR #247 b8f0a18a / PR #248 508e7fee / PR #249 630ec19c / PR #251 0a596d8e）: KanbanBoard.tsx 复制 + 适配 — 部分落地（桌面端为适配重写，非逐行复制）

- crates/star-desktop/frontend/src/components/KanbanBoard.tsx: 已落地 (commit 630ec19c, PR #249), 由 frontend/src/components/board/KanbanBoard.tsx (当前 580 行) 移植重写为 243 行的 Vite/React 19 版本: 去除 "use client" 与 Next.js 依赖; 数据经 props (workItems) 注入, 组件本身不调用 Tauri IPC; IPC 由 BoardView.tsx 经 hooks/useTauriWorkItems.ts (invoke list_work_items) 提供并作为 props 传入。
- crates/star-desktop/frontend/src/components/KanbanCard.tsx: 已落地 (508e7fee, PR #248), 159 行移植版 (根目录 234 行): 以 react-router-dom 的 useNavigate 替代 next/navigation 的 useRouter, 去除 lucide-react 图标依赖与 StatusPill、i18n 等项目模块依赖; 展示型组件, 不直接调用 Tauri IPC (components/ 内无 invoke), 数据经 KanbanBoard props 传入。
- crates/star-desktop/frontend/src/types/ids.ts: 已落地 (b8f0a18a, PR #247): WorkItemStatus (6 值 type union, 与根目录一致)、Wtm (type alias 'W' | 'T' | 'M'; 根目录无同名类型, 对应根目录 WtmCategory 去掉 uncategorized)、WorkItem (interface; 字段名部分一致, 但 id 为 string、priority 为大写 P0-P3、assignee 与 w_t_m 的类型和可选性不同)。为 TypeScript 类型的移植, 非 enum/struct 逐字复制。
- crates/star-desktop/frontend/src/lib/wasm/layout-engine.ts 未创建 (桌面端无 src/lib/)。布局能力由 hooks/useLayoutEngine.ts 承接 (0a596d8e, PR #251), 为 Tauri 侧移植 (头注释: 源自根仓 frontend/src/lib/wasm/layout-engine.ts, 285 行), 非拷贝; 仓内无已提交的 layout-engine .wasm 产物, 运行时回退 JS (ready 为 false)。消费方: components/BoardView.tsx (算法选择与 WASM 状态显示) 与 pages/WorktreeVizPage.tsx (120 节点布局演示, dd045314, PR #270)。
- crates/star-desktop/frontend/src/lib/wasm/query-engine.ts 未创建。查询能力由 hooks/useQueryEngine.ts 承接 (0a596d8e, PR #251), 为 Tauri 侧移植 (源自根仓 query-engine.ts, 283 行), 仅导出 Keyword、parseDslKeywords 与 useQueryEngine; 仓内无 query-engine .wasm 产物, 运行时回退 JS。

估: 3 天（规划估算，仓内无实际工时记录）

### 3.3 阶段 3（原计划 PR-247，实际 PR #250 afbed082）: Tauri IPC 数据接入 — 部分落地（实时更新未做）

- crates/star-desktop/frontend/src/hooks/useTauriWorkItems.ts 已存在（afbed082，随 PR #250 落地，计划编号 PR-247）：通过 invoke('list_work_items') 拉取 Run-scoped WorkItem[]，并由 tests/useTauriWorkItems.test.ts 覆盖。实时更新（tauri::Window listen）仍是未完成项：前端未调用 listen，后端也未 emit。当前后端在未配置 Run provider 时 fail closed 返回错误（2f7ac2a3），真实 Tauri 运行时该 hook 进入 error 状态而非得到数据；该 hook 没有浏览器 mock 回退。
- crates/star-desktop/frontend/src/hooks/useTauriBoardInfo.ts 已实现（afbed082，PR #250）：通过 invoke('get_board_info') 获取 BoardInfo { board_kind_count, swimlane_group_by_count, default_column_count }；后端返回常量 3 / 4 / 6（src-tauri/src/lib.rs:244-250）；非 Tauri 运行时回退 MOCK_BOARD_INFO；由 tests/useTauriBoardInfo.test.ts 覆盖。
- crates/star-desktop/frontend/src/hooks/useTauriKeyboardLayout.ts 已实现（afbed082，PR #250；cd443f4c 修复构建与安全门禁）：通过 invoke('get_keyboard_layout') 获取 KeyboardLayout { swimlanes: W/T/M, statuses: 6 }；后端当前返回字面量（src-tauri/src/lib.rs:215-228）；非 Tauri 运行时回退 MOCK_LAYOUT；由 tests/useTauriKeyboardLayout.test.ts 覆盖。

估: 2 天

### 3.4 阶段 4（原计划 PR-248，实际 PR #251 0a596d8e / PR #252 a1616b5a）: WASM lazy load 集成 — 部分落地（未做懒加载）

- crates/star-desktop/frontend/src/lib/wasm/loader.ts 未创建: 根仓 frontend/src/lib/wasm/loader.ts 从未存在 (git 历史无此路径); PR #229 (fd7f770b) 仅新增 layout-engine.ts 与测试。WASM 加载逻辑 (fetch /wasm/*.wasm, 失败回退 JS) 已内联于 hooks/useLayoutEngine.ts 与 hooks/useQueryEngine.ts (0a596d8e), 仍无懒加载 (无 React.lazy 或动态 import); 尚未完成的是 wasm-pack 产物打包到 crates/star-desktop/frontend/public/wasm/ (桌面端无 public/ 目录)。
- crates/star-desktop/frontend/src/components/BoardView.tsx (a1616b5a, PR #252; 计划编号 PR-248): 已集成 useLayoutEngine() 与 useQueryEngine() (BoardView.tsx:18-19, 49-50), board-meta 显示 WASM 状态 (147-148 行)。WASM 懒加载未生效: 两个 hook fetch /wasm/layout_engine_wasm_bg.wasm 与 /wasm/query_engine_wasm_bg.wasm; 仓内无对应 .wasm 且桌面端无 public/ 目录, 运行时 ready 为 false, version 为 0.1.0-js-fallback, 实际走 JS 回退。阶段 4 的懒加载集成待完成。

估: 2 天

### 3.5 阶段 5（原计划 PR-249）: 集成测试 — 部分落地（无 board.test.tsx，无 E2E）

- tests/board.test.tsx 这一文件名未采用；对应覆盖为 crates/star-desktop/frontend/tests/BoardView.test.tsx（8 个用例，a1616b5a，PR #252）：覆盖工具栏渲染、board-meta、看板渲染三个工作项、视图切换、DSL show 过滤、Refresh 重新调用 IPC、drop 状态迁移，以及 IPC 失败时的 board-view-error 状态，IPC 通过 globalThis.__TAURI_INTERNALS__ 模拟。WASM 回退未在该文件中断言；JS 回退由 tests/useLayoutEngine.test.ts:46、tests/useQueryEngine.test.ts:43 与 tests/WorktreeVizPage.test.tsx:99-107 覆盖。
- crates/star-desktop/frontend/tests/setup.ts (已存在; 首次提交 df099df1, PR #246; 后经 1068ea40、2f7ac2a3、cd443f4c 修改), 由 vite.config.ts 的 setupFiles 注册 (jsdom 环境)。它是手写的 Tauri IPC 桩: 在 globalThis.__TAURI_INTERNALS__.invoke 上提供 mockInvoke, 覆盖 list_work_items、list_worktree_groups、get_app_version、get_keyboard_layout 四个命令, 其余命令 reject (包括 get_board_info); 不是 @tauri-apps/api 的 mockIPC 或模块级 vi.mock。

估: 1 天

## 4. B-mini 守门

| 守门 | 状态 |
|---|---|
| **#7 unsafe_code = forbid** | 已完成 — Tauri Rust 部分: PR #239 落地 workspace lint (根 Cargo.toml [workspace.lints.rust] unsafe_code = forbid; star-desktop 清单 [lints] workspace = true, 现位于 crates/star-desktop/src-tauri/Cargo.toml); crate 源码级 #![forbid(unsafe_code)] 随后续 PR 补齐: lib.rs (81b03503, #240)、ipc_adapter.rs (44e386bb, #241)、db_adapter.rs (e8156b43, #245)、main.rs (4de395d4) |
| **#11 缺标比错标** | 待完成: 前端组件仅部分移植。KanbanCard (508e7fee, 159 行, 根目录 234 行) 与 KanbanBoard (630ec19c, 243 行, 根目录 525 行) 已落地为 Tauri 端适配副本; 根目录 board 目录的 KanbanFilters、WorkItemDetailDrawer、TaskCardCommand、ArchGraphModal 未移植; 逐文件差异 (行数、列增删改名、未分类泳道) 需显式列出, 未列出的项不视为已同步。 |
| **#19 0 动 V0.1 业务 logic** | ⚠️ 非逐字副本: 根目录 frontend/src/components/board/KanbanBoard.tsx 未被 Tauri 端修改; Tauri 端 crates/star-desktop/frontend/src/components/KanbanBoard.tsx 为 243 行的适配重写, 而非 525 行原件副本: 保留 W/T/M 泳道与 WIP 超限提示, 列集合为 6 列 (含 wontfix; 根目录 KANBAN_COLUMNS 为 5 列), 未分类项落入 W 泳道, 不含 StatusPill、Tooltip、粒子提示与 i18n; 根目录该文件此后有 3 次提交 (08b03c2d 主题 token、d3370e65 测试对齐、f1d2fa40 文案去除第三方产品名), 工作区另新增无授权数据诚实态 (unavailableState); 业务逻辑差异未逐行核对。 |
| **#14 v4 PR review** | 无正式评审记录: PR #244 已于 2026-09-30 合并, GitHub 正式 review 数为 0 (可用 gh api repos/UlyssesLeoLee/Star/pulls/244/reviews 核验), 仅有 bot 评论; P5 实施 PR #246 至 #252 已全部合并 (df099df1 至 a1616b5a), 同样无正式 review。仓库不保存评审结论, 评审状态以 GitHub 记录为准。 |

## 5. 风险

| Risk | Mitigation |
|---|---|
| Next.js dynamic import 依赖 (已消除) | KanbanBoard.tsx 本身不使用 dynamic import; Next 的 dynamic (ssr: false) 位于 GasParticlesHint.tsx, 由根 KanbanBoard 间接引入。Tauri 副本不引入该组件, 风险已消除, 无需额外的移除步骤。 |
| React 19 + Tailwind 兼容 | 仓内无 React 19 + Tailwind 3 的构建或测试证据。根 Next 应用使用 React 19 与 Tailwind 4 (未使用 Tailwind 3); Tauri 前端 (React 19 + Vite 7) 当前未安装 Tailwind, 其样式方案待定, 兼容性尚未在 Vite 环境中验证。 |
| Tauri IPC vs fetch 延迟差异（未测量） | 仓内无 Tauri IPC 与 fetch 的延迟测量记录; 需在 P7 性能基线中以同一主机上的 invoke 往返与 HTTP 请求实测后替换。 |
| WASM bundle size | CI 实测 (wasm-build 运行 43604213157, 2026-11-14, 未压缩): layout_engine_wasm_bg.wasm 17,401 字节、query_engine_wasm_bg.wasm 157,559 字节, 合计约 175 KB; relationship_engine_wasm_bg.wasm 321,041 字节, 三模块合计约 496 KB。其后两次构建运行 (43819223631、43860875844, 均基于 HEAD 80830732) 失败, 上述尺寸来自 2026-11-14 的构建; 当前体积以新日志为准。 |

## 6. 总估

- 阶段 1-5 累计: 1 + 3 + 2 + 2 + 1 = **9 天**
- 加 review + polish: **2 周**

## 7. 不在 PR-244 范围 (未排期的后续项; 原标注 留 PR-245+ 已不符, 见 §3.0)

> 注: 以下各项均为未排期的后续项; 原标注的 PR 编号已与实际 PR 不符, 见 §3.0。

- ⏳ 待确认: Tauri build 结果。桌面 cargo check/test 由 ci.yml 的 desktop-rust job 覆盖 (仅在 desktop 变更时运行); 打包 (deb/AppImage、msi、dmg) 由 star-stable-desktop.yml 覆盖; PR #242 的独立 Linux workflow 已于 32fa4a8c 删除。仓内 CI 记录: ci.yml:23 注释引用 run 43329468929 (star-desktop 因缺 glib-2.0 失败); ci.yml:339 注释称同一 run 的前端三步通过; 仓内未留存桌面构建的通过记录, 结果以 GitHub Actions 为准。
- ❌ 待完成: W/T/M swimlane 接入 crates/domain-board 的持久化数据 (PR #245 (e8156b43) 仅落地 DbAdapter trait 与 MockDb; 桌面端生产代码当前未引用 crates/domain-board, get_board_info 返回硬编码常量 3/4/6, domain-board 仅在 src-tauri/src/ipc_adapter.rs 的 #[cfg(test)] 中被引用)。
- ❌ 待完成: Tauri 真实 IPC 接入 crates/domain-worktree 与 crates/canvas-engine (list_worktree_groups、list_canvas_entities 仍返回 MockDb 数据); src-tauri/Cargo.toml 当前未声明 canvas-engine 依赖, 接入时需新增; 后续 PR 接入。
- ❌ 待完成: 桌面端 Tauri E2E 测试 (5 services + Tauri 集成; 未实现, CI 仅运行 vitest 单元测试与打包)。PR #249 (630ec19c) 为 KanbanBoard 移植, 不含 E2E, 原标注的 'PR-249+' 标签应删除。仓内 scripts/verify-star-services.sh (169f236f) 为需人工指定 K3s 上下文的三服务健康检查 (canvas-engine、domain-canvas、canvas-realtime), 不替代 E2E; 旧五服务脚本 verify-5-services.sh (e44d02f4) 已由其替换。

## 8. Refs

- [docs/architecture/2026-09-30-upgrade/star-desktop-p0p3-impl-summary.md](../2026-09-30-upgrade/star-desktop-p0p3-impl-summary.md)
- [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2](../../architecture/2026-09-29-upgrade/rust-app-end-research.md)
- PR #229 [WASM frontend hook](https://github.com/UlyssesLeoLee/Star/pull/229)
- PR #231 [query-engine React hook](https://github.com/UlyssesLeoLee/Star/pull/231)
- PR #234 [/wasm-demo 演示页](https://github.com/UlyssesLeoLee/Star/pull/234) — 09c8ee2f 新增 frontend/src/app/wasm-demo/page.tsx 与测试, 演示 layout-engine / query-engine WASM 的布局算法选择; 代码与提交历史中不存在 KafkaBoardLayout (仅本计划文档提及该名称)。
- frontend/src/components/board/KanbanBoard.tsx (580 LOC)
- crates/star-desktop/README.md (Tauri 2.0 PoC P0→P10 架构 + IPC 命令 + 已完成步骤记录; 不含本计划的 React/Vite 移植步骤)
