# F10 游戏化画布

> 状态摘要: 弹幕 Roguelike 的前端逻辑（机器人状态机、子弹碰撞、敌人弹幕、道具、大招）与游戏覆盖层已实现并通过 17 项前端测试，但游戏层当前未挂载到任何页面，原入口已于 `23d60e75` 移除。SRS-CANVAS-GAMIFY-001 的 G1–G12 游戏化能力、服务端游戏模型与 3渲2 机器人渲染均未实现；其依赖的 V0.1 agent-game 组件已由 `a3365edd` 移除，`crates/star-game` 已由 `fb42a132` 移除。

> 代码基线: HEAD `cdd00a78`; 核对方法: 路径与符号 grep、逐文件读取（带行号）、`vitest run` 复跑 `frontend/src/lib/canvas-game` 与 `frontend/src/components/canvas-game` 两组测试（2 文件、17 项通过）; 核对日期 2026-11-16.

## 1. 目标与边界

### 1.1 目标

把画布（`/canvas/[id]`）上的工作项卡片（`work_item_card`）与智能体光标（`agent_cursor`）转为可操控的像素风弹幕 Roguelike 关卡素材：玩家操控机器人，敌人由画布节点生成并发射弹幕，击杀后掉落升级道具，形成局内成长。

远期目标（设计源 SRS-CANVAS-GAMIFY-001）是把积分、等级、技能树、徽章、投票、表情回应、排行榜与每日挑战接入画布协作，并由 AI 对便签进行聚类。

### 1.2 范围内 / 范围外

| 类别 | 范围内 | 范围外 |
|---|---|---|
| 客户端玩法 | 机器人移动/攻击/翻滚/大招、子弹与碰撞、敌人弹幕、道具掉落与拾取、HUD、重开 | 3渲2 Three.js 机器人渲染（设计项，未实现） |
| 画布集成 | 画布节点到敌人种子的映射（设计于 `canvas/[id]/page.tsx`，已移除） | 基础画布 `CanvasView` 本身（属画布基础功能，不在本文展开） |
| 游戏化 G1–G11 | 作为待办设计项登记 | 本文不因设计存在而标记为已实现 |
| 服务端 | 游戏实体表与 REST/WS 端点（设计项） | — |
| 退役内容 | V0.1 agent-game（RPG/Manga/Settings）、agent 小游戏 PoC（`star-game`）、ADR-0027 方案 | 仅记录依赖断裂，不作为当前能力 |

## 2. 用户与场景

| 角色 | 主要故事 | 当前可达性 |
|---|---|---|
| 画布编辑者（玩家） | 在画布上开启游戏模式，操控机器人击杀由节点生成的敌人，拾取道具，死亡后重开 | 不可达：入口已移除 |
| 画布编辑者 | 工作项与智能体节点被转为敌人，敌人强度随节点类型不同 | 仅逻辑层可用，无挂载 |
| 团队成员（设计项） | 对便签投票、表情回应、查看进度与徽章 | ⬜ 未实现 |
| 管理员（设计项） | 查看 workspace/tenant 排行榜 | ⬜ 未实现 |
| 智能体（设计项） | 作为游戏角色或敌人参与关卡 | ⬜ 未实现 |

## 3. 需求

优先级取自源文档的域标注（BD-STAR-CANVAS-GAME-001 §3.1 的 P0 域）；G1–G11 的逐条优先级以 SRS-CANVAS-GAMIFY-001 §4.1 为准，本表未逐条核对，记为 P1。

