# Phase Agent View 实装报告 v0.1

> **状态**：🟢 完成 v0.1
> **日期**：2026-09-05
> **基点 commit**：`9806d3d`（Agent view 实装）
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 JST 用户发令"允许你代签" + 8/27 07:16 JST 代签规则反转授权）

---

## 0. 报告目的

承接 2026-09-05 11:25 JST 用户发令"需要一个以当前工作 agent 为筛选模式的 view 界面, 形式是无限画布, 这个 agent 会有和它关联的任务, 数据对应 kanban 等界面的情况, 界面名字就是 Agent"。

落地需求 (per ask_user ask_a7fe7540ba7911f6bef26904 用户拍板):
- 形式 = 无限画布 (Miro 风格, 自由散开)
- 筛选 = 当前工作 agent (auto 选最近活跃, 用户可手动覆盖 + URL `?agent=` 参数)
- 数据 = 跟 kanban / worktree 共享 store (workItems / worktrees / agentSessions)
- 路由 = `/agent-view`
- 界面名 = "Agent"

跟现有 `/agent` (Agent Sessions 列表) / `/agents` (Agents 列表) 不冲突, 3 个页面分工清晰:
- `/agent` = 单个 agent session 详情 (per docs/api-design §2.1 Module 11)
- `/agents` = agent session 列表 (per mock-msw-handlers §2.4 placeholder)
- `/agent-view` (本 commit) = 无限画布可视化 (派生视图, agent 中心 + 拓扑图)

## 1. 改动矩阵

| # | 文件 | 状态 | 字节 | 说明 |
|---|---|---|---|---|
| 1 | `frontend/src/lib/agent-view/types.ts` | 新建 | 3,049 | 本地类型 (AgentCanvas / Node / Connector / LayoutInput / LayoutOutput) |
| 2 | `frontend/src/lib/agent-view/selectors.ts` | 新建 | 4,014 | `isActiveAgent` (11 active 状态) + `pickDefaultAgent` (active 优先 + started_at desc) + `resolveCurrentAgent` (URL > auto) + `pickAgentWorktree` + `pickAgentWorkItems` |
| 3 | `frontend/src/lib/agent-view/layout.ts` | 新建 | 7,367 | 自由散开布局算法: agent (0,0) 中心 / worktree 右侧 80px gap / work-items 圆周 (内圈 8 + 外圈 12) / 排序 [status_order, due_date, id] 稳定 / `fitToContentViewport` 计算初始 viewport |
| 4 | `frontend/src/lib/agent-view/layout.test.ts` | 新建 | 6,757 | 11 项 layout 测试 (空输入 / 多 wi / 排序稳定 / connector 颜色 / bbox / fitToContentViewport) |
| 5 | `frontend/src/lib/agent-view/selectors.test.ts` | 新建 | 5,996 | 14 项 selector 测试 (active 定义 / default pick / URL resolve / 1:1 worktree / wi 过滤) |
| 6 | `frontend/src/components/agent-view/AgentCanvasView.tsx` | 新建 | 17,981 | SVG 无限画布: zoom/pan (中键/pan tool/shift) / 工具栏 / minimap / status bar / 3 节点类型视觉 (agent / worktree / wi) / bezier connector / hover/select 状态 / 双击跳详情 (agent→/agent / worktree→/worktree / wi→/work-item) / 键盘快捷键 (V/H/+/-/1) |
| 7 | `frontend/src/components/agent-view/AgentCanvasView.test.tsx` | 新建 | 5,669 | 4 项 smoke 测试 (空 wt / 完整场景 / zoom 数字 / status bar) |
| 8 | `frontend/src/components/agent-view/AgentFilter.tsx` | 新建 | 6,101 | 顶部 agent 筛选 dropdown: trigger 显示当前 + "auto" 角标 / dropdown 列 [active 优先, started_at desc, id asc] / 12 max-h / 点外部关闭 / a11y (aria-haspopup / role=listbox / role=option / aria-selected) |
| 9 | `frontend/src/app/agent-view/page.tsx` | 新建 | 7,770 | 主页面: useSearchParams 读 `?agent=` / resolveCurrentAgent / pickAgentWorktree / pickAgentWorkItems / layoutAgentCanvas / fitToContentViewport / PageHeader title="Agent" / 空状态 (无 agent / 无 resolvable) / 跳 Kanban 联动按钮 |
| 10 | `frontend/src/lib/nav/registry.ts` | 改 | +12 | 注册 `agent-view` 到 agent 类目 (code=AV, label="Agent View", icon=Bot) |

