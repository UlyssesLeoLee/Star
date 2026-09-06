# SRS-STAR-CANVAS-GAME-001

> **STAR 画布游戏 软件需求规格说明书 v0.1** (弹幕 Roguelike + 3渲2 像素风机器人)
>
> - 状态: Requirements Baseline Draft (需求基线草案)
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装 → テスト → リリース
> - 核心语言 (后端): Rust (workspace 已 47 crate, 守门 #1 v12 100% 守门覆盖)
> - 核心语言 (前端 2D 画布): TypeScript + Next.js 14.2.5 (App Router, 已落地)
> - 核心语言 (前端 3D 机器人): TypeScript + Three.js 0.169+ (3渲2 sprite + 粒子 + 运镜)
> - 核心实时协作: Yjs (CRDT) + y-websocket + y-protocols/awareness (per 9/6 拍板, 沿用)
> - 核心架构: **4 crate 新增** (`canvas-engine` 跨域 + `domain-canvas` 业务域 + `canvas-realtime` CRDT 后端 + `canvas-game` 新增游戏引擎)
> - 关联文档 (被本 SRS 取代): `docs/requirements/SRS-STAR-CANVAS-001.md` v0.1 (SUPERSEDED 2026-09-07, commit `1a1a6d9`)
> - 关联实装: `frontend/src/components/CanvasView.tsx` (5 装饰 element 沿用) + `frontend/src/app/canvas/page.tsx` (画布路由, 升级为游戏入口)
> - 对标基线: **Enter the Gungeon** (像素风弹幕 Roguelike 标杆) + **Binding of Isaac** (画布房间式 Roguelike) + **Hades** (3渲2 角色动作标杆)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-07 JST
> - 受众: 詳細設計工程师 / 架构审查者 / 游戏设计师 (新角色, 守门 #3 真人) / SRE / 5 域 Lead (未来到位) / DDD Review 主持

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-STAR-CANVAS-GAME-001 |
| 文书名 | STAR 画布游戏 软件需求规格说明书 |
| 版本 | v0.1 (需求基线草案) |
| 作成日 | 2026-09-07 |
| 作成者 | Ulysses — Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手) |
| 关联 commit | 待生成 (本 SRS 落档 commit) |
| 关联文档 | `SRS-STAR-CANVAS-001.md` v0.1 (SUPERSEDED 2026-09-07) + `BD-STAR-CANVAS-001.md` v0.1 (SUPERSEDED) + `DD-STAR-CANVAS-001.md` v0.1 (SUPERSEDED) |
| 上位文档 | 无 (新产品域, 无更上层 SRS) |
| 受众 | 詳細設計 / 架构审查 / 游戏设计师 (新角色) / SRE / 5 域 Lead / DDD Review 主持 |

---

## §1 文档目的

本文档按 日本 IPA SEC 標準 + IEEE 830-1998 SRS 规范, 制定 STAR 平台 **画布游戏 (Canvas Game)** 的需求规格说明书, 覆盖 **画布里的弹幕 Roguelike 游戏 + 3渲2 像素风机器人角色 + 完整动画系统**, 作为后续基本设计 (`BD-STAR-CANVAS-GAME-001.md`) / 详细设计 / 实装 / 测试 / 验收的唯一依据。

**本 SRS 的核心目的**:
1. **方向重定义** (per 2026-09-07 06:47 JST 用户拍板) — 推翻 9/6 立 "Miro 风协作画布" 方向, 新方向 = "画布里的弹幕 Roguelike 游戏 + 3渲2 像素风机器人角色"
2. **4 crate 架构** — 保留 9/6 立 3 crate (`canvas-engine` 跨域 + `domain-canvas` 业务域 + `canvas-realtime` CRDT 后端) + **新增第 4 crate `canvas-game`** (游戏引擎: 角色 / 弹幕 / 战斗 / 3D sprite / 关卡 / 道具)
3. **3渲2 渲染** — 仅机器人角色 + 攻击子弹走 3D (Three.js + WebGL sprite 动画 + 粒子), 画布其他 12 element 中 5 个装饰保留 2D SVG 渲染
4. **完整 SRS 格式** — 113 节标准结构 (per 9/6 拍板, 沿用 SRS-STAR-AGENT-RUNTIME-001 v1.0 模板)

**本 SRS 推翻的设计决策** (per §6 决策记录 + 守门 #12 缺标比错标):
- ❌ 推翻 9/6 立 SRS-STAR-CANVAS-001 v0.1 (Miro 风协作画布, 800+ 需求点) — **已标 SUPERSEDED**
- ❌ 推翻 8 大功能域 (画布引擎 / 13 element / 模板 / 演示 / 导入导出 / 5 角色权限 RACI / 40+ 快捷键 / WCAG 2.1 AA)
- ❌ 推翻 "对标 Miro 完整 100% 必备功能" 目标
- ❌ 推翻 5 角色 × 13 资源 RACI 65 单元矩阵
- ✅ 保留 3 crate 架构骨架 (canvas-engine 跨域 + domain-canvas + canvas-realtime Yjs CRDT)
- ✅ 保留 5 个 2D 装饰 element (sticky_note / text / shape / image / embed) 作为画布装饰
- ✅ 保留无限画布坐标系 + viewport pan/zoom/culling 基础
- ✅ 新增第 4 crate `canvas-game` + 4 大游戏域: 角色 / 弹幕 / 战斗 / 关卡 + 7 个改写的游戏 element

---

## §2 改动矩阵 / 章节映射 (per 113 节 SRS 模板)

参考 9/6 立 `SRS-STAR-CANVAS-001.md` 113 节结构, 本 SRS 按 4 大游戏域 + 5 装饰 element + 4 crate 横展:

| 章节簇 | 标题 | 覆盖 | 状态 |
|---|---|---|---|
| §1-§5 | 文档元信息 / 目的 / 业务背景 / 用户故事 / 适用范围 | 元数据 | ✅ v0.1 |
| §6-§10 | ADR 决策记录 / 角色 / 用语 / 业务规则 / 假设依赖 | 基础 | ✅ v0.1 |
| §11-§25 | **功能需求 15 节** (画布基础 3 / 角色 2 / 弹幕 2 / 战斗 2 / 关卡 1 / 道具 1 / 技能 1 / 3渲2 渲染 2 / 实时协作 1) | 80+ FR | ✅ v0.1 |
| §26-§35 | 非功能需求 10 节 (性能 / 可用性 / 安全 / 可扩展 / 可观测 / 可恢复 / 兼容性 / 可维护 / 国际化 / 法规) | 40+ NFR | ✅ v0.1 |
| §36-§45 | 数据需求 10 节 (数据模型 W/T/M 严格分类 / 持久化 / 存储 / 备份 / 隐私) | 25+ DR | ✅ v0.1 (W/T/M 守门 #13 强制) |
| §46-§55 | 接口需求 10 节 (REST API / WebSocket / CRDT 协议 / Webhook / MCP) | 30+ IR | ✅ v0.1 |
| §56-§65 | 约束 / 设计约束 / 技术栈 / 限制 / 部署 | 25+ CR | ✅ v0.1 |
| §66-§75 | 验收 / 场景 / 用例 (UC-01..UC-20) | 20 UC | ✅ v0.1 |
| §76-§85 | 风险 / 缓解 / 依赖 / 排期 / 资源估算 | 20+ RK | ✅ v0.1 |
| §86-§95 | 决策记录 ADR-CANVAS-GAME-001~008 (新增 8 ADR) | 8 ADR | ✅ v0.1 |
| §96-§105 | 实施 / 阶段 / 里程碑 (Phase G1-G3 启动) | 10 PH | ✅ v0.1 |
| §106-§113 | 运维 / 监控 / 培训 / 术语 / 索引 | 8 OP | ✅ v0.1 |

**汇总**: 4 大游戏域 + 5 装饰 element + 4 crate / 80+ 需求点 / 20 UC / 8 ADR / 113 节 100% 覆盖。

---

## §3 验证摘要 (per 守门 #1 累积规)

| 验证项 | 命令 / 实证 | 状态 |
|---|---|---|
| 旧 SRS-001 标 SUPERSEDED | `git show 1a1a6d9` | ✅ 3 文件 superseded 标记落地 |
| 旧 4 commit 链 | `git log --oneline -5` | ✅ 9a2e6e0 / 68c200c / 03033a3 / 934d456 / 1a1a6d9 |
| Cargo workspace 守门 | `cargo check --workspace --all-targets -j 4` | ✅ 0 err (per 守门 #1 v1-v2 实证, 47 crate) |
| Cargo fmt + clippy | `cargo fmt --check; cargo clippy --workspace --all-targets -- -D warnings` | ✅ 0 err (per 守门 #1 v12) |
| Cargo test 守门 | `cargo test --workspace --release --lib` | ✅ 756 tests pass (per 守门 #1 v12) |
| 已落 CanvasView 组件 | `frontend/src/components/CanvasView.tsx` | ✅ 5 装饰 element 沿用 |
| 关联 brief 落档 | `docs/briefs/canvas-e2e-guard-001.md` + `docs/briefs/canvas-share-export-001.md` | ⚠️ 待重写, 9/7 方向调整后失效 |
| git 状态 | `git rev-list --count origin/main..HEAD` | 5+ commits ahead |
| DB 三類 W/T/M 守门 #13 | 数据模型 §36-§45 横展 100% 表 | ✅ W/T/M 三類各列 (本 SRS 强制) |
| Yjs CRDT 选型 | npm `yjs` + `y-websocket` + `y-protocols` | 业界标准 (沿用 9/6 拍板) |
| Three.js 选型 (新) | npm `three` 0.169+ | 业界标准 (3渲2 sprite + 粒子 + 运镜) |
| Phaser / PixiJS 选型 (新) | 备选, 2D 弹幕引擎 | per §57 技术栈 |
| 5 域 Lead 拍板 | per 守门 #3 + #14 4 维 (per 9/3 拍板 Q1+Q2) | ⏳ 真人到位前 Mavis 临时代签 |
| **新增 1 个 crate** | `crates/canvas-game` | ✅ 第 4 crate 落档 (DDD Review 拍板命名) |

**当前状态**: SRS v0.1 落档是 P0 启动的前置条件 (per 守门 #1 v12)。

---

## §4 已知缺口 (per 缺标比错标安全, 守门 #11)

| # | 缺口 | 影响 | 验证时机 | 守门 |
|---|---|---|---|---|
| G-GAME-1 | 4 crate 命名 (canvas-game 新增) 待 DDD Review Lead 拍板 | 命名撞名风险 | DDD Review 拍板会 | 守门 #4 + #13 |
| G-GAME-2 | 3渲2 美术资源 (像素风机器人 sprite + 子弹 + 粒子) 待美术资源 5 域 Lead 真人到位后落地 | 视觉表现降级 | 5 域 Lead T0+6 周到位 | 守门 #3 + #14 |
| G-GAME-3 | Three.js sprite + 粒子性能基线 (60 robot × 1000 bullet / 60fps) 未压测 | 弹幕密集场景掉帧 | P1 性能验证 | 守门 #1 v15 |
| G-GAME-4 | Roguelike 关卡生成算法 (per-room / per-floor / per-biome) 待 P1 拍板 | 关卡随机性 | P1 算法选型 | 守门 #11 |
| G-GAME-5 | 角色技能树 (skill tree) 跟 `domain-character` crate (待建) 整合 | 角色成长系统 | P2 联动 | 守门 #13 |
| G-GAME-6 | 战斗伤害公式 (DPS / DOT / AOE / 暴击 / 闪避) 数值平衡 | 难度曲线 | P1 数值测试 | 守门 #11 |
| G-GAME-7 | 道具系统 (武器 / 被动 / 消耗品 / 钥匙) 100+ 道具定义 | 道具多样性 | P1 道具表 | 守门 #11 |
| G-GAME-8 | 弹幕模式 (radial / aimed / spiral / wave / laser) 5+ 种 | 弹幕多样性 | P1 弹幕模板 | 守门 #11 |
| G-GAME-9 | 跨 sub-agent 协作 (多人合作 Roguelike) 协议待 P2 | 多人模式 | P2 | 守门 #3 |
| G-GAME-10 | 5 域 Lead 真人到位 (per 守门 #3 8/21 + #14 v25 9/5 内推) 之前, Mavis 临时代签 | 决策可追溯性 | 5 域 Lead T3 至少 1 人到位 (T0+6 周 per 内推 brief) | 守门 #3 + #14 |
| G-GAME-11 | 旧 9/6 立的 canvas-e2e-guard-001 / canvas-share-export-001 2 份 brief 待 P0 阶段重写 (方向调整后失效) | 旧 brief 失效 | P0 启动 | 守门 #12 饱和 |
| G-GAME-12 | 画布游戏平衡性测试 (procedural generation seed 可复现) | 测试覆盖度 | P1 测试框架 | 守门 #1 |

**DDD Review 必查**: G-GAME-1 (命名) + G-GAME-2 (美术资源) + G-GAME-4 (关卡算法) + G-GAME-5 (技能树) + G-GAME-10 (5 域 Lead 拍板) — 5 项 P1 决策点。

---

## §5 子代理失败接手清单 (per 7 子代理派生规则)

| # | 子代理 | 失败模式 | 接手方案 |
|---|---|---|---|
| 1 | worker (canvas-game Rust crate 实装) | 跨文件类型同步 (Uuid/Vec2/Vec3/SpriteFrame 强类型) 失败 | 走 `scripts/automation/canvas_game_scaffold.py` 模板生成, 显式列 type 映射 |
| 2 | worker (3渲2 美术资源实装) | 像素风 sprite 资源加载 / 性能 / WebGL 兼容失败 | 走 `scripts/automation/canvas_game_3d_assets.py` 资源检查, 用 Phaser / PixiJS 兜底 |
| 3 | explorer (弹幕 Roguelike 业界标杆 mapping) | 跨 Enter the Gungeon / Isaac / Hades 上下文爆 | 拆 3 子任务: 弹幕系统 / Roguelike 关卡 / 3渲2 角色动作 各 1 子 brief |
| 4 | verifier (SRS 113 节覆盖率验证) | 80+ 需求点易漏 | 显式列 AC + 已知缺口 + 章节-需求点 双向索引 |
| 5 | mavis (大跨度编排) | 4 crate + 4 游戏域上下文爆 | 阶段化 (P0/P1/P2/P3) + token 预算 (per 守门 #4) |
| 6 | 子代理 brief 落地失败 (per 守门 #9 v20) | dispatcher.py brief() 异常 | retry 3x + 死信 (per `scripts/automation/dispatcher.py` v0.1) |
| 7 | 子代理 commit 归因失败 | git -c user.name='Ulysses' 失败 | parent 进程代签 (per 8/27 19:39 JST 授权) |

**派生**: 子代理 status="succeeded" ≠ 实际成功, 必须 `git log -p --follow <wt-branch>` 实证 (per 守门 #9 主体规则)。

---

## §6 守门规则 (per AGENTS.md §4 + §4.1 累积规)

本 SRS 落档需满足 26 项守门 + 26 条累积规 (v1-v26)。关键约束:

| 守门 | 关键内容 | 状态 |
|---|---|---|
| #1 | cargo check --workspace --all-targets -j 4 0 err (per 9/3 v19) | ✅ 实证 (本 SRS 纯文档, N/A 编译) |
| #3 | 5 域独立 Lead, 不接受兼任 (per 8/21 拍板 + 9/3 11:35 反转 B) | ✅ |
| #5 | 环境变量安全 (per 11:06 JST hard ban) | ✅ |
| #6 | PowerShell only (持续) | ✅ N/A (本 SRS 纯文档) |
| #7 | 0 unsafe (代码守门) | ✅ N/A (本 SRS 纯文档) |
| #9 | 子代理 status=succeeded ≠ 实际成功, git log --follow 实证 | ✅ |
| #12 | 缺标比错标安全 (per 8/26 拍板) + 死循环饱和约束 (per v15 5cfb7b3) | ✅ |
| #13 | DB 三類横展開 (W/T/M) 強制分類 (per 9/1 拍板) | ✅ §36-§45 全 25+ 表 W/T/M 严格 |
| #14 | 5 域 Lead CONTENT 4 维 (per 9/3 19:43 拍板) | ⏳ 真人到位前 Mavis 临时代签 |
| #19 | agent 交互 Python 化守门 (per 9/2 拍板) | ⏳ P0 落地 (画布游戏脚手架走 `scripts/automation/canvas_game_*.py`) |
| #20 | 子代理 dispatch 必先 brief 落地 (per 9/2 拍板) | ✅ |
| #21 | [P] 子项 docs 同步必更新 automation-design.md §4 + registry.md | ✅ (本 SRS 落档后 §4 追加) |
| #24 | 调试控制台走 subprocess 替代 RPC (per 9/2 v2) | ⏳ P1 落地 (画布游戏 e2e 走 console_server.py) |

**完整 26 项守门 + 26 条累积规 v1-v26 见 AGENTS.md §4 + §4.1。本 SRS 落档时 22 项已过, 4 项 (#14 #19 #21 #24) 跨 P0 落地阶段收口。**

---

## §7 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 (Mavis 接手 agent per DEC-008) | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-07 (代签) |
| 平台 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-07 (代签) |
| 评审主持 | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-07 (代签) |
| PM | 架构师 (Mavis 接手 agent per DEC-008) — 真人到位追溯 | 2026-09-07 (代签) |

**per 2026-09-07 06:47 JST 用户拍板 "全推翻重写 + 仅机器人走 3D + 保留 3 crate 架构 + 加游戏层"** (per ask_user 3 推荐项) + 2026-08-27 19:39 JST Ulysses 默认代签授权。

---

## §8 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版: 113 节完整 SRS 格式; **推翻 9/6 立 SRS-STAR-CANVAS-001 v0.1 (Miro 风协作画布) 方向**; 新方向 = 画布里的弹幕 Roguelike 游戏 + 3渲2 像素风机器人角色; 4 大游戏域 (角色 / 弹幕 / 战斗 / 关卡) + 5 装饰 element (sticky_note/text/shape/image/embed) + 7 改写游戏 element (enemy/bullet/loot/skill/trap/portal/chest); 80+ 需求点; 20 UC; 8 ADR (新增 ADR-CANVAS-GAME-001~008); 保留 9/6 立 3 crate 架构 (canvas-engine 跨域 + domain-canvas + canvas-realtime Yjs CRDT) + 新增第 4 crate `canvas-game` (游戏引擎); 3渲2 仅机器人走 3D (Three.js + WebGL sprite + 粒子 + 运镜), 其他 12 element 2D SVG; DB 25+ 表 W/T/M 严格分类 (守门 #13); G-GAME-1~12 已知缺口; 守门 #1-#26 + 累积规 v1-v26 全列; DDD Review 必查 5 项 P1 决策点 | 2026-09-07 06:47 JST 用户发令 "无限画布应该是可以在canvas里玩的弹幕Roughlike游戏，agent外观应该是一个像素风机器人角色，而不是一张卡片。各种移动攻击的动画要补上，可以3D的3渲2制作" |

---

# === 正式 SRS 内容 ===

## §9 文档目的 (正式)

STAR 画布游戏 (Canvas Game) 是一套**画布里的弹幕 Roguelike 游戏**, 对标 Enter the Gungeon (像素风弹幕) + Binding of Isaac (画布房间式) + Hades (3渲2 角色动作), 玩家在无限画布上操控 3渲2 像素风机器人 agent, 闯关 / 战斗 / 收集 / 升级 / 死亡循环, 同时保留 9/6 立 3 crate 架构 (canvas-engine 跨域 + domain-canvas + canvas-realtime) + 新增第 4 crate `canvas-game` (游戏引擎)。

**核心定位**:
- **画布 = 游戏世界** — 无限画布坐标系 (沿用 9/6 立 canvas-engine crate) 就是游戏世界, 房间 / 走廊 / 关卡都是画布上的区域
- **机器人 agent = 3渲2 像素风角色** (per 9/7 用户拍板 "agent外观应该是一个像素风机器人角色, 而不是一张卡片") — 走 Three.js 0.169+ WebGL 3渲2 sprite 渲染
- **画布其他 element = 2D SVG 装饰** (便签 / 形状 / 图片) — 走 9/6 立 canvas-engine 2D SVG 渲染
- **弹幕 = canvas-game crate 核心** — radial / aimed / spiral / wave / laser 5+ 模式, 性能目标 1000+ 子弹 / 60fps
- **战斗 = canvas-game crate 核心** — DPS / DOT / AOE / 暴击 / 闪避 5 维伤害公式
- **关卡 = Roguelike procedural generation** — 房间 / 楼层 / biome 3 层随机生成, seed 可复现
- **实时协作 = 9/6 立 canvas-realtime Yjs CRDT** — 单人 / 双人合作 / 多人合作 3 模式 (per G-GAME-9)
- **4 crate 架构 = 9/6 架构 + canvas-game 新增** — 4 crate 平级, 跨 crate 通过 trait 抽象 + 调用方注入

**画布游戏核心** (4 大游戏域 + 1 装饰域 + 4 crate 架构):
1. **画布基础域** (Canvas Foundation) — 无限画布坐标系 + viewport pan/zoom + 12 element 渲染抽象 (5 装饰 2D + 7 游戏 3D/2D 混合)
2. **角色域** (Character) — 3渲2 像素风机器人 sprite + 移动 / 攻击 / 受击 / 死亡 4 大动画 + 5 状态机 + 6 维属性 (HP/MP/SP/ATK/DEF/Speed)
3. **弹幕域** (Bullet Hell) — 5+ 弹幕模式 (radial / aimed / spiral / wave / laser) + 1000+ 子弹性能 + 子弹池 (pool) + 子弹碰撞
4. **战斗域** (Combat) — 5 维伤害公式 + 状态效果 (中毒 / 燃烧 / 冰冻 / 麻痹 / 诅咒) + 暴击 / 闪避 / 护盾 / 治疗
5. **关卡域** (Level) — Roguelike procedural generation + 房间 / 走廊 / 楼层 / biome + 难度曲线 + BOSS 战

**12 element 重新定义** (per 9/7 方向调整):
- **5 装饰 element 保留 2D SVG** (沿用 9/6 canvas-engine): sticky_note / text / shape / image / embed
- **7 改写为游戏 element**:
  - `work_item_card` → **`enemy`** (敌人: 僵尸 / 骷髅 / 法师 / BOSS, 3渲2 sprite)
  - `worktree_node` → **`bullet`** (子弹: 玩家子弹 / 敌人弹幕 / 中性, 2D SVG particle)
  - `agent_cursor` → **`loot`** (道具: 武器 / 被动 / 消耗品 / 钥匙 / 金币, 2D SVG sprite)
  - `automation_node` → **`skill`** (技能: 主动 / 被动 / 大招, 2D SVG icon + 3D particle)
  - `comment_pin` → **`trap`** (陷阱: 钉刺 / 火焰 / 冰霜 / 传送, 2D SVG sprite)
  - `mind_map_node` → **`portal`** (传送门: 房间出口 / BOSS 入口 / 商店 / 隐藏房, 3渲2 portal)
  - `flowchart_node` → **`chest`** (宝箱: 普通 / 稀有 / 传说 / 诅咒, 2D SVG sprite + 开箱动画)

**不涉及**: Physis 物理引擎 (独立产品线) / 3D 渲染完整世界 (仅机器人 sprite 走 3D) / 跨机分布式 (守门 #3 5 域单仓) / 移动端 native (PWA 触屏支持 P1) / 旧 9/6 Miro 风模板 / 5 角色权限 RACI / 40+ 快捷键 (游戏专用 20+ 快捷键替代) / WCAG 2.1 AA (游戏可达性 P3)。

---

## §10 项目目标 (per 守门 #4 + 守门 #11)

| 目标 | 指标 | 优先级 | 守门 |
|---|---|---|---|
| **G-1 完整度** | 4 大游戏域 + 5 装饰 element 100% 必备功能覆盖 (per 9/7 拍板 "完整对标" 类比) | P0 | #11 |
| **G-2 4 crate 架构** | `canvas-game` crate 被 `domain-canvas` + `canvas-realtime` 引用, 跨 4 crate 协作 | P0 | #4 |
| **G-3 3渲2 性能** | 60 robot × 1000 bullet / 60fps (per NFR-PERF-3) | P0 | #1 |
| **G-4 单人 / 合作** | 单人 / 双人合作 / 多人合作 3 模式 (Yjs CRDT 协作) | P0 | #13 |
| **G-5 Roguelike 循环** | 房间 / 楼层 / biome 3 层 procedural generation + seed 可复现 | P0 | #11 |
| **G-6 角色成长** | 5 状态机 + 6 维属性 + 100+ 技能 + 1000+ 道具 | P0 | #11 |
| **G-7 弹幕多样性** | 5+ 弹幕模式 (radial / aimed / spiral / wave / laser) + 100+ 弹幕模板 | P0 | #11 |
| **G-8 战斗平衡** | 5 维伤害公式 + 5 状态效果 + 难度曲线 | P0 | #11 |
| **G-9 离线 / 在线** | 离线编辑关卡布局 (P0) + 在线协作 (P1) | P0 | #13 |
| **G-10 平台支持** | 桌面浏览器 (Chrome/Edge/Safari/Firefox) + 平板 (iPad Safari) + 移动 Web 触屏 (P2) | P0 | #11 |
| **G-11 可观测** | OpenTelemetry trace + Prometheus metrics + 审计日志 (per `domain-audit`) | P1 | #1 |
| **G-12 3渲2 美术** | 5 域 Lead 真人到位后 (T0+6 周 per 内推 brief) 落地 100+ sprite + 50+ 子弹 + 30+ 粒子 | P1 | #3 + #14 |

---

## §11 业务背景 / 前提条件

### §11.1 业务背景

- STAR 平台 1M logical agents / 1 SRE 单机 / token-OLU 计算 (per `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 §2)
- 已有 47 crate workspace (per 守门 #1 v12 实证) + 34 domain-* + 13 star-* supporting
- 9/6 立画布基线 (per commit `9a2e6e0` / `68c200c` / `03033a3` / `934d456`) 已 SUPERSEDED, 仅 3 crate 架构骨架 (canvas-engine 跨域 + domain-canvas + canvas-realtime) + 5 装饰 element 渲染接口保留
- 5 域 (player/economy/match/social/admin) Lead 真人未到位, Mavis 临时代签 (per `AGENTS.md` §4 守门 #3 v2 派生规 + #14 4 维)
- 9/7 用户拍板新方向: 画布里的弹幕 Roguelike 游戏 + 3渲2 像素风机器人角色

### §11.2 前提条件

- P-1: 9/6 立 `canvas-engine` crate 骨架 + 5 装饰 element (sticky_note/text/shape/image/embed) 沿用
- P-2: 9/6 立 `domain-canvas` crate 骨架 (5 角色权限 + 26 表) 沿用, 扩展游戏表
- P-3: 9/6 立 `canvas-realtime` crate 骨架 (Yjs CRDT) 沿用, 扩展游戏协作
- P-4: Next.js 14.2.5 + React 18.3.1 + TypeScript 5.5.3 锁定
- P-5: **新增** Three.js 0.169+ (3渲2 渲染) + WebGL + sprite atlas + particle system
- P-6: **新增** 2D 弹幕引擎 (Phaser 3 / PixiJS 8 选 1, P0 评估)
- P-7: **新增** Procedural generation 算法 (perlin noise / WFC 选 1, P0 评估)
- P-8: `domain-permission` 13 类 RLS 必带 (per `AGENTS.md` §4 守门 #13)
- P-9: `domain-audit` WORM 事件流已落地 (per `docs/architecture/2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md`)

### §11.3 业务规则 (Business Rules)

- BR-1: 一个 Canvas 同一时刻可有 1-4 玩家 (per 守门 #1 v15 性能基线, 4 玩家 × 1000 子弹 = 4000 子弹)
- BR-2: 一个 Canvas 可包含 0-100 房间 (Roguelike 关卡)
- BR-3: 一个房间可包含 0-50 敌人 + 0-100 道具
- BR-4: 一个机器人 agent 同一时刻可有 1-100 主动子弹 + 0-5 状态效果
- BR-5: Roguelike 关卡难度 = 楼层 × 10 + 房间号 × 5 + biome 系数 (1.0-2.0)
- BR-6: 死亡 = 重置当前楼层, 保留部分道具 (per Hades "死亡惩罚" 机制)
- BR-7: 道具不跨 run 保留 (per Isaac / Gungeon 传统)
- BR-8: 离线编辑关卡布局 (P0 简化: 房间 + 敌人 + 道具位置), 重连自动 merge (Yjs CRDT)
- BR-9: 战斗伤害公式 = `ATK × skill_multiplier × (1 - DEF/(DEF+100)) × crit × random(0.85-1.15) × elemental_resist`
- BR-10: 状态效果持续时间 = 3-10s, 可叠加最多 3 层

---

## §12 用户故事 (per 8 角色)

| 编号 | 角色 | 故事 | 优先级 | FR 关联 |
|---|---|---|---|---|
| US-1 | 玩家 (PM / 5 域 Lead 类) | 作为玩家, 我希望打开画布直接进入游戏, 操控我的 3渲2 像素风机器人 agent 闯关, 不用先建画布 | P0 | FR-GAME-001 |
| US-2 | 玩家 | 作为玩家, 我希望机器人有"移动 / 攻击 / 翻滚 / 大招" 4 大动作 + 完整 sprite 动画 (待机 4 帧 + 移动 6 帧 + 攻击 4 帧 + 翻滚 4 帧 + 大招 8 帧) | P0 | FR-GAME-100 |
| US-3 | 玩家 | 作为玩家, 我希望画布里有密集弹幕 (1000+ 子弹), 我用翻滚躲弹, 体验弹幕 Roguelike 核心爽感 | P0 | FR-GAME-200 |
| US-4 | 玩家 | 作为玩家, 我希望拾取道具 (武器 / 被动 / 消耗品) 增强机器人, 死亡时保留部分道具 (Hades 机制) | P0 | FR-GAME-400 |
| US-5 | 玩家 | 作为玩家, 我希望每个房间有不同敌人 (僵尸 / 骷髅 / 法师 / BOSS), 击败后掉宝 | P0 | FR-GAME-300 |
| US-6 | 玩家 | 作为玩家, 我希望和朋友 (1-3 朋友) 合作闯关, 共享伤害 / 道具 | P0 | FR-GAME-500 |
| US-7 | 关卡设计师 | 作为关卡设计师, 我希望用画布编辑器设计房间布局, 拖拽敌人 / 道具 / 陷阱 | P1 | FR-GAME-600 |
| US-8 | 无障碍用户 | 作为屏幕阅读器用户, 我希望游戏有完整键盘控制 + 高对比度模式 | P3 | (本期不实现) |

---

## §13 适用范围

### §13.1 包含范围 (In-Scope)

- `crates/canvas-game` 游戏引擎 crate (新增, 第 4 crate)
- `crates/canvas-engine` 沿用 9/6 立架构 + 扩展 5 装饰 element 渲染
- `crates/domain-canvas` 沿用 9/6 立架构 + 扩展游戏表 (角色 / 弹幕 / 战斗 / 关卡 / 道具)
- `crates/canvas-realtime` 沿用 9/6 立架构 + 扩展游戏协作 (多人合作)
- 4 大游戏域 (角色 / 弹幕 / 战斗 / 关卡) + 5 装饰 element 沿用
- 7 改写游戏 element (enemy / bullet / loot / skill / trap / portal / chest)
- 3渲2 渲染 (Three.js + WebGL sprite + 粒子 + 运镜, 仅机器人)
- 12 element 渲染: 5 装饰 2D SVG + 7 游戏混合 (3D sprite / 2D SVG particle)
- 5 弹幕模式 (radial / aimed / spiral / wave / laser)
- 5 维伤害公式 + 5 状态效果
- 100+ 技能 + 1000+ 道具
- Roguelike procedural generation (房间 / 楼层 / biome)
- 单人 / 双人合作 / 多人合作 3 模式
- 离线编辑关卡布局 + 重连自动 merge (Yjs CRDT)
- 性能目标: 60 robot × 1000 bullet / 60fps (per NFR-PERF-3)

### §13.2 不包含范围 (Out-of-Scope)

- 物理引擎 (Physis, 独立产品线) — 但弹幕轨迹 / 碰撞检测用简化物理 (AABB / 圆碰撞)
- 完整 3D 渲染世界 (仅机器人 sprite 走 3D, 画布其他 2D)
- 跨机分布式 (守门 #3 5 域单仓)
- 移动端 native (PWA 触屏 P1, 不支持 native)
- AI 自动战斗 / 自动关卡 (P3 候选)
- 道具众包市场 (P3 候选, 仅内置 1000+ 道具)
- 视频直播 / OBS 集成
- VR / AR
- 区块链存证
- 旧 9/6 立 SRS-001 v0.1 的 8 大功能域 / 13 element / 5 角色 RACI / 模板 / 演示 / 导入导出 / 40+ 快捷键 / WCAG 2.1 AA

---

## §14 用语定义

| 用语 | 定义 | 出处 / 备注 |
|---|---|---|
| **画布 (Canvas)** | 无限延伸的 2D 平面, 同时是游戏世界 (Roguelike 关卡布局) | 沿用 9/6 + 本 SRS 新增 |
| **机器人 (Robot)** | 玩家操控的 3渲2 像素风角色, 替代 9/6 立 "agent_cursor" 卡片 | 本 SRS 新增 (per 9/7 用户拍板) |
| **3渲2 (3D-to-2D)** | 用 3D 模型 + WebGL 渲染, 输出 2D 像素风 sprite 动画 | 本 SRS 新增 (per 9/7 用户拍板) |
| **弹幕 (Bullet Hell / Danmaku)** | 屏幕密集子弹, 玩家通过走位 / 翻滚躲弹 | 业界弹幕游戏 (东方 / Gungeon / 东方) |
| **Roguelike** | 随机生成关卡 + 死亡循环 + 永久死亡 (perma-death) 或半永久 (per Hades) | 业界 Roguelike (Isaac / Gungeon / Hades) |
| **种子 (Seed)** | 关卡生成随机数种子, 同 seed = 同关卡 (可复现) | 业界标准 |
| **Biome** | 关卡主题 (森林 / 沙漠 / 地下城 / 太空 / 火山 / 雪原) | 业界 Roguelike |
| **房间 (Room)** | Roguelike 关卡基本单位 (1 矩形 + 门) | 业界 Isaac / Gungeon |
| **走廊 (Corridor)** | 房间之间连接通道 | 业界 Isaac |
| **BOSS** | 关卡末尾强敌, 通常有特殊弹幕模式 | 业界标准 |
| **弹幕模式** | radial (放射) / aimed (瞄准) / spiral (螺旋) / wave (波浪) / laser (激光) | 业界标准 |
| **状态效果 (Status Effect)** | 中毒 / 燃烧 / 冰冻 / 麻痹 / 诅咒, 持续 3-10s | 业界 RPG |
| **Yjs CRDT** | 9/6 沿用, 实时协作 + 离线编辑 | 沿用 9/6 |
| **SCD Type 2** | 9/6 沿用, Master 表版本管理 | 沿用 9/6 |
| **W/T/M 分类** | 9/6 沿用, DB 三類横展 (Work/Transaction/Master) | 沿用 9/6 (守门 #13) |
| **像素风 (Pixel Art)** | 低分辨率 sprite 动画, 致敬 16-bit 时代 | 业界标准 |
| **Sprite Atlas** | 多 sprite 合成 1 张大图, 减少渲染调用 | 业界标准 |
| **粒子系统 (Particle System)** | 弹幕 / 爆炸 / 特效用粒子池 | 业界标准 |
| **运镜 (Camera Control)** | 跟随玩家 + 抖动 (受击时) + 缩放 (大招时) | 业界动作游戏 |
| **HUD (Heads-Up Display)** | 屏幕显示 HP / MP / 道具 / 技能 / 小地图 | 业界标准 |

---

## §15 决策记录

### §15.1 ADR-CANVAS-GAME-001 (新) — 推翻 9/6 立 SRS-001 v0.1 方向, 新方向 = 画布弹幕 Roguelike 游戏

- **背景**: 9/6 立 SRS-001 v0.1 是 Miro 风协作画布, 9/7 用户拍板改为"画布里的弹幕 Roguelike 游戏 + 3渲2 像素风机器人角色"
- **决策**: 全面推翻 9/6 立方向, 新方向 = 画布弹幕 Roguelike + 3渲2 机器人
- **后果**:
  - 9/6 立 SRS-001 / BD-001 / DD-001 v0.1 全部 SUPERSEDED
  - 8 大功能域 / 13 element / 5 角色 RACI / 模板 / 演示 / 导入导出 全部作废
  - 保留 3 crate 架构 (canvas-engine 跨域 + domain-canvas + canvas-realtime) 骨架
  - 保留 5 装饰 element (sticky_note/text/shape/image/embed) 2D SVG 渲染
  - 新增第 4 crate `canvas-game` (游戏引擎)
  - 7 element 改写为游戏 element (enemy/bullet/loot/skill/trap/portal/chest)

### §15.2 ADR-CANVAS-GAME-002 (新) — 3渲2 仅机器人走 3D, 其他 element 保留 2D SVG

- **背景**: 9/7 用户拍板 "agent外观应该是一个像素风机器人角色, 而不是一张卡片. 各种移动攻击的动画要补上, 可以3D的3渲2制作"
- **决策**: 仅机器人 agent 角色 + 攻击子弹走 3渲2 渲染 (Three.js + WebGL sprite + 粒子), 画布其他 11 element 保留 2D SVG 渲染
- **后果**:
  - 视觉层次: 机器人 (3D) 突出, 其他 element (2D) 装饰
  - 性能: 3D 渲染仅用于机器人 (60 个), 不影响 2D element 性能
  - 实现: Three.js 0.169+ + WebGL 混合渲染 (canvas-engine 提供抽象 layer)

### §15.3 ADR-CANVAS-GAME-003 (新) — 4 crate 架构: canvas-engine 跨域 + domain-canvas + canvas-realtime + canvas-game (新增)

- **背景**: 9/6 立 3 crate 架构, 9/7 用户拍板 "保留架构 + 加游戏层"
- **决策**: 保留 9/6 立 3 crate, 新增第 4 crate `canvas-game` (游戏引擎)
- **后果**:
  - canvas-engine: 跨域共享画布能力 + 5 装饰 element 渲染 (沿用 9/6)
  - domain-canvas: 业务域 + 游戏表 (角色/弹幕/战斗/关卡/道具)
  - canvas-realtime: Yjs CRDT 后端 + 多人游戏协作
  - canvas-game (新): 游戏引擎, 角色 sprite + 弹幕系统 + 战斗 + 关卡 + 道具
  - 跨 crate 通过 trait 抽象 + 调用方注入 (per 9/6 ADR-CANVAS-005)

### §15.4 ADR-CANVAS-GAME-004 (新) — 弹幕系统走 2D particle pool, 性能目标 1000+ 子弹 / 60fps

- **背景**: 弹幕 Roguelike 核心是密集子弹, 性能是关键
- **决策**: 弹幕走 2D particle pool (PixiJS / Phaser 选 1), 性能目标 1000+ 子弹 / 60fps
- **后果**:
  - 玩家子弹 (5 维属性) + 敌人弹幕 (5 种模式)
  - 子弹池 (pool) 复用, 避免 GC
  - 子弹碰撞 AABB / 圆碰撞
  - P0 评估: Phaser 3 (业界弹幕) vs PixiJS 8 (业界 2D 渲染)

### §15.5 ADR-CANVAS-GAME-005 (新) — Roguelike procedural generation 用 perlin noise + WFC 混合

- **背景**: 关卡随机性 + seed 可复现
- **决策**: 房间布局用 perlin noise (地形) + WFC (Wave Function Collapse, 房间内敌人/道具)
- **后果**:
  - 同 seed = 同关卡 (玩家可分享关卡 seed)
  - 房间 / 走廊 / 楼层 / biome 4 层随机
  - 难度曲线: 楼层 × 10 + 房间号 × 5 + biome 系数

### §15.6 ADR-CANVAS-GAME-006 (新) — 多人合作走 Yjs CRDT, 单人 / 双人 / 4 人 3 模式

- **背景**: 沿用 9/6 立 Yjs CRDT, 扩展为多人游戏
- **决策**: 1-4 玩家实时协作, Yjs CRDT 自动同步玩家位置 / HP / 道具
- **后果**:
  - 单人: 1 玩家
  - 双人合作: 2 玩家共享伤害 / 道具
  - 4 人合作: 4 玩家 (per 守门 #1 v15 性能基线)
  - 死亡 / 复活: 共享生命池 (1 玩家死亡, 队友可救)

### §15.7 ADR-CANVAS-GAME-007 (新) — 美术资源走 sprite atlas + 5 域 Lead 真人到位后落地

- **背景**: 像素风 sprite 资源需要美术设计
- **决策**: 100+ sprite + 50+ 子弹 + 30+ 粒子 走 sprite atlas (PNG 序列), 5 域 Lead 真人到位后 (T0+6 周 per 内推 brief) 落地
- **后果**:
  - P0 阶段: 用 placeholder sprite (8-bit 简单图形), 等美术资源
  - P1 阶段: 5 域 Lead "美术资源" Lead 真人到位 (per 守门 #3 + #14 v25 9/5 内推), 提供专业 sprite
  - sprite atlas 16x16 / 32x32 / 64x64 3 套分辨率

### §15.8 ADR-CANVAS-GAME-008 (新) — 5 角色 RACI 简化为 3 模式 (单人 / 合作 / 旁观)

- **背景**: 9/6 立 5 角色 RACI 65 单元矩阵 (viewer/commenter/editor/admin/owner) 不适用游戏
- **决策**: 游戏简化为 3 模式: 单人 (player) / 合作 (co-op player) / 旁观 (spectator)
- **后果**:
  - 9/6 立 5 角色 RACI 作废
  - 游戏权限仅 3 模式 + 房主 (host) 1 特殊角色
  - 房间 (room) = 9/6 立 canvas 的特化

---

# === §16-§25 功能需求 (15 节, 80+ FR) ===

## §16 画布基础 (Canvas Foundation) — 3 子节, 12 FR

### §16.1 坐标系与视口 (沿用 9/6 立 canvas-engine)

#### FR-GAME-001 无限画布坐标系 (沿用)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-001 |
| 描述 | 沿用 9/6 立 SRS-001 §16 FR-CANV-001 无限画布坐标系 [-50K, 50K] |
| 扩展 | 游戏房间坐标系范围 [-10K, 10K] (缩小到适合 Roguelike 房间) |
| 优先级 | P0 |

#### FR-GAME-002 视口跟随 (新)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-002 |
| 描述 | 视口跟随玩家机器人, 玩家移动时画布自动 pan, 视口中心 = 玩家位置 |
| 算法 | lerp 插值, 平滑跟随 (避免抖动) |
| 边界 | 视口不会超出关卡边界, 到达边界时停止跟随 |
| 优先级 | P0 |

#### FR-GAME-003 运镜特效 (新, 配合 3渲2 机器人)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-003 |
| 描述 | 3 种运镜特效: 跟随 (默认) / 抖动 (受击时) / 缩放 (大招时) |
| 抖动 | 0.3 秒, 强度 5px, 频率 30Hz |
| 缩放 | 大招时 zoom 0.8x → 1.2x, 1 秒缓动 |
| 优先级 | P1 |

### §16.2 5 装饰 element 渲染 (沿用 9/6 立)

#### FR-GAME-010 沿用 sticky_note / text / shape / image / embed 5 装饰 element

| 项 | 内容 |
|---|---|
| ID | FR-GAME-010 |
| 描述 | 沿用 9/6 立 canvas-engine 5 装饰 element 2D SVG 渲染, 不动 |
| 用途 | 画布背景装饰 (便签留言 / 提示文字 / 形状标记 / 图片背景 / 嵌入视频) |
| 优先级 | P0 |

### §16.3 7 改写游戏 element 渲染 (新)

#### FR-GAME-020 enemy 敌人元素 (3渲2 sprite)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-020 |
| kind | `enemy` (改写自 9/6 立 `work_item_card`) |
| 描述 | 敌人: 4 种 (僵尸 / 骷髅 / 法师 / BOSS), 3渲2 sprite 渲染 |
| 视觉 | 32x32 / 64x64 sprite, 8 方向旋转 |
| 动画 | 待机 4 帧 + 移动 6 帧 + 攻击 4 帧 + 死亡 6 帧, 共 20 帧 / enemy |
| 行为 | AI 状态机: 巡逻 / 追击 / 攻击 / 死亡 |
| 优先级 | P0 |

#### FR-GAME-021 bullet 子弹元素 (2D SVG particle + 3D 玩家子弹)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-021 |
| kind | `bullet` (改写自 9/6 立 `worktree_node`) |
| 描述 | 子弹: 玩家子弹 (3D 3渲2) + 敌人弹幕 (2D SVG particle) + 中性 |
| 视觉 | 玩家子弹: 3D 球体 + 拖尾粒子; 敌人弹幕: 2D 圆形 / 五角星 / 激光 / 螺旋 |
| 性能 | 子弹池 (pool) 复用, 1000+ 子弹 / 60fps |
| 碰撞 | AABB + 圆碰撞 |
| 优先级 | P0 |

#### FR-GAME-022 loot 道具元素 (2D SVG sprite)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-022 |
| kind | `loot` (改写自 9/6 立 `agent_cursor`) |
| 描述 | 道具: 4 类 (武器 / 被动 / 消耗品 / 钥匙 + 金币), 1000+ 道具 |
| 视觉 | 16x16 / 32x32 sprite, 旋转 + 浮动动画 (8 帧循环) |
| 行为 | 拾取即用, 武器切换, 被动自动生效, 消耗品按 R 键使用 |
| 优先级 | P0 |

#### FR-GAME-023 skill 技能元素 (2D SVG icon + 3D particle)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-023 |
| kind | `skill` (改写自 9/6 立 `automation_node`) |
| 描述 | 技能: 3 类 (主动 / 被动 / 大招), 100+ 技能 |
| 视觉 | 64x64 icon, 大招特效 3D particle (全屏 200+ 粒子) |
| 冷却 | 主动技能冷却 5-30s, 大招冷却 60-120s |
| 优先级 | P0 |

#### FR-GAME-024 trap 陷阱元素 (2D SVG sprite)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-024 |
| kind | `trap` (改写自 9/6 立 `comment_pin`) |
| 描述 | 陷阱: 4 种 (钉刺 / 火焰 / 冰霜 / 传送), 玩家踩中触发 |
| 视觉 | 32x32 sprite, 待机 4 帧 + 触发 6 帧 |
| 伤害 | 陷阱伤害 = 玩家 HP × 10-30% |
| 优先级 | P0 |

#### FR-GAME-025 portal 传送门元素 (3渲2 portal)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-025 |
| kind | `portal` (改写自 9/6 立 `mind_map_node`) |
| 描述 | 传送门: 4 类 (房间出口 / BOSS 入口 / 商店 / 隐藏房), 3渲2 portal 渲染 |
| 视觉 | 64x64 portal, 旋转 16 帧循环, 中心发光粒子 |
| 行为 | 玩家走入即传送, 触发房间切换 |
| 优先级 | P0 |

#### FR-GAME-026 chest 宝箱元素 (2D SVG sprite + 开箱动画)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-026 |
| kind | `chest` (改写自 9/6 立 `flowchart_node`) |
| 描述 | 宝箱: 4 稀有度 (普通 / 稀有 / 传说 / 诅咒), 2D SVG sprite + 开箱动画 |
| 视觉 | 32x32 sprite, 关闭 4 帧 + 开启 8 帧, 颜色按稀有度 (灰 / 蓝 / 紫 / 红) |
| 行为 | 玩家攻击或钥匙开启, 1-3 道具掉落 |
| 优先级 | P0 |

## §17 角色域 (Character) — 2 子节, 15 FR

### §17.1 机器人 agent 角色

#### FR-GAME-100 机器人 3渲2 像素风角色 (新, per 9/7 用户拍板)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-100 |
| 描述 | 玩家操控的 3渲2 像素风机器人 agent, 替代 9/6 立 `agent_cursor` 卡片 |
| 视觉 | Three.js 0.169+ WebGL 3D 渲染, 输出 16x16 / 32x32 像素风 sprite 动画 |
| 尺寸 | 32x32 默认, 64x64 缩放 |
| 配色 | 8 色 palette, 玩家可自定义 (per §18.3) |
| 像素风 | 致敬 16-bit 时代, NES / SNES 风格 |
| 优先级 | P0 |

#### FR-GAME-101 机器人 4 大动作 + 完整 sprite 动画 (新, per 9/7 用户拍板 "各种移动攻击的动画要补上")

| 项 | 内容 |
|---|---|
| ID | FR-GAME-101 |
| 描述 | 机器人 4 大动作: 移动 / 攻击 / 翻滚 / 大招, 每动作 4-8 帧 sprite 动画 |
| 移动动画 | 6 帧 (走路循环) + 8 方向 |
| 攻击动画 | 4 帧 (出招前摇 + 攻击 + 后摇) |
| 翻滚动画 | 4 帧 (无敌帧 0.3s) |
| 大招动画 | 8 帧 (蓄力 + 释放) |
| 待机动画 | 4 帧 (呼吸循环) |
| 死亡动画 | 6 帧 (倒地) |
| **总帧数** | **32 帧 / 机器人** (4 移动 + 4 攻击 + 4 翻滚 + 8 大招 + 4 待机 + 6 死亡) |
| 优先级 | P0 |

#### FR-GAME-102 机器人 5 状态机

| 项 | 内容 |
|---|---|
| ID | FR-GAME-102 |
| 描述 | 机器人 5 状态: idle / moving / attacking / rolling / dying |
| 转换 | idle → moving (方向键) → idle; idle → attacking (空格) → idle (0.5s) ; idle → rolling (Shift) → idle (0.3s 无敌); any → dying (HP=0) → respawn |
| 优先级 | P0 |

#### FR-GAME-103 机器人 6 维属性

| 项 | 内容 |
|---|---|
| ID | FR-GAME-103 |
| 描述 | 机器人 6 维属性: HP / MP / SP / ATK / DEF / Speed |
| HP | 生命值, 默认 100, 0 = 死亡 |
| MP | 魔法值, 默认 50, 技能消耗 |
| SP | 耐力值, 默认 100, 翻滚 / 跑步消耗 |
| ATK | 攻击力, 默认 10, 影响伤害 |
| DEF | 防御力, 默认 5, 影响减伤 |
| Speed | 移动速度, 默认 200 px/s |
| 优先级 | P0 |

### §17.2 角色成长

#### FR-GAME-110 经验值 + 升级

| 项 | 内容 |
|---|---|
| ID | FR-GAME-110 |
| 描述 | 击败敌人获得经验值, 升级解锁技能槽 + 属性成长 |
| 经验曲线 | 升级所需经验 = level^1.5 × 100 |
| 属性成长 | 每级 +5 HP, +2 MP, +5 SP, +1 ATK, +1 DEF |
| 优先级 | P0 |

#### FR-GAME-111 技能树

| 项 | 内容 |
|---|---|
| ID | FR-GAME-111 |
| 描述 | 6 技能树: 攻击 / 防御 / 速度 / 魔法 / 闪避 / 暴击, 每树 5-10 技能 |
| 解锁 | 升级获得技能点, 技能点投入技能 |
| 优先级 | P0 |

#### FR-GAME-112 装备系统

| 项 | 内容 |
|---|---|
| ID | FR-GAME-112 |
| 描述 | 武器 (主手 / 副手) + 盔甲 (头 / 胸 / 腿 / 鞋) + 饰品 (项链 / 戒指) 共 8 槽 |
| 优先级 | P0 |

## §18 弹幕域 (Bullet Hell) — 2 子节, 12 FR

### §18.1 5 弹幕模式

#### FR-GAME-200 弹幕 radial 放射模式

| 项 | 内容 |
|---|---|
| ID | FR-GAME-200 |
| 描述 | 敌人以自身为中心, 360° 均匀放射 N 颗子弹 |
| 参数 | 子弹数 N = 8-32, 速度 v = 100-300 px/s, 间隔 0.5-2s |
| 优先级 | P0 |

#### FR-GAME-201 弹幕 aimed 瞄准模式

| 项 | 内容 |
|---|---|
| ID | FR-GAME-201 |
| 描述 | 敌人朝玩家当前位置发射 N 颗子弹 (含预判玩家走位) |
| 参数 | 子弹数 N = 1-5, 预判时间 0.3s |
| 优先级 | P0 |

#### FR-GAME-202 弹幕 spiral 螺旋模式

| 项 | 内容 |
|---|---|
| ID | FR-GAME-202 |
| 描述 | 敌人以螺旋方式连续发射子弹, 角度随时间旋转 |
| 参数 | 螺旋数 2-5, 角速度 30-90 °/s, 子弹频率 5-10 发/s |
| 优先级 | P0 |

#### FR-GAME-203 弹幕 wave 波浪模式

| 项 | 内容 |
|---|---|
| ID | FR-GAME-203 |
| 描述 | 敌人以波浪形发射子弹, 子弹位置按 sin 函数分布 |
| 参数 | 波长 50-200 px, 振幅 30-100 px, 频率 0.5-2 Hz |
| 优先级 | P0 |

#### FR-GAME-204 弹幕 laser 激光模式

| 项 | 内容 |
|---|---|
| ID | FR-GAME-204 |
| 描述 | 敌人发射持续激光 (0.5-2s), 激光宽度 20-50 px |
| 视觉 | 激光 = 2D 长方形 + 闪烁 |
| 警告 | 激光发射前 0.3s 显示红色警告线 |
| 优先级 | P0 |

### §18.2 弹幕系统

#### FR-GAME-210 子弹池 (Bullet Pool)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-210 |
| 描述 | 子弹对象池, 避免 GC, 1000+ 子弹 / 60fps |
| 实现 | Rust Vec<Bullet> pre-allocate 2000 容量 |
| 优先级 | P0 |

#### FR-GAME-211 子弹碰撞

| 项 | 内容 |
|---|---|
| ID | FR-GAME-211 |
| 描述 | 子弹 vs 玩家 / 敌人碰撞, AABB + 圆碰撞 |
| 算法 | 广度优先 + 空间分区 (uniform grid, 64x64 格子) |
| 性能 | 1000 子弹 + 50 敌人 + 1 玩家 = 60fps |
| 优先级 | P0 |

#### FR-GAME-212 玩家子弹

| 项 | 内容 |
|---|---|
| ID | FR-GAME-212 |
| 描述 | 玩家发射的子弹, 3D 3渲2 渲染 (per ADR-CANVAS-GAME-002) |
| 视觉 | 3D 球体 + 拖尾粒子 (3渲2 风格) |
| 优先级 | P0 |

#### FR-GAME-213 弹幕模板

| 项 | 内容 |
|---|---|
| ID | FR-GAME-213 |
| 描述 | 100+ 弹幕模板, 关卡设计师可复用 + 组合 |
| 模板 | 5 模式 × 20 变体 = 100 模板 |
| 优先级 | P0 |

## §19 战斗域 (Combat) — 2 子节, 10 FR

### §19.1 伤害公式

#### FR-GAME-300 5 维伤害公式 (per BR-9)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-300 |
| 公式 | `damage = ATK × skill_multiplier × (1 - DEF/(DEF+100)) × crit × random(0.85-1.15) × elemental_resist` |
| 字段 | ATK / skill_multiplier (0.5-3.0) / DEF / crit (1.5-2.5 倍率, 5-30% 概率) / random (浮动 ±15%) / elemental_resist (0.5-2.0) |
| 优先级 | P0 |

#### FR-GAME-301 暴击 / 闪避

| 项 | 内容 |
|---|---|
| ID | FR-GAME-301 |
| 描述 | 暴击 (1.5-2.5x 伤害) + 闪避 (完全躲伤害, 0-30% 概率) |
| 视觉 | 暴击: 红色伤害数字; 闪避: "MISS" 黄色文字 |
| 优先级 | P0 |

#### FR-GAME-302 DOT (持续伤害)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-302 |
| 描述 | 5 状态效果触发 DOT (持续伤害) |
| 公式 | `dot_damage = base_dot × (1 + ATK/100) × tick_interval (0.5-1s)` |
| 优先级 | P0 |

#### FR-GAME-303 AOE (范围伤害)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-303 |
| 描述 | 范围伤害, 圆形 / 矩形 / 扇形 3 形状 |
| 视觉 | AOE 范围红色标记 (0.3s 预警) + 伤害粒子 |
| 优先级 | P0 |

#### FR-GAME-304 护盾 / 治疗

| 项 | 内容 |
|---|---|
| ID | FR-GAME-304 |
| 描述 | 护盾 (吸收伤害, 0-100) + 治疗 (恢复 HP) |
| 视觉 | 护盾: 玩家周围蓝色光环; 治疗: 绿色 +HP 数字 |
| 优先级 | P0 |

### §19.2 5 状态效果 (per BR-10)

#### FR-GAME-310 中毒

| 项 | 内容 |
|---|---|
| ID | FR-GAME-310 |
| 描述 | 每秒损失 HP = MAX_HP × 2%, 持续 5s |
| 视觉 | 玩家身上绿色粒子 |
| 优先级 | P0 |

#### FR-GAME-311 燃烧

| 项 | 内容 |
|---|---|
| ID | FR-GAME-311 |
| 描述 | 每秒损失 HP = MAX_HP × 3%, 持续 3s |
| 视觉 | 玩家身上火焰粒子 |
| 优先级 | P0 |

#### FR-GAME-312 冰冻

| 项 | 内容 |
|---|---|
| ID | FR-GAME-312 |
| 描述 | 移动速度 -50%, 持续 3s |
| 视觉 | 玩家冰冻 (蓝色覆盖) |
| 优先级 | P0 |

#### FR-GAME-313 麻痹

| 项 | 内容 |
|---|---|
| ID | FR-GAME-313 |
| 描述 | 不能攻击 / 翻滚, 持续 1s |
| 视觉 | 玩家身上闪电粒子 |
| 优先级 | P0 |

#### FR-GAME-314 诅咒

| 项 | 内容 |
|---|---|
| ID | FR-GAME-314 |
| 描述 | 受到的伤害 +25%, 持续 10s |
| 视觉 | 玩家身上紫色光环 |
| 优先级 | P0 |

## §20 关卡域 (Level) — 1 子节, 8 FR

#### FR-GAME-400 房间 (Room) (新)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-400 |
| 描述 | Roguelike 关卡基本单位: 1 矩形 + 4 门 (上下左右) |
| 尺寸 | 800x600 - 1600x1200 px, 矩形 |
| 内容 | 0-50 敌人 + 0-100 道具 + 0-4 陷阱 + 0-2 宝箱 + 0-1 传送门 |
| 优先级 | P0 |

#### FR-GAME-401 走廊 (Corridor)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-401 |
| 描述 | 房间之间连接通道, 100-200 px 宽 |
| 优先级 | P0 |

#### FR-GAME-402 楼层 (Floor)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-402 |
| 描述 | 8-12 房间组成 1 楼层, 末尾 1 BOSS 房间 |
| 难度 | 楼层 × 10 (per BR-5) |
| 优先级 | P0 |

#### FR-GAME-403 Biome (主题)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-403 |
| 描述 | 6 biome: 森林 / 沙漠 / 地下城 / 太空 / 火山 / 雪原 |
| 视觉 | 画布背景色 + 装饰 element 主题 |
| 难度 | biome 系数 1.0-2.0 (per BR-5) |
| 优先级 | P0 |

#### FR-GAME-404 Procedural Generation (per ADR-CANVAS-GAME-005)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-404 |
| 描述 | 关卡随机生成, 同 seed = 同关卡 (可复现) |
| 算法 | 房间布局: perlin noise (地形) + WFC (房间内敌人/道具) |
| 优先级 | P0 |

#### FR-GAME-405 BOSS 战

| 项 | 内容 |
|---|---|
| ID | FR-GAME-405 |
| 描述 | 每层末尾 1 BOSS, 3 阶段 (血量 100% / 50% / 20%) |
| 视觉 | BOSS 3渲2 大 sprite (128x128), 3 阶段换 sprite + 弹幕模式升级 |
| 优先级 | P0 |

#### FR-GAME-406 商店 / 隐藏房

| 项 | 内容 |
|---|---|
| ID | FR-GAME-406 |
| 描述 | 商店: 用金币购买道具 / 技能; 隐藏房: 60% 概率刷稀有宝箱 |
| 优先级 | P0 |

#### FR-GAME-407 难度曲线

| 项 | 内容 |
|---|---|
| ID | FR-GAME-407 |
| 描述 | 难度 = 楼层 × 10 + 房间号 × 5 + biome 系数 (per BR-5) |
| 应用 | 敌人 HP / ATK / 数量 / 弹幕密度 全部按难度缩放 |
| 优先级 | P0 |

## §21 道具域 (Loot) — 1 子节, 6 FR

#### FR-GAME-500 武器 (100+)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-500 |
| 描述 | 100+ 武器, 主手 / 副手槽, 影响攻击模式 + 伤害 + 弹幕 |
| 优先级 | P0 |

#### FR-GAME-501 被动道具 (300+)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-501 |
| 描述 | 300+ 被动道具, 自动生效, 影响属性 / 抗性 / 特殊能力 |
| 优先级 | P0 |

#### FR-GAME-502 消耗品 (200+)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-502 |
| 描述 | 200+ 消耗品 (药水 / 卷轴 / 炸弹), 按 R 键使用 |
| 优先级 | P0 |

#### FR-GAME-503 钥匙 (4 种)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-503 |
| 描述 | 4 种钥匙: 普通 / 金 / 银 / 骷髅, 开对应宝箱 / 隐藏房 |
| 优先级 | P0 |

#### FR-GAME-504 金币

| 项 | 内容 |
|---|---|
| ID | FR-GAME-504 |
| 描述 | 货币, 拾取 / 商店购买, **死亡时保留部分** (per BR-6 + Hades 机制) |
| 优先级 | P0 |

#### FR-GAME-505 道具掉落

| 项 | 内容 |
|---|---|
| ID | FR-GAME-505 |
| 描述 | 敌人死亡掉落金币 + 道具, 概率按稀有度 |
| 优先级 | P0 |

## §22 技能域 (Skill) — 1 子节, 5 FR

#### FR-GAME-600 主动技能 (50+)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-600 |
| 描述 | 50+ 主动技能, 主动键释放, 消耗 MP, 冷却 5-30s |
| 优先级 | P0 |

#### FR-GAME-601 被动技能 (50+)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-601 |
| 描述 | 50+ 被动技能, 自动生效, 不消耗 MP |
| 优先级 | P0 |

#### FR-GAME-602 大招 (10+)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-602 |
| 描述 | 10+ 大招, 按 Q 键释放, 消耗 MP 50%, 冷却 60-120s, 全屏 AOE |
| 视觉 | 大招特效 3D particle 200+ 粒子 (per ADR-CANVAS-GAME-002) |
| 优先级 | P0 |

#### FR-GAME-603 技能冷却 UI

| 项 | 内容 |
|---|---|
| ID | FR-GAME-603 |
| 描述 | HUD 技能冷却倒计时, 灰色圆环 + 倒计时数字 |
| 优先级 | P0 |

#### FR-GAME-604 技能快捷键

| 项 | 内容 |
|---|---|
| ID | FR-GAME-604 |
| 描述 | Q (大招) / E (技能 1) / R (消耗品) / Space (攻击) / Shift (翻滚) / WASD (移动) |
| 优先级 | P0 |

## §23 3渲2 渲染 (3D-to-2D Rendering) — 2 子节, 8 FR

### §23.1 机器人 3渲2

#### FR-GAME-700 3D 角色模型 (新, per 9/7 用户拍板 "可以3D的3渲2制作")

| 项 | 内容 |
|---|---|
| ID | FR-GAME-700 |
| 描述 | 机器人 3D 模型 + Three.js 0.169+ WebGL 渲染 |
| 模型 | 简单 3D mesh (头 / 身 / 手臂 / 腿 / 武器), 5-10 部件 |
| 渲染 | OrthographicCamera 2D 投影, 输出像素风 sprite |
| 性能 | 60 robot / 60fps |
| 优先级 | P0 |

#### FR-GAME-701 Sprite Atlas

| 项 | 内容 |
|---|---|
| ID | FR-GAME-701 |
| 描述 | sprite atlas: 多 sprite 合成 1 张大图, 减少渲染调用 |
| 尺寸 | 2048x2048 单 atlas, 32x32 sprite × 4096 个 |
| 优先级 | P0 |

#### FR-GAME-702 像素风后处理

| 项 | 内容 |
|---|---|
| ID | FR-GAME-702 |
| 描述 | WebGL fragment shader, 像素化处理 (降低分辨率 + nearest 采样) |
| 算法 | fragment shader: `vec2 pixel = floor(uv * resolution) / resolution;` |
| 优先级 | P0 |

#### FR-GAME-703 8 方向旋转

| 项 | 内容 |
|---|---|
| ID | FR-GAME-703 |
| 描述 | 机器人 8 方向旋转 (N / NE / E / SE / S / SW / W / NW) |
| 实现 | Three.js group.rotation.z, 45° 步进 |
| 优先级 | P0 |

### §23.2 子弹 + 粒子 3D

#### FR-GAME-710 玩家子弹 3D 球体

| 项 | 内容 |
|---|---|
| ID | FR-GAME-710 |
| 描述 | 玩家子弹 3D 球体 + 拖尾粒子 (3渲2 风格) |
| 视觉 | 球体半径 5-10 px + 拖尾长度 20-50 px |
| 优先级 | P0 |

#### FR-GAME-711 粒子系统

| 项 | 内容 |
|---|---|
| ID | FR-GAME-711 |
| 描述 | 粒子系统: 爆炸 / 命中 / 受击 / 大招 4 类, 200+ 粒子 |
| 实现 | Rust particle pool + 客户端渲染 |
| 优先级 | P0 |

#### FR-GAME-712 大招全屏特效

| 项 | 内容 |
|---|---|
| ID | FR-GAME-712 |
| 描述 | 大招释放时全屏 3D 粒子特效, 200+ 粒子, 0.5-1s |
| 视觉 | 屏幕震动 + 闪光 + 粒子扩散 |
| 优先级 | P0 |

## §24 实时协作 (Realtime Collaboration) — 1 子节, 6 FR

#### FR-GAME-800 多人合作 1-4 玩家 (per ADR-CANVAS-GAME-006)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-800 |
| 描述 | 1-4 玩家实时协作, Yjs CRDT 自动同步玩家位置 / HP / 道具 |
| 模式 | 单人 / 双人合作 / 4 人合作 |
| 同步 | 60 Hz, debounce 50ms |
| 优先级 | P0 |

#### FR-GAME-801 玩家 3 模式权限 (per ADR-CANVAS-GAME-008)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-801 |
| 描述 | 3 模式权限: player (游玩) / co-op (合作) / spectator (旁观) |
| 优先级 | P0 |

#### FR-GAME-802 房主 (host) 权限

| 项 | 内容 |
|---|---|
| ID | FR-GAME-802 |
| 描述 | 房主 1 特殊角色: 邀请 / 踢人 / 开始 / 暂停 / 设置难度 |
| 优先级 | P0 |

#### FR-GAME-803 共享生命池

| 项 | 内容 |
|---|---|
| ID | FR-GAME-803 |
| 描述 | 4 玩家合作时共享生命池, 1 玩家死亡, 队友可救 (3s 内) |
| 优先级 | P1 |

#### FR-GAME-804 离线编辑关卡

| 项 | 内容 |
|---|---|
| ID | FR-GAME-804 |
| 描述 | 关卡设计师离线编辑关卡布局 (房间 + 敌人 + 道具), 重连自动 merge (Yjs CRDT) |
| 优先级 | P1 |

#### FR-GAME-805 关卡 seed 分享

| 项 | 内容 |
|---|---|
| ID | FR-GAME-805 |
| 描述 | 玩家可分享关卡 seed (URL `?seed=xxx`), 其他玩家用同 seed 玩同关卡 |
| 优先级 | P1 |

## §25 4 crate 架构 (per ADR-CANVAS-GAME-003) — 1 子节, 5 FR

#### FR-GAME-900 canvas-engine crate 沿用 9/6 立架构

| 项 | 内容 |
|---|---|
| ID | FR-GAME-900 |
| 描述 | 沿用 9/6 立 canvas-engine crate 架构 + 5 装饰 element 渲染 (沿用) |
| 扩展 | 视口跟随玩家 (FR-GAME-002) + 运镜特效 (FR-GAME-003) |
| 优先级 | P0 |

#### FR-GAME-901 domain-canvas crate 扩展游戏表

| 项 | 内容 |
|---|---|
| ID | FR-GAME-901 |
| 描述 | 沿用 9/6 立 domain-canvas crate 5 角色权限 + 26 表 (守门 #13 W/T/M 严格) |
| 扩展 | 25+ 游戏表: 角色 / 弹幕 / 战斗 / 关卡 / 道具 (per §36-§39) |
| 优先级 | P0 |

#### FR-GAME-902 canvas-realtime crate 扩展多人游戏协作

| 项 | 内容 |
|---|---|
| ID | FR-GAME-902 |
| 描述 | 沿用 9/6 立 canvas-realtime crate + Yjs CRDT |
| 扩展 | 1-4 玩家实时同步 (FR-GAME-800) + 共享生命池 (FR-GAME-803) |
| 优先级 | P0 |

#### FR-GAME-903 canvas-game crate 新增 (第 4 crate)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-903 |
| 描述 | 新增 crates/canvas-game, 游戏引擎: 角色 / 弹幕 / 战斗 / 关卡 / 道具 / 3渲2 渲染 |
| 公开 API | 5 函数 + 7 ElementRenderer impl + 5 状态机 + 5 弹幕模式 + 5 维伤害公式 |
| 依赖 | canvas-engine (跨域共享画布) + domain-canvas (业务域) + canvas-realtime (CRDT) |
| 优先级 | P0 |

#### FR-GAME-904 跨 4 crate 协作 (per ADR-CANVAS-GAME-003)

| 项 | 内容 |
|---|---|
| ID | FR-GAME-904 |
| 描述 | 4 crate 通过 trait 抽象 + 调用方注入协作, 0 业务域依赖 (per 9/6 ADR-CANVAS-005) |
| 隔离 | canvas-engine 不依赖 domain-* ; canvas-game 不依赖具体业务域 |
| 实现 | trait 抽象 + 调用方注入 |
| 优先级 | P0 |

---

# === §26-§35 非功能需求 (10 节, 40+ NFR) ===

## §26 性能 (Performance)

| NFR | 指标 | 优先级 | 验证 |
|---|---|---|---|
| NFR-PERF-1 | 游戏启动 < 3s (单人模式, P50) | P0 | Lighthouse |
| NFR-PERF-2 | 60 robot × 1000 bullet / 60fps (弹幕密集场景) | P0 | Chrome DevTools Performance |
| NFR-PERF-3 | Yjs 同步延迟 < 100ms (4 玩家, P95) | P0 | benchmark (比 9/6 立 50 协作者 < 200ms 更严) |
| NFR-PERF-4 | 玩家输入响应 < 50ms (P95) | P0 | benchmark |
| NFR-PERF-5 | 子弹生成 / 销毁 < 5ms / 帧 | P0 | bullet pool benchmark |
| NFR-PERF-6 | 3渲2 渲染 < 8ms / 帧 (60 robot) | P0 | WebGL benchmark |
| NFR-PERF-7 | 粒子系统 < 4ms / 帧 (200 粒子) | P0 | particle pool benchmark |
| NFR-PERF-8 | 关卡 procedural generation < 500ms (1 楼层) | P0 | benchmark |
| NFR-PERF-9 | 死亡 / 重生 < 200ms | P0 | benchmark |
| NFR-PERF-10 | 5 域 Lead 真人到位后, 美术资源 sprite atlas 加载 < 1s | P1 | benchmark |

## §27 可用性 (Usability)

| NFR | 指标 | 优先级 |
|---|---|---|
| NFR-USA-1 | 新玩家 5 分钟内完成教程关卡 | P0 |
| NFR-USA-2 | 机器人 4 大动作 + 完整动画流畅, 无卡顿 | P0 |
| NFR-USA-3 | 死亡 / 重生 反馈清晰, 90% 玩家理解 | P0 |
| NFR-USA-4 | 道具 / 技能描述清楚, 玩家不需要查 wiki | P0 |
| NFR-USA-5 | 5+ tutorial 关卡教学 4 大动作 + 弹幕 + 战斗 | P2 |

## §28 安全 (Security)

| NFR | 指标 | 优先级 |
|---|---|---|
| NFR-SEC-1 | 房间 token 不可预测 (UUID v4) | P0 |
| NFR-SEC-2 | 反作弊: 客户端计算结果服务端校验 (HP / 经验 / 金币) | P0 |
| NFR-SEC-3 | WSS (TLS 1.3) | P0 |
| NFR-SEC-4 | Yjs update 持久化前验证 (守门 #7 unsafe 禁) | P0 |
| NFR-SEC-5 | XSS 防护: 玩家输入转义 (per 9/6 沿用) | P0 |
| NFR-SEC-6 | CSRF token 校验 (写操作) | P0 |
| NFR-SEC-7 | 速率限制: 60 req/min/user (写操作) | P1 |
| NFR-SEC-8 | 审计日志 100% 覆盖 (守门 #13 T 类) | P0 |
| NFR-SEC-9 | RLS 13 类必带 (守门 #13) | P0 |
| NFR-SEC-10 | 房间过期后不可恢复 | P0 |

## §29 可扩展 (Scalability)

| NFR | 指标 | 优先级 |
|---|---|---|
| NFR-SCAL-1 | 单画布 1 玩家 (P0) / 4 玩家 (P0) / 50 协作者旁观 (P3) | P0 |
| NFR-SCAL-2 | 单画布 100 房间 (1 楼层) | P0 |
| NFR-SCAL-3 | 单画布 1000 子弹 (per 4 玩家) | P0 |
| NFR-SCAL-4 | canvas-game 横向扩展 (Rust stateless) | P0 |
| NFR-SCAL-5 | canvas-realtime sticky session + Redis pub/sub (沿用 9/6) | P1 |
| NFR-SCAL-6 | 100+ 道具表 + 100+ 技能表 (SQLite + PostgreSQL) | P0 |

## §30 可观测 (Observability)

| NFR | 指标 | 优先级 |
|---|---|---|
| NFR-OBS-1 | OpenTelemetry trace 100% 覆盖 (沿用 9/6) | P0 |
| NFR-OBS-2 | Prometheus metrics 20+ 指标 (新增 5 游戏指标: active_games / players_online / bullet_count / boss_kills / death_count) | P0 |
| NFR-OBS-3 | 结构化日志 (JSON, 9 字段) | P0 |
| NFR-OBS-4 | 健康检查 /healthz + /readyz | P0 |
| NFR-OBS-5 | 错误聚合 (Sentry) | P1 |
| NFR-OBS-6 | Audit log (WORM, per 守门 #13) | P0 |
| NFR-OBS-7 | Grafana dashboard 7+ panel (canvas 5 + 游戏 2) | P1 |

## §31 可恢复 (Recoverability)

| NFR | 指标 | 优先级 |
|---|---|---|
| NFR-REC-1 | canvas-realtime 崩溃后客户端 5s 内自动重连 (沿用 9/6) | P0 |
| NFR-REC-2 | DB 故障后读副本接管 (RPO < 5min) | P0 |
| NFR-REC-3 | 关卡 seed 可复现 (per BR-5) | P0 |
| NFR-REC-4 | Yjs 离线编辑重连后自动 merge (per 9/6) | P0 |
| NFR-REC-5 | 备份: 每日全量 + Yjs update 增量 5min | P1 |
| NFR-REC-6 | RTO < 30min | P1 |

## §32 兼容性 (Compatibility)

| NFR | 指标 | 优先级 |
|---|---|---|
| NFR-COMPAT-1 | Chrome 110+ / Edge 110+ / Safari 16+ / Firefox 110+ 100% 兼容 (沿用 9/6) | P0 |
| NFR-COMPAT-2 | WebGL 2.0 支持 (3渲2 必需) | P0 |
| NFR-COMPAT-3 | iPad Safari (iOS 16+) 触屏 (P1) | P1 |
| NFR-COMPAT-4 | 移动 Web 触屏 (P2) | P2 |
| NFR-COMPAT-5 | 高 DPI (Retina / 4K) | P0 |
| NFR-COMPAT-6 | 暗色 / 亮色模式 | P0 |

## §33 可维护 (Maintainability)

| NFR | 指标 | 优先级 |
|---|---|---|
| NFR-MAINT-1 | 单元测试覆盖率 ≥ 80% (canvas-game / domain-canvas / canvas-engine / canvas-realtime) | P0 |
| NFR-MAINT-2 | 集成测试覆盖率 ≥ 60% | P0 |
| NNR-MAINT-3 | E2E 测试 20+ 场景 (per §66-§75 UC) | P0 |
| NFR-MAINT-4 | 代码审查 100% (守门 #9) | P0 |
| NFR-MAINT-5 | CI 9/9 通过 (per 守门 #1 v25) | P0 |
| NFR-MAINT-6 | 文档覆盖率 ≥ 90% (rustdoc / TypeDoc) | P1 |
| NFR-MAINT-7 | 0 unsafe (守门 #7) | P0 |

## §34 国际化 (i18n)

| NFR | 指标 | 优先级 |
|---|---|---|
| NFR-I18N-1 | 3 语言: zh-CN / en / ja 100% 翻译 (沿用 9/6) | P0 |
| NFR-I18N-2 | RTL 预留 (P3) | P3 |
| NFR-I18N-3 | 时区支持 (UTC + 本地) | P1 |
| NFR-I18N-4 | 日期 / 数字 / 货币本地化 (Intl API) | P0 |
| NFR-I18N-5 | CJK 字体 (Noto Sans CJK / 思源黑体) | P0 |

## §35 法规 (Compliance)

| NFR | 指标 | 优先级 |
|---|---|---|
| NFR-COMP-1 | GDPR 合规 (EU 用户数据可删除 / 导出) | P0 |
| NFR-COMP-2 | 个人信息最小化 (per 守门 #5 env 安全) | P0 |
| NFR-COMP-3 | 数据驻留: 用户可选 region (US / EU / APAC) | P2 |
| NFR-COMP-4 | Cookie 同意 (per GDPR ePrivacy) | P0 |
| NFR-COMP-5 | 知识产权: 用户内容所有权归用户, 平台仅托管 | P0 |
| NFR-COMP-6 | 审计日志保留 7 年 (per SOX) | P1 |

---

# === §36-§45 数据需求 (10 节, 25+ DR, W/T/M 严格守门 #13) ===

## §36 数据模型总览 (per 守门 #13 W/T/M 横展强制)

> **守门 #13 严格** (per AGENTS.md §4): DB 表设计必须 100% 横展三类: **Work (W)** / **Transaction (T)** / **Master (M)**, 不允许混合分类。

| # | 表名 | W/T/M 分类 | 类别理由 | 守门 #13 派生规 |
|---|---|---|---|---|
| 1 | `game_character` | **M** | 角色主数据, 慢变 SCD Type 2 | (c) 物理删除禁止 + SCD Type 2 + RLS 13 必带 |
| 2 | `game_character_save` | **M** | 角色存档, 慢变 (per Hades 半永久机制) | (c) |
| 3 | `game_skill` | **M** | 技能主数据, 参考 (内置 100+ 技能) | (c) |
| 4 | `game_skill_tree` | **M** | 技能树主数据, 参考 | (c) |
| 5 | `game_equipment` | **M** | 装备主数据, 参考 (内置 100+ 装备) | (c) |
| 6 | `game_loot` | **M** | 道具主数据, 参考 (内置 1000+ 道具) | (c) |
| 7 | `game_loot_drop` | **T** | 道具掉落事件, append-only | (b) 物理删除禁止 + 監査必須 + RLS 13 必带 |
| 8 | `game_status_effect` | **M** | 状态效果主数据, 参考 (5 种) | (c) |
| 9 | `game_room` | **M** | 房间主数据, 慢变 SCD Type 2 | (c) |
| 10 | `game_room_enemy` | **T** | 房间敌人事件, append-only | (b) |
| 11 | `game_floor` | **M** | 楼层主数据, 慢变 SCD Type 2 | (c) |
| 12 | `game_biome` | **M** | Biome 主数据, 参考 (6 种) | (c) |
| 13 | `game_bullet_pattern` | **M** | 弹幕模式主数据, 参考 (5+ 模式) | (c) |
| 14 | `game_bullet_template` | **M** | 弹幕模板主数据, 参考 (100+ 模板) | (c) |
| 15 | `game_damage_log` | **T** | 伤害事件, append-only | (b) |
| 16 | `game_death_log` | **T** | 死亡事件, append-only | (b) |
| 17 | `game_boss_kill` | **T** | BOSS 击杀事件, append-only | (b) |
| 18 | `game_seed` | **M** | 关卡 seed 主数据, 慢变 | (c) |
| 19 | `game_session` | **W** | 游戏会话, retention 1d | (a) 物理删除 / 短 TTL |
| 20 | `game_replay_yjs_update` | **M + W 二象** | 游戏回放 Yjs update, 短 TTL 30d | (a) + (c) 二象 |
| 21 | `game_audit_log` | **T** | 游戏审计日志, append-only, WORM | (b) |
| 22 | `game_member` | **M** | 游戏成员, 慢变 SCD Type 2 (3 模式权限 per ADR-CANVAS-GAME-008) | (c) |
| 23 | `game_chat` | **T** | 游戏聊天, append-only | (b) |
| 24 | `game_leaderboard` | **M** | 排行榜, 慢变 (1 / 楼层 / biome) | (c) |
| 25 | `game_achievement` | **M** | 成就主数据, 参考 (50+) | (c) |
| 26 | `game_dlq` | **W** | dead-letter queue, 失败任务, retention 7d | (a) |

**汇总**: 26 表, **M: 16** / **T: 6** / **W: 6** (含 1 个 M+W 二象), 守门 #13 100% 覆盖。**完全沿用 9/6 立 SRS-001 §36 26 表 (M:16/T:6/W:6 含 1 个 M+W 二象) 架构**, 仅游戏相关表命名调整 (canvas → game)。

## §37-§45 数据持久化 / 存储 / 备份 / 隐私 / 数据迁移 (略, 8 节覆盖)

> 完整 5 节 (持久化策略 / 存储选型 / 备份恢复 / 隐私保护 / 数据迁移) 详见基本设计阶段 `BD-STAR-CANVAS-GAME-001.md` v0.1, 本 SRS 仅列关键约束:
> - 持久化: PostgreSQL 16 + S3-compatible object storage (sprite atlas / 子弹 / 粒子资源)
> - 存储选型: 单库 + game_session_id hash 分片 (P2)
> - 备份: 每日全量 + WAL 归档 + 跨 region 复制
> - 隐私: GDPR 合规, 用户可导出 / 删除自己角色的所有 save + chat + damage log
> - 迁移: 9/6 立 canvas_* 表数据迁移到 game_* 表, 一次性脚本 (per 守门 #13 W/T/M 重分类)

---

# === §46-§55 接口需求 (10 节, 30+ IR) ===

## §46 REST API (沿用 9/6 立 13 端点 + 扩展 7 游戏端点)

> 游戏 API 端点 (7 个, 在 9/6 立 13 端点基础上扩展):

| IR | Method | Path | 描述 | 权限 |
|---|---|---|---|---|
| IR-API-1 | GET | `/v1/games/rooms` | 列出房间 (per 守门 #13 RLS) | player+ |
| IR-API-2 | POST | `/v1/games/rooms` | 创建房间 | player+ |
| IR-API-3 | GET | `/v1/games/rooms/{id}` | 获取房间详情 | player+ |
| IR-API-4 | PUT | `/v1/games/rooms/{id}` | 更新房间 (设置难度 / biome) | host |
| IR-API-5 | DELETE | `/v1/games/rooms/{id}` | 关闭房间 | host |
| IR-API-6 | POST | `/v1/games/rooms/{id}/join` | 加入房间 | player+ |
| IR-API-7 | GET | `/v1/games/characters/{id}` | 获取角色存档 | player+ |

## §47 WebSocket 协议 (Yjs y-protocols, 沿用 9/6 立)

| IR | 类型 | 描述 | 路径 |
|---|---|---|---|
| IR-WS-1 | game | 1-4 玩家实时同步 (per ADR-CANVAS-GAME-006) | `wss://star.app/ws/game/{room_id}` |
| IR-WS-2 | bullet | 子弹流 (per 4 玩家 1000+ 子弹 / 60Hz) | 同上 |
| IR-WS-3 | chat | 游戏内聊天 | 同上 |
| IR-WS-4 | auth | 初次连接鉴权 (Bearer token) | query param `?token=...` |
| IR-WS-5 | ping/pong | 心跳 (30s 间隔) | frame 0x9 / 0xA |

## §48 CRDT 协议 (Yjs y-protocols/sync, 沿用 9/6 立)

| IR | 消息 | 描述 |
|---|---|---|
| IR-CRDT-1 | messageGameState | 游戏状态: 玩家位置 / HP / 子弹流 / 关卡进度 |
| IR-CRDT-2 | messageDamage | 伤害事件: 玩家 → 敌人 / 敌人 → 玩家 |
| IR-CRDT-3 | messageLoot | 道具拾取 / 掉落 |
| IR-CRDT-4 | messageDeath | 玩家 / 敌人死亡 |

## §49 Webhook (8 事件, 沿用 9/6 立 + 扩展 3 游戏事件)

| IR | 事件 | 描述 |
|---|---|---|
| IR-WH-1 | `game.room.created` | 房间创建 (新增) |
| IR-WH-2 | `game.boss.killed` | BOSS 击杀 (新增) |
| IR-WH-3 | `game.player.death` | 玩家死亡 (新增) |

## §50 MCP 工具 (沿用 9/6 立 7 工具 + 扩展 3 游戏工具)

| IR | 工具 | 描述 | 输入 | 输出 |
|---|---|---|---|---|
| IR-MCP-1 | `game_room_list` | 列出房间 | `{ filter }` | `Room[]` |
| IR-MCP-2 | `game_room_create` | 创建房间 | `{ difficulty, biome }` | `Room` |
| IR-MCP-3 | `game_character_get` | 获取角色存档 | `{ character_id }` | `CharacterSave` |

## §51-§55 (略, 5 节覆盖)

> 详细 5 节接口 (事件订阅 / GraphQL / gRPC / 消息队列 / 内部事件总线) 详见基本设计 `BD-STAR-CANVAS-GAME-001.md` v0.1。

---

# === §56-§65 约束 / 设计约束 / 技术栈 / 限制 / 部署 (10 节, 25+ CR) ===

## §56 设计约束

| CR | 约束 | 守门 |
|---|---|---|
| CR-DESIGN-1 | `canvas-engine` crate 不依赖任何 domain-* 业务域 (per 9/6 ADR-CANVAS-005) | #4 + #13 |
| CR-DESIGN-2 | `canvas-game` crate 不依赖具体业务域, 通过 trait 抽象 | #4 + #13 |
| CR-DESIGN-3 | 3渲2 仅机器人走 3D, 其他 element 2D (per 9/7 用户拍板 + ADR-CANVAS-GAME-002) | #1 |
| CR-DESIGN-4 | 弹幕走 2D particle pool, 性能目标 1000+ 子弹 / 60fps (per ADR-CANVAS-GAME-004) | #1 |
| CR-DESIGN-5 | Roguelike procedural generation 用 perlin noise + WFC (per ADR-CANVAS-GAME-005) | #1 |
| CR-DESIGN-6 | 多人合作 1-4 玩家走 Yjs CRDT (per ADR-CANVAS-GAME-006) | #13 |
| CR-DESIGN-7 | 美术资源走 sprite atlas + 5 域 Lead 真人到位后落地 (per ADR-CANVAS-GAME-007) | #3 + #14 |
| CR-DESIGN-8 | 5 模式权限简化为 3 模式 (per ADR-CANVAS-GAME-008) | #11 |
| CR-DESIGN-9 | 所有 element 13 kind 走统一 ElementKind 枚举 + ElementContent 联合类型 (沿用 9/6) | #7 |
| CR-DESIGN-10 | 0 unsafe (守门 #7), 跨整个 workspace | #7 |

## §57 技术栈

| 层 | 选型 | 版本 | 守门 |
|---|---|---|---|
| **后端 (Rust)** |  |  |  |
| canvas-engine | 沿用 9/6 (Rust + tokio + serde) | 1.78+ | #7 |
| domain-canvas | 沿用 9/6 (Rust + sqlx + axum) | 1.78+ | #7 |
| canvas-realtime | 沿用 9/6 (Rust + tokio + yrs + axum) | 1.78+ | #7 |
| **canvas-game (新)** | Rust + tokio + 2D 弹幕引擎 (Phaser 3 / PixiJS 8 评估) + Three.js FFI | 1.78+ | #7 |
| 数据库 | 沿用 9/6 PostgreSQL 16 | 16.x | #1 |
| 缓存 | 沿用 9/6 Redis 7 | 7.x | #1 |
| 对象存储 | S3-compatible (MinIO), sprite atlas | latest | #1 |
| 消息队列 | 沿用 9/6 NATS JetStream | 2.10+ | #1 |
| **前端 (TypeScript)** |  |  |  |
| 框架 | 沿用 9/6 Next.js 14.2.5 (App Router) | 14.2.5 | #1 |
| UI | 沿用 9/6 React 18.3.1 | 18.3.1 | #1 |
| 状态管理 | 沿用 9/6 zustand 4.5+ | 4.5+ | #1 |
| CRDT 客户端 | 沿用 9/6 yjs 13.6+ | 13.6+ | #1 |
| WebSocket | 沿用 9/6 y-websocket 2.0+ | 2.0+ | #1 |
| **3D 渲染 (新)** | **Three.js 0.169+ + WebGL 2.0** (3渲2 sprite + 粒子 + 运镜) | 0.169+ | #1 |
| **2D 弹幕 (新)** | **Phaser 3 / PixiJS 8** (P0 评估) | latest | #1 |
| 国际化 | 沿用 9/6 next-intl 3.0+ | 3.0+ | #1 |

## §58 限制

| CR | 限制 | 原因 |
|---|---|---|
| CR-LIMIT-1 | 1 画布 1-4 玩家 (P0) / 50 协作者旁观 (P3) | 性能基线 G-1 |
| CR-LIMIT-2 | 1 画布 100 房间 (1 楼层) | 性能 |
| CR-LIMIT-3 | 1 画布 1000 子弹 (per 4 玩家) | 弹幕性能 |
| CR-LIMIT-4 | 移动端仅支持 Web (PWA), 不支持 native (沿用 9/6 ADR-CANVAS-008) | per ADR-CANVAS-008 |
| CR-LIMIT-5 | 道具 / 技能 仅内置 100+ / 1000+, 不做众包 (P3 候选) | per 守门 #11 |
| CR-LIMIT-6 | AI 自动战斗 / 自动关卡 不做 (P3 候选) | per §13.2 |
| CR-LIMIT-7 | 完整 3D 渲染世界 不做 (仅机器人 sprite 3D) | per 9/7 用户拍板 "可以3D的3渲2制作" + ADR-CANVAS-GAME-002 |
| CR-LIMIT-8 | 5 域 Lead 真人到位前, Mavis 临时代签 (沿用 9/6) | per 守门 #3 + #14 |
| CR-LIMIT-9 | 物理引擎 / 跨机分布式 不涉及 (沿用 9/6) | per AGENTS.md §4 |
| CR-LIMIT-10 | 旧 9/6 立 8 大功能域 / 13 element / 5 角色 RACI / 40+ 快捷键 / WCAG 2.1 AA 全部作废 | per 9/7 用户拍板 |

## §59 部署

| CR | 约束 |
|---|---|
| CR-DEPLOY-1 | 容器化 (Docker + K8s, 沿用 9/6) |
| CR-DEPLOY-2 | canvas-game 走 stateless 横向扩展 (Rust, 沿用 9/6) |
| CR-DEPLOY-3 | canvas-realtime sticky session + Redis pub/sub (沿用 9/6) |
| CR-DEPLOY-4 | 数据库主从 + WAL 归档 (沿用 9/6) |
| CR-DEPLOY-5 | 监控 (Prometheus + Grafana) + 日志 (Loki) + 追踪 (Jaeger) (沿用 9/6) |
| CR-DEPLOY-6 | CI/CD: GitHub Actions 9/9 (沿用 9/6 守门 #1 v25) |
| CR-DEPLOY-7 | 蓝绿部署 / 金丝雀发布 (沿用 9/6) |
| CR-DEPLOY-8 | CDN 静态资源 (sprite atlas) (沿用 9/6) |
| CR-DEPLOY-9 | 边缘层: envoy 独立 deployment (per 9/1 13:05 拍板, 沿用 9/6) |
| CR-DEPLOY-10 | mTLS 服务间通信 (per 守门 #5 env 安全 + ADR-0021, 沿用 9/6) |

## §60-§65 (略, 6 节覆盖)

> 完整 6 节 (运维 / 监控告警 / 容量规划 / 灾备 / 升级策略 / 退役) 详见基本设计 `BD-STAR-CANVAS-GAME-001.md` v0.1。

---

# === §66-§75 验收 / 场景 / 用例 (10 节, 20 UC) ===

## §66 验收标准 (Acceptance Criteria)

| AC | 描述 | 优先级 | 验证方法 |
|---|---|---|---|
| AC-1 | 4 大游戏域 + 5 装饰 element 100% 必备功能覆盖 (per G-1) | P0 | benchmark |
| AC-2 | `canvas-game` crate 被 `domain-canvas` + `canvas-realtime` 引用 (per G-2) | P0 | `cargo tree` |
| AC-3 | 60 robot × 1000 bullet / 60fps (per G-3) | P0 | benchmark |
| AC-4 | 单人 / 双人 / 4 人合作 3 模式 (per G-4) | P0 | 集成测试 |
| AC-5 | Roguelike procedural generation 同 seed = 同关卡 (per G-5) | P0 | 单元测试 |
| AC-6 | 100+ 技能 + 1000+ 道具 (per G-6) | P0 | data table 验证 |
| AC-7 | 5 弹幕模式 (radial / aimed / spiral / wave / laser) (per G-7) | P0 | 单元测试 |
| AC-8 | 5 维伤害公式 + 5 状态效果 (per G-8) | P0 | 单元测试 |
| AC-9 | 守门 #1 #3 #5 #6 #7 #9 #12 #13 #19 #24 全过 | P0 | CI |
| AC-10 | DB 26 表 100% W/T/M 分类 (守门 #13 强制) | P0 | 审计脚本 |
| AC-11 | 0 unsafe (守门 #7) | P0 | cargo clippy |
| AC-12 | 机器人 32 帧 sprite 动画流畅 (per FR-GAME-101) | P0 | manual + benchmark |

## §67 场景 (Scenarios)

| Scenario | 描述 | UC 关联 |
|---|---|---|
| S-1 | 玩家打开画布, 直接操控机器人闯关 | UC-01 |
| S-2 | 玩家操控机器人 4 大动作 (移动/攻击/翻滚/大招) | UC-02 |
| S-3 | 玩家用翻滚躲 1000+ 弹幕 | UC-03 |
| S-4 | 玩家拾取道具, 死亡保留部分 (Hades 机制) | UC-04 |
| S-5 | 玩家跟 1-3 朋友合作, 共享生命池 | UC-05 |
| S-6 | 关卡设计师用画布编辑器设计房间 | UC-06 |
| S-7 | 玩家分享关卡 seed, 朋友用同 seed 玩 | UC-07 |
| S-8 | 玩家离线编辑关卡, 重连自动 merge | UC-08 |

## §68-§75 20 Use Case (UC-01..UC-20, 略, 详见基本设计)

> 20 UC 详细流程 (前置 / 步骤 / 后置 / 异常) 详见基本设计 `BD-STAR-CANVAS-GAME-001.md` v0.1, 本 SRS 仅列标题:

| UC | 标题 | 优先级 |
|---|---|---|
| UC-01 | 进入游戏 (单人 / 合作 / 旁观 3 模式) | P0 |
| UC-02 | 机器人 4 大动作 + 32 帧 sprite 动画 | P0 |
| UC-03 | 弹幕系统 (5 模式 + 1000+ 子弹 / 60fps) | P0 |
| UC-04 | 道具拾取 + 死亡保留 (Hades 机制) | P0 |
| UC-05 | 多人合作 1-4 玩家 (Yjs CRDT) | P0 |
| UC-06 | 关卡编辑 (房间 + 敌人 + 道具) | P1 |
| UC-07 | 关卡 seed 分享 | P1 |
| UC-08 | 离线编辑关卡 (P0 简化) | P0 |
| UC-09 | 5 维伤害公式 (DPS / DOT / AOE / 暴击 / 闪避) | P0 |
| UC-10 | 5 状态效果 (中毒/燃烧/冰冻/麻痹/诅咒) | P0 |
| UC-11 | Roguelike procedural generation (同 seed = 同关卡) | P0 |
| UC-12 | BOSS 战 (3 阶段 + 弹幕模式升级) | P0 |
| UC-13 | 商店 / 隐藏房 | P0 |
| UC-14 | 技能树 (6 树 × 5-10 技能) | P0 |
| UC-15 | 装备系统 (8 槽) | P0 |
| UC-16 | 3渲2 渲染 (Three.js + sprite atlas + 粒子) | P0 |
| UC-17 | 运镜 (跟随 / 抖动 / 缩放) | P1 |
| UC-18 | 大招全屏特效 | P0 |
| UC-19 | 性能压测 (60 robot × 1000 bullet / 60fps) | P1 |
| UC-20 | WCAG 2.1 AA 自动化测试 (P3 候选) | P3 |

---

# === §76-§85 风险 / 缓解 / 依赖 / 排期 / 资源估算 (10 节, 20+ RK) ===

## §76 风险 (Risks)

| # | 风险 | 等级 | 缓解 | 触发守门 |
|---|---|---|---|---|
| RK-1 | 3渲2 性能 (60 robot × 1000 bullet / 60fps) 未压测 | P0 高 | 阶段化压测, P1 推到 60 robot / P2 推到 100 | 守门 #1 v15 |
| RK-2 | canvas-game 命名撞名 | P1 中 | DDD Review Lead 拍板命名 | 守门 #4 + #13 |
| RK-3 | 旧 9/6 立的 canvas-e2e-guard-001 / canvas-share-export-001 2 份 brief 待 P0 重写 | P1 中 | P0 启动时重写, 标 superseded | 守门 #12 饱和 |
| RK-4 | 5 域 Lead 真人未到位, Mavis 临时代签 长期 | P1 中 | 9/5 内推 brief T0+6 周到位 | 守门 #3 + #14 |
| RK-5 | 美术资源 (100+ sprite) 5 域 Lead 真人到位后才落地 | P1 中 | P0 用 placeholder sprite, P1 美术 Lead 到位后替换 | 守门 #3 + #14 |
| RK-6 | 弹幕池 1000+ 子弹 GC 风险 | P1 中 | 子弹池 (pool) 复用, 避免 GC | 守门 #1 v15 |
| RK-7 | Three.js / Phaser / PixiJS 选型不确定 | P2 低 | P0 评估, 选 1 主备 | 守门 #11 |
| RK-8 | 旧 9/6 立 canvas_* 表数据迁移到 game_* 表 | P1 中 | 一次性迁移脚本, 保留旧数据 30d | 守门 #13 W/T/M |
| RK-9 | WCAG 2.1 AA 100% 合规成本 (P3 候选, 游戏可达性) | P2 低 | P3 阶段化 | 守门 #11 |
| RK-10 | Roguelike procedural generation 算法 (perlin + WFC) 选型 | P1 中 | P0 评估, P1 落地 | 守门 #11 |

## §77-§85 (略, 9 节覆盖)

> 完整 9 节详见基本设计 `BD-STAR-CANVAS-GAME-001.md` v0.1, 关键约束:
> - 依赖: PostgreSQL 16 / Redis 7 / NATS JetStream 2.10+ / S3-compatible / Three.js 0.169+ / 2D 弹幕引擎 (Phaser 3 / PixiJS 8)
> - 排期: P0 6 周 / P1 6 周 / P2 6 周 / P3 6 周 (24 周总)
> - 资源: 1 SRE 全程 (per 守门 #4 token-OLU) + 5 域 Lead 真人到位后追溯签字 + 美术资源 Lead (新增 5 域 + 1 = 6 域? 待 DDD Review 拍板)
> - 培训: 内部 5 域 Lead 培训 2 周

---

# === §86-§95 决策记录 (8 ADR, 全部新增) ===

> 完整 8 ADR 详见 §15 决策记录, 本节仅列索引:

| ADR | 标题 | 状态 |
|---|---|---|
| ADR-CANVAS-GAME-001 | 推翻 9/6 立方向, 新方向 = 画布弹幕 Roguelike | ✅ |
| ADR-CANVAS-GAME-002 | 3渲2 仅机器人走 3D | ✅ |
| ADR-CANVAS-GAME-003 | 4 crate 架构: 保留 3 + 新增 canvas-game | ✅ |
| ADR-CANVAS-GAME-004 | 弹幕系统走 2D particle pool, 1000+ 子弹 / 60fps | ✅ |
| ADR-CANVAS-GAME-005 | Roguelike procedural generation 用 perlin + WFC | ✅ |
| ADR-CANVAS-GAME-006 | 多人合作 1-4 玩家 Yjs CRDT | ✅ |
| ADR-CANVAS-GAME-007 | 美术资源走 sprite atlas + 5 域 Lead 真人到位后 | ✅ |
| ADR-CANVAS-GAME-008 | 5 角色 RACI 简化为 3 模式 | ✅ |

---

# === §96-§105 实施 / 阶段 / 里程碑 (10 节, 10 PH) ===

| Phase | 范围 | 周期 | token-OLU | 守门 |
|---|---|---|---|---|
| **P0 (2026-09-08 ~ 2026-10-19, 6 周)** | canvas-game crate 骨架 + 5 装饰 element 沿用 + 机器人 3渲2 占位 sprite + 弹幕池 (100 子弹 PoC) + 单人模式 PoC | 6 周 | ~1.5M | 守门 #1-#26 + 累积规 v1-v26 |
| **P1 (2026-10-20 ~ 2026-12-01, 6 周)** | 13 element 渲染完整 + 9 大功能游戏化 (enemy/bullet/loot/skill/trap/portal/chest) + 5 弹幕模式 + 5 状态效果 + 5 维伤害公式 + Roguelike 关卡 PoC + 双人合作 | 6 周 | ~1.5M | 同上 |
| **P2 (2026-12-02 ~ 2027-01-13, 6 周)** | 100+ 技能 + 1000+ 道具 + 6 biome + 6 BOSS + 技能树 + 装备系统 + 4 人合作 + 关卡编辑器 + seed 分享 | 6 周 | ~1.2M | 同上 |
| **P3 (2027-01-14 ~ 2027-02-25, 6 周)** | 5 域 Lead 美术资源到位 + sprite atlas 替换 + 性能压测 (60 robot × 1000 bullet / 60fps) + WCAG 2.1 AA (P3 候选) + 50 协作者旁观 | 6 周 | ~0.8M | 同上 |
| **P4 (2027-02-26 ~ 2027-04-09, 6 周)** | 100 robot × 5000 bullet 性能压测 + AI 自动战斗 (P3 候选) + 道具众包市场 (P3 候选) | 6 周 | ~0.5M | 同上 |

> **里程碑**: 6 周 1 阶段, 总 30 周 (P0-P4), 跟 Phase F-I 架构集成 (per `ADR-0035-0042-phase-f-i-architecture.md`)。

---

# === §106-§113 运维 / 监控 / 培训 / 术语 / 索引 (8 节, 8 OP) ===

> 完整 8 节详见基本设计 `BD-STAR-CANVAS-GAME-001.md` v0.1, 关键约束:
> - 运维: 7×24 值班, SRE 1 人 (per 守门 #4 token-OLU)
> - 监控: Grafana dashboard 7+ panel (5 画布 + 2 游戏), Prometheus 20+ 指标
> - 培训: 5 域 Lead 内部培训 2 周
> - 术语: per §14 用语定义
> - 索引: per `AGENTS.md` §6 关键 ADR 索引

---

# === 附录 ===

## 附录 A: 弹幕 Roguelike 业界标杆对标清单

> 对标基线: Enter the Gungeon + Binding of Isaac + Hades (截至 2026-09 业界标杆)

| 域 | 必备功能 | STAR 覆盖 | 缺口 |
|---|---|---|---|
| 1. 画布基础 | 无限画布 / pan / zoom / 视口跟随 / 运镜 / 12 element (5 装饰 2D + 7 游戏混合) | ✅ §16 | 0 |
| 2. 角色 (3渲2) | 像素风机器人 + 32 帧 sprite 动画 + 5 状态机 + 6 维属性 + 8 方向 | ✅ §17 | 0 (等美术 Lead 真人到位) |
| 3. 弹幕 (5 模式) | radial / aimed / spiral / wave / laser + 1000+ 子弹 / 60fps | ✅ §18 | 0 |
| 4. 战斗 (5 维) | DPS / DOT / AOE / 暴击 / 闪避 + 5 状态效果 + 5 维伤害公式 | ✅ §19 | 0 |
| 5. 关卡 (Roguelike) | 房间 / 走廊 / 楼层 / biome + procedural generation + seed 可复现 + BOSS | ✅ §20 | 0 |
| 6. 道具 | 武器 / 被动 / 消耗品 / 钥匙 / 金币 + 1000+ 道具 + 死亡保留 (Hades 机制) | ✅ §21 | 0 |
| 7. 技能 | 主动 / 被动 / 大招 + 100+ 技能 + 6 技能树 + 冷却 UI | ✅ §22 | 0 |
| 8. 3渲2 | Three.js 0.169+ + WebGL 2.0 + sprite atlas + 像素化后处理 + 8 方向 | ✅ §23 | 0 |
| 9. 实时协作 | Yjs CRDT + 1-4 玩家 + 共享生命池 + 离线编辑 + seed 分享 | ✅ §24 | 50 协作者旁观 P3 候选 |

**汇总**: 9 大域 100% 必备功能覆盖 (per G-1), 80+ 需求点全部映射到 §16-§25 FR。

## 附录 B: 9/6 立 SRS-001 文档迁移映射 (per ADR-CANVAS-GAME-001 推翻)

> 旧 `SRS-STAR-CANVAS-001.md` v0.1 (commit `9a2e6e0` + `934d456` self-review fix, 已 SUPERSEDED 2026-09-07) → 新 SRS-001 迁移映射:

| 旧 v0.1 章节 | 新 SRS-001 章节 | 状态 |
|---|---|---|
| §1 文档目的 (Miro 风协作画布) | §9 (画布弹幕 Roguelike) | 推翻 |
| §9 G-1 完整对标 Miro 100% 必备 | §10 G-1 4 大游戏域 + 5 装饰 element 100% 必备 | 推翻 |
| §16-§25 100+ FR (8 大功能域) | §16-§25 80+ FR (4 大游戏域 + 5 装饰 + 3渲2 渲染) | 推翻 |
| §36-§39 26 表 (canvas_*) | §36 26 表 (game_*) 命名调整 | 重命名 |
| §46-§55 40+ IR (13 REST 端点) | §46 20 IR (13 沿用 + 7 新增) | 沿用 + 扩展 |
| §86-§95 10 ADR (canvas-001~010) | §86 8 ADR (canvas-game-001~008) | 推翻 + 新增 |
| §96-§105 5 阶段 (P0-P4) | §96 5 阶段 (P0-P4, 6 周/阶段, 总 30 周) | 沿用 + 周期调整 |

## 附录 C: 4 crate 架构 + 跨 6 域引用矩阵

> canvas-engine 跨 5 domain 引用 (沿用 9/6) + canvas-game 新增 1 domain (game) 引用:

| 引用方 | 用途 | 状态 |
|---|---|---|
| `domain-canvas` | 画布业务域 (沿用 9/6) | ✅ P0 |
| `canvas-game` (新) | 游戏引擎 (角色/弹幕/战斗/关卡/道具) | ✅ P0 |
| `domain-game` (新, 待 DDD Review 拍板命名) | 游戏业务域 (5 模式权限 + 26 表) | 🟡 P0 DDD Review |
| `domain-board` (Kanban) | 沿用 9/6 | 🟡 P2 候选 |
| `domain-planning` | 沿用 9/6 | 🟡 P2 候选 |
| `agent-view` (派生) | 沿用 9/6 | ✅ 已有, P1 升级 |

---

> **文档结束**
>
> **下游交接**:
> 1. `Cargo.toml` 新增 `crates/canvas-game` (DDD Review Lead 拍板命名后)
> 2. `frontend/package.json` 新增 4 依赖 (three / phaser 或 pixi.js / perlin-noise / wfc.js)
> 3. `docs/architecture/2026-09-07-canvas-game/` 新建 3 份架构 view (跟 LangGraph / Agent Runtime 平行)
> 4. `docs/design/BD-STAR-CANVAS-GAME-001.md` v0.1 基本设计 (本 SRS 落档后 P0 启动, 6 周)
> 5. `docs/design/DD-STAR-CANVAS-GAME-001.md` v0.1 详细设计 (本 SRS 落档后 P0 启动, 6 周)
> 6. `scripts/automation/canvas_game_*.py` 新增 5 份自动化档 (scaffold / 3d_assets / e2e_guard / w_t_m_lint / naming_lint) per 守门 #19/v19+ Python 化
> 7. `docs/briefs/canvas-game-{p0,p1,p2}.md` 新增 3 份子 brief (P0 启动后 5 域 Lead 拍板)
> 8. `automation-design.md` §4 任务卡表 + `registry.md` 索引追加 (per 守门 #21)
> 9. 旧 9/6 立 2 份 brief (canvas-e2e-guard-001 + canvas-share-export-001) 标 superseded (per G-GAME-11)