| 编号 | 需求 | 优先级 | 状态 | 证据 |
|---|---|---|---|---|
| FR-10-01 | 玩家可在画布上进入弹幕 Roguelike 游戏模式并正常游玩 | P0 | 🟡 | 游戏层组件存在（`frontend/src/components/canvas-game/CanvasGameOverlay.tsx:200-213`），但无任何页面引用（grep `CanvasGameOverlay` 仅命中自身与测试）；原挂载点 `frontend/src/app/canvas/[id]/page.tsx` 在 `633b4264:113-122,143-146,185-190` 曾有入口，被 `23d60e75` 移除；当前该文件 333 行且无 game 引用 |
| FR-10-02 | 机器人 5 状态机（idle/moving/attacking/rolling/dying） | P0 | ✅ | `frontend/src/lib/canvas-game/types.ts:21`；`frontend/src/lib/canvas-game/robotController.ts:93-121`；`frontend/src/lib/canvas-game/collision.ts:95`；测试 `canvasGame.test.ts:183` |
| FR-10-03 | WASD 移动，8 方向朝向编码 | P0 | ✅ | `frontend/src/lib/canvas-game/useCanvasGame.ts:87-94`；`robotController.ts:52-65,112-121`；测试 `canvasGame.test.ts:36,50` |
| FR-10-04 | 攻击（空格/J）：冷却 350ms，出弹并带方向 | P0 | ✅ | `useCanvasGame.ts:75-78,132-134`；`robotController.ts:30,125-128,150-164`；测试 `canvasGame.test.ts:72,112` |
| FR-10-05 | 翻滚（Shift/K）：消耗 SP 20，0.3s 无敌帧，滑动加速 1.6 倍 | P0 | ✅ | `useCanvasGame.ts:79-82`；`robotController.ts:22-29,104-108,113`；`collision.ts:75`；测试 `canvasGame.test.ts:97` |
| FR-10-06 | 大招（Q）：16 向全屏爆发，冷却 8s，伤害随升级增长 | P0 | ✅ | `useCanvasGame.ts:83-86,145-153`；`robotController.ts:301-319`。无测试覆盖（见 §7） |
| FR-10-07 | 玩家子弹与敌人 AABB 碰撞，同帧伤害累计后统一结算击杀 | P0 | ✅ | `frontend/src/lib/canvas-game/collision.ts:19-24,58-72`；测试 `canvasGame.test.ts:141` |
| FR-10-08 | 敌方弹幕命中玩家扣血；翻滚无敌帧免伤；HP 归零进入 dying | P0 | ✅ | `collision.ts:73-80,95`；测试 `canvasGame.test.ts:162,183` |
| FR-10-09 | 敌人由画布节点生成（work_item 与 agent 两类强度不同） | P0 | 🟡 | 生成逻辑 `frontend/src/lib/canvas-game/enemySpawner.ts:28-49`；种子映射 `633b4264:113-122`（筛选 `work_item_card` / `agent_cursor`）仅存在于已移除的页面代码；节点类型定义 `frontend/src/types/ids.ts:792,794` |
| FR-10-10 | 敌人弹幕 5 模式（aimed/radial/spiral/wave/laser）按敌人类别轮换 | P1 | 🟡 | 子弹生成函数 `robotController.ts:167-298`（5 种）；轮换逻辑 `enemySpawner.ts:73-86`：agent 轮换 radial/spiral/wave/laser，work_item 只用 aimed；`laser` 的 `targetX/targetY` 未被碰撞结算使用（`collision.ts` 无引用） |
| FR-10-11 | 击杀掉落 Roguelike 道具（伤害/射速/回血），接触拾取 | P1 | 🟡 | 掉落 `enemySpawner.ts:102-111`（掉率 agent 60%、work_item 35%）；拾取与效果 `useCanvasGame.ts:165-188`（拾取半径 28，回血 25）；类型 `types.ts:67-75`；无测试 |
| FR-10-12 | 局内升级：伤害 +4/级，射速 +15 帧/级 | P1 | 🟡 | `useCanvasGame.ts:135-143`；无测试 |
| FR-10-13 | HUD 显示 HP、SP、击杀数，提供退出按钮与操作提示 | P0 | 🟡 | `CanvasGameOverlay.tsx:158-182`；测试 `CanvasGameOverlay.test.tsx:37-52` |
| FR-10-14 | Game Over 界面与重开 | P0 | 🟡 | 状态 `useCanvasGame.ts:200,232-234`；界面 `CanvasGameOverlay.tsx:185-195`；无测试覆盖重开 |
| FR-10-15 | 游戏文案国际化（zh-CN/en/ja） | P1 | ✅ | `frontend/src/lib/i18n/dictionary.ts:633`、`en.ts:721`、`ja.ts:722`、`zh-CN.ts:736` |
| FR-10-16 | 32 帧 sprite 动画（idle/move/attack/roll/dying）（FR-GAME-101） | P0 | ⬜ | 仅两帧静态矩阵 `frontend/src/lib/canvas-game/sprites.ts:18-35,38-55`；`robotController.ts:13` 注释自述 "P0: idle/move 2 帧" |
| FR-10-17 | 3渲2 机器人（3D 模型离屏渲染为 2D 精灵，FR-GAME-100/700–703） | P0 | ⬜ | 无 three/3D 相关模块于游戏代码路径；机器人为 SVG 矩阵 `CanvasGameOverlay.tsx:146-153` |
| FR-10-18 | 8 方向机器人旋转渲染（FR-GAME-703） | P1 | 🟡 | 方向已计算 `robotController.ts:52-65`；渲染未使用方向，`CanvasGameOverlay.tsx:146-153` 无旋转 |
| FR-10-19 | 程序化关卡生成：房间/走廊/楼层/Biome（FR-GAME-400–404） | P0 | ⬜ | 无关卡生成代码；敌人仅来自画布节点 `enemySpawner.ts:28-49`；随机数未受种子控制 `enemySpawner.ts:43,108` |
| FR-10-20 | 弹幕系统的子弹池上限与 60fps 性能（FR-GAME-210，1000+ 子弹） | P0 | ⬜ | 子弹为普通数组，每帧复制 `useCanvasGame.ts:117-218`；无池化；无性能测试 |
| FR-10-21 | 多人合作 1–4 玩家、共享生命池、房主权限（FR-GAME-800–803） | P0/P1 | ⬜ | 无协议或状态代码；画布实时层未发现游戏协议（`crates/canvas-realtime`、`crates/domain-canvas`、`crates/canvas-engine` 中 grep `game` 无命中） |
| FR-10-22 | 服务端游戏实体与 API（`game_*` 表、22 个 REST、2 个 WebSocket） | P0 | ⬜ | 设计源 `docs/design/BD-STAR-CANVAS-GAME-001.md` §4.1、`docs/design/BD-CANVAS-GAMIFY-001.md` §5.1–5.2；代码中无 `game_character`/`game_room`/`game_session` 迁移，`crates/` 无游戏路由 |
| FR-10-23 | G1 节点：avatar/level/xp/skill_tree/class/badge/quest/inventory | P1 | ⬜ | SRS-CANVAS-GAMIFY-001 §4.1.1 L241-378；代码中无相应实体（V0.1 实现已移除，见 FR-10-29） |
| FR-10-24 | G2 奖励与成就（解锁规则、通知、confetti、徽章自动授予） | P1 | ⬜ | SRS §4.1.2 L379-444；`crates/arg/src/models/achievement.rs` 存在但未与画布关联（见 §4.6） |
| FR-10-25 | G3 积分规则、累计、排行榜入口 | P1 | ⬜ | SRS §4.1.3 L445-494 |
| FR-10-26 | G4 经验/升级动画/技能树解锁 | P1 | ⬜ | SRS §4.1.4 L495-544；原 `leveling.ts` 已移除（`a3365edd`） |
| FR-10-27 | G5 便签 AI 聚类（走 mock） | P1 | ⬜ | SRS §4.1.5 L545-578；无 mock 处理器（`frontend/src/mocks/handlers` 中无 gamify/cluster） |
| FR-10-28 | G6 点投票、G7 表情回应、G8 confetti | P1 | ⬜ | SRS §4.1.6–4.1.8 L579-649；无 `dot vote`/`confetti` 代码（grep 命中为空） |
| FR-10-29 | G12 与 V0.1 agent-game 集成（Roguelike 角色/怪物映射、Manga 主题节点） | P0 | 🗑 | 依赖组件已移除：`a3365edd`（2026-10-31，30 个文件，4584 行删除）删除 `frontend/src/components/agent-game/*` 与 `frontend/src/lib/agent-game/*`（含 `leveling.ts`、`perks.ts`、`mapgen.ts`、`movement.ts`、`theme.ts`、`characters.tsx`、`enemies.tsx`） |
| FR-10-30 | G9 排行榜（workspace/tenant）、G10 每日挑战与连续登录、G11 power-up 与物品栏（W/T/M 三类横展） | P1 | ⬜ | SRS §4.1.9–4.1.11 L650-780；`frontend/src/mocks/handlers/analytics.ts` 的 leaderboard 与游戏无关 |
| FR-10-31 | 独立 `star-game` crate（agent 小游戏 PoC，6 维属性 + ECS + GameBackend） | — | 🗑 | 由 `fb42a132`（chore(workspace): remove retired star-game crate）删除；`crates/star-game` 目录不存在；`Cargo.toml:141` 仅保留注释；退役依据 `docs/plans/plan-032-rust-pivot-agent-game.md:3` |
| FR-10-32 | 旧路由别名 `/chrono-vibe` | — | ✅ | `frontend/src/app/chrono-vibe/page.tsx:1-5` 重定向到 `/inbox`，不含游戏逻辑 |

