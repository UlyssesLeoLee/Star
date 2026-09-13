# ARG.5 Brief — frontend/src/app/agent-relationships/ 5 UI 组件

> **Task ID**: arg-05-frontend-5ui
> **WT**: `wt-arg-05-frontend` (branch: `wt-arg-05-frontend`, base: main @ f1207e2 含 ARG.1+ARG.2+ARG.3+ARG.4 merge)
> **触发**: 2026-09-10 07:46 JST 用户发令"按顺序推进" + ARG.3 merge 走守门后父会话自驱
> **依赖**: 
> - ARG.1 (`crates/arg`) 已 merge @ `651117e`
> - ARG.2 (`crates/arg-bridge`) 已 merge @ `87e1618`
> - ARG.3 (`crates/arg-effect`) 已 merge @ `f1207e2` ← 刚 merge
> - ARG.4 (`crates/api/src/arg`) 已 merge @ `1d894ab` (提供 14 routes 13 REST + 1 WS API)
> **拍板来源**: per WBS-001 v0.50 §14.11 ARG.5 (3M tokens / 0.5 周) + v0.50 跨 session 续做规划 (估 4-6M / 3-5 session)
> **守门合规**: #1 v15 (第 46 次新事件) + #1 v19 ([M] 必先 `scripts/automation/arg_ui_test.py`) + #1 v25 + #3 + #5 + #6 + #7 + #9 + #10 + #12 + #13 + #14 v3 + #19 v19

---

## 1. Objective

落地 ARG 4 新 crate 的**第 4 个 — Frontend UI**：`frontend/src/app/agent-relationships/` 5 UI 组件 + zustand store 5 channel。

ARG.5 完成后:
- P3-C W4 收官
- 5 UI 组件 (RelationshipEditor / RelationshipView / AchievementWall / EdgeTypeSelector / TemplateGallery)
- zustand useARGStore 5 channel (agents / edges / templates / achievements / events)
- Agent View 1 tab 集成 (Relationship 视角切换)

---

## 2. Scope (per DD-AGENT-RELATIONSHIP-001.md v0.1.1 §3.1 + §4.13 + §5.1)

### 2.1 必做

#### A. `frontend/src/app/agent-relationships/` 5 UI 组件

- `page.tsx` (3 tab container: Editor / View / Achievements)
- `editor/RelationshipEditor.tsx` (画布拖拽建关系, 复用 frontend-canvas-design.md v0.1 无限画布)
- `editor/EdgeTypeSelector.tsx` (10 类关系选择器, Dropdown)
- `view/RelationshipView.tsx` (图谱浏览, 节点详情侧栏)
- `view/NodeDetail.tsx` (节点详情: name + archetype + trust_score + edge 数)
- `achievements/AchievementWall.tsx` (成就墙, 4 稀有度筛选 COMMON/RARE/EPIC/LEGENDARY)
- `templates/TemplateGallery.tsx` (5 模板库: Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council)
- `AgentViewTab.tsx` (集成到 /agent-view, 1 tab 切到 Relationship 视角)

#### B. `frontend/src/lib/arg/` 状态管理层

- `store.ts` (useARGStore zustand 5 channel, per DD §4.2.3):
  - `agents: Map<string, Agent>`
  - `edges: Map<string, Edge>`
  - `templates: Template[]`
  - `achievements: Achievement[]`
  - `unlockedAchievements: Set<string>`
  - `argEvents: ARGEvent[]`
  - `wsConnected: boolean`
  - actions: `loadAgents / loadEdges / createEdge / updateEdge / archiveEdge / instantiateTemplate / loadAchievements / unlockAchievement / subscribeEvents`
- `api.ts` (13 REST 客户端封装, fetch + retry + RLS header)
- `ws.ts` (WebSocket 客户端, /ws/arg/events 订阅, 6 事件类型)
- `types.ts` (TypeScript types: Agent / Edge / RelationshipType / Achievement / Template / ARGEvent)

#### C. 1 脚本 (per 守门 #1 v19 [M])

- `scripts/automation/arg_ui_test.py` v0.1 (~150 行):
  - 测 13 REST 端点可达性 (跟 ARG.4 IT 配合)
  - 测 5 UI 组件 TSX 编译 (via `tsc --noEmit`)
  - 测 zustand store 类型一致性

#### D. 2 文档更新

- `docs/automation-design.md` §4 追加 ARG.5 行
- `scripts/automation/registry.md` §1 + §5 索引追加

#### E. 1 报告

- `docs/reports/PHASE-ARG-05-IMPL-REPORT.md` v0.1

#### F. 1 commit + merge

- 1 commit: `feat(arg-frontend): frontend/src/app/agent-relationships/ 5 UI 组件 + zustand store 5 channel (ARG.5 子项, P3-C W4)`
- author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)

### 2.2 不做

- 不写 e2e test (ARG.7 后续, 8 E2E Playwright)
- 不写 PT 性能测试 (ARG.7 后续)
- 不写 ARG.8 行为/产出成就 evaluator (前端显示先用 placeholder)
- 不写实际 WebSocket 集成 (UI 部分订阅, 真实 WS 在 ARG.4 backend 已就绪, mock fallback)
- 不写 实际 OAuth2 / JWT 鉴权 UI (ARG.4 鉴权后续)

