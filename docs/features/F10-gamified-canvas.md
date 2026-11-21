# F10 游戏化画布

> 状态摘要: 弹幕 Roguelike 的前端逻辑（机器人状态机、子弹碰撞、敌方弹幕 5 模式、道具、大招）与覆盖层组件存在于 `frontend/src/lib/canvas-game/` 与 `frontend/src/components/canvas-game/`，并有 17 个单测用例，但 637ed19d 上游戏层没有任何挂载点，用户无法进入；服务端 canvas-game 与 star-game 已退役，SRS-CANVAS-GAMIFY-001 的 G1–G12 游戏化能力仅为 Draft，无代码。

> 代码基线: 637ed19d; 核对方法: git reads at the pin (paths and grep); 核对日期 2026-11-19.

## 1. 目标与边界

### 1.1 目标

把画布上的工作项卡片（`work_item_card`）与智能体光标（`agent_cursor`）转为像素风弹幕 Roguelike 的敌人：玩家操控机器人击杀敌人、拾取升级道具、死亡后重开。前端可玩版的设计依据是 `docs/design/BD-STAR-CANVAS-GAME-001.md` 及其 SRS / DD，用户命令与范围见 `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:971`（23.36）。

另一套设计 `docs/design/BD-CANVAS-GAMIFY-001.md`（对应 `SRS-CANVAS-GAMIFY-001`）的目标是积分、等级、技能树、徽章、投票、表情回应、排行榜与每日挑战接入画布协作，目前为 Draft（见 §3、§8）。

### 1.2 范围内 / 范围外

| 类别 | 范围内 | 范围外 |
|---|---|---|
| 客户端玩法 | 机器人移动、攻击、翻滚、大招；子弹与碰撞；敌方弹幕；道具掉落与拾取；HUD；重开 | 3渲2（3D 离屏渲染为 2D 精灵）机器人；程序化关卡；多人合作 |
| 画布集成 | 由画布元素生成敌人种子（`EnemySpawnSeed`） | 画布页本身（归 F06 统一画布，见 §4.7） |
| 游戏化 G1–G11 | 作为需求条目登记（§3） | 因设计存在而标记为已实现 |
| 服务端 | 无（canvas-game crate 已退役） | 游戏实体表、REST 与 WebSocket 端点（设计项） |
| 退役内容 | 记录依赖断裂与退役依据 | 不作为当前能力 |
| 相邻组件 | — | `frontend/src/components/effects/PhysisHud.tsx:23`（physis 主题 HUD，与本功能无关） |

## 2. 用户与场景

| 角色 | 主要场景 | 当前可达性 |
|---|---|---|
| 画布编辑者（玩家） | 开启游戏模式，操控机器人击杀由节点生成的敌人，拾取道具，死亡后重开 | 不可达：挂载点已移除（FR-10-01） |
| 画布编辑者（节点作者） | 工作项与智能体节点分别生成强度不同的敌人 | 仅逻辑层存在，无生产者（FR-10-09） |
| 团队成员（草案） | 对便签投票、表情回应、查看进度与徽章 | ⬜ 未实现 |
| 管理员（草案） | 查看 workspace / tenant 排行榜 | ⬜ 未实现 |
| 智能体（草案） | 作为游戏角色或敌人参与关卡 | ⬜ 未实现 |

## 3. 需求

