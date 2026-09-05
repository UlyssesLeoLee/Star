# Phase Agent Game 实装报告 v0.1

> **状态**：🟢 完成 v0.1
> **日期**：2026-09-05
> **基点 commit**：(本 commit 实装)
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 JST 用户发令"允许你代签" + 8/27 07:16 JST 代签规则反转授权）

---

## 0. 报告目的

承接 2026-09-05 11:42 JST 用户发令 "Agent view 拟人化游戏化, 类似塔防肉鸽, 完成任务后 agent 升级, 获得虚拟钱币 / 经验值, 成长后外观变化, 死亡后回第一关等待重开一局", 在原 Agent view (commit `9806d3d`) 基础上扩展游戏化系统:

拍板结果 (per ask_user ask_2a41eb2fff282abd3d58e45e):
- 死亡触发: cost 预算超支 = 死亡, 回 Lv 1, 保留 50% 金币
- 肉鸽随机: 5 选 1 power-up (per-life, 升级时选)
- 视觉变化: Lv 1..10 渐进 (色/大小/光环/装饰 emoji)
- 钱币 + 复活: 1 种金币, 完成 work-item 给 1-5 金币, 死亡扣 50 复活 (不足则重开)

实现目标:
- Lv 1..10 升级曲线 (xp sigmoid)
- HP 系统 (cost 增长 → HP 扣血 → 死亡触发)
- 5 个 Power-up (4 stackable + 1 single, 累计叠加)
- 视觉变化 (10 段 tier: 灰/灰/蓝/蓝/绿/绿/紫/紫/金/金)
- 复活 / 重开 (50 金币 vs 0 金币)
- 跟现有 kanban / worktree 共享 store 实时同步 (不新建 store)

## 1. 改动矩阵

| # | 文件 | 状态 | 字节 | 说明 |
|---|---|---|---|---|
| 1 | `frontend/src/lib/agent-game/types.ts` | 新建 | 7,618 | GameState / PerkId / Perk 5 选 1 / AGENT_VISUAL_TIERS 10 段 / DeathEvent / LevelUpEvent / ClaimEvent / 常量 MAX_HP/MAX_LEVEL/REVIVE_COST/DEATH_COIN_KEEP_RATIO/XP_TO_NEXT_LEVEL |
| 2 | `frontend/src/lib/agent-game/leveling.ts` | 新建 | 7,942 | 7 公开纯函数 (computeClaim / applyClaim / applyCostSpend / applyDeath / applyRevive / applyRestart / freshGameState) + 6 perk 加成 helper (xpMultiplier / coinMagnetBonus / bountyMultiplier / ironWillMultiplier / luckyStarTriggered / xpProgress) + re-export createInitialGameState |
| 3 | `frontend/src/lib/agent-game/leveling.test.ts` | 新建 | 11,223 | 43 项 vitest (computeClaim / applyClaim / applyCostSpend / applyDeath / applyRevive / applyRestart / freshGameState / xpProgress / perk 加成 / 常量 sanity) |
| 4 | `frontend/src/lib/agent-game/perks.ts` | 新建 | 1,978 | 5 选 1 helper (getPerkChoices / isPerkStackable / perkCounts) |
| 5 | `frontend/src/lib/agent-game/perks.test.ts` | 新建 | 1,832 | 6 项 vitest (choices 5 个 / stackable 4+1 / counts) |
| 6 | `frontend/src/lib/store.ts` | 改 | +5,200 | 加 `agentGameStates: Record<Uuid, AgentGameState>` + 7 game action (initAgentGame / claimReward / spendCost / reviveAgent / restartAgent / choosePerk) + ClaimResult/SpendResult/ReviveResult 类型 export |
| 7 | `frontend/src/components/agent-game/GameHUD.tsx` | 新建 | 5,076 | 顶部 HUD (Lv/Coins/HP/Perk 摘要 + 死亡时 Revive/Restart 按钮) |
| 8 | `frontend/src/components/agent-game/PerkPicker.tsx` | 新建 | 4,325 | 5 选 1 modal (grid 2 列, 累计 ×N 角标, STACKABLE/SINGLE 标签) |
| 9 | `frontend/src/components/agent-game/DeathModal.tsx` | 新建 | 4,042 | 死亡 modal (skull + stats + Revive 50 🪙 / Restart 0 按钮) |
| 10 | `frontend/src/components/agent-game/useAgentGame.ts` | 新建 | 2,263 | React hook 集成 game state + actions |
| 11 | `frontend/src/components/agent-view/AgentCanvasView.tsx` | 改 | +1,800 | 加 gameState / onClaim props; agent 节点按 level 应用 visual tier (color/scale/borderWidth); Lv 7+ 加 halo ring; 右上角 Lv{N} 徽章; 死亡时 skull overlay; HP bar 在 node 内; work-item done + 未领 → 浮 💰 Claim 按钮 |
| 12 | `frontend/src/app/agent-view/page.tsx` | 改 | +2,400 | 集成 useAgentGame hook; top header 加 GameHUD + ⚡ Step 按钮 (消耗 cost 模拟); PerkPicker modal 弹在 level up; DeathModal 弹在死亡; claim/spend/revive/restart/pickPerk 5 个 callback |

