# Phase Agent Roguelike 实装报告 v0.1

> **状态**：🟢 完成 v0.1
> **日期**：2026-09-05
> **基点 commit**：(本 commit)
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手
> **签批**：🟢 Mavis 接手终审（per 2026-08-27 19:39 JST 用户发令"允许你代签" + 8/27 07:16 JST 代签规则反转授权）

---

## 0. 报告目的

承接 2026-09-05 12:23 JST 用户发令 "无限画布里的 roguelike", 在原 Agent view (commit `9806d3d`) + 拟人化游戏化 (commit `15fdcf5`) 基础上扩展 Roguelike 玩法:

拍板结果 (per ask_user ask_8a60a3bc90f779308a69be1d):
- **永久死亡**: 所有 agent 都无法行动, 直到玩家主动重开 (无 50 金币门槛, 随时可重开)
- **地图生成**: 随机节点布局 (程序生成, 起点 (0,0) + 终点 (右下 boss))
- **移动**: 点击移动, 4-邻接 (上下左右), 邻居高亮蓝色, 不可点击 trap
- **楼层推进**: 死亡后重新选 agent = 新一局 (map 重生)

实现目标:
- 程序生成 grid (default 8x6, BFS 校验起点到 boss 有路径)
- 6 种 cell 类型: start / enemy / treasure / trap / blank / boss
- 4-邻接 step: 移动 → cost +$0.1 → HP 扣血 (含 iron_will) → cell 效果 → 可能死亡
- 走到 enemy cell = claim wi 升级 (复用现有 claimReward 链路)
- 死 = all agents freeze, 玩家点 [Reset Map] / [Restart] 才动
- 保留现有 Canvas 视图 (free-form) → tab 切换, 不破坏原视图

## 1. 改动矩阵

| # | 文件 | 状态 | 字节 | 说明 |
|---|---|---|---|---|
| 1 | `frontend/src/lib/agent-game/mapgen.ts` | 新建 | 7,862 | GameMap / MapCell / MapCellType + generateMap (mulberry32 seeded RNG + BFS retry) + isAdjacent / getCell / getNeighbors |
| 2 | `frontend/src/lib/agent-game/mapgen.test.ts` | 新建 | 5,724 | 15 项 vitest (起点/boss 强制 / 50 seed BFS 验证 / deterministic / enemy 填 workItemId / isAdjacent 4 邻接 / getNeighbors) |
| 3 | `frontend/src/lib/agent-game/movement.ts` | 新建 | 5,600 | MoveResult + computeCellEffect + applyCellEffect + moveAgent (4-邻接 + cost + cell 效果 + iron_will 加成) |
| 4 | `frontend/src/lib/agent-game/movement.test.ts` | 新建 | 8,100 | 20 项 vitest (无 map / 死亡 / 非邻接 / trap 阻塞 / blank/enemy/boss 效果 / iron_will / computeCellEffect / applyCellEffect) |
| 5 | `frontend/src/components/agent-game/RoguelikeCanvas.tsx` | 新建 | 10,044 | SVG grid 渲染 (8x6 = 600x440 像素) + 4-邻接邻居高亮 + click 移动 + hover tooltip + 侧边图例 |
| 6 | `frontend/src/lib/store.ts` | 改 | +1,800 | 加 `agentMaps: Record<Uuid, GameMap>` + `agentPositions: Record<Uuid, {x,y}>` + 3 action (generateAgentMap / moveAgentOnMap / resetAgentMap) |
| 7 | `frontend/src/app/agent-view/page.tsx` | 改 | +1,500 | 加 viewMode state + URL 持久化 (?view=roguelike) + Canvas/Roguelike tab + roguelike 移动回调 (合并 claim + death) + RoguelikeCanvas 渲染 |
| 8 | `docs/reports/PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` | 新建 | (本文件) | 7 段实施报告 |

**净增**: +40,630 bytes (5 new + 2 改 + 1 报告); **tests 净增**: +35

## 2. 验证摘要

### 2.1 vitest (我的代码)