## 4. 设计

### 4.1 概念模型

- 关卡素材：画布元素（工作项卡片、智能体光标）→ `EnemySpawnSeed`（`frontend/src/lib/canvas-game/types.ts:128-134`）→ `Enemy`。
- 玩家：`PlayerRobot`（HP/SP/速度/状态/朝向，`types.ts:26-44`）。
- 战斗对象：`Bullet`（玩家与敌方共用，`types.ts:46-64`），`ExplosionParticle`（8×8 像素，`types.ts:97-104`）。
- 成长：`PowerUp`（伤害、射速、回血，`types.ts:67-75`）与局内等级 `damageLevel` / `firerateLevel`（`types.ts:117-118`）。
- 局状态：`GameState`（`types.ts:106-125`），由 rAF 循环每帧推进（`useCanvasGame.ts:115-229`）。

全部为客户端内存状态，无持久化。

### 4.2 数据模型

| 对象 | 定义 | 状态 |
|---|---|---|
| 客户端类型 GameState / PlayerRobot / Bullet / Enemy / PowerUp / ExplosionParticle / EnemySpawnSeed | `frontend/src/lib/canvas-game/types.ts:20-134` | ✅ |
| 设计：15 张游戏化表（W/T/M 分类） | `docs/design/BD-CANVAS-GAMIFY-001.md` §4.1–4.2（L486-766）；DDL 目标 `crates/storage/migrations/2026_09_10_gamify_w_t_m.sql` | ⬜（无该迁移文件） |
| 设计：26 张 `game_*` 表（守门 #13 W/T/M） | `docs/design/BD-STAR-CANVAS-GAME-001.md` §4.1–4.3（L649-895）；`docs/design/DD-STAR-CANVAS-GAME-001.md` §4.1（L847-1160） | ⬜（无 `game_*` 表） |
| 设计：`game_character`（M，SCD Type 2）、`game_room`（M）、`game_audit_log`（T，WORM）、`game_session`（W） | `DD-STAR-CANVAS-GAME-001.md` §4.1.1–4.1.4 | ⬜ |