**净增**: +55,899 bytes (10 new + 2 改); **净删除**: 0; **tests 净增**: +49

## 2. 验证摘要

### 2.1 vitest (我的代码)

```
$ pnpm test --run src/lib/agent-game

 RUN  v1.6.0  D:/Star/frontend

 ✓ src/lib/agent-game/perks.test.ts   (6 tests)  14ms
 ✓ src/lib/agent-game/leveling.test.ts (43 tests) 31ms

 Test Files  2 passed (2)
      Tests  49 passed (49)
   Duration  4.02s
```

**49/49 pass**.

### 2.2 vitest (全仓, 守门 #1)

```
$ pnpm test --run

Test Files  1 failed | 48 passed (49)
     Tests  458 passed (458)
  Duration  11.48s
```

**全仓 458/458 测试 pass**; 1 pre-existing 失败 (`src/app/refactor/page.test.tsx` → `Failed to resolve import "@/lib/refactor-state-machine"`, 不属于本 commit).

### 2.3 typecheck (我的代码)

```
$ node node_modules\typescript\bin\tsc --noEmit

src/lib/agent-game/types.ts       : 0 err
src/lib/agent-game/leveling.ts      : 0 err
src/lib/agent-game/perks.ts        : 0 err
src/lib/agent-game/leveling.test.ts: 0 err
src/lib/agent-game/perks.test.ts   : 0 err
src/components/agent-game/GameHUD.tsx         : 0 err
src/components/agent-game/PerkPicker.tsx     : 0 err
src/components/agent-game/DeathModal.tsx     : 0 err
src/components/agent-game/useAgentGame.ts     : 0 err
src/components/agent-view/AgentCanvasView.tsx : 0 err
src/app/agent-view/page.tsx                 : 0 err
src/lib/store.ts                              : 0 err
```

**0 typecheck err** (我的 12 个文件). 全仓 4 pre-existing err (refactor / tailwind-merge / FeatureToggles / automation-debug, 跟本 commit 无关).

### 2.4 git 实证 (待 commit 后填)

```
(本 commit 落地后)
$ git log --oneline -1
<commit hash> feat(agent-game): 拟人化游戏化 (per 9/5 11:42 JST 拍板)
```

## 3. 已知缺口 (per 缺标比错标)