优先级取自源文档的 FR 表（`docs/requirements/SRS-STAR-CANVAS-GAME-001.md` 的 FR-GAME 条目，`docs/requirements/SRS-CANVAS-GAMIFY-001.md` §1.3 子能力表）；无源条目对应的项标注"项目约束"。状态 ✅ 表示该逻辑在 637ed19d 上有代码与单测引用，不表示用户可达（可达性见 FR-10-01）。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-10-01 | 玩家可在画布页进入弹幕 Roguelike 游戏模式并开始游玩 | P0 | 🗑 | 入口按钮 `data-testid="canvas-game-mode-btn"` 与 `<CanvasGameOverlay>` 挂载由 `633b4264` 加入 `frontend/src/app/canvas/[id]/page.tsx:185-189`，由 `23d60e75`（2026-11-14）移除（该提交 diff 删除上述按钮与挂载）。637ed19d 上没有任何文件引用 `CanvasGameOverlay` 或 `useCanvasGame`，`git grep` 仅命中两个游戏目录自身 |
| FR-10-02 | 机器人 5 状态机：idle / moving / attacking / rolling / dying | P0 | ✅ | `frontend/src/lib/canvas-game/types.ts:21`；`robotController.ts:82-143`；测试 `canvasGame.test.ts:183` |
| FR-10-03 | WASD 移动，8 方向朝向编码 | P0 | ✅ | 按键 `useCanvasGame.ts:87-94`；方向 `robotController.ts:52-65,112-118`；测试 `canvasGame.test.ts:36,50` |
| FR-10-04 | 攻击（空格 / J），冷却 350ms，出弹带朝向 | P0 | ✅ | 按键 `useCanvasGame.ts:75-78`；冷却 `robotController.ts:30,124-128`；出弹 `robotController.ts:150-164`；测试 `canvasGame.test.ts:72,112` |
| FR-10-05 | 翻滚（Shift / K）：消耗 SP 20，0.3s 无敌帧，滑动 1.6 倍速 | P0 | ✅ | 按键 `useCanvasGame.ts:79-82`；`robotController.ts:29,103-113`；无敌判定 `collision.ts:75`；测试 `canvasGame.test.ts:97` |
| FR-10-06 | 大招（Q）：16 向全屏爆发，冷却 8s，伤害随伤害等级增长 | P0 | ✅ | 按键与冷却 `useCanvasGame.ts:83-86,147-153`；`robotController.ts:300-319`（伤害 `:313`）。无单测覆盖 `fireUltimate` |
| FR-10-07 | 玩家子弹与敌人 AABB 命中；同帧多发伤害累计后统一结算 | P0 | ✅ | `collision.ts:19-24,58-72`；测试 `canvasGame.test.ts:141` |
| FR-10-08 | 敌方弹幕命中扣血；翻滚无敌帧免伤；HP 归零进入 dying | P0 | ✅ | `collision.ts:73-80,95`；测试 `canvasGame.test.ts:162,183` |
| FR-10-09 | 敌人由画布节点生成：work_item 与 agent 两类，强度不同 | P0 | 🟡 | 生成 `enemySpawner.ts:28-49`（agent 60HP，work_item 30HP，`:38`）；节点类型 `frontend/src/types/ids.ts:786`（`CanvasElementKind` 含 work_item_card 与 agent_cursor）。节点筛选 work_item_card 或 agent_cursor 仅存在于已移除的页面代码（`633b4264:113-120`）；637ed19d 无生产者提供 `EnemySpawnSeed` |
| FR-10-10 | 敌方弹幕 5 模式 aimed / radial / spiral / wave / laser，按敌人类别轮换 | P0 | 🟡 | 生成器 `robotController.ts:167-298`；轮换 `enemySpawner.ts:73-86`（agent 循环 radial / spiral / wave / laser；work_item 仅 aimed，`:85`）；laser 的 `targetX`、`targetY` 未被结算（`collision.ts` 无引用，见 §6） |
| FR-10-11 | 击杀掉落升级道具（伤害 / 射速 / 回血），接触拾取 | P0 | 🟡 | 掉落 `enemySpawner.ts:102-111`（agent 60%，work_item 35%，`:107`）；拾取 `useCanvasGame.ts:177-188`（半径 28，回血 25）；无单测。源条目 FR-GAME-505 要求按稀有度，未给数值（§6） |
| FR-10-12 | 局内 Roguelike 升级：伤害每级 +4，射速每级 +15 帧 | P0 | 🟡 | 伤害 `useCanvasGame.ts:139`；"射速"实现为子弹存活帧数 `ttlFrames + firerateLevel*15`（`:140`），攻击冷却 `ATTACK_COOLDOWN_MS` 不随等级变化；无单测 |
| FR-10-13 | HUD 显示 HP、SP、击杀数，提供退出按钮与操作提示 | P0 | 🟡 | HUD `CanvasGameOverlay.tsx:158-182`；测试 `CanvasGameOverlay.test.tsx:37`（B）、`:53`（C）。操作提示 `:181` 列出 WASD / SPACE / SHIFT，未列 Q（大招）、J、K |
| FR-10-14 | Game Over 界面与重开 | P0 | 🟡 | 状态 `useCanvasGame.ts:198-200`；界面 `CanvasGameOverlay.tsx:185-195`（重开按钮 `:191`）；重开 `useCanvasGame.ts:231-234`。覆盖层测试仅有 A / B / C，无重开用例 |
| FR-10-15 | 游戏文案国际化（zh-CN / en / ja） | P1（项目约束） | ✅ | `frontend/src/lib/i18n/dictionary.ts:633`、`en.ts:722`、`ja.ts:723`、`zh-CN.ts:737`（均为 `canvasGame` 键） |
| FR-10-16 | 机器人动作 sprite 动画：移动、攻击、翻滚、大招各多帧 | P0 | ⬜ | 仅 idle 与 move 两个静态矩阵 `sprites.ts:18`、`:38`；`robotController.ts:13` 注释为 "P0: idle/move 2 帧"。源条目帧数不一致（§8） |
| FR-10-17 | 3渲2 机器人：3D 模型离屏渲染为 2D 像素精灵 | P0 | ⬜ | 游戏代码路径不引用 three 或 @react-three（`frontend/package.json:23,24,47` 仅声明依赖）；机器人为 SVG 矩阵 `CanvasGameOverlay.tsx:146-153` |
| FR-10-18 | 8 方向旋转渲染 | P0 | 🟡 | 方向已计算 `robotController.ts:52-65,118`；渲染未使用方向（`CanvasGameOverlay.tsx:147-153` 无旋转） |
| FR-10-19 | 程序化关卡生成：房间、走廊、楼层、Biome | P0 | ⬜ | 无生成代码；敌人只来自外部种子（`enemySpawner.ts:28-49`）；随机数未播种（`enemySpawner.ts:43,107-109` 使用 `Math.random`） |
| FR-10-20 | 子弹池与 60fps 性能（1000+ 子弹） | P0 | ⬜ | 子弹为普通数组，每帧复制（`useCanvasGame.ts:134,150`；`enemySpawner.ts:58`），无池化；无性能测试 |
| FR-10-21 | 多人合作 1–4 人，共享生命池，房主权限 | P0 / P1 | ⬜ | 无协议或状态代码；`crates/canvas-realtime/src/lib.rs:98` 的 `/ws` 仅回显（`:133-134`），无游戏消息 |
| FR-10-22 | 服务端游戏服务 canvas-game（角色、弹幕、战斗；端口 8084 占位） | P0 | 🗑 | crate 由 `1868a13c`（2026-10-31）删除（`crates/canvas-game/` 下 Cargo.toml、Dockerfile、src/lib.rs、src/main.rs、tests/smoke.rs）；`Cargo.toml:116-120` 的成员列表不含该 crate |
| FR-10-23 | 游戏实体表（gamify_* 或 game_*，W/T/M 分类） | P1 | 🟡 | DDL 存在：`db/migrations/2026-09-10-p3d6-13-tables.sql`（6 张，自 `:1332`）与 `db/migrations/2026-09-10-p3d6-15-more-tables.sql`（9 张，自 `:530`）；无代码路径引用这些文件，应用状态 ❓。设计表名不一致（§4.2） |
| FR-10-24 | 游戏 REST 端点（BD-CANVAS-GAMIFY §5.1 的 22 端点；BD-STAR-CANVAS-GAME §8.1 的 7 端点） | P0 / P1 | ⬜ | `crates/canvas-engine/src/lib.rs:88-92` 仅 `/healthz`、`/ready`、`/version`、`/api/v1/services`；`crates/domain-canvas/src/lib.rs:87` 仅占位 `/api/v1/canvas`；无游戏端点 |
| FR-10-25 | G1 游戏化节点：avatar、level、xp、skill_tree、class、badge、quest、inventory 8 种 element kind | P1 | ⬜ | `CanvasElementKind`（`frontend/src/types/ids.ts:786`）仅含 sticky_note、text、shape、image、embed、work_item_card、worktree_node、agent_cursor、automation_node、comment_pin、path；SRS-CANVAS-GAMIFY-001 §4.1.1 无对应代码 |
| FR-10-26 | G2 奖励与成就：解锁规则、通知、confetti、徽章授予 | P1 | ⬜ | SRS-CANVAS-GAMIFY-001 §4.1.2；画布侧无代码。`crates/arg/src/models/achievement.rs` 存在，画布代码中未发现引用 |
| FR-10-27 | G3 积分规则与排行榜入口；G4 经验、升级动画、技能树解锁 | P0 / P1 | ⬜ | SRS-CANVAS-GAMIFY-001 §1.3 表（G3.1–3.2 P0，G4.1–4.2 P0，其余 P1）；无代码 |
| FR-10-28 | G5 便签 AI 聚类（走 mock） | P1 | ⬜ | `frontend/src/mocks/handlers/` 无聚类处理器；SRS-CANVAS-GAMIFY-001 §4.1.5 |
| FR-10-29 | G6 点投票、G7 表情回应、G8 confetti | G6 P0，G7 P1，G8 P0 | ⬜ | 无 `VoteButton`、`ReactionPicker`、`ConfettiOverlay` 等符号（`git grep` 于 frontend/src 与 crates 无命中） |
| FR-10-30 | G9 排行榜（workspace / tenant）、G10 每日挑战与连续登录、G11 道具与物品栏（W/T/M 横展） | P1 / P2 | ⬜ | SRS-CANVAS-GAMIFY-001 §1.3 表（G9.1 P1，G9.2 P2，G10 P1，G11 P2）；代码无。`frontend/src/mocks/handlers/analytics.ts:10` 的 leaderboard 注释与游戏无关 |
| FR-10-31 | G12 与 V0.1 agent-game 集成：Roguelike 角色与怪物映射、Manga 主题节点 | P0 | 🗑 | 依赖组件由 `a3365edd`（2026-10-31，30 个文件，删除 4584 行）删除：`frontend/src/components/agent-game/*`、`frontend/src/lib/agent-game/*`；637ed19d 上两路径不存在 |
| FR-10-32 | 独立 star-game crate（Agent 小游戏 PoC：6 维属性、ECS、GameBackend） | 项目约束 | 🗑 | 由 `fb42a132`（2026-11-13）删除 `crates/star-game/*`（6 个文件）；`Cargo.toml:144,148` 仅保留注释；`.github/workflows/publish-star-canvas-services.yml:320-321` 断言 canvas 闭包不含 star-game；`docs/WBS.md:218`（12.6）为"已取消" |