### 4.3 接口

| 接口 | 位置 | 状态 |
|---|---|---|
| `CanvasGameOverlay` props：`seeds`、`enabled`、`onExit` | `frontend/src/components/canvas-game/CanvasGameOverlay.tsx:33-37` | ✅（无调用方挂载，见 FR-10-01） |
| `useCanvasGame({ seeds, enabled })` 返回 `{ gameState, restart }` | `frontend/src/lib/canvas-game/useCanvasGame.ts:29-32,236` | ✅ |
| 键盘事件：window `keydown`/`keyup`，方向键与空格默认行为被阻止 | `useCanvasGame.ts:68-113` | ✅ |
| BFF 22 个 REST 端点、2 个 WebSocket、Mock 规范 | 设计：`docs/design/BD-CANVAS-GAMIFY-001.md` §5.1–5.4；`docs/design/DD-CANVAS-GAMIFY-001.md` §7（L2064-2447）；`docs/design/BD-STAR-CANVAS-GAME-001.md` §8（L1255-1292，7 个游戏端点、3 个 MCP 工具） | ⬜（代码中无对应路由） |

### 4.4 状态机与关键流程

- 机器人：`idle ↔ moving`；攻击进入 `attacking`（持续 500ms，`robotController.ts:22-26,96-98`）；翻滚进入 `rolling`（300ms，`robotController.ts:99-101,104-108`）；HP 归零进入 `dying`，dying 不可中断（`robotController.ts:93-95`，`collision.ts:95`）。
- 游戏结束：`dying` 且 HP ≤ 0 时置 `gameOver`（`useCanvasGame.ts:198-200`）。注释标明为 P0 简化，即时结束。
- 击杀与掉落：子弹结算后对击杀者位置匹配敌人并按概率掉落（`useCanvasGame.ts:165-175`，`enemySpawner.ts:102-111`）。
- 重开：`restart` 以同一 seeds 重建初始状态（`useCanvasGame.ts:231-234`）。
- 设计项 SM-1..SM-5（簇、投票、表情、每日挑战、连续登录，`docs/design/DD-CANVAS-GAMIFY-001.md` §5，L1107-1495）：⬜ 未实现。