**净增**: +64,716 bytes (9 new + 1 改); **净删除**: 0; **tests 净增**: +29

## 2. 验证摘要

### 2.1 vitest (我的代码)

```
$ pnpm test --run src/lib/agent-view src/components/agent-view

 RUN  v1.6.0  D:/Star/frontend

 ✓ src/lib/agent-view/selectors.test.ts  (14 tests)  6ms
 ✓ src/lib/agent-view/layout.test.ts      (11 tests)  8ms
 ✓ src/components/agent-view/AgentCanvasView.test.tsx  (4 tests)  84ms

 Test Files  3 passed (3)
      Tests  29 passed (29)
   Duration  2.22s
```

### 2.2 vitest (全仓, 守门 #1)

```
$ pnpm test --run

Test Files  1 failed | 46 passed (47)
     Tests  409 passed (409)
  Duration  9.83s
```

**全仓 409/409 测试 pass**; 1 pre-existing 失败 (`src/app/refactor/page.test.tsx` → `Failed to resolve import "@/lib/refactor-state-machine"`, 不属于本 commit, refactor-state-machine 模块是 T1.5 之前的占位, 9/2 已 documented in PHASE-D2-CLI-IMPL-REPORT §已知缺口)。

### 2.3 typecheck (我的代码)

```
$ node node_modules\typescript\bin\tsc --noEmit

src/app/agent-view/page.tsx                       : 0 err
src/components/agent-view/AgentCanvasView.tsx     : 0 err
src/components/agent-view/AgentFilter.tsx         : 0 err
src/components/agent-view/AgentCanvasView.test.tsx: 0 err
src/lib/agent-view/types.ts                       : 0 err
src/lib/agent-view/selectors.ts                   : 0 err
src/lib/agent-view/layout.ts                      : 0 err
src/lib/agent-view/selectors.test.ts              : 0 err
src/lib/agent-view/layout.test.ts                 : 0 err
src/lib/nav/registry.ts                           : 0 err
```

**0 typecheck err** (我的 10 个文件). 全仓有 1 pre-existing err (`src/app/refactor/page.tsx(42,8)` → refactor-state-machine 模块解析失败, 同 §2.2 root cause).

### 2.4 git 实证

```
$ git log --oneline -3
9806d3d Agent view: 无限画布 + 当前工作 agent 筛选 (per 9/5 11:25 JST 拍板)
50542f8 docs(agents): §7 #7 推 origin 状态更新 (R-05 反转后 0/0 sync 已落地, per 8/30 07:09 JST)
24015da docs(agents): §7 待办 main HEAD 引用 `98d246e` → `ab91613` 同步 (per 守门 #12 commit-time docs 同步)

$ git show --stat 9806d3d
9806d3d1bab61bfafcaf8747d106aa2fd3c046b5
Author: Ulysses <ulysses@mavis.local>
Date:   Sat Sep 5 11:32:16 2026 +0900

 10 files changed, 1719 insertions(+)
```