## 4. 设计

### 4.1 概念模型

- 敌人来源：`EnemySpawnSeed`（`frontend/src/lib/canvas-game/types.ts:128-134`），字段 id、x、y、kind（work_item 或 agent）、difficulty。
- 敌人：`Enemy`（`types.ts:77-94`），含 hp、variant（3 色）、速度、开火间隔、已开火次数、来源元素 id。
- 玩家：`PlayerRobot`（`types.ts:26-44`），含 hp、sp、状态、朝向、攻击冷却、翻滚无敌截止时间、击杀数。
- 战斗对象：`Bullet`（`types.ts:48-64`，`BulletKind` 为 player 与 5 种敌方模式，`:46`）；`PowerUp`（`types.ts:69-75`，damage / firerate / heal）；`ExplosionParticle`（`types.ts:97-104`，8x8 像素矩阵）。
- 局状态：`GameState`（`types.ts:106-125`），汇总机器人、子弹、敌人、爆炸、道具、输入、升级等级、大招冷却、gameOver、startedAt。

### 4.2 数据模型

运行时数据仅为内存中的 `GameState`（`useCanvasGame.ts:38-53`）；游戏层不读写 localStorage 或网络（`useCanvasGame.ts:15-24` 的导入只有 react 与游戏内部模块）。

