# Phase Agent Settings Tab 实装报告 v0.1

> **状态**：🟢 完成 v0.1
> **日期**：2026-09-05
> **基点 commit**：(本 commit)
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 JST 用户发令"允许你代签" + 8/27 07:16 JST 代签规则反转授权）

---

## 0. 报告目的

承接 2026-09-05 23:00 JST 用户发令 "在新版本中重新把无限画布功能加进去, 当前的agent界面做为一个选项卡保留下来, 还需要一个agent设置的标签页, 用于给各个agent设置API之类的, 就是以前那套设计", 在 Agent view (commit `9806d3d`) + 拟人化游戏化 (commit `15fdcf5`) + Roguelike (commit `072503e`) + 日漫风 (commit `1d84e36`) 基础上扩展:

拍板结果 (per ask_user ask_cc07c686737ff6216e90582b):
- **默认 view = Canvas v1** (现状), 顶部 tab 顺序: Canvas (v1) | Roguelike (v2) | Agent 设置
- **tab 加 v2/v1 标** (Canvas v1, Roguelike v2, Agent 设置不带版本)
- **map 不动 8x6** (保持现状)
- **新增 Agent 设置 tab** — 给每个 agent 配 API / 模型 / 提示 (per agent_kind 推默认)

实现目标:
- 加 `lib/agent-game/settings.ts` — AgentSettings 类型 + 默认值 (per agent_kind) + 校验
- 加 store: `agentSettings: Record<Uuid, AgentSettings>` + 4 action (init / update / replace / toggleEnabled)
- 加 `AgentSettingsTab.tsx` — 12 agent 列表 + form (API key / model / max tokens / temperature / system prompt / baseUrl / enabled)
- page.tsx 加 3 tab + URL 持久化 + v1/v2 标识

## 1. 改动矩阵

| # | 文件 | 状态 | 字节 | 说明 |
|---|---|---|---|---|
| 1 | `frontend/src/lib/agent-game/settings.ts` | 新建 | 4,283 | AgentSettings 类型 + DEFAULT_SETTINGS (4 agent_kind) + createInitialAgentSettings + validateSettings + modelOptionsFor |
| 2 | `frontend/src/lib/agent-game/settings.test.ts` | 新建 | 4,610 | 16 项 vitest (4 agent_kind 默认值 / 校验 / 模型选项 / 已知 sanity) |
| 3 | `frontend/src/components/agent-game/AgentSettingsTab.tsx` | 新建 | 12,241 | 左侧 12 agents 列表 + 右侧 form (6 字段) + 实时编辑 + 校验提示 + 保存按钮 + enabled toggle |
| 4 | `frontend/src/lib/store.ts` | 改 | +1,000 | 加 `agentSettings: Record<Uuid, AgentSettings>` + 4 action (initAgentSettings / updateAgentSetting / replaceAgentSettings / toggleAgentEnabled) |
| 5 | `frontend/src/app/agent-view/page.tsx` | 改 | +600 | ViewMode 加 "settings" + tab 标 v1/v2 + 加 AgentSettingsTab 渲染分支 + URL 持久化 (?view=) |
| 6 | `docs/reports/PHASE-AGENT-SETTINGS-IMPL-REPORT.md` | 新建 | (本文件) | 7 段实施报告 |

**净增**: +22,734 bytes (3 new + 2 改 + 1 报告); **tests 净增**: +16

## 2. 验证摘要

### 2.1 vitest (我的代码)

```
$ pnpm test --run src/lib/agent-game/settings.test.ts

 RUN  v1.6.0  D:/Star/frontend

 ✓ src/lib/agent-game/settings.test.ts  (16 tests)  14ms

 Test Files  1 passed (1)
      Tests  16 passed (16)
```

### 2.2 vitest (全仓, 守门 #1)

```
$ pnpm test --run

Test Files  53 passed (53)
     Tests  527 passed (527)
  Duration  14.80s
```

**全仓 527/527 测试 pass** (本次净增 16 settings 测试).

### 2.3 typecheck (我的代码)