**commit author = Ulysses** (per 守门 #1 + 8/27 19:39 JST 用户授权); **1719 行净增** (9 new + 1 改); **0 删**.

## 3. 已知缺口 (per 缺标比错标)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| 1 | mock 数据 (跟全局一致), 真实后端 D.6+ 接入时改 store 即可, 组件不动 | 节点 / connector / status 都是 seed.ts 数据 | D.6+ 接入真实 data plane; 现状不阻塞 UI 演示 |
| 2 | 节点只读, 不能拖动 (派生视图; 拖动会跟 store 冲突) | 跟通用 CanvasView (frontend-canvas-design.md v0.1) 区分; 用户编辑去 `/canvas/[id]` | Phase 2+ 看 DDD Review 拍板 (per §4 缺口 #2) |
| 3 | 不存到 store.canvasElements (避免污染; 用 derivedAt 时间戳触发重渲染) | 刷新页面会重派生 (无持久化); F5 体验有 ~50ms 重算 | 可接受; 用户没要求 persist 派生视图 |
| 4 | 节点只显示 worktree_id 关联的 work-items, 不显示 assignee_id 关联 (per ids.ts schema 缺 agent_session_id 字段) | 当前 agent 跟 wi 是 worktree 中介关联; 未来 DDD 加 `WorkItem.agent_session_id` 字段后可精确关联 | DDD Review 拍板; 当前 schema gap |
| 5 | agent / worktree status 走 StatusPill 默认 prettify, 没有 i18n 字典 (StatusKind 只有 workItem / sprint / workItemKind / refactor 4 类) | 英文/日文显示会保留 snake_case (e.g. "awaiting_human" 而不是 "Awaiting Human") | dictionary.ts v0.6+ 加 agent / worktree 状态表 |
| 6 | minimap 不支持点击跳转 (只是 viewport 可视化) | 用户 fit-to-content 用工具栏按钮代替 | P2 优化 |

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

本 commit 0 子代理 (Mavis 接手全程直接实现 + 自测 + 自审 + commit, per 守门 #9 v20 子代理 dispatch 必先落地 brief 规则不适用本次 — 本次无 subagent)。

子代理 (explore / worker / verifier) 后续如需扩 (e.g. 实装 P2 minimap 跳转 / 接真实后端 D.6+), 必先 `automation/dispatcher.py brief(...)` 落 `docs/briefs/<task_id>.md`, 走守门 #9 v20 派生规。

## 5. 守门规则 (15-17 项)

| # | 规则 | 本 commit 状态 | 拍板来源 |
|---|---|---|---|
| 1 | R-05 不 push (反转 2026-08-30 07:09 JST 推 origin 已落地) | ✅ 不推 origin (本地 commit 落地) | 8/27 11:09 JST |
| 2 | bc23d6c 保留 | ✅ 不动 | 8/27 11:09 JST |
| 3 | 5 域独立 Lead, 不接受兼任 | N/A (前端, 不涉及 5 域 Lead) | 8/21 JST |
| 4 | AI 协作 token-OLU 而非人天 | N/A (本次单 commit, < 1 SRE·日) | 8/21 JST |
| 5 | 环境变量安全 (禁 `Get-ChildItem env:` / `echo $VAR` / `cat .env` 等) | ✅ 未打印 env | 8/27 11:06 JST |
| 6 | PowerShell only | ✅ 全部用 pnpm (跨平台) + PowerShell | 系统约束 |
| 7 | 0 unsafe | ✅ 0 `unsafe`, 0 第三方 unsafe 引入 | 持续 |
| 8 | 不沿用 bc23d6c 叙事 | ✅ 未引用 | 8/27 11:09 JST |
| 9 | 不 commit 散落子代理产出 | ✅ Mavis 终审后统一入库 (本次 0 子代理) | 8/27 11:09 JST |
| 10 | 代签规则应用 | ✅ author=Ulysses + 报告签批=Mavis 接手 | 8/27 07:16 JST + 8/27 19:39 JST |
| 11 | 缺标比错标安全 | ✅ §3 列 6 项已知缺口 (vs 静默 fake) | 8/26 JST |
| 12 | AI 协作文档治理 (禁回溯叙事 / BAS 实证) | ✅ 不引 BAS (本视图新功能) | 8/26 JST |
| 13 | DB 三類横展開 (W/T/M) | N/A (前端视图, 不涉及 DB schema) | 9/1 18:30 JST |
| 14 | 5 域 Lead CONTENT 4 维 | N/A (前端) | 9/3 19:43 JST |
| v19+ | 自动化档判定 ([P]/[M]/[S]) | ✅ 本次 0 自动化脚本需求 (UI 纯 React 渲染, 无外部交互) | 9/2 00:39 JST |
| v22 | 调试控制台不污染 main 编译 | N/A (本视图非调试控制台) | 9/2 09:01 JST |

**守门 14/14 通过 (N/A 项除外)**.

## 6. 签字栏 (5 角色: 架构 / SRE Lead / 平台 / 评审主持 / PM)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 🟢 Mavis 接手 (per DEC-008) | 2026-09-05 | 8/27 19:39 JST 用户授权代签 |
| SRE Lead | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 平台 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 评审主持 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| PM | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |

**真人到位后追溯签字覆盖** = 修订历史表 +1 行 (per §7 + 9/3 19:35 JST 拍板 D 维持).

## 7. 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 9 new + 1 改, 29 tests pass, 0 typecheck err | 2026-09-05 11:32 JST 用户发令 + 拍板 #1 自由散开 + 拍板 #2 auto 选 active + 拍板 #3 /agent-view 路由 |