服务端 DDL 存在但未接入：

| 文件 | 表（CREATE TABLE 行号） | 数量 |
|---|---|---|
| `db/migrations/2026-09-10-p3d6-13-tables.sql` | gamify_avatars（`:1332`）、gamify_levels（`:1476`）、gamify_sticky_notes（`:1620`）、gamify_confetti（`:1764`）、gamify_votes（`:1908`）、gamify_streaks（`:2052`） | 6 |
| `db/migrations/2026-09-10-p3d6-15-more-tables.sql` | gamify_achievements（`:530`）、gamify_xp_history（`:630`）、gamify_level_progress（`:742`）、gamify_rewards（`:832`）、gamify_leaderboards（`:933`）、gamify_avatar_customization（`:1035`）、gamify_vote_audit（`:1133`）、gamify_sticky_note_audit（`:1245`）、gamify_confetti_audit（`:1357`） | 9 |

gamify_avatars 的 RLS 与 FORCE RLS 见 `db/migrations/2026-09-10-p3d6-13-tables.sql:1397-1398`。637ed19d 上没有任何迁移执行器或代码路径引用这些文件（`git grep "db/migrations"` 仅命中 bi-owner-runtime 的 2026-10-24 迁移与注释）。

设计表名不一致，三者互不对应：

- `docs/design/BD-STAR-CANVAS-GAME-001.md` §4.1 标题称 26 张 game_* 表。
- `docs/design/DD-STAR-CANVAS-GAME-001.md` §4.1 全文只有 4 条 `CREATE TABLE game_*`（game_character、game_room、game_audit_log、game_session）。
- 迁移文件实际为 15 张 gamify_*。

BD-CANVAS-GAMIFY-001 §4.1 与 DD-CANVAS-GAMIFY-001 §9.2 引用的 `crates/storage/migrations/2026_09_10_gamify_w_t_m.sql` 在 637ed19d 上不存在（`git ls-tree -r -- crates/storage` 为空）。

### 4.3 接口

| 接口 | 位置 | 状态 | 说明 |
|---|---|---|---|
| `useCanvasGame({ seeds, enabled })` 返回 `{ gameState, restart }` | `frontend/src/lib/canvas-game/useCanvasGame.ts:55`，返回 `:236` | ✅ | 唯一调用方为 `CanvasGameOverlay` |
| `CanvasGameOverlay({ seeds, enabled, onExit })` | `frontend/src/components/canvas-game/CanvasGameOverlay.tsx:33-37`，组件 `:200` | 🟡 | 637ed19d 上无调用方 |
| `GameModeIcon()` | `CanvasGameOverlay.tsx:216-218` | 🟡 | 导出，注释称供入口页复用；无调用方 |
| `enemiesFromCanvas`、`stepEnemies`、`rollPowerUpDrop` | `enemySpawner.ts:28`、`:52`、`:102` | ✅ | 无生产者提供 seeds（FR-10-09） |
| `createRobot`、`directionFromVector`、`stepRobot`、`fireBullet`、`fireUltimate` | `robotController.ts:32`、`:52`、`:82`、`:150`、`:301` | ✅ | — |
| `stepBullets`、`aabbHit` | `collision.ts:38`、`:19` | ✅ | — |
| i18n 键 `t.canvasGame.{exit, move, attack, roll, gameOver, killsLabel, restart}` | `CanvasGameOverlay.tsx:175,181,187,189,192`；字典 `dictionary.ts:633` | ✅ | — |
| 游戏 REST 端点 | 无 | ⬜ | `canvas-engine` 仅 `/healthz`、`/ready`、`/version`、`/api/v1/services`（`crates/canvas-engine/src/lib.rs:88-92`）；`domain-canvas` 仅占位 `/api/v1/canvas`（`crates/domain-canvas/src/lib.rs:87`） |
| 游戏 WebSocket | 无 | ⬜ | `crates/canvas-realtime/src/lib.rs:98` 的 `/ws` 回显消息（`:133-134`） |

### 4.4 状态机与关键流程

机器人状态（`robotController.ts`）：

| 状态 | 进入条件 | 退出条件 | 证据 |
|---|---|---|---|
| idle | 初始状态（`createRobot`，`:41`）；moving 无输入时回落（`:119-120`）；attacking 或 rolling 超时（`:96-101`） | 有方向输入进入 moving；攻击键进入 attacking；翻滚键进入 rolling | `robotController.ts:82-143` |
| moving | idle 下有方向输入（`:117`） | 无输入回落 idle | 同上 |
| attacking | 攻击键按下，冷却 350ms 已过，且非 attacking 或 rolling（`:124-128`） | 持续 `STATE_DURATIONS.attacking` = 500ms（`:22-23`，`:96-98`）；攻击期间定身（`:112`） | 同上 |
| rolling | 翻滚键按下，SP ≥ 20，且非 rolling（`:104-107`） | 持续 `ROLL_INVULN_MS` = 300ms（`:29`，`:99-101`）；速度 1.6 倍（`:113`） | 同上 |
| dying | 敌方弹幕使 HP ≤ 0（`collision.ts:95`） | 不可中断（`robotController.ts:93-95`）；`useCanvasGame.ts:200` 设 gameOver | 同上 |