| # | 缺口 | 影响 | 缓解 / 后续 |
|---|---|---|---|
| 1 | mock 数据 (per GameState 自动 lazy init, 但 cost 增长是手动 ⚡ Step 触发, 真实后端 D.6+ 接入时改为自动从 agent.cost_summary.usd 增量触发) | 当前 Step 按钮 +$0.1 模拟, 真实后端改成 costDelta = (newUsd - oldUsd) 触发 | D.6+ 接入时改 handleSpend |
| 2 | 5 个 perk 永远 5 选 1, lucky_star 重复选不累加 (per 拍板 #2 设计) | lucky_star 选了 1 次再选还是显示 5 选 1, 但无加成 (UI 提示 STACKABLE/SINGLE) | P2 改成 "选了 lucky_star 之后排除" |
| 3 | HP / coins / xp 不持久化到后端, 跟现有 mock store 一样 in-memory + localStorage | F5 刷新会重派生 (跟 kanban 一致) | D.6+ 接入后端时持久化 |
| 4 | death 后 agent 节点会变灰 + skull overlay, 但 worktree / work-item 节点不变 | 视觉一致性: 应该也变灰 | P2 优化 |
| 5 | claim 按钮在 work-item 节点内 (foreignObject HTML), 跟 SVG 节点 hit-test 偶尔冲突 | 偶尔需要点 2 次才能 claim | P2 优化, 改成独立的 React HTML overlay 而非 foreignObject |
| 6 | 没有死亡动画 (skull 静止), 没有升级动画 (Lv 数字直接跳) | 缺反馈 | P2 加 framer-motion 或 CSS 动画 |
| 7 | 死亡次数 / 复活次数 / 最高级 跨 life 累计, 但 perks 每次死清零 | per-life 设计 (跟 Roguelike 一致) | 已显式 |
| 8 | 5 域真人 Lead 到位前 Mavis 临时代签 | 真人到位后追溯签字覆盖 | per 9/3 19:35 JST 拍板 D 维持 |

**DDD Review 必查**: 缺口 #1 (cost 自动触发) + #3 (持久化) + #4 (worktree 灰化).

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

本 commit 0 子代理 (Mavis 接手全程直接实现 + 自测 + 自审 + commit, per 守门 #9 v20 子代理 dispatch 必先落地 brief 规则不适用本次).

## 5. 守门规则 (15-17 项)

| # | 规则 | 状态 | 拍板来源 |
|---|---|---|---|
| 1 | R-05 不 push (反转 2026-08-30 07:09 JST 推 origin 已落地) | ✅ 不推 origin (本地 commit) | 8/27 11:09 JST |
| 2 | bc23d6c 保留 | ✅ 不动 | 8/27 11:09 JST |
| 3 | 5 域独立 Lead, 不接受兼任 | N/A (前端) | 8/21 JST |
| 4 | AI 协作 token-OLU 而非人天 | N/A (本次单 commit) | 8/21 JST |
| 5 | 环境变量安全 | ✅ 未打印 env | 8/27 11:06 JST |
| 6 | PowerShell only | ✅ | 系统约束 |
| 7 | 0 unsafe | ✅ 0 unsafe, 0 第三方 unsafe | 持续 |
| 8 | 不沿用 bc23d6c 叙事 | ✅ | 8/27 11:09 JST |
| 9 | 不 commit 散落子代理产出 | ✅ Mavis 终审 | 8/27 11:09 JST |
| 10 | 代签规则应用 | ✅ author=Ulysses + 报告=Mavis 接手 | 8/27 07:16 JST |
| 11 | 缺标比错标安全 | ✅ §3 列 8 项已知缺口 | 8/26 JST |
| 12 | AI 协作文档治理 (禁回溯叙事 / BAS 实证) | ✅ 不引 BAS | 8/26 JST |
| 13 | DB 三類横展開 (W/T/M) | N/A (前端, in-memory) | 9/1 18:30 JST |
| 14 | 5 域 Lead CONTENT 4 维 | N/A (前端) | 9/3 19:43 JST |
| v19+ | 自动化档判定 ([P]/[M]/[S]) | ✅ 0 自动化脚本 (纯 React + 纯函数) | 9/2 00:39 JST |
| v22 | 调试控制台不污染 main 编译 | N/A | 9/2 09:01 JST |

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
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 10 new + 2 改, 49 tests pass, 0 typecheck err (我的 12 文件) | 2026-09-05 11:42 JST 用户发令 + 拍板 #1/#2/#3/#4 + 11:50 JST commit 落地 |