### 4.5 权限与安全

- 游戏为纯客户端，无认证、无授权、无服务端校验；分数与击杀数不可信。
- 设计项中的 RLS 13 类、反作弊（`docs/design/DD-STAR-CANVAS-GAME-001.md` §9.1）与多人权限（ADR-CANVAS-GAME-008，3 模式）均为 ⬜。
- 与安全相关的现有行为：键盘处理阻止方向键与空格的页面滚动（`useCanvasGame.ts:72-74`）。

### 4.6 依赖关系

| 依赖 | 说明 | 状态 |
|---|---|---|
| React hooks、requestAnimationFrame | `useCanvasGame.ts:15,220-229` | ✅ |
| 画布元素类型 `work_item_card`、`agent_cursor`、`sticky_note` | `frontend/src/types/ids.ts:787-794` | ✅ |
| i18n `canvasGame` 字典 | `CanvasGameOverlay.tsx:31`（使用 `t.canvasGame.*`） | ✅ |
| 图标库 lucide-react | `CanvasGameOverlay.tsx:21` | ✅ |
| V0.1 agent-game 组件（RoguelikeCanvas、GameHUD、leveling、perks、theme） | 设计依赖；已由 `a3365edd` 移除 | 🗑 |
| `crates/arg` 成就模型（`achievement.rs`，含 streak） | 与 G2 徽章语义相近，但画布与成就之间无引用 | ⬜（未关联） |
| `star-game` crate | 由 `fb42a132` 移除 | 🗑 |
| 设计依赖：Physis / GVPE `GameBackend` 集成点 | 见 `docs/adr/0027-rust-pivot-agent-game.md:150`，随 star-game 退役 | 🗑 |

## 5. 当前实现