每帧顺序（`useCanvasGame.ts:115-218`）：

1. 机器人步进（`:122-128`）。
2. 进入 attacking 的首帧开火，并应用伤害与射速等级（`:130-143`）。
3. 大招（Q），冷却 8s（`:145-153`）。
4. 敌人步进与弹幕（`:155-158`）。
5. 子弹碰撞结算（`:160-163`）。
6. 击杀掉落道具（`:165-175`）。
7. 道具拾取（`:177-188`）。
8. 爆炸与道具衰减（`:190-196`）。
9. 死亡判定（`:198-200`）。

覆盖层状态矩阵（`CanvasGameOverlay.tsx`）：

| 状态 | 触发 | 呈现 | 现状 |
|---|---|---|---|
| 禁用或未挂载 | `enabled` 为 false，或 `gameState` 为 null | 返回 null（`:203`） | 有实现 |
| 正常 | `enabled` 为 true 且初始化完成 | 游戏 SVG 与 HUD（`:206-210`） | 有实现；见 §6 的开启路径缺陷 |
| 空 | seeds 为空 | 敌人数组为空，仅渲染机器人 | 无空状态提示 |
| 错误 | — | — | 无错误分支 |
| 未授权 | — | — | 不适用（无鉴权） |
| 未接入 | 游戏层未挂载 | 无 | 当前状态（FR-10-01） |
| 游戏结束 | `gameOver` 为 true | 遮罩与重开按钮（`:185-195`） | 有实现 |
| 退出 | 点击退出 | 调用 `onExit`（`:174-176`） | 有实现 |

开启路径（代码阅读判定，需交互测试确认，见 §6）：`useState` 的初始化器只在首次挂载时执行（`useCanvasGame.ts:56-60`）。`enabled` 由 false 变为 true 时，`gameState` 保持 null，覆盖层返回 null（`CanvasGameOverlay.tsx:203`）。历史接线为无条件挂载并传入 `enabled={gameMode}`（`633b4264:185-189`），按此接线开启模式不会出现游戏。

### 4.5 权限与安全

- 纯客户端功能，无鉴权、无租户上下文、无服务端写入。
- 键盘监听挂在 `window`，仅在 `enabled` 时注册（`useCanvasGame.ts:68-113`）；方向键与空格键会 `preventDefault`（`:72-74`），开启期间阻止页面滚动。
- 分数与击杀数仅在内存中，不外发。
- 服务端 DDL 含 RLS 策略（见 §4.2），但未被应用代码引用，无生效的访问控制。

### 4.6 依赖关系

- 外部：react（`useCanvasGame.ts:15`）、lucide-react（`CanvasGameOverlay.tsx:21`）、项目 i18n（`:31`）。
- 内部：`useCanvasGame` 依赖 `collision`、`enemySpawner`、`robotController`（`useCanvasGame.ts:16-24`）；`enemySpawner` 依赖 `robotController`（`enemySpawner.ts:17-25`）；`CanvasGameOverlay` 依赖 `useCanvasGame`、`sprites`、`types`（`CanvasGameOverlay.tsx:22-30`）。
- 与画布的关系：历史上由画布页按 `work_item_card`、`agent_cursor` 过滤元素生成种子（`633b4264:113-120`）。637ed19d 上画布页不再生成种子，`enemiesFromCanvas` 的输入没有生产者。
- three 与 @react-three/fiber 已在 `frontend/package.json:23,24,47` 声明，但游戏代码不使用。

4.7 页面设计: 本功能无独立页面，入口见 F06 统一画布（`docs/features/F06-unified-canvas.md`）的 `/canvas/[id]` 页面；游戏层曾挂载于该页，挂载已由 `23d60e75` 移除。

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 类型定义 | `frontend/src/lib/canvas-game/types.ts`（134 行） | 🟡 | 被游戏层导入；游戏层未挂载 |
| 机器人控制器（状态机、敌方弹幕生成器） | `frontend/src/lib/canvas-game/robotController.ts`（319 行） | 🟡 | 单测覆盖部分（§7） |
| 碰撞与子弹推进 | `frontend/src/lib/canvas-game/collision.ts`（113 行） | 🟡 | 单测覆盖部分 |
| 敌人生成与敌方 AI | `frontend/src/lib/canvas-game/enemySpawner.ts`（111 行） | 🟡 | 无生产者提供种子 |
| 精灵矩阵 | `frontend/src/lib/canvas-game/sprites.ts`（114 行） | 🟡 | 仅 2 帧（FR-10-16） |
| 游戏循环 hook | `frontend/src/lib/canvas-game/useCanvasGame.ts`（237 行） | 🟡 | 无直接单测；开启路径缺陷（§4.4） |
| 覆盖层组件 | `frontend/src/components/canvas-game/CanvasGameOverlay.tsx`（218 行） | 🟡 | 无挂载点 |
| 单测 | `frontend/src/lib/canvas-game/canvasGame.test.ts`（14 例）、`frontend/src/components/canvas-game/CanvasGameOverlay.test.tsx`（3 例） | 🟡 | 执行状态 ❓（§7） |
| 画布页游戏入口与挂载 | `frontend/src/app/canvas/[id]/page.tsx` | 🗑 | 入口与挂载由 `23d60e75` 移除；当前 333 行，无 game 字样（页面本身归 F06） |
| 旧路由 /chrono-vibe | `frontend/src/app/chrono-vibe/page.tsx:1-5` | ✅ | 仅 `redirect("/inbox")`，不含游戏逻辑 |
| 游戏文案 | `frontend/src/lib/i18n/dictionary.ts`、`en.ts`、`ja.ts`、`zh-CN.ts` | ✅ | `canvasGame` 键 |
| 游戏实体 DDL | `db/migrations/2026-09-10-p3d6-13-tables.sql`、`db/migrations/2026-09-10-p3d6-15-more-tables.sql` | 🟡 | 15 张 gamify_*，无应用代码引用 |
| 服务端游戏 crate | `crates/canvas-game/` | 🗑 | 由 `1868a13c` 删除 |
| 早期 star-game crate | `crates/star-game/` | 🗑 | 由 `fb42a132` 删除 |
| agent-game 组件与库 | `frontend/src/components/agent-game/`、`frontend/src/lib/agent-game/` | 🗑 | 由 `a3365edd` 删除 |