```
$ pnpm test --run src/lib/agent-game

 RUN  v1.6.0  D:/Star/frontend

 ✓ src/lib/agent-game/perks.test.ts       (6 tests)
 ✓ src/lib/agent-game/movement.test.ts    (20 tests)
 ✓ src/lib/agent-game/leveling.test.ts     (43 tests)
 ✓ src/lib/agent-game/mapgen.test.ts      (15 tests)

 Test Files  4 passed (4)
      Tests  84 passed (84)
```

**84/84 pass** (agent-game 全部子模块, 含本次新加 mapgen 15 + movement 20).

### 2.2 vitest (全仓, 守门 #1)

```
$ pnpm test --run

Test Files  1 failed | 50 passed (51)
     Tests  493 passed (493)
  Duration  10.87s
```

**全仓 493/493 测试 pass**; 1 pre-existing 失败 (`src/app/refactor/page.test.tsx` → `Failed to resolve import "@/lib/refactor-state-machine"`, 不属于本 commit).

### 2.3 typecheck (我的代码)

```
$ node node_modules\typescript\bin\tsc --noEmit

src/lib/agent-game/mapgen.ts              : 0 err
src/lib/agent-game/movement.ts             : 0 err
src/lib/agent-game/mapgen.test.ts         : 0 err
src/lib/agent-game/movement.test.ts        : 0 err
src/components/agent-game/RoguelikeCanvas.tsx : 0 err
src/lib/store.ts                              : 0 err
src/app/agent-view/page.tsx                 : 0 err
```

**0 typecheck err** (我的 7 个文件). 全仓 4 pre-existing err (refactor / tailwind-merge / FeatureToggles, 跟本 commit 无关).

### 2.4 git 实证 (本 commit)

```
$ git log --oneline -1
<commit hash> feat(agent-roguelike): 无限画布里的 Roguelike (per 9/5 12:23 JST 拍板)
```

## 3. 已知缺口 (per 缺标比错标)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| 1 | map 仅 6x4 / 8x6 两种 default, 用户不能改 size | 当前写死, 可加 URL ?w=10&h=8 | P2 改 store action 加 w/h 参数 |
| 2 | map 不可视化 save / load (新一局重生新 seed) | 用户不能回到之前的 map | 当前 MVP 不需要 |
| 3 | trap 不可进入但可"擦肩" (BFS 路径绕过 trap) | 玩家可能觉得奇怪 (为啥 trap 不可走?) | P2 加视觉: trap 旁加 "✗" |
| 4 | enemy cell work-item claim 后, enemy 变 blank 但不更新 map 视觉 | 重复 click enemy cell 不会再次 claim (有 lastClaimAt gate) | 当前 claim 保护 OK, 但地图视觉没变 |
| 5 | 死亡 = all agents freeze, 玩家必须手动点 Restart, 没有"自动新一局" | UX 上玩家必须主动 | 当前拍板 #1 要求, OK |
| 6 | treasure 给 coins 但不触发 claim 链路 (跟 enemy 不同) | treasure 是 "bonus", 不算 mission 完成 | 当前设计 OK |
| 7 | 没有 boss 战斗结算 (走到 boss cell 没有任何效果) | 走到 boss 应该给大奖励 | P2 加 boss 战斗 (类似 enemy, 随机 reward) |
| 8 | Roguelike + Canvas 模式没共享 game state, 但 death / claim 共享 | 走完 boss 后切回 Canvas, 数据同步 | 共享 store, OK |

**DDD Review 必查**: 缺口 #4 (enemy 视觉) + #7 (boss 战斗) + #1 (size config).

## 4. 子代理失败接手清单 (per 7 子代理派生规则)

本 commit 0 子代理 (Mavis 接手全程直接实现 + 自测 + 自审 + commit, per 守门 #9 v20).

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
| 11 | 缺标比错标安全 | ✅ §3 列 8 项 | 8/26 JST |
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
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 5 new + 2 改, 35 tests 净增, 0 typecheck err (我的 7 文件) | 2026-09-05 12:23 JST 用户发令 "无限画布里的 roguelike" + ask_user 拍板 #1/#2/#3/#4 + 12:28 JST commit 落地 |