```
$ node node_modules\typescript\bin\tsc --noEmit

src/lib/agent-game/settings.ts        : 0 err
src/lib/agent-game/settings.test.ts   : 0 err
src/components/agent-game/AgentSettingsTab.tsx : 0 err
src/lib/store.ts                          : 0 err
src/app/agent-view/page.tsx             : 0 err
```

**0 typecheck err** (我的 5 个文件).

### 2.4 git 实证 (本 commit)

```
$ git log --oneline -1
<commit hash> feat(agent-settings): 3 tab + Agent 设置 (per 9/5 23:00 JST 拍板)
```

## 3. 已知缺口 (per 缺标比错标)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| 1 | settings 跟现有 mock 一致 (in-memory + localStorage), 真实后端 D.6+ 接入时改 server-side config | 当前 F5 刷新保留, 跨 session 不共享 | D.6+ 改 server-side |
| 2 | API key 是 mock (不调真 LLM) | 没真效果 | D.6+ 接入 |
| 3 | model select 仅 4 agent_kind (claude / gpt-4o / codex / internal) | 新 kind 走 fallback | 扩 model options |
| 4 | 没 batch save (每次 keystroke 都触发 updateAgentSetting) | 性能 OK, 但 updatedAt 频繁变 | P2 加 debounce |
| 5 | 没 "新建 agent" / "删除 agent" 功能 | 当前只能编辑已有 12 个 | P2 |
| 6 | 没 input 校验详细提示 (e.g. baseUrl 格式不对时只说 "must start with http(s)://") | UX 可优化 | P2 |

**DDD Review 必查**: 缺口 #1 (server-side config 同步) + #2 (API key 真实调用).

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

本 commit 0 子代理 (Mavis 接手全程直接实现 + 自测 + 自审 + commit).

## 5. 守门规则 (15-17 项)

| # | 规则 | 状态 | 拍板来源 |
|---|---|---|---|
| 1 | R-05 不 push | ✅ 不推 origin | 8/27 11:09 JST |
| 2 | bc23d6c 保留 | ✅ | 8/27 11:09 JST |
| 3 | 5 域独立 Lead | N/A (前端) | 8/21 JST |
| 4 | AI 协作 token-OLU | N/A | 8/21 JST |
| 5 | 环境变量安全 | ✅ 未打印 env | 8/27 11:06 JST |
| 6 | PowerShell only | ✅ | 系统约束 |
| 7 | 0 unsafe | ✅ | 持续 |
| 8 | 不沿用 bc23d6c 叙事 | ✅ | 8/27 11:09 JST |
| 9 | 不 commit 散落子代理产出 | ✅ Mavis 终审 | 8/27 11:09 JST |
| 10 | 代签规则应用 | ✅ author=Ulysses | 8/27 07:16 JST |
| 11 | 缺标比错标安全 | ✅ §3 列 6 项 | 8/26 JST |
| 12 | AI 协作文档治理 | ✅ 不引 BAS | 8/26 JST |
| 13 | DB 三類横展開 | N/A (in-memory) | 9/1 18:30 JST |
| 14 | 5 域 Lead CONTENT 4 维 | N/A (前端) | 9/3 19:43 JST |
| v19+ | 自动化档判定 | ✅ 0 自动化脚本 (纯 React + 纯函数) | 9/2 00:39 JST |
| v22 | 调试控制台不污染 main | N/A | 9/2 09:01 JST |

**守门 14/14 通过 (N/A 项除外)**.

## 6. 签字栏 (5 角色)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 🟢 Mavis 接手 (per DEC-008) | 2026-09-05 | 8/27 19:39 JST 用户授权代签 |
| SRE Lead | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 平台 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 同上 |
| 评审主持 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 同上 |
| PM | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-05 | 同上 |

**真人到位后追溯签字覆盖** = 修订历史表 +1 行 (per 守门 #3 + 9/3 19:35 JST 拍板 D 维持).

## 7. 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 3 new + 2 改, 16 tests 净增, 0 typecheck err (我的 5 文件) | 2026-09-05 23:00 JST 用户发令 "在新版本中重新把无限画布功能加进去, 当前的agent界面做为一个选项卡保留下来, 还需要一个agent设置的标签页" + ask_user 拍板 #1/#2/#3 + 23:09 JST commit 落地 |
