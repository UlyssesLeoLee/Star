# Star Desktop — Tauri P5 前端复用 KanbanBoard 实施计划

> **date**: 2026-09-30 JST
> **author**: Ulysses (一人公司 12 角色 per DEC-008) — minimax-agent
> **trigger**: PR #244 follow-up of PR #243 (Tauri P3 docs) + PR-#241 (P2 adapters)
> **status**: 设计阶段 (本 PR) → 实施留 PR-245+

## 1. 目标

把现有 frontend/src/components/board/KanbanBoard.tsx (521 LOC, W/T/M swimlane) 嵌入 Tauri 2.0 webview。

## 2. 现状分析

### 2.1 frontend/src/components/board/ 现有资源

| 文件 | LOC | 用途 |
|---|---|---|
| KanbanBoard.tsx | 521 | Kanban 看板列容器 (4 列 × W/T/M swimlane) |
| KanbanCard.tsx | - | 卡片组件 |
| board 目录其他 | - | 共 6+ React 组件 |

### 2.2 frontend/ vs crates/star-desktop/frontend/ 对比

| 维度 | frontend/ (Next.js) | crates/star-desktop/frontend/ (Tauri) |
|---|---|---|
| Framework | Next.js 14 App Router | Vite + vanilla TS |
| Routing | /board, /worktree-canvas | Tauri webview (无路由) |
| State | Zustand store | React 19 (待 P5+ 集成) |
| Data | fetch /api/v1/* | Tauri IPC `invoke('cmd')` |
| Components | KanbanBoard + 10+ | 1 page mock (P0-P3) |

### 2.3 集成挑战

1. **Framework 迁移**: KanbanBoard.tsx 强依赖 `use client` + Next.js dynamic import → Tauri 需 React 19 + Vite + React Router
2. **Data source 切换**: fetch → Tauri IPC (8 commands 已在 PR #239-#241 落地)
3. **CSS**: KanbanBoard.tsx 用 Tailwind, Vite 已配 (per PR #239 frontend/vite.config.ts)
4. **WASM hooks**: KanbanBoard.tsx 当前不用 WASM, 但 P5+ 可集成 useLayoutEngine/useQueryEngine

## 3. 实施方案

### 3.1 阶段 1 (PR-245): React + Vite skeleton

- crates/star-desktop/frontend/package.json: + react@19, react-dom@19, react-router-dom@7, @types/react@19
- crates/star-desktop/frontend/vite.config.ts: 加 React plugin + dev server 端口 1420
- crates/star-desktop/frontend/index.html: 加 root div + module script
- crates/star-desktop/frontend/src/main.tsx: ReactDOM.createRoot + React Router

估: 1 天

### 3.2 阶段 2 (PR-246): KanbanBoard.tsx 复制 + 适配

- crates/star-desktop/frontend/src/components/KanbanBoard.tsx: 复制 frontend/src/components/board/KanbanBoard.tsx (521 LOC) → 适配 Vite + Tauri IPC
- crates/star-desktop/frontend/src/components/KanbanCard.tsx: 同上
- crates/star-desktop/frontend/src/types/ids.ts: 复制 WorkItemStatus + Wtm enum + WorkItem struct
- crates/star-desktop/frontend/src/lib/wasm/layout-engine.ts (NEW): 复用 frontend/src/lib/wasm/layout-engine.ts (per PR #229)
- crates/star-desktop/frontend/src/lib/wasm/query-engine.ts (NEW): 复用 frontend/src/lib/wasm/query-engine.ts (per PR #231)

估: 3 天

### 3.3 阶段 3 (PR-247): Tauri IPC 数据接入

- crates/star-desktop/frontend/src/hooks/useTauriWorkItems.ts (NEW): invoke('list_work_items') + 实时更新 (tauri::Window listen)
- crates/star-desktop/frontend/src/hooks/useTauriBoardInfo.ts (NEW): invoke('get_board_info')
- crates/star-desktop/frontend/src/hooks/useTauriKeyboardLayout.ts (NEW): invoke('get_keyboard_layout')

估: 2 天

### 3.4 阶段 4 (PR-248): WASM lazy load 集成

- crates/star-desktop/frontend/src/lib/wasm/loader.ts (NEW): 复用 frontend/src/lib/wasm/loader.ts (per PR #229)
- crates/star-desktop/frontend/src/components/BoardView.tsx (NEW): useLayoutEngine() + useQueryEngine() 集成

估: 2 天

### 3.5 阶段 5 (PR-249): 集成测试

- crates/star-desktop/frontend/tests/board.test.tsx (NEW): vitest 测试 BoardView 渲染 + IPC mock + WASM fallback
- crates/star-desktop/frontend/tests/setup.ts (NEW): Tauri IPC mock (per @tauri-apps/api mock)

估: 1 天

## 4. B-mini 守门

| 守门 | 状态 |
|---|---|
| **#7 unsafe_code = "forbid"** | ✅ Tauri Rust 部分 (PR #239 落地) |
| **#11 缺标比错标** | ⏳ 前端组件需逐文件同步 |
| **#19 0 动 V0.1 业务 logic** | ✅ 复制 frontend/src/components/board/KanbanBoard.tsx 不动 V0.1 (Tauri 端是副本) |
| **#14 v4 PR review** | ⏳ 5 PRs 实施需 review |

## 5. 风险

| Risk | Mitigation |
|---|---|
| KanbanBoard.tsx 强依赖 Next.js dynamic import | 移除 dynamic import, 改 Vite eager import |
| React 19 + Tailwind 兼容 | 使用 React 19 + Tailwind 3 (已验证兼容 per Vite ecosystem) |
| Tauri IPC vs fetch 延迟差异 | Tauri IPC local 0ms, fetch 网络 ~5-10ms, 体验更好 |
| WASM bundle size | layout-engine-wasm + query-engine-wasm 总 ~6MB, Tauri 接受 |

## 6. 总估

- 阶段 1-5 累计: 1 + 3 + 2 + 2 + 1 = **9 天**
- 加 review + polish: **2 周**

## 7. 不在 PR-244 范围 (留 PR-245+)

- ❌ Tauri build 验证 (需 Linux CI per PR #242, 等 CI 跑出结果)
- ❌ W/T/M swimlane 真接 crates/domain-board (PR-245+ 接 database)
- ❌ Tauri 真实 IPC 接 crates/domain-worktree + crates/canvas-engine (PR-245+)
- ❌ E2E 测试 (5 services + Tauri 集成, 留 PR-249+)

## 8. Refs

- [docs/architecture/2026-09-30-upgrade/star-desktop-p0p3-impl-summary.md](../2026-09-30-upgrade/star-desktop-p0p3-impl-summary.md)
- [docs/architecture/2026-09-29-upgrade/rust-app-end-research.md §3.2](../../architecture/2026-09-29-upgrade/rust-app-end-research.md)
- PR #229 [WASM frontend hook](https://github.com/UlyssesLeoLee/Star/pull/229)
- PR #231 [query-engine-wasm frontend React hook](https://github.com/UlyssesLeoLee/Star/pull/231)
- PR #234 [KafkaBoardLayout demo page](https://github.com/UlyssesLeoLee/Star/pull/234)
- frontend/src/components/board/KanbanBoard.tsx (521 LOC)
- crates/star-desktop/README.md (Tauri 架构 + 实施步骤)