| 模块 | 路径 | 状态 | 证据 |
|---|---|---|---|
| 游戏类型定义 | `frontend/src/lib/canvas-game/types.ts` | ✅ | 20–134 |
| 机器人控制与状态机 | `frontend/src/lib/canvas-game/robotController.ts` | ✅ | 22–319 |
| 碰撞与子弹推进 | `frontend/src/lib/canvas-game/collision.ts` | ✅ | 19–113 |
| 敌人生成、AI 与道具掉落 | `frontend/src/lib/canvas-game/enemySpawner.ts` | ✅ | 28–111 |
| 游戏循环 hook | `frontend/src/lib/canvas-game/useCanvasGame.ts` | ✅ | 27–237 |
| 像素精灵矩阵 | `frontend/src/lib/canvas-game/sprites.ts` | ✅ | 18–114 |
| 游戏覆盖层与 HUD | `frontend/src/components/canvas-game/CanvasGameOverlay.tsx` | 🟡 | 存在且有测试，无挂载点 |
| 画布页游戏模式按钮与挂载 | `frontend/src/app/canvas/[id]/page.tsx`（`633b4264:143-146,185-190`） | 🗑 | 由 `23d60e75` 移除，当前文件无 game 引用 |
| 旧路由别名 | `frontend/src/app/chrono-vibe/page.tsx` | ✅ | 1–5（重定向至 `/inbox`） |
| 画布基础（三个 crate） | `crates/canvas-engine`、`crates/domain-canvas`、`crates/canvas-realtime`（`Cargo.toml:113-117`） | ✅ | 存在；无游戏代码 |
| agent-game 前端组件与逻辑 | `frontend/src/components/agent-game/`、`frontend/src/lib/agent-game/` | 🗑 | `a3365edd` |
| agent 小游戏 crate | `crates/star-game` | 🗑 | `fb42a132` |
| 游戏服务端实体与 API | `crates/` 与迁移目录 | ⬜ | grep `game_character`、`game_room`、`game_session` 无命中 |
| 3渲2 机器人渲染 | 游戏代码路径 | ⬜ | 无 three 相关代码 |

## 6. 差距与后续

| 差距 | 影响 | 下一步 |
|---|---|---|
| 游戏层无挂载点（入口于 `23d60e75` 移除） | 用户无法进入游戏模式；WBS 8.5.2 与 8.5.3 标记"已完成"（`docs/WBS.md:148-149`）与可达性不符 | 所有者决定：重新挂载（候选：画布页按钮 + `?mode=game`）或退役代码；随后更新 WBS |
| G12 依赖的 V0.1 组件已删除 | G12.1–12.2 与 G1.2/G4 的等级数据源不存在；SRS-CANVAS-GAMIFY-001 §1.1 与 §4.1.12 指向已删除组件 | 按当前模型重写 G1–G4 与 G12 的接口依赖，或降级为待办 |
| G1–G11 游戏化功能全部未实现 | 设计范围大，与代码差距大 | 所有者确认是否纳入当前产品范围；纳入则以 BD/DD 与当前代码对齐后分批实现 |
| 服务端游戏模型与 API 未实现 | 无法持久化分数、关卡、投票；多人与排行榜不可能 | 确认范围后执行 W/T/M 迁移，并补 BFF 路由 |
| 动画仅 2 帧，8 方向未渲染 | 与设计 32 帧和 8 向不符，视觉不完整 | 补 sprite 帧与方向渲染（`CanvasGameOverlay.tsx:146-153`） |
| 程序化关卡生成与随机种子缺失 | 关卡不可复现（`enemySpawner.ts:43,108` 使用 `Math.random`） | 实现种子化随机与关卡生成器 |
| 子弹数组每帧复制，无池化 | 与 1000+ 子弹 / 60fps 目标不符（FR-GAME-210） | 引入子弹池并补性能测试 |
| 激光 `targetX/targetY` 与波浪相位未参与结算 | 设计中的激光锁定与波浪摆动不生效 | 在 `collision.ts` 中实现对应结算，或删除未用字段 |
| `star-game` 已删除但去重报告仍称其在 workspace 与 CI 中 | `docs/refactor/dedupe-20261113-lane-d.md` L29、L45 与当前 `Cargo.toml`（仅注释，L141）及 `.github/workflows`（仅排除断言，`publish-star-canvas-services.yml:320-341`）不符；发行面要求排除（`docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md:27,46`） | 无需动作；该报告的过时描述应修正 |
| 退役状态在源文档之间表述不一 | BD-STAR-CANVAS-GAME-001 自标 2026-10-30 退役；`docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:974` 记录用户拍板恢复；`633b4264` 提交信息亦称"恢复" | 所有者确认当前有效版本，统一文档状态 |
| 无游戏相关测试覆盖大招、道具、重开 | 关键路径无回归保护 | 补 `fireUltimate`、`rollPowerUpDrop`、`restart` 单元测试 |