## 6. 差距与后续

| 差距 | 影响 | 闭合它的下一步 |
|---|---|---|
| 游戏层无挂载点（FR-10-01） | 用户无法进入游戏。`docs/WBS.md:148-149` 将 8.5.2 与 8.5.3 标为"已完成"，与可达性不符 | 决定挂载位置（F06 `/canvas/[id]` 或其他入口），接线后补端到端用例，并订正 WBS 状态 |
| 开启路径：`enabled` 由 false 变 true 时游戏不启动（❓，代码阅读判定） | `useState` 初始化只执行一次（`useCanvasGame.ts:56-60`），`gameState` 保持 null，覆盖层返回 null（`CanvasGameOverlay.tsx:203`）；历史接线为 `enabled={gameMode}`（`633b4264:185-189`） | 交互测试确认；修复为在 `enabled` 变化时初始化（effect），或由调用方按需挂载 |
| StrictMode 下 ref 在 updater 内被消费（❓） | `frame` 的 `setGameState` 回调内读取并清零 `attackHeldRef`、`rollHeldRef`、`ultimateHeldRef`（`useCanvasGame.ts:124-127,147-148`）。`reactStrictMode: true`（`frontend/next.config.js:29`）下开发环境会执行 updater 两次，第二次看到的按键为 false，攻击、翻滚、大招可能被丢弃 | 写 StrictMode 渲染测试确认；将输入消费移到 updater 之外（rAF 中读取后再 setState） |
| 射速等级映射为子弹存活帧数（FR-10-12） | 与"射速 +15 帧/级"的文字含义不符：实际改变射程，不改变冷却 `ATTACK_COOLDOWN_MS` | 明确"射速"定义，改为冷却或发射间隔，并补单测 |
| 激光目标点未结算（FR-10-10） | `targetX`、`targetY` 在 `types.ts:62-63` 与 `robotController.ts:293-294` 赋值，`collision.ts` 无引用，激光与普通直线子弹行为相同 | 按 BD §5.1.2 实现到达判定，或删除字段 |
| 掉落率注释与代码不一致（FR-10-11） | `enemySpawner.ts:101` 注释为 40%，代码为 60% / 35%（`:107`）；FR-GAME-505 要求按稀有度（`SRS-STAR-CANVAS-GAME-001.md:957-965`），无数值 | 源文档给出稀有度表后按表实现，并修正注释 |
| 操作提示不完整（FR-10-13） | `CanvasGameOverlay.tsx:181` 不含 Q（大招）、J、K | 补全提示，或由 i18n 文案统一生成 |
| 测试覆盖缺口（§7） | 无 `useCanvasGame` 直接测试；大招、掉落拾取、升级、重开、wave / spiral / laser 生成均无用例 | 为 hook 增加 rAF 与 act 测试，覆盖 §3 中 ✅ 与 🟡 项 |
| 动画与 3D 未实现（FR-10-16、FR-10-17） | 仅 2 帧静态；源条目"每动作 4–8 帧"（FR-GAME-101）与"32 帧"（DD-STAR §3.1）不一致 | 先在源文档中统一帧数，再决定 sprite atlas 与 3D 离屏渲染方案（ADR-CANVAS-GAME-002 的有效性需确认） |
| 关卡生成与随机数（FR-10-19） | 无程序化关卡；`Math.random` 未播种（`enemySpawner.ts:43,107-109`） | 引入可播种 RNG，再实现 FR-GAME-400–404 |
| 性能（FR-10-20） | 子弹与粒子以数组复制（`useCanvasGame.ts:134,150`；`enemySpawner.ts:58`），未达 FR-GAME-210 的 1000+ 子弹目标，无性能测试 | 子弹池（固定容量加回收）与 PT 测试 |
| 服务端与多人（FR-10-21 至 FR-10-25） | 无代码；`canvas-realtime` 的 `/ws` 仅回显 | 在实时层协议确定后再排期 |
| 数据设计分歧（FR-10-23） | 15 张 gamify_*（迁移）、26 张 game_*（BD-STAR §4.1）、4 条 CREATE（DD-STAR §4.1）三者不一致；迁移无应用路径 | 所有者选定一套表名与 W/T/M 分类，删除或迁移另一套，并确认迁移机制 |
| 报告缺失 | BD-STAR 与 DD-STAR 引用的 `docs/reports/PHASE-CANVAS-GAME-IMPL-REPORT.md` 在 637ed19d 上不存在 | 补实现报告，或删除引用 |
| 设计状态与退役标注冲突 | SRS、BD、DD-STAR-CANVAS-GAME-001 头部标注"已退役（2026-10-30）"；`WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:971`（23.36，2026-11-13）以用户命令"恢复原设计"接续前端层；SRS"阶段 1 实现状态"声称 canvas-game 骨架，与 `1868a13c` 冲突 | 所有者确认前端层设计基线，并更新各文档头部状态（§8 裁决） |
| G1–G12 草案无实现（FR-10-25 至 FR-10-31） | SRS、BD、DD-CANVAS-GAMIFY-001 均为 Draft v0.1，对应条目全部 ⬜ | 所有者决定是否批准草案范围；批准后按 G 组拆分交付 |