---

## 3. Acceptance Criteria (per DD §4.13 + §5.1 + zustand 5 channel)

### AC-1: 编译守门
- [ ] `cd frontend && pnpm install` exit 0
- [ ] `pnpm tsc --noEmit` exit 0, 0 type error
- [ ] `pnpm lint` exit 0
- [ ] `pnpm test` (jest) exit 0
- [ ] `pnpm build` exit 0 (next build 0 err)

### AC-2: 5 UI 组件落地
- [ ] `RelationshipEditor.tsx` 画布拖拽 + 10 类关系 Dropdown
- [ ] `RelationshipView.tsx` 图谱 + NodeDetail
- [ ] `AchievementWall.tsx` 20 成就 + 4 稀有度筛选
- [ ] `EdgeTypeSelector.tsx` 10 类关系
- [ ] `TemplateGallery.tsx` 5 模板
- [ ] `page.tsx` 3 tab 容器
- [ ] `AgentViewTab.tsx` /agent-view 集成

### AC-3: zustand store 5 channel
- [ ] useARGStore 含 5 channel (agents / edges / templates / achievements / events)
- [ ] 9 actions (loadAgents / loadEdges / createEdge / updateEdge / archiveEdge / instantiateTemplate / loadAchievements / unlockAchievement / subscribeEvents)
- [ ] WS 自动重连 + 6 事件类型处理

### AC-4: API 集成
- [ ] 13 REST 客户端封装 (per ARG.4 §4.12)
- [ ] 1 WS /ws/arg/events (per ARG.4 §4.12)
- [ ] 5 模板 instantiate 走 POST /api/arg/templates/instantiate

### AC-5: Agent View 集成
- [ ] /agent-view 页面加 1 tab "Relationship" 切到 ARG 视角
- [ ] Tab 间数据共享 (per Agent View 既有 zustand store pattern)

### AC-6: 自动化档守门
- [ ] `scripts/automation/arg_ui_test.py` 存在
- [ ] `docs/automation-design.md` §4 追加 1 行
- [ ] `scripts/automation/registry.md` 追加 1 行

### AC-7: 守门 v25
- [ ] frontend typecheck + build 0 err
- [ ] workspace 兼容: `cargo check --workspace --lib -j 4` 0 err
- [ ] author = Ulysses

---

## 4. 已知约束

| 约束 | 处理 |
|---|---|
| 守门 #1 v19 [M] | 落 `arg_ui_test.py` |
| 守门 #6 PowerShell only | PowerShell 跑 pnpm 命令 |
| 守门 #10 代签 | `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local'` |
| 守门 #9 RPC 不可靠 | mock WS / mock fetch, 不调外部服务 |
| Next.js 既有 | 复用 frontend/src/app/ 既有 layout + 路由 |
| zustand 5 channel | 跟 BD §4.2.3 一致 |
| WebSocket mock fallback | 真实 WS 在 ARG.4 backend 就绪, 客户端降级 |

---

## 5. Deliverable

- `frontend/src/app/agent-relationships/` 8 文件
- `frontend/src/lib/arg/` 4 文件
- 1 脚本
- 2 文档更新
- 1 报告
- 1 commit

---

## 6. Token 预算

- 软预算 3M (per WBS-001 v0.18 §14.11 ARG.5)
- v0.50 跨 session 估 4-6M
- 实际预期 4-5M
- 触发熔断 7M
- 超出记录 (per 2026-09-10 用户拍板)

---

## 7. 子代理执行守门

- 不调任何 RPC
- 不用 RPC 派子代理
- 不写后端 (本任务只动 frontend/)
- 不调 git push
- 每步必跑守门

---

## 8. 父会话职责

- 接收子代理 final report
- self-review 子代理产出
- 跑守门 #1 v3 + v25
- merge 走 `git merge --no-ff wt-arg-05-frontend`
- WBS 升版
- 触发 ARG.6 (30 UT) 或 ARG.7 (10 IT + 8 E2E + 4 PT)

---

## 9. 失败接手

| 失败 | 接手 |
|---|---|
| 子代理 RPC 失败 | task_stop, 父会话接手 |
| pnpm tsc 失败 | 本地 fix |
| 5 UI 组件缺一 | 必补齐 |
| zustand store 不一致 | 必对齐 BD §4.2.3 |
| Token 超 7M | task_stop, 报告 |

---

## 10. 引用

- `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1
- `docs/design/BD-AGENT-RELATIONSHIP-001.md` v0.1
- `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 §4.13 + §5.1
- `docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md` v0.50 (411 行)
- `docs/architecture/2026-09-03-arg/14-arg-10-arg-frontend.md` v0.50 (315 行, 5 UI + 14 routes + WebSocket)
- `docs/reports/STAR-P3-WBS-001.md` v0.50 §14.11
- `frontend-canvas-design.md` v0.1 (画布 + bezier connector 公式)
- `BD-AGENT-VIEW-001.md` (Agent View 既有 zustand store pattern)
- `crates/api/src/arg/` ARG.4 14 routes 实证
- `docs/briefs/arg-01-arg-crate-skeleton.md` (模板)
- `AGENTS.md` §4 守门 + §3 7 段报告