## 7. 验收与测试

| 测试 | 位置 | 覆盖 |
|---|---|---|
| 机器人与控制（6 项） | `frontend/src/lib/canvas-game/canvasGame.test.ts:35-120` | 8 方向编码、边界钳制、攻击冷却、翻滚耗 SP 与无敌帧、开火方向 |
| 碰撞（4 项） | `canvasGame.test.ts:121-200` | AABB 命中、玩家子弹击杀与爆炸、敌方弹幕与翻滚免伤、HP 归零进入 dying |
| 敌人生成与弹幕（4 项） | `canvasGame.test.ts:202-250` | agent 强度、追踪与定时弹幕、aimed 预判、radial 均分 |
| 覆盖层（3 项） | `frontend/src/components/canvas-game/CanvasGameOverlay.test.tsx:29-63` | enabled=false 不渲染、渲染机器人/敌人/HUD、退出按钮 |

2026-11-16 复跑：`vitest run src/lib/canvas-game src/components/canvas-game`，2 个文件、17 项测试全部通过（vitest 5.1.0，耗时 2.98s）。

已验证：机器人移动、攻击、翻滚、dying 转换；AABB；子弹伤害与击杀；敌人生成与 aimed/radial 弹幕；覆盖层挂载、HUD 与退出。

未验证：大招与冷却、道具掉落与拾取、回血、重开、spiral/wave/laser 弹幕、性能目标（60fps、1000+ 子弹）、端到端游玩、服务端与多人能力、画布页的游戏模式入口。G1–G11、G12 与 3渲2 没有测试，因为没有实现。

## 8. 来源与裁决