## 7. 验收与测试

现有单测（17 例，位于两个文件，执行状态 ❓）：

| 文件 | 用例 | 行号 |
|---|---|---|
| `frontend/src/lib/canvas-game/canvasGame.test.ts` | robotController：WASD 移动、方向向量、移动边界钳制、攻击与冷却、翻滚耗 SP 与无敌帧、开火朝向（6 例） | `:36`、`:50`、`:57`、`:72`、`:97`、`:112` |
| 同上 | collision：AABB、玩家子弹命中与击杀、敌方弹幕命中与翻滚免伤、HP 归零进入 dying（4 例） | `:136`、`:141`、`:162`、`:183` |
| 同上 | enemySpawner：agent 更强、追踪与定时弹幕、aimed 预判、radial 均匀分布（4 例） | `:203`、`:218`、`:230`、`:237` |
| `frontend/src/components/canvas-game/CanvasGameOverlay.test.tsx` | enabled=false 不渲染、游戏层渲染（机器人、敌人、HUD）、退出按钮触发 onExit（3 例） | `:32`、`:37`、`:53` |

已覆盖的行为：机器人状态转换、攻击冷却、翻滚与无敌帧、子弹碰撞与击杀、敌方弹幕命中、敌人强度、aimed 预判与 radial 分布、覆盖层禁用、HUD 渲染、退出回调。

未覆盖的行为：大招（FR-10-06）、道具掉落与拾取（FR-10-11）、升级（FR-10-12）、重开（FR-10-14）、wave / spiral / laser 生成（FR-10-10）、`useCanvasGame` 的按键消费与 `enabled` 切换、StrictMode、性能（FR-10-20）。`frontend/e2e` 中无游戏用例。

执行状态：本文档未在 637ed19d 上运行测试。工作区不是该提交，其测试结果不可复用，因此执行与通过状态标为 ❓。

