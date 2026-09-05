# Brief: ContextPanel 5 页面 2-Pane 改造 (worker-5 / cp-tab-5pages, 批 2)

> **per AGENTS.md 守门 #9 v20**: 子代理 dispatch 必先落 brief
> **author**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手代签
> **触发**: 2026-09-06 07:34 JST Ulysses 拍板 Q1=C (12 缺口) / Q2=B / Q4=B (批 2)

---

## §1 Objective & Why

**5 页面 2-Pane 改造** —— per `docs/frontend/design/ui-detailed-design.md` §1.1 第一设计原则 "2-Pane (主列表 + 粘性右侧抽屉)" 拍板,**当前只在 Kanban 卡片实装**,其余 4 页面 (/agent /notification /analytics /planning) 都没 2-Pane 抽屉。

**Why**: spec 写明 5 页面共用同款 2-Pane 模式,但实装只覆盖 1/5;本 worker 把模式扩展到全 5 页面,ContextPanel 5 tab 容器复用 worker-1 的实现。

---

## §2 已知事实

- `frontend/src/components/contextpanel/ContextPanel.tsx` 5 slot API 由 worker-1 拍板,本 worker 等批 1 合并后从 main 链 cherry-pick。
- `ui-detailed-design.md` §5.1-§5.6 5 页面规格已存在,本 worker 按规格补实装。
- 当前 `/agent` `/notification` `/analytics` `/planning` 4 页面用各自独立的非 ContextPanel 详情展示 (3-Pane 或无抽屉)。
- 5 wt 已建,本 worker wt = `feat/cp-5pages`,**批 2 启动**。

---

## §3 Scope

**必做 (5 页面 2-Pane 改造)**:
1. `/agent` 页面 (`/projects?tab=agents`):
   - 现有左 360px agent 列表 + 中详情 + 右 320px lease/heartbeat 固定面板 → 改为 2-Pane: 左列表 + 右 ContextPanel (5 tab 容器,默认 active "属性" tab)
2. `/notification` 页面 (`/projects?tab=inbox`):
   - 现有 3 column: 通知源/通知列表/详情 → 改为 2-Pane: 中列表 + 右 ContextPanel (默认 "活动" tab 显示通知已读记录)
3. `/analytics` 页面:
   - 当前 6 KPI + 2 chart + 1 table → 补 2-Pane: 主图表 + 右 ContextPanel (默认 "活动" tab 显示数据刷新历史)
4. `/planning` 页面:
   - 当前 Calendar/Timeline → 补 2-Pane: 主日历 + 右 ContextPanel (默认 "关联" tab 显示 milestone → work-item 依赖图)
5. `/projects` 页面 Kanban 视图:
   - 已有 WorkItemDetailDrawer (worker-1 重构) → 验证 ContextPanel 容器兼容,**不重复实装**

**6 个新组件**:
- `frontend/src/components/contextpanel/AgentDetailPanel.tsx`
- `frontend/src/components/contextpanel/NotificationDetailPanel.tsx`
- `frontend/src/components/contextpanel/AnalyticsDetailPanel.tsx`
- `frontend/src/components/contextpanel/PlanningDetailPanel.tsx`
- `frontend/src/components/contextpanel/hooks/useDetailDrawer.ts` (统一开关 + active tab 状态)
- `frontend/src/components/contextpanel/__tests__/DetailDrawer.integration.test.tsx` (跨页面集成测试)

**i18n**: 4 语言 × 4 key × 4 页面 = 64 行。

**报告**: `PHASE-CONTEXTPANEL-5PAGES-REPORT.md` 7 段。

---

## §4 Deliverable

| 文件 | 估行 |
|---|---|
| `frontend/src/components/contextpanel/AgentDetailPanel.tsx` | ~200 |
| `frontend/src/components/contextpanel/NotificationDetailPanel.tsx` | ~200 |
| `frontend/src/components/contextpanel/AnalyticsDetailPanel.tsx` | ~200 |
| `frontend/src/components/contextpanel/PlanningDetailPanel.tsx` | ~200 |
| `frontend/src/components/contextpanel/hooks/useDetailDrawer.ts` | ~80 |
| `frontend/src/components/contextpanel/__tests__/DetailDrawer.integration.test.tsx` | ~150 |
| 4 页面文件 (改) | 各 +30 行 |
| `frontend/src/lib/i18n/{zh-CN,ja,en,ts}.ts` | +64 行 |
| `docs/reports/PHASE-CONTEXTPANEL-5PAGES-REPORT.md` | 7 段 |

---

## §5 Acceptance Criteria

- [ ] 4 页面 2-Pane 模式实装
- [ ] `useDetailDrawer` hook 统一 4 页面开关逻辑
- [ ] 键盘快捷键 `Cmd+.` 全折叠 / `Cmd+[` `Cmd+]` 切 tab (per ui-3pane-arch.md §1.4 line 202)
- [ ] i18n 4 语言覆盖
- [ ] 集成测试 4 页面 drawer 打开/关闭/切换 tab 全 pass
- [ ] 守门 4 项 0 err
- [ ] commit author = Ulysses, 引用 brief + 自动化档
- [ ] 7 段报告含 §3 已知缺口 (Phase 2+ 留: Export / 离线缓存 / 移动端)

---

## §6 启动条件 (批 2)

- ✅ 批 1 三 wt (property / comment / relation) 全部 merge 到 main 链
- ✅ ContextPanel.tsx 5 slot API 稳定
- ✅ TabProperty / TabComment / TabRelation / TabAi 4 组件可复用

**启动机制**: 同 worker-4,Mavis 接手在批 1 验证通过后启动。

---

## §7 Risk

| 风险 | 缓解 |
|---|---|
| 4 页面现有 3-Pane 改动影响现有用户 | 2-Pane 模式保留视觉位置 (右 320px),只是从"固定面板"改为"可折叠抽屉" |
| `useDetailDrawer` hook 4 页面状态不独立 | hook 接受 `pageId` 参数,每个页面独立 state |
| 键盘快捷键跟现有 Cmd+K (CommandBar) 冲突 | 走 `Cmd+.` / `Cmd+[` / `Cmd+]`,不冲突 |
| 4 页面同时改动导致 commit 大 | 本 worker 拆 5 commit: 1 hook + 4 页面各 1 commit |
