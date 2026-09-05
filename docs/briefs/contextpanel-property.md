# Brief: ContextPanel Tab 1 — 属性 (worker-1 / cp-tab-property)

> **per AGENTS.md 守门 #9 v20 (2026-09-02 00:39 JST 拍板)**: 子代理 dispatch 必先落 brief → `docs/briefs/<task_id>.md`
> **author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签
> **触发**: 2026-09-06 07:34 JST Ulysses 拍板 Q1=C / Q2=B / Q3=A1 / Q4=B

---

## §1 Objective & Why

**实装 ContextPanel 5 tab 中第 1 个 Tab"属性"**——把现有 `frontend/src/components/board/WorkItemDetailDrawer.tsx` (29KB 单页滚动) 重构为"5 tab 容器"中的"属性"子页面。

**Why**: per `docs/frontend/design/ui-3pane-arch.md` §1.4 + `ui-detailed-design.md` §5.3,ContextPanel 5 tab 拍板但 0 实装,Drawer 当前是单页 7 字段编辑(标题/状态/分配/Worktree 关联/描述/Save 按钮),需要拆出 5 tab 容器架构 + 把现有字段迁到"属性" tab。

---

## §2 已知事实 + 排除路径

**已确认** (Mavis 接手探查):
- `frontend/src/components/board/WorkItemDetailDrawer.tsx` 现状: 7 字段 + 关联 Worktree 下拉 + 描述 textarea + Save 按钮,无 tab 切分。
- `frontend/src/mocks/data/kanban.ts` + `frontend/src/mocks/data/index.ts` 已有 mock 模式可复用。
- `domain-work-item` Service InMemory 已存在 (`crates/domain-work-item/src/service.rs`)。
- 5 wt 拓扑已建好: `feat/cp-tab-property` 从 main `7028c8f` 拉出。
- 测试: `frontend/src/components/board/KanbanBoard.test.tsx` + `KanbanCard.test.tsx` 现有。

**已排除** (不要做):
- ❌ 不起 HTTP server — Q1=B1 拍板走 MSW 兜底 (per `docs/architecture/msw-real-mode.md`)。
- ❌ 不改 7 字段 UI/字段名/字段顺序 — 只迁位置,不改逻辑。
- ❌ 不实现 Tab 2-5 (评论/关联/活动/AI 助手) — 那 4 个 Tab 由 worker-2/3/4/5 负责。
- ❌ 不动 KanbanBoard.tsx 主路径 — Drawer 由 KanbanBoard 触发,只改 Drawer 内部。
- ❌ 不写 5 页面 2-Pane 改造 — 那归 worker-5 (cp-tab-5pages) 在批 2 做。

---

## §3 Scope / Ownership / Out-of-scope

**In-scope (本 wt 必做)**:
1. 新建 `frontend/src/components/contextpanel/ContextPanel.tsx` — 5 tab 容器组件(预留 5 slot,本 wt 只填 slot 1 "属性")。
2. 新建 `frontend/src/components/contextpanel/TabProperty.tsx` — 把现有 WorkItemDetailDrawer 7 字段迁过来。
3. 重构 `WorkItemDetailDrawer.tsx` — 内部包 `<ContextPanel activeTab="property" />`,**保留 Drawer 触发逻辑 + 公开 API 100% 不变**。
4. 配套 `frontend/src/components/contextpanel/__tests__/ContextPanel.test.tsx` — 至少 3 个测试: 默认 tab 渲染 / tab 切换 / Drawer API 兼容。
5. `frontend/src/lib/i18n/{zh-CN,ja,en,ts}.ts` 补 5 个 key (per `docs/reports/STAR-I18N-EXTRACT.json` i18n 化基线)。

**Out-of-scope (本 wt 不做)**:
- Tab 2-5 内容(留 `<EmptyTabPlaceholder tabId="comment" />` 占位组件 + `data-tab="comment"` 属性)。
- MSW handler (worker-2/3 写,本 wt 复用 `frontend/src/mocks/handlers/work-item.ts` 既有)。
- 键盘快捷键 (Cmd+[ / Cmd+]) — 跟 Tab 容器一起做,本 wt 只做容器 + Tab 1。
- 5 页面 2-Pane 改造 — 归 worker-5。

---

## §4 Deliverable

| # | 文件 | 行数估 | 状态 |
|---|---|---|---|
| 1 | `frontend/src/components/contextpanel/ContextPanel.tsx` | ~120 | 新建 |
| 2 | `frontend/src/components/contextpanel/TabProperty.tsx` | ~280 (从 WorkItemDetailDrawer 拆) | 新建 |
| 3 | `frontend/src/components/contextpanel/EmptyTabPlaceholder.tsx` | ~30 | 新建 |
| 4 | `frontend/src/components/board/WorkItemDetailDrawer.tsx` | -300 / +50 净改 -250 | 重构 |
| 5 | `frontend/src/components/contextpanel/__tests__/ContextPanel.test.tsx` | ~80 | 新建 |
| 6 | `frontend/src/lib/i18n/{zh-CN,ja,en,ts}.ts` | +5 key × 4 = 20 行 | 改 |
| 7 | `frontend/src/mocks/handlers/_index.ts` (注册位) | 0 改 (worker-2/3 写) | 不动 |
| 8 | `docs/reports/PHASE-CONTEXTPANEL-TAB1-PROPERTY-REPORT.md` | 7 段 (per AGENTS.md §3) | 新建 |