源文档验收标准（`SRS-CANVAS-GAMIFY-001.md` §8.1；`SRS-STAR-CANVAS-GAME-001.md` §66）未在本功能测试中一一对应。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-CANVAS-GAMIFY-001.md`（1338 行；头部、§1.3 表、§4.1 条目标题） | G1–G12 范围与优先级（FR-10-25 至 FR-10-31） | 头部为"Draft v0.1"；无代码 | 草案，条目状态 ⬜ |
| `docs/design/BD-CANVAS-GAMIFY-001.md`（1751 行；头部、§1、§3、§4.1、§5.1–5.2） | 22 REST 端点与 2 WebSocket；15 张表的 W/T/M 分类 | Draft；pin 上无对应符号 | 草案，⬜ |
| `docs/design/DD-CANVAS-GAMIFY-001.md`（3439 行；头部与 §3–§9 标题） | 组件、状态机、共享类型的设计结构 | §9.2 的 `crates/storage/...` 路径不存在；DDL 实际位于 `db/migrations/` | 草案；DDL 位置以 637ed19d 上的 `db/migrations/` 为准 |
| `docs/requirements/SRS-STAR-CANVAS-GAME-001.md`（1698 行；头部、§15 ADR-CANVAS-GAME-001 至 008 标题、FR-GAME-001 至 904 标题与优先级、末尾"阶段 1 实现状态"、§66） | FR-GAME 条目与优先级（FR-10-01 至 FR-10-24 的依据）；ADR-002（3渲2 仅机器人）、ADR-004（2D 粒子池）、ADR-006（Yjs）、ADR-008（3 模式）作为设计依据 | 头部标注"已退役（2026-10-30）"；"阶段 1 实现状态"声称 canvas-game 骨架，与 `1868a13c` 冲突；FR-GAME-101 写"每动作 4–8 帧" | 前端层以 WBS 23.36 的用户命令为准（裁决 A）；服务端以 `1868a13c` 为准（裁决 B）；"阶段 1 实现状态"不采信 |
| `docs/design/BD-STAR-CANVAS-GAME-001.md`（1546 行；头部、§2.1、§3.1、§4.1、§5.1、§8.1） | 机器人 5 状态、5 弹幕模式、功能清单 | 头部退役标注；26 张 game_* 表与迁移的 15 张 gamify_* 不一致 | 机器人状态与弹幕模式采用（前端层）；数据设计不采信，待所有者定稿 |
| `docs/design/DD-STAR-CANVAS-GAME-001.md`（2119 行；头部、§2.2、§3.1、§4.1） | canvas-game 公开 API 结构；伤害公式与随机关卡设计（未在 637ed19d 实现） | 头部退役标注；§4.1 标题称 26 表，全文仅 4 条 CREATE；§3.1 称 32 帧 | 同上 |
| `docs/features/F06-unified-canvas.md`（行 29、280） | `/canvas/[id]` 页面归 F06；游戏叠加层代码不属 F06 | — | 采用：页面归 F06，游戏代码归 F10 |
| `docs/features/F10-gamified-canvas.md`（637ed19d 上的既有版本，基线 cdd00a78） | 文档结构参考 | 结论与行号多处与 637ed19d 不符（如 i18n 行号）；"通过 17 项测试"未在 637ed19d 复核 | 不作为事实来源；本文逐条以 637ed19d 代码重核 |
| `docs/adr/0027-rust-pivot-agent-game.md` | Agent 小游戏概念与 star-game 组件设计 | 头部标注"当前状态（2026-10-30）：已退役" | 退役，仅作历史背景 |
| `docs/plans/plan-032-rust-pivot-agent-game.md` | 退役依据 | 头部标注"Retired Historical Record（2026-10-30）" | 退役，不作为事实来源；未展开读取 |
| `docs/implementation-plans/CANVAS-GAME-IMPL-PLAN-001.md` | 阶段 1 骨架记录 | 头部标注"已退役（2026-10-30）"；骨架声明与 `1868a13c` 冲突 | 不采信其骨架声明 |
| `docs/WBS.md`（行 148、149、218） | 8.5.2、8.5.3 标为"已完成"；12.6 star-game 已取消 | 8.5.2 与 8.5.3 与挂载移除（`23d60e75`）不符 | 代码为准；任务状态待订正 |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md`（行 971 起，23.36） | 用户命令（2026-11-13）恢复原设计；P0 范围与资源盘点 | 计划中的 `bulletSystem.ts` 在 637ed19d 上不存在，子弹逻辑位于 `collision.ts` 与 `robotController.ts` | 采用该用户命令作为前端层的最近决策；文件清单以代码为准 |
| `docs/refactor/ui-audit-20261113.md`（行 53） | 提及 chrono-vibe 为演示页 | 当前为重定向 | 代码为准（✅ 重定向） |
| `docs/briefs/{srs,bd,dd}-canvas-gamify-001.md` | 草案输入 | 未展开读取 | 不作为事实来源 |
| `docs/reports/PHASE-CANVAS-GAME-IMPL-REPORT.md` | 被 BD 与 DD 引用 | 637ed19d 上不存在 | 缺失，见 §6 |
| `frontend/src/components/effects/PhysisHud.tsx:23`、`AppShell.tsx:37-39` | 名称含"游戏化 HUD" | 属 physis 主题，与本功能无关 | 不属 F10，范围外 |
| `crates/star-agent-runtime`（ECS）；已删除的 `crates/star-game/src/ecs.rs` | 关键词"ecs"命中 | 另一功能（Agent 运行时） | 不属本功能 |
| `crates/star-canvas/src/lib.rs`（行 14、64、415 注释） | 注释引用 star-game 的 3D 概念 | 3D 白板域，非游戏实现 | 不属本功能 |
| `frontend/src/app/agent-view`（637ed19d 上无目录）；`frontend/src/lib/i18n/en.ts:450` | "gamified avatars" 文案 | 路由不存在，文案为残留 | 不属本功能 |
| 代码与提交 | `frontend/src/lib/canvas-game/*`（7 文件）、`frontend/src/components/canvas-game/*`（2 文件）、`frontend/src/app/canvas/[id]/page.tsx`、`frontend/src/app/canvas/layout.tsx`、`frontend/src/components/AppShell.tsx`、`frontend/src/app/chrono-vibe/page.tsx`、`frontend/src/lib/i18n/*`、`frontend/src/types/ids.ts`、`frontend/next.config.js`、`frontend/package.json`、`Cargo.toml`、`db/migrations/*`、`crates/canvas-engine`、`crates/domain-canvas`、`crates/canvas-realtime`；提交 `633b4264`、`23d60e75`、`a3365edd`、`1868a13c`、`fb42a132` | 现状的主要依据 | — | 代码为准 |

裁决汇总：

- 裁决 A（前端层设计基线）：采用 WBS 23.36 记录的 2026-11-13 用户命令，它晚于 2026-10-30 的退役标注。因此 BD、DD、SRS-STAR-CANVAS-GAME-001 的退役标注对前端层失效，待所有者更新头部。
- 裁决 B（服务端）：canvas-game 与 star-game 保持退役。`1868a13c`（2026-10-31）与 `fb42a132`（2026-11-13）的删除为现行决定，SRS"阶段 1 实现状态"不采信。
- 裁决 C（G1–G12 草案）：SRS、BD、DD-CANVAS-GAMIFY-001 为 Draft v0.1，未被替代，按草案处理，条目状态 ⬜。
- 裁决 D（页面归属）：`/canvas/[id]` 归 F06；本功能的游戏代码归 F10。
- 裁决 E（冲突规则）：源文档与代码的现状冲突时，以代码为准；源文档之间的设计冲突，以最近的已接受决定为准，并在本节记录理由。
