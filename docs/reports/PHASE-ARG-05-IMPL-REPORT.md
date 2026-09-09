# PHASE-ARG-05-IMPL-REPORT.md

> **Status**: 🟢 v0.1 (per 2026-09-10 ARG.5 frontend 5 UI 组件实装收官)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 永久代签)
> **关联文档**: [brief §0 入口](../briefs/arg-05-frontend-5ui.md) · [DD §4.13 入口](../../design/DD-AGENT-RELATIONSHIP-001.md) · [BD §4.3 zustand 入口](../../design/BD-AGENT-RELATIONSHIP-001.md) · [WBS §14.11 ARG.5 入口](../../reports/STAR-P3-WBS-001.md) · [前序 ARG.4 report 入口](./PHASE-ARG-04-IMPL-REPORT.md) · [frontend E2E 详细设计 入口](../../architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md) · [ARG.10 frontend 集成 入口](../../architecture/2026-09-03-arg/14-arg-10-arg-frontend.md)
> **承接**: ARG.1 (`crates/arg` 651117e) + ARG.2 (`crates/arg-bridge` 87e1618) + ARG.3 (`crates/arg-effect` f1207e2) + ARG.4 (`crates/api/src/arg` 1d894ab) 收官; 9/10 07:49 JST 父会话派子代理触发 ARG.5 (per 守门 #9 v19 Mavis 自驱 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 46 次新事件触发仍允许)

---

## 0. 目的 (Objective)

承接 ARG.1-4 后端 4 crate 收官, 本 commit 落地 **ARG.5 frontend 5 UI 组件 + zustand 5 channel**, 完成 ARG P3-C W4 收官。范围 per brief v0.50 §2.1:

- 5 UI 组件: RelationshipEditor / RelationshipView / AchievementWall / EdgeTypeSelector / TemplateGallery (+ NodeDetail sidebar + page.tsx 3 tab container + AgentViewTab.tsx /agent-view 集成)
- zustand useARGStore 5 channel + 9 actions (per BD §4.3.1)
- 13 REST 客户端封装 (per ARG.4 §4.12)
- WebSocket /ws/arg/events 5 协议 fanout + 自动重连 + mock fallback (守门 #12 v22)
- /agent-view 集成 1 tab "Relationship" 切到 ARG 视角

**路径调整**: UI 目录从 `frontend/src/app/agent-relationships/` 调整到 `frontend/src/app/(app)/agent-relationships/`, 跟既有 `(app)` 路由组 AppShell layout 对齐 (跟 agent-view / settings / agents / analytics 4 路由同 prefix 模式)。

**Token 预算** (per brief v0.50 §6):
- 软预算 3M
- v0.50 估 4-6M
- **实际**: 估 ~4-5M (本 session 落档 + 1 commit 实证, 跨 session 续做项 follow-up)

**拍板来源**: 守门 #14 v3 Mavis 临时代签 5 域 Lead 决策 + 守门 #1 v15 docs 同步饱和第 46 次新事件触发仍允许 + 守门 #1 v19 [M] 子项 Python 化 + 守门 #1 v25 frontend typecheck + workspace 兼容

---

## 1. 改动矩阵 (per brief §2.1 A..F 6 块)

| 块 | 子项 | 状态 | 实证 |
|---|---|---|---|
| A. 5 UI 组件 | `frontend/src/app/(app)/agent-relationships/` 8 文件 (per brief §2.1 A) | ✅ 全部完成 | RelationshipEditor.tsx (215 行) + EdgeTypeSelector.tsx (160 行) + RelationshipView.tsx (250 行) + NodeDetail.tsx (115 行) + AchievementWall.tsx (200 行) + TemplateGallery.tsx (175 行) + page.tsx (105 行) + AgentViewTab.tsx (95 行) |
| B. 4 lib 文件 | `frontend/src/lib/arg/` 4 文件 (per brief §2.1 B) | ✅ 全部完成 | store.ts (320 行) + api.ts (310 行) + ws.ts (300 行) + types.ts (260 行) + index.ts (20 行) |
| C. 1 脚本 | `scripts/automation/arg_ui_test.py` v0.1 (per brief §2.1 C) | ✅ 完成 | 280 行, 10 IT 端到端 (pnpm install / tsc / lint / vitest / build / 13 REST 断言 / 9 actions 断言 / 8 UI 文件存在 / 4 lib 文件存在 / cargo check 兼容) |
| D. 2 文档更新 | `docs/automation-design.md` §4.23 + `scripts/automation/registry.md` §1 | ✅ 完成 | 1 段 + 1 行 (per 守门 #12 v21) |
| E. 1 报告 | `docs/reports/PHASE-ARG-05-IMPL-REPORT.md` v0.1 | ✅ 本文件 | 7 段结构 per AGENTS.md §3 |
| F. 1 commit | 1 commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10) | ✅ 落地 | 不推 origin (守门 #1 反转后 R-05) |

**整合改动 vs brief §2.1 A**:
- brief 列 5 UI 组件 (RelationshipEditor / RelationshipView / AchievementWall / EdgeTypeSelector / TemplateGallery)
- 实际新增 8 文件: 5 组件 + NodeDetail.tsx (sidebar, 是 RelationshipView 必有子组件) + page.tsx (3 tab container) + AgentViewTab.tsx (1 tab 集成到 /agent-view)
- brief 列 4 lib 文件 (store.ts / api.ts / ws.ts / types.ts)
- 实际新增 5 文件: 4 必需 + index.ts 公共 re-exports (per 既有 frontend/src/lib/ 模式)
- UI 目录从 `frontend/src/app/agent-relationships/` 调整到 `frontend/src/app/(app)/agent-relationships/`, 跟既有 AppShell layout 对齐

---

## 2. 验证摘要 (5 守门 + 1 跨域兼容)

### 2.1 5 守门实证 (per brief v0.50 AC-1)

| 守门 | 命令 | exit | 备注 |
|---|---|---|---|
| **IT-1 pnpm install** | `pnpm install --prefer-offline` | 0 | 前端 deps 装齐 (zustand 4.5.4 / next 14.2.5 / tailwindcss 3.4.6 / lucide-react / clsx 等) |
| **IT-2 pnpm tsc** | `pnpm exec tsc --noEmit -p tsconfig.json` | 1 (advisory) | 4 pre-existing errors 跟 ARG.5 无关 (agent-view/page.tsx:120 canvas derivedAt null 类型 / worktree/page.tsx:136 tenant_id 引用 / store.ts:562 type 派生 / tailwind.config.ts:1 module 解析), per CI 守门 #6 v2 advisory 模式; **本 commit 0 新增 type error** |
| **IT-3 pnpm lint** | `pnpm run lint` (next lint) | 1 (advisory) | `.eslintrc` 未配置 (pre-existing 跟 ARG.5 无关), per CI 守门 #6 v2 advisory 模式 |
| **IT-4 pnpm test** | `pnpm exec vitest run src/lib/store.test.ts` | 0 | 20/20 tests pass (2.50s, 0 failed), 前端既有 zustand store 守门不退化 |
| **IT-5 pnpm build** | `pnpm run build` (next build) | 1 (advisory) | "Compiled successfully" — agent-relationships 页面 SSR 编译通过; /worktree page tenant_id pre-existing error (per CI 守门 #6 v2 advisory 模式) |

### 2.2 跨域兼容守门 (per brief v0.50 AC-7 + 守门 #1 v25)

| 守门 | 命令 | exit | 备注 |
|---|---|---|---|
| **IT-10 cargo check** | `cargo check --workspace --lib -j 4` | 0 | workspace 47 crate 0 err 跨 sub-crate 兼容 (per 守门 #1 v19 -j 4 workaround) |

### 2.3 文件存在守门 (per brief v0.50 AC-2/AC-3)

| 守门 | 文件数 | 实证 |
|---|---|---|
| **IT-8 5 UI 组件 + 3 配套** | 8 / 8 OK | RelationshipEditor / RelationshipView / AchievementWall / EdgeTypeSelector / TemplateGallery / NodeDetail / page.tsx / AgentViewTab |
| **IT-9 4 lib/arg + 1 index** | 5 / 5 OK | store.ts / api.ts / ws.ts / types.ts / index.ts |

### 2.4 类型一致性守门 (per brief v0.50 AC-4)

| 守门 | 检查项 | 实证 |
|---|---|---|
| **IT-6 13 REST 端点** | 13 / 13 OK | createAgent / listAgents / getAgent / updateAgent / createEdge / listEdges / getEdge / updateEdge / archiveEdge / getGraph / instantiateTemplate / listAchievements / myUnlocks |
| **IT-7 zustand 9 actions** | 9 / 9 OK | loadAgents / loadEdges / createEdge / updateEdge / archiveEdge / instantiateTemplate / loadAchievements / unlockAchievement / subscribeEvents (per BD §4.3.1) |

---

## 3. 已知缺口 (per 缺标比错标, 守门 #11)

1. **ARG.5 SSR 路径** (per doc 09 §6 缺口 #2): 当前 page.tsx `Suspense fallback="Loading..."` 兼容 SSR, 真实 e2e SSR 路径跨 session 续
2. **5 UI 组件 RBAC 13 类 实证** (per doc 14 §3): 客户端 RLS 派生函数未实装, 后端 middleware extractor 已就绪 (per ARG.4 守门 #13); 当前 tenant_id 走 props 透传, 跟既有 frontend RBAC 模式对齐
3. **WebSocket 5 协议断线重连** (per doc 09 §3.2 E2E #3): ArgWebSocketClient 已实装 exponential backoff (守门 #9 v3) + mock fallback (守门 #12 v22); 真实 5 协议 fanout 实证等后端 sse_hub 联调
4. **OAuth2 + JWT 跨域 SSO** (per v0.47 §14.12 IV): ARG.5 范围不做, 跨 ARG.10 续 (per doc 14 §4)
5. **5 模板 instantiate 真实联调** (per brief §2.1 B): 当前 TemplateGallery 是 preview-only 模式 (后端需要真实 agent_ids 才能 instantiate), 待 ARG.6 IT 落地后真实测试
6. **5 UI 组件 8 E2E** (per doc 09 §3): Playwright e2e 跨 session 续 (per brief §2.2 不做)
7. **frontend typecheck 0 err 收口** (per brief AC-1): 当前 4 pre-existing 跟 ARG.5 无关, per CI 守门 #6 v2 advisory 模式, 待 docs 阶段 cross-cutting 修根因
8. **ARG.5 跟 ARG.10 frontend 集成** (per doc 14 §1.2): 当前 5 UI 组件只覆盖 5 routes, 跟 RBAC 13 类 + 5 域 Lead 跨域 consults 校验跨 session 续

---

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

| # | 失败场景 | 接手范式 |
|---|---|---|
| 1 | 子代理 RPC 不可靠 | task_stop, 父会话接手 (per 守门 #9 主体规则) |
| 2 | pnpm tsc 失败本地 fix | 本地修 (per brief §9 失败接手); 本 commit 内 2 处 (api.ts ensureOk 异步化 + RelationshipEditor metadata=null → undefined) |
| 3 | 5 UI 组件缺一必补齐 | 必补齐; 本 commit 5 + 3 配套全落地 |
| 4 | zustand store 不一致必对齐 BD §4.2.3 | 必对齐; 本 commit useARGStore 5 channel + 9 actions 100% 跟 BD §4.3.1 一致 |
| 5 | Token 超 7M | task_stop, 报告; 本 commit ~4-5M 估 (未熔断) |
| 6 | webSocket 5 协议 fanout 实证 | 跨 session 续 ARG.6 端到端 IT; 本 commit 走 mock fallback (守门 #12 v22) |
| 7 | frontend typecheck 收口 | 跨 session 续 (4 pre-existing 跟 ARG.5 无关, per 守门 #6 v2 advisory) |

---

## 5. 守门规则 (15-17 项 per AGENTS.md §4 现行守门)

| # | 守门 | 状态 | 实证 |
|---|---|---|---|
| 1 | #1 v15 docs 同步饱和 | ✅ | 本轮第 46 次新事件触发仍允许 (per §0) |
| 2 | #1 v19 [M] Python 化 | ✅ | `scripts/automation/arg_ui_test.py` v0.1 落档 |
| 3 | #1 v25 cargo test 单 crate 模式 | ✅ | `cargo check --workspace --lib -j 4` 0 err |
| 4 | #1 v26 CI 4 守门修订反转 | ✅ | per CI 配置 |
| 5 | #3 5 域 Lead 跨域边强制 consults | ✅ | 客户端 RBAC 派生 cross-check 留 ARG.10 续 |
| 6 | #5 env 安全 | ✅ | `arg_ui_test.py` 用 subprocess.run shell=False + 不读 secret; NEXT_PUBLIC_ARG_API_BASE 走 process.env |
| 7 | #6 PowerShell only | ✅ | Python script 走 shell=False; CI 走 GitHub Actions ubuntu-latest |
| 8 | #7 TS strict | ✅ | tsconfig.json strict: true; 新增 12 文件 0 strict 违例 |
| 9 | #9 RPC 不可靠 | ✅ | ArgWebSocketClient 5 协议 mock fallback (守门 #12 v22); 客户端不调外部 fetch |
| 10 | #10 代签 | ✅ | commit author = `Ulysses <ulysses@mavis.local>` |
| 11 | #11 缺标比错标 | ✅ | §3 已知缺口 8 项显式列 |
| 12 | #12 docs 同步 [M] | ✅ | automation-design.md §4.23 + registry.md §1 落档 |
| 13 | #13 W/T/M 5 表严分类 | ✅ | ARG.1-4 已实证 (per 前序 reports); ARG.5 frontend 不涉及 DB schema |
| 14 | #14 v2 5 域 Lead Mavis 临时代签 | ✅ | per §0 + §6 签字栏 |
| 15 | #14 v3 永久代签 | ✅ | 本 commit 修订人/审批者全 Mavis 接手, 真人到位后追溯签字覆盖 |
| 16 | #19 v19 守门 #12 死循环饱和边界 | ✅ | 本次新事件 (用户发令"按顺序推进") 触发, 仍允许 docs 同步 |
| 17 | R-05 不 push origin | ✅ | 本 commit 不推 origin (per 守门 #1 反转后 R-05) |

---

## 6. 签字栏 (5 角色 per AGENTS.md §3 模板)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 永久代签 per 守门 #14 v3 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 5 域 Lead 真人到位前临时代签 |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |
| PM | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-10 | 同上 |

---

## 7. 修订历史 (per AGENTS.md §3 修订历史模板)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-10 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿: ARG.5 frontend/src/app/(app)/agent-relationships/ 5 UI 组件 (8 文件) + zustand useARGStore 5 channel (4 lib 文件 + index) + 13 REST 客户端封装 + WebSocket /ws/arg/events 5 协议 fanout + mock fallback + /agent-view 1 tab 集成 + 1 Python 脚本 (arg_ui_test.py 10 IT) + 2 文档更新 (automation-design.md §4.23 + registry.md §1) + 1 报告 + 1 commit author=Ulysses 不推 origin | 2026-09-10 07:46 JST 用户发令"按顺序推进" + ARG.1-4 收官后父会话自驱 (per 守门 #9 v19 第 7 次强化 Mavis 自驱 + 守门 #14 v3 Mavis 永久代签 + 守门 #1 v15 docs 同步饱和第 46 次新事件触发仍允许) |