| 源文档 | 吸收内容 | 与代码或其他源的差异 | 裁决 |
|---|---|---|---|
| `docs/requirements/SRS-CANVAS-GAMIFY-001.md` | G1–G12 功能清单、验收项、§1.1 V0.1 依赖（L48-56）、G12 集成（L781-815） | 设计前提 agent-game 组件已由 `a3365edd` 删除；G1–G11 无代码 | 状态以代码为准：G1–G11 ⬜，G12 🗑；功能定义作为待办保留 |
| `docs/design/BD-CANVAS-GAMIFY-001.md` | 4 层架构、15 张 W/T/M 表、22 REST、2 WS | 表与端点均无代码 | ⬜；保留为设计意图 |
| `docs/design/DD-CANVAS-GAMIFY-001.md` | 13 个关键类、SM-1..SM-5、11 个共享类型、OpenAPI 示例、15 张表 DDL（§9.2） | 无对应 Rust 或 TS 实现 | ⬜；状态机与类型保留为设计 |
| `docs/requirements/SRS-STAR-CANVAS-GAME-001.md` | 弹幕 Roguelike 全部 FR（§16–§25）、NFR（§26–§35）、ADR-CANVAS-GAME-001..008（§15）、用户故事（§12） | 客户端仅实现子集（见 §3） | 与 BD/DD 合并为本文 FR-10-* 行；具体条目以代码为准 |
| `docs/design/BD-STAR-CANVAS-GAME-001.md` | 4 crate 架构、5 状态机、26 张 `game_*` 表、接口（§8）、性能与安全 NFR（§9） | 四 crate 中 `canvas-game` 不存在；26 表无迁移；代码为纯前端 | 架构与表为 ⬜；前端部分与代码一致，采用 P0 子集 |
| `docs/design/DD-STAR-CANVAS-GAME-001.md` | 机器人 32 帧、弹幕 5 模式、伤害公式、Procedural generation、Yjs 协议、测试清单 | 与代码差异见 §3、§6；`canvas-game` crate 未建立 | 设计意图保留，状态按代码标注 |
| `docs/adr/0027-rust-pivot-agent-game.md` | 原 agent 游戏方向与 star-game 定义 | 文档自标 2026-09-07 SUPERSEDED（第 5 行），保留骨架 | 退役：不作为当前方案，仅记录依赖断裂 |
| `docs/plans/plan-032-rust-pivot-agent-game.md` | R5 agent 小游戏 PoC、GameBackend 集成点 | 第 3 行标注 2026-10-30 退役 | 🗑（star-game 相关） |
| `docs/implementation-plans/CANVAS-GAME-IMPL-PLAN-001.md` | 4 crate 骨架与阶段 2 待办（§3.4 canvas-game） | 第 3 行标注 2026-10-30 退役；canvas-game crate 未落地 | 🗑（历史记录） |
| `docs/WBS.md` §8.5.2、§8.5.3、§12.5 | P0/P1 "已完成"（L148-149）；交付资产清理（L217） | 与游戏层不可达的代码事实不符 | 以代码为准；WBS 状态列为待所有者修正的差距（§6） |
| `docs/implementation-plans/WBS-STAR-STABLE-RELEASE-20261031.md` | 发行载荷必须排除 `star-game`（L27、L46） | 与 `fb42a132` 删除一致 | 作为发行约束保留 |
| `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` L974 | 资源盘点，记录 BD-STAR-CANVAS-GAME-001 "文档自标退役但用户拍板恢复" | 与 BD 文档自标退役不一致 | 记为差距（§6），以所有者确认为准 |
| `docs/refactor/dedupe-20261113-lane-d.md` L29、L45 | 称 `star-game` 仍在 workspace 成员与 CI 中 | 当前 `Cargo.toml` 仅注释（L141），`.github/workflows` 仅发布工作流的排除断言（`publish-star-canvas-services.yml:320-341`） | 以代码为准：已删除 |
| `docs/refactor/ui-audit-20261113.md` L53 | 提及 `chrono-vibe` 为演示页 | 当前为重定向 | 以代码为准：✅ 重定向 |
| `docs/reports/PHASE-AGENT-GAME-IMPL-REPORT.md` | V0.1 agent-game 实装报告（仅核对其存在与状态行） | 对应组件已由 `a3365edd` 删除 | 🗑（组件）；报告作为历史记录 |
| `docs/requirements/SRS-CANVAS-001.md`、`docs/design/BD-CANVAS-001.md`、`docs/design/DD-CANVAS-001.md`、`docs/requirements/SRS-STAR-CANVAS-001.md` | 基础画布需求与设计（提及游戏化节点的扩展点） | 基础画布不属于 F10 | 不吸收；仅作为画布基础的边界依据（§1.2） |
| `docs/adr/`、`docs/plans/`、`docs/reviews/`、`docs/automation-design.md` 中的 gamify 相关条目 | 检索命中 | 与游戏化功能无直接设计内容或仅为提及 | 仅用于确认退役与依赖关系，未单独吸收 |

读取范围说明：六份种子文档均已读取目录结构，并读取与 FR 编号、依赖、退役标记相关的段落；未逐行通读全部约 1.2 万行设计内容。未在本文逐条复述的 FR 条目，其状态按代码 grep 结果给出，并标注 ⬜ 或 🟡。

Scope note for ECS: `crates/star-agent-runtime/src/ecs.rs` 为 Agent Runtime 的 ECS 实现，与游戏无关，不并入本文。