**commit** (per AGENTS.md §2.1):
```bash
git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m 'feat(cp-tab-property): ContextPanel Tab 1 属性 5 tab 容器实装

- 新建 ContextPanel.tsx (5 slot, 本 wt 填 slot 1)
- 拆 WorkItemDetailDrawer 7 字段到 TabProperty.tsx
- Drawer 公开 API 100% 兼容 (KanbanBoard 不改)
- 4 key i18n × 4 语言 = 20 行
- 测试: ContextPanel.test.tsx 3/3 pass
- 守门: cargo check --workspace --all-targets -j 4 (0 err, 跟 main 一致)
- 自动化档: scripts/automation/contextpanel_property.py
- brief: docs/briefs/contextpanel-property.md
- per 守门 #1+#19+#20+#9 v20

Co-Authored-By: Mavis 接手 agent <mavis@MiniMax.local>'
```

---

## §5 Acceptance Criteria

**功能**:
- [ ] `WorkItemDetailDrawer` 公开 props (`open`, `onClose`, `projectId`, `projectKey`, `tenantId`, `reporterId`) 100% 不变
- [ ] KanbanBoard 点击卡片 → Drawer 打开 → 默认显示 Tab 1 "属性" (跟当前行为一致)
- [ ] Tab 容器显示 5 个 tab 标签 (属性/评论/关联/活动/AI 助手),仅属性 tab 可点击,其余显示 EmptyTabPlaceholder
- [ ] 现有 7 字段 (Title/Status/Assignee/Worktree/Priority/DueDate/Description) 全部迁到 TabProperty,功能不变

**测试**:
- [ ] `pnpm test src/components/contextpanel/__tests__/ContextPanel.test.tsx` 3/3 pass
- [ ] `pnpm test src/components/board/KanbanBoard.test.tsx` 现有测试 100% pass (Drawer 兼容)
- [ ] `pnpm test src/components/board/WorkItemDetailDrawer` (如有) 100% pass

**守门**:
- [ ] `cargo check --workspace --all-targets -j 4` 0 err (跟 main HEAD 7028c8f 一致,本 wt 不动 Rust)
- [ ] `pnpm typecheck` 0 err
- [ ] `pnpm lint` 0 err (advisory mode, per 守门 #7 v3 PR #12)

**文档**:
- [ ] `PHASE-CONTEXTPANEL-TAB1-PROPERTY-REPORT.md` 7 段 (§0 目的 / §1 改动矩阵 / §2 验证摘要 / §3 已知缺口 / §4 子代理失败接手清单 / §5 守门规则 / §6 签字栏 / §7 修订历史)
- [ ] §3 已知缺口显式列 "Tab 2-5 由 worker-2/3/4/5 接手"

**提交**:
- [ ] commit author = `Ulysses <ulysses@mavis.local>` (per AGENTS.md §2.1)
- [ ] 1 commit, message 引用 brief 路径 + 自动化档路径
- [ ] wt 分支 `feat/cp-tab-property`,**不推 origin** (per AGENTS.md 守门 #1 + 推到 Mavis 接手统一推)

---

## §6 Risk

| 风险 | 概率 | 缓解 |
|---|---|---|
| WorkItemDetailDrawer 重构影响 KanbanBoard 测试 | 中 | 公开 API 100% 兼容,只在 Drawer 内部包 ContextPanel |
| i18n key 跟现有 STAR-I18N-EXTRACT.json 冲突 | 低 | 用 `cp.tab.property.*` 命名空间隔离 |
| 5 tab 容器组件后续 worker 命名不一致 | 中 | 本 wt 落 `contextpanel/CONTRACT.md` 列出 5 tab id 常量 (`property` / `comment` / `relation` / `activity` / `ai`) |
| commit 跟 worker-2/3 在 main 链上冲突 | 低 | 5 wt 互不重叠,本 wt 只动 contextpanel/ + board/WorkItemDetailDrawer.tsx |

---

## §7 Out of Band (跨 wt 协调)

- 通知 worker-2 (cp-tab-comment): Tab 容器已就绪,Tab 2 slot id = `comment`,ContextPanel.tsx 留空 slot。
- 通知 worker-3 (cp-tab-relation): Tab 3 slot id = `relation`,可读 GraphNode/GraphEdge。
- 通知 worker-4 (cp-tab-activity, 批 2): Tab 4 slot id = `activity`,批 1 合并后启动。
- 通知 worker-5 (cp-tab-5pages, 批 2): 5 页面 2-Pane 改造,批 1 合并后启动,Container 复用本 wt 的 ContextPanel。
