# SRS-CANVAS-001

> **无限画布 (Infinite Canvas) 总册 SRS v1.0** (per 日本 IPA SEC 標準 / 要件定義書 テンプレート)
>
> **⚠️ 方向重置 (per 2026-09-10 17:08 JST Ulysses 拍板)**
> 核心功能 = **管理 agent + 游戏化**, 避免过度冗余
>
> - 状态: Requirements Baseline
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: (root 统一 commit 时填, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
> - 关联 V0.1 实装: `docs/frontend-canvas-design.md` v0.1 + `frontend/src/components/CanvasView.tsx` + 6+3 e2e 守门
> - 上位要件: `docs/requirements/SRS-STAR-OPS-001.md` v1.0 (STAR 平台运营 SRS)
> - 平行专题 SRS (2 份并行撰写中, root 协调):
>   - `SRS-CANVAS-AGENT-001` (双核心 1: agent 管理域, 28 项)
>   - `SRS-CANVAS-GAMIFY-001` (双核心 2: 游戏化域, 32 项)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 9/8 15:19 JST 第 6 次强化)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) (per 守门 #14 v3 Mavis 永久代签)
> - 日期: 2026-09-10 JST
> - 受众: 詳細設計工程師 / 架構審查者 / UI/UX 設計師 / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-CANVAS-001 |
| 文书名 | 无限画布 总册 SRS (Infinite Canvas Master SRS) |
| 版本 | v1.0 (方向重置版) |
| 作成日 | 2026-09-10 |
| 作成者 | Ulysses — Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手) (per 守门 #14 v3) |
| 关联 commit | (root 统一 commit, per 守门 #1 v15) |
| 关联文档 | `frontend-canvas-design.md` v0.1 (V0.1 MVP design) + `CanvasView.tsx` (V0.1 实装) + 2 份平行专题 SRS + 2 份 e2e spec |
| 上位文档 | `SRS-STAR-OPS-001.md` v1.0 (STAR 平台运营 SRS, 9/8 落档) |
| 平行文档 | `SRS-CANVAS-AGENT-001` + `SRS-CANVAS-GAMIFY-001` (2 专题并行撰写) |

### 0.1 修订履历 (本总册)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 (撤回) | 2026-09-10 17:10 JST | Ulysses — Mavis 接手 | 旧方向: Miro 全功能对标 12 大类 50 项, 3 专题拆分 (collab/content/integration) | Ulysses 17:00 JST 拍板 (后撤回) |
| v1.0 (方向重置版) | 2026-09-10 17:20 JST | Ulysses — Mavis 接手 (per 守门 #14 v3) | 新方向: 双核心 = 管理 agent + 游戏化, 2 专题拆分, 砍掉 Miro 通用功能, 字节数目标 ~20K (vs v0.1 47.5K 砍 58%) | Ulysses 17:08 JST 拍板"管理 agent 和游戏化, 避免过度冗余" |
| **v1.1 (当前, 三次更新版)** | **2026-09-10 17:34 JST** | **Ulysses — Mavis 接手 (per 守门 #14 v3)** | **v0.63 反转: 撤回 17:08 JST 砍多人编辑决定, A12 多人编辑 8 项新增, AGENT 38 → 46 项, 双核心 70 → 78 项, 31 P0 → 36 P0** | **Ulysses 17:34 JST 拍板"多人编辑是要的"** |

### 0.2 平行 2 专题 SRS 修订履历 (独立跟踪)

| 文书 | 版本 | 状态 | 撰写方 | 触发 |
|---|---|---|---|---|
| `SRS-CANVAS-AGENT-001` | v1.0 (目标) | 撰写中 | worker 子代理 1 (bg_fc33dfea) | 本拍板派生 (per 17:08 JST 方向重置) |
| `SRS-CANVAS-GAMIFY-001` | v1.0 (目标) | 撰写中 | worker 子代理 2 (bg_84cf0613) | 本拍板派生 (per 17:08 JST 方向重置) |

### 0.3 撤回记录 (per 守门 #1 禁回溯叙事)

| 撤回时间 | 撤回对象 | 撤回理由 | 替代物 |
|---|---|---|---|
| 2026-09-10 17:15 JST | 3 子代理 (bg_f85f553e / bg_0b4e50e6 / bg_85c3c459) | 旧方向: Miro 全功能 12 大类 50 项 | 2 子代理 (bg_fc33dfea / bg_84cf0613) 双核心 |
| 2026-09-10 17:15 JST | 3 旧 brief (srs-canvas-collab/content/integration-001) | 旧方向 | 2 新 brief (srs-canvas-agent/gamify-001) + 1 deprecated placeholder |
| 2026-09-10 17:17 JST | 总册 v0.1 (47.5KB) | 旧方向 | 总册 v1.0 (本文件, 聚焦双核心) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 制定 STAR 平台 **无限画布 (Infinite Canvas)** 的总册 SRS, 明确**双核心定位**: **管理 agent + 游戏化**。

涵盖:
1. **双核心 ~ 60 项需求** (28 agent + 32 gamify) 的索引与优先级
2. **3 份 SRS 文档结构** (本总册 + 2 平行专题) 的职责划分与跨块接口
3. **共享约束 / 非功能需求 / 接口协议** 跨 2 专题的统一基线
4. **V0.1 MVP 衔接** (14 element + 4 frame + 8 connector + 9 e2e 守门)

2 份平行专题 SRS 各自展开详细 FR / NFR / AC / US, 本总册不重复展开。

### 1.2 背景

STAR 平台已在 2026-09-04 落地 V0.1 MVP 无限画布, 满足基础无限画布需求。

**方向重置** (per 2026-09-10 17:08 JST Ulysses 拍板):
- 旧方向 (17:00 JST): "对标 Miro, 12 大类 50 项全面铺开" — **撤回**, 过度冗余
- 新方向 (17:08 JST): **核心 = 管理 agent + 游戏化**, 避免过度冗余

**用户痛点** (per 2026-09-10 17:08 JST 用户发令): 团队成员 / PM / 5 域 Lead 需要在画布上**集中管理 agent 节点** (拓扑 / 状态 / handoff / worktree 关联) + 通过**游戏化机制** (奖励 / 积分 / 等级 / 投票 / confetti) 提升 agent 协作积极性; Miro 通用功能 (12 种 diagram / 模板 / PDF / Slack / Jira / 移动 / a11y) 超出核心范围, 砍掉。

### 1.3 包含范围

#### 1.3.1 3 份 SRS 文档职责划分

| SRS | 域 | 子能力 | 项数 | 详细展开 |
|---|---|---|---|---|
| **SRS-CANVAS-001 (本总册)** | 跨域 / 索引 | 双核心 60 项 (索引级) | 60 项 (索引) | 双核心索引 + 优先级 + 跨块接口 + 共享约束 |
| `SRS-CANVAS-AGENT-001` | 双核心 1: agent 管理 | A1-A12 (12 子能力, 含 ARG 图论构造 + 多人编辑 per 17:34 JST v0.63 反转) | **46 项** | 节点 / 拓扑 / 状态 / worktree / work-item / 操作 / 监控 / 聚类 / 跨域 / settings 集成 / **ARG (10 类关系 + 4 维度 + 5 模板 + 同步桥 + 成就)** / **多人编辑 (8 项: 多人同时编辑 + 实时 cursor + 元素增删改 + Follow mode + 评论线程 + @ + 冲突解决 + audit)** (FR/NFR/AC/US 详细) |
| `SRS-CANVAS-GAMIFY-001` | 双核心 2: 游戏化 | G1-G12 (12 子能力) | **32 项** | 节点 / 奖励 / 积分 / 升级 / AI 聚类 / 投票 / 反应 / confetti / 排行榜 / 任务 / 道具 / game 集成 (FR/NFR/AC/US 详细) |
| **合计** | | | **78 项 (展开)** | |

#### 1.3.2 V0.1 MVP 衔接 (per 守门 #11 缺标比错标)

V0.1 MVP 已实装能力, 本总册**保留**为 V0.1 不重新设计:

| V0.1 已实装 | 引用 | 衔接方式 |
|---|---|---|
| 无限世界坐标 + pan/zoom (0.1x-4x) | `CanvasView.tsx` §3 | 不重构, 仅在 NFR 引用 |
| 8 element kind + 3 routing connector | `CanvasView.tsx` line 156-340 | 类型扩展, 不破坏 schema |
| 5 联动 (WorkItem/Worktree/Relation/Comment/Search URL) | `frontend-canvas-design.md` §4 | 跨块接口 §6 引用 |
| 工具栏 + Minimap | `CanvasView.tsx` line 376 / 365 | 工具栏扩展 (本总册 §6.2) |
| 6 e2e 守门 | `canvas-view.spec.ts` | 验证基线 (本总册 §8.1) |
| 3 e2e 守门 (share + export) | `canvas-share-export.spec.ts` | 验证基线 (本总册 §8.1) |
| agent_cursor (line 218) | AGENT A1 扩展为 agent_node | A1.1 |
| comment_pin (line 253) | GAMIFY G7 reaction 共用 | G7.1 |
| sticky_note (line 156) | GAMIFY G5 聚类 / G6 投票 共用 | G5.1 + G6.1 |
| automation_node (line 236) | GAMIFY G2 reward rule 触发 | G2.1 |
| **agent-game V0.1** (Roguelike + Manga + Theme + Settings) | GAMIFY G12 集成 | G12.1-12.2 |

### 1.4 不包含范围 (per 守门 #11 缺标比错标)

**核心原则**: **避免过度冗余**, 以下 Miro 通用功能**全部砍掉**, 留 P3+ 评估:

| 不包含类别 | 不包含内容 | 砍掉理由 |
|---|---|---|
| **Miro 通用协作** | 多人同时编辑 / 实时 cursor / Follow mode / 评论线程 / @ 提醒 / 内置视频通话 | 超出核心, 协作工具 (Slack / 飞书) 已有 |
| **Miro 通用演示** | Frame as slide / Guided Tour / Speaker notes / Timer 演讲 | 演示用 Keynote / PowerPoint 已成熟 |
| **Miro 通用导出** | PDF / Word / Excel / CSV / SVG | PNG 已有 (V0.1), 文本导出超出核心 |
| **Miro 通用互动** | Cursor chat / Async video / Timer workshop 用 | 互动用 Slack / 飞书 / 邮件 |
| **Miro 通用版本** | Version history / Branching / Restore | 跟 Git worktree 重复, 用 Git 即可 |
| **Miro 通用集成** | Slack / Jira / Asana / Figma / Notion / GitHub / Zoom | Star 25 module 已有, 集成超出画布核心 |
| **Miro 通用移动** | iOS / Android native / Touch / Stylus / Offline | 客户端重投入, 短期 web 触控够用 |
| **Miro 12 种 diagram** | Mind map / Flowchart / BPMN / ER / Wireframe / Kanban / Sequence / Smart drawing / Draw | 内容生产用专门工具 (Figma / Lucidchart) |
| **Miro 模板库** | 2500+ 模板 | 评估做 5-10 个, 留 P3+ |
| **Miro AI 通用** | 文本生成 diagram / 翻译 / 图像识别 | 仅留 sticky note 聚类 (GAMIFY G5, 走 mock) |
| **Miro Tables / Chart widget / Form** | 表格 / 数据源 / Chart / 表单 | 跟 Star 数据模块重复, 留 P3+ |
| **Miro 完整 a11y (WCAG 2.1 AA)** | 完整合规 | 部分 a11y 跟随 V0.1, 完整合规留 P3+ |
| **5 域 Lead 真人到位前跨域编排决策** | 真人 Lead 决策 | per 9/3 11:35 JST 拍板 B: Mavis 临时代签, 真人到位后追溯签字 (per守门 #14 v2 + 9/5 10:43 JST 拍板 D), **不沿用代签决策** (per守门 #1 禁回溯叙事) |
| **5 域独立 Lead ≠ Star 22 DDD bounded context** | 业务子域↔DDD 映射 | per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer |
| **Star 25 module DDD bounded context 业务子域映射** | 业务子域↔DDD 映射 | per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer, 不建立 |

### 1.5 用户故事 (总册级, 跨 2 专题)

| 编号 | 角色 | 故事 | 优先级 | 引用专题 |
|---|---|---|---|---|
| US-T-1 | 5 域 Lead (per 守门 #3, 真人未到位 Mavis 临时代签) | 作为 Lead, 我希望在画布上一眼看到 5 域所有 agent 节点的拓扑图, 实时状态色码同步, 双击跳详情 | P0 | AGENT A1+A2+A3 |
| US-T-2 | 5 域 Lead | 作为 Lead, 我希望在画布上启停 / 重启 agent, 查看 token 用量 / runtime, 异常告警 | P0 | AGENT A6+A7 |
| US-T-3 | PM (Ulysses 类) | 作为 PM, 我希望在画布上看到 agent 关联的 worktree + work-item 实时状态, 不切换 4 个 Tab | P0 | AGENT A4+A5 |
| US-T-4 | Dev / 团队成员 | 作为成员, 我希望画布上有 RPG 元素 (avatar / level / xp / skill tree), 完成 agent 任务获得 reward, 解锁 badge | P1 | GAMIFY G1+G2+G3+G4 |
| US-T-5 | Dev | 作为成员, 我希望在画布上 sticky note 聚类成主题 (AI mock), dot voting 决策下一步, reaction emoji 表达态度 | P1 | GAMIFY G5+G6+G7 |
| US-T-6 | 团队成员 | 作为成员, 我希望完成关键任务时 confetti 庆祝, daily challenge 鼓励日常参与, streak 连续天数 | P2 | GAMIFY G8+G10 |
| US-T-7 | 5 域 Lead | 作为 Lead, 我希望看到 per workspace 排行榜, 鼓励团队协作 | P2 | GAMIFY G9 |
| US-T-8 | Dev (RPG 玩家) | 作为 RPG 玩家, 我希望有 power-up 道具 (双倍积分 / 自动聚类 / 隐身), inventory 物品栏管理 | P3 | GAMIFY G11 |
| US-T-9 | 团队成员 | 作为成员, 我希望画布上跟 Roguelike / Manga 主题游戏节点集成, 一边画架构一边玩 | P2 | GAMIFY G12 (V0.1 集成) |
| US-T-10 | 5 域 Lead | 作为 Lead, 我希望 agent 聚类 / 排序 / 过滤, 按 role / kind / status 分组 | P1 | AGENT A8 |

---

## §2 用語定義 (用語集 / Ubiquitous Language)

### 2.1 无限画布核心 (per frontend-canvas-design.md v0.1)

| 用語 | 定義 | 出处 / 备注 |
|---|---|---|
| 无限画布 (Infinite Canvas) | Miro 风格画布, 世界坐标 + viewport 转换, 鼠标 pan/zoom, 不强制栅格 | per `frontend-canvas-design.md` v0.1 |
| 世界坐标 (World) | 画布无限延伸, 所有 element 真实位置 (x, y ∈ [-100K, 100K]) | per design §3.2 |
| 屏幕坐标 (Screen) | 浏览器视口, 跟 world 转换公式 `screen = (world - viewport) * zoom` | per design §3.2 |
| Viewport | 画布观察窗口, 含 pan (x, y) + zoom (0.1x ~ 4x) | per design §3.2 |
| 最小画布尺寸 | 100,000 × 100,000 px (超过则 panic 给 warning) | per design §3.2 |
| Frame | 画布分区, 矩形 + 标题, 可作 slide 演示 | per design §2.1 + §3.6 |
| Element | 画布可视对象 (V0.1 8 kind + 双核心扩展) | per design §2.2 |
| Connector | 画布连线 (3 routing: straight / curved / orthogonal) | per design §2.3 + §3.5 |
| Bezier Connector | 三次贝塞尔曲线, 复用 SmView 算法 (c1x = fx + dx*0.25, c2x = tx - dx*0.25) | per design §3.5 |

### 2.2 双核心 1: Agent 管理 (per SRS-CANVAS-AGENT-001)

| 用語 | 定義 | 出处 |
|---|---|---|
| Agent Session | 1 个 AI Agent 执行的会话实例, 14 状态机 (queued/spawning/initializing/compiling_context/planning/executing/awaiting_feedback/awaiting_human/awaiting_tool/validating/paused/completed/failed/cancelled) | per `SRS-STAR-AGENT-RUNTIME-001.md` §8 |
| Active Agent | 处于上述 14 状态中前 11 个 (排除 completed/failed/cancelled 终态) 的 agent session | per `SRS-AGENT-VIEW-001.md` §2 |
| agent_node | V0.1 agent_cursor 升级为完整卡 (头像/名字/role/kind/status/token 用量/启动时间) | AGENT A1.1 |
| Agent Handoff | 1 个 agent 完成后, 上下文传给下 1 个 agent, 含时间 / 状态 / 备注 | AGENT A2.1 |
| 5 域分组 | 5 域 (player/economy/match/social/admin per 8/21 JST RGS 治理命名) Frame 分组 | AGENT A2.2 |
| Token OLU | 1 SRE·周 ≈ 1.2M tokens (per STAR-OLU-001 v0.1 2026-08-29 落档) | per 守门 #4 |

### 2.3 双核心 2: 游戏化 (per SRS-CANVAS-GAMIFY-001)

| 用語 | 定義 | 出处 |
|---|---|---|
| Gamification 节点 | V0.1 8 kind 扩展: avatar / level / xp / skill_tree / class / badge / quest / inventory | GAMIFY G1.1-1.8 |
| Reward / Achievement | 解锁条件 + 通知 + 画布特效 (跟 V0.1 automation rule 触发) | GAMIFY G2.1-2.4 |
| Score / Points | 操作积分 (完成任务 +10, 投票 +1, 分享 +5) | GAMIFY G3.1-3.2 |
| Leveling | xp → level 公式 (默认线性: level = sqrt(xp / 100)) | GAMIFY G4.1 |
| Skill Tree | 升级达到解锁条件, 技能树解锁 | GAMIFY G4.3 |
| Sticky Note 聚类 (AI) | 选中 N 张 sticky_note → AI 聚类成 K 个主题 (K 用户可调) — **走 mock 接口** (per 守门 #23 v2, 不引入第三方 LLM 凭据) | GAMIFY G5.1 |
| Dot Voting | 每用户 N 票 (默认 5 票/session), 点 sticky_note 投票 | GAMIFY G6.1 |
| Reaction | emoji 表情回应 (8-12 种: 👍 ❤️ 🎉 😄 🤔 👀 🔥 ⭐), 短时显示 (3s 淡出) | GAMIFY G7.1 |
| Confetti | 庆祝特效 (CSS / Lottie, 触发: 完成 / 解锁 / 升级) | GAMIFY G8.1 |
| Leaderboard | per workspace / per tenant 排行榜 (token 用量 / 任务完成数 / vote 数量) | GAMIFY G9.1-9.2 |
| Daily Challenge | 每日 3 个任务, 完成得 reward | GAMIFY G10.1 |
| Streak | 连续天数 (中断清零, 7 天奖励) | GAMIFY G10.2 |
| Power-up | 道具 (双倍积分 / 自动聚类 / 隐身模式, 限时) | GAMIFY G11.1 |
| Inventory | 物品栏 (持有 + 使用, per 守门 #13 W/T/M 三類横展) | GAMIFY G11.2 |
| DB W/T/M 三類 | Work (短 TTL 作業中) / Transaction (業務事実 Append-only) / Master (SCD Type 2) | per 守门 #13 (per AGENTS.md §4 #13) |

### 2.4 跨域共享 (本总册)

| 用語 | 定義 | 出处 |
|---|---|---|
| 派生视图 (Projection) | 从已有数据派生计算的视图, 不是业务事实源, 不可写 | DDD 概念, per SRS-Runtime §4.5 |
| 跨域编排 (Cross-Domain Orchestration) | 多 domain 协作完成 1 业务用例, 由 Saga orchestrator 协调 | per 守门 #3 v2 + 9/3 11:35 JST 拍板 B |
| Token-OLU | 1 SRE·周 ≈ 1.2M tokens (per STAR-OLU-001 v0.1 2026-08-29 落档) | per 守门 #4 |
| Mavis 接手 | Mavis 接手代理 Ulysses 决策 (per 守门 #14 v3 Mavis 永久代签 + 9/8 15:19 第 6 次强化) | per AGENTS.md §4 #14 |
| Mavis 审核 | per 2026-09-10 12:45 JST v0.62 反转, 真人代签流程全部取消, 改为 Mavis 审核 决定 author=Ulysses | per 守门 #14 v4 + v0.62 反转 |

---

## §3 業務背景 / 前提条件

### 3.1 业务背景

STAR 平台是 5 域 (player/economy/match/social/admin per 8/21 JST RGS 治理命名) 分布式系统, 已有 25 module + 6 状态机 + V0.1 无限画布 MVP + LangGraph 编排 + Agent Runtime 9/3 落档。

**画布能力现状** (V0.1 MVP, 2026-09-04 落档):
- 14 element (work-item 2 + worktree 3 + agent 3 + feedback 2 + sticky 1 + automation 2 + text 1)
- 4 frame + 8 connector
- 6 e2e 守门 (pan / zoom / fit / highlight / delete / minimap)
- 3 e2e 守门 (share / export PNG)
- 5 联动 (WorkItem/Worktree/Relation/Comment/Search URL)
- 工具栏 + Minimap

**双核心方向** (per 2026-09-10 17:08 JST Ulysses 拍板):
1. **管理 agent**: 画布上**集中管理 agent 节点** (节点 / 拓扑 / 状态 / handoff / worktree 关联)
2. **游戏化**: 画布上**游戏化机制** (节点 / 奖励 / 积分 / 等级 / 投票 / confetti) 提升 agent 协作积极性

**业务目标** (本 SRS 派生):
1. **3 个月内**: P0 落地 (agent_node 完整卡 + 5 域拓扑 + 状态实时同步 + worktree/work-item 关联 + 启停 + 监控 + 基础 RPG 元素)
2. **6 个月内**: P1 补齐 (聚类 / 投票 / reaction / confetti / daily challenge / leaderboard / 排行)
3. **12 个月内**: P2 扩展 (跟 V0.1 game 4 份集成 + 跨域引用 / 完整 avatar 系统)
4. **12 个月+**: 评估 P3 (power-up / inventory / 5 域 Lead 真人到位后追溯签字 / 跨 workspace 排行榜)

### 3.2 前提条件 (per 守门 #11 缺标比错标)

| 前提 | 说明 | 引用 |
|---|---|---|
| V0.1 MVP 已实装 | 14 element + 4 frame + 8 connector + 9 e2e 守门 | `frontend-canvas-design.md` v0.1 + `CanvasView.tsx` + 2 e2e spec |
| 25 module 已有 work-item / worktree / agent / relation / comment / search / notification 7 类可对接 | 跨块接口已实装 | per frontend-canvas-design §4 联动矩阵 |
| Star 平台前端为 Next.js 14 + React 18 + zustand + lucide-react + react-hot-toast | 不引入新前端框架 | per `frontend/package.json` |
| **5 域 Lead 真人未到位** | 跨域编排决策延后, Mavis 临时代签 (per 守门 #14 v2 + 9/5 10:43 JST 拍板 D) | per AGENTS.md §4 #3 + #14 v2 |
| **5 域独立 Lead ≠ Star 22 DDD bounded context** | per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer, 不建立业务子域↔DDD映射 | per AGENTS.md §4 #3 + #4 #1 |
| **AGENT A3 实时状态同步 WebSocket 选型未拍板** | P0 阻塞, 拍板后启动 | per AGENT §7 风险 |
| **AGENT A4 agent ↔ worktree 1:N 关系当前 schema 可能缺** | per `SRS-AGENT-VIEW-001.md` §3 缺口 #4 schema 缺 `WorkItem.agent_session_id` 字段 | per AGENT §7 风险 |
| **GAMIFY G5 sticky note 聚类 AI 走 mock 接口** | per 守门 #23 v2 ai-edit-mode=本地 mock, 不引入第三方 LLM 凭据 | per GAMIFY §7 风险 |
| **ARG (Agent Relationship Graph) 已落档** | `SRS-AGENT-RELATIONSHIP-001.md` v0.1 (9/8 落档, 37KB) + `BD-AGENT-RELATIONSHIP-001.md` (55KB) + `DD-AGENT-RELATIONSHIP-001.md` (94KB) + `DDD-REVIEW-AGENT-RELATIONSHIP-001.md` (30KB) | per 2026-09-08 22:35 JST 拍板, AGENT A11 必含 |
| **ARG 10 类关系 + 4 维度 + 5 模板** | 画布 A11 子能力主源, 不重写 ARG 逻辑, 仅画布集成 | per `SRS-AGENT-RELATIONSHIP-001.md` §4.1+4.3+4.5 |
| **Memgraph 图数据库** | Docker 启动 port 7687 Bolt + 7444 HTTP, 数据卷持久化 | per `SRS-AGENT-RELATIONSHIP-001.md` §3.2 PR-4 |
| **ARG 4 表 W/T/M 分类已定** | agents = Master / agent_relationship_edges = Master / agent_relationship_edges_audit = Transaction / team_template_instances = Work, 100% 表覆盖 | per 守门 #13 |
| **GAMIFY G11 power-up / inventory 必含 W/T/M 三類横展** | per 守门 #13 | per AGENTS.md §4 #13 |
| **html2canvas 1.4.1 不渲染 SVG foreignObject** | PNG export 文字丢失已知缺口, 留 V0.1 | per `canvas-share-export-001.md` §3 缺口 #1 |
| **代签规则** | Mavis 接手代签 Ulysses (per 守门 #14 v3 + 9/8 15:19 第 6 次强化) | per AGENTS.md §4 #10 + #14 v3 |
| **DB W/T/M 三類横展** | GAMIFY G11 必含 Work/Transaction/Master, 100% 表覆盖 | per 守门 #13 |
| **AI mock 接口** | 不引入第三方 API 凭据 (per 守门 #23 v2) | per GAMIFY §10.x |

---

## §4 功能需求 (双核心 60 项索引 + 跨块接口)

### 4.1 双核心 60 项 总览 (per 17:08 JST Ulysses 拍板)

| 双核心 | 子能力 | 项数 | 优先级 P0 | P1 | P2 | P3 | 引用专题 SRS |
|---|---|---|---|---|---|---|---|
| **1. Agent 管理** | A1-A12 (12, 含 ARG 图论构造 + 多人编辑 per 17:34 JST v0.63 反转) | **46** | 24 | 13 | 9 | 0 | `SRS-CANVAS-AGENT-001` |
| **2. 游戏化** | G1-G12 (12) | **32** | 12 | 9 | 9 | 2 | `SRS-CANVAS-GAMIFY-001` |
| **合计** | **24 子能力** | **78** | **36** | **22** | **18** | **2** | |

### 4.2 Agent 管理 28 项 子能力索引 (per `SRS-CANVAS-AGENT-001`)

| 子能力 | 项数 | 描述 | 引用 |
|---|---|---|---|
| A1 agent 节点渲染 | 3 | agent_cursor 升级为 agent_node 完整卡 + StatusPill 色码 + 双击跳详情 | AGENT §4.1 |
| A2 agent 拓扑图 | 4 | 1:N handoff + 5 域分组 + 父子关系 + pipeline 视图 | AGENT §4.2 |
| A3 agent 状态实时同步 | 3 | 14 状态机实时色码 + audit + notification | AGENT §4.3 |
| A4 agent 关联 worktree | 2 | 1 agent → N worktree, status 联动 | AGENT §4.4 |
| A5 agent 关联 work-item | 2 | 1 agent → N work-item, status 联动 | AGENT §4.5 |
| A6 agent 操作菜单 | 4 | 启停 / 重启 / logs / settings | AGENT §4.6 |
| A7 agent 监控面板 | 3 | token / cost / runtime 仪表 + budget 对比 + 告警 | AGENT §4.7 |
| A8 agent 聚类 / 排序 / 过滤 | 3 | 按 role / kind / status 聚类, 按 token / 启动时间 排序, 按 status / token 过滤 | AGENT §4.8 |
| A9 agent session 跨域引用 | 2 | 跟 SRS-AGENT-VIEW-001 / SRS-AGENT-RELATIONSHIP-001 协同 | AGENT §4.9 |
| A10 agent settings 集成 | 2 | 跟 V0.1 AgentSettingsTab 集成, 画布调用 settings | AGENT §4.10 |
| **A11 ARG 图论构造 (per 17:21 JST 补充)** | **10** | **10 类关系边 + 4 维度协作影响 + 5 团队模板 + 同步桥 + 成就 + audit + 版本** (主源 `SRS-AGENT-RELATIONSHIP-001.md` v0.1) | **AGENT §4.11** |
| **A12 多人编辑 (per 17:34 JST 拍板"多人编辑是要的", v0.63 反转)** | **8** | **多人同时编辑 + 实时 cursor + 元素增删改 + Follow mode + 评论线程 + @ + 冲突解决 (CRDT 选型) + view-comment-edit 3 级权限 + audit log** (撤回 17:08 JST 砍多人编辑决定) | **AGENT §4.12** |

### 4.3 游戏化 32 项 子能力索引 (per `SRS-CANVAS-GAMIFY-001`)

| 子能力 | 项数 | 描述 | 引用 |
|---|---|---|---|
| G1 gamification 节点 | 8 | avatar / level / xp / skill_tree / class / badge / quest / inventory | GAMIFY §4.1 |
| G2 reward / achievement | 4 | 解锁条件 + 通知 + 画布特效 + 自动授予 | GAMIFY §4.2 |
| G3 score / points | 3 | 操作积分规则 + 累计 + 排行榜入口 | GAMIFY §4.3 |
| G4 leveling / skill tree | 3 | xp → level 公式 + 升级动画 + skill tree 解锁 | GAMIFY §4.4 |
| G5 sticky note 聚类 (AI, mock) | 2 | 选中 N 张 → AI 聚类成 K 主题 + 主题 Frame | GAMIFY §4.5 |
| G6 dot voting | 2 | 每用户 N 票 + 实时显示 | GAMIFY §4.6 |
| G7 reaction | 1 | emoji 表情回应 (8-12 种) | GAMIFY §4.7 |
| G8 confetti | 1 | 庆祝特效 (CSS / Lottie) | GAMIFY §4.8 |
| G9 leaderboard | 2 | per workspace / per tenant | GAMIFY §4.9 |
| G10 daily challenge / streak | 2 | 每日 3 任务 + 连续天数 | GAMIFY §4.10 |
| G11 power-up / inventory | 2 | 道具 + 物品栏 (**W/T/M 三類横展**) | GAMIFY §4.11 |
| G12 跟 agent-game V0.1 集成 | 2 | Roguelike + Manga 集成 | GAMIFY §4.12 |

### 4.4 P0 优先级清单 (3 个月内, 必含)

| 编号 | 标题 | 双核心 | 子能力 | 引用 |
|---|---|---|---|---|
| F-P0-A1.1 | agent_cursor 升级为 agent_node 完整卡 | Agent | A1 | AGENT A1.1 |
| F-P0-A1.2 | agent_node 状态色码 走 StatusPill 60+ | Agent | A1 | AGENT A1.2 |
| F-P0-A1.3 | agent_node 双击跳详情 | Agent | A1 | AGENT A1.3 |
| F-P0-A2.1 | 1:N handoff connector | Agent | A2 | AGENT A2.1 |
| F-P0-A2.2 | 跨 5 域拓扑图 | Agent | A2 | AGENT A2.2 |
| F-P0-A3.1 | 14 状态机实时色码同步 | Agent | A3 | AGENT A3.1 |
| F-P0-A4.1 | agent → worktree 1:N 关联 | Agent | A4 | AGENT A4.1 |
| F-P0-A4.2 | worktree status → agent 状态联动 | Agent | A4 | AGENT A4.2 |
| F-P0-A5.1 | agent → work-item 1:N 关联 | Agent | A5 | AGENT A5.1 |
| F-P0-A5.2 | work-item status → agent 状态联动 | Agent | A5 | AGENT A5.2 |
| F-P0-A6.1 | agent 启停 | Agent | A6 | AGENT A6.1 |
| F-P0-A6.2 | agent 重启 | Agent | A6 | AGENT A6.2 |
| F-P0-G1.1 | avatar_node 玩家头像 | Game | G1 | GAMIFY G1.1 |
| F-P0-G1.2 | level_node 等级 | Game | G1 | GAMIFY G1.2 |
| F-P0-G1.3 | xp_node 经验值 | Game | G1 | GAMIFY G1.3 |
| F-P0-G1.4 | skill_tree_node 技能树 | Game | G1 | GAMIFY G1.4 |
| F-P0-G1.5 | class_node 职业 | Game | G1 | GAMIFY G1.5 |
| F-P0-G1.6 | badge_node 徽章 | Game | G1 | GAMIFY G1.6 |
| F-P0-G3.1 | 操作积分规则 | Game | G3 | GAMIFY G3.1 |
| F-P0-G3.2 | score 累计 | Game | G3 | GAMIFY G3.2 |
| F-P0-G4.1 | xp → level 公式 | Game | G4 | GAMIFY G4.1 |
| F-P0-G4.2 | 升级动画 + 通知 | Game | G4 | GAMIFY G4.2 |
| F-P0-G6.1 | dot voting 每用户 N 票 | Game | G6 | GAMIFY G6.1 |
| F-P0-G6.2 | dot voting 实时显示 | Game | G6 | GAMIFY G6.2 |
| F-P0-G8.1 | confetti 庆祝特效 | Game | G8 | GAMIFY G8.1 |
| F-P0-G12.1 | Roguelike 集成 (V0.1) | Game | G12 | GAMIFY G12.1 |
| F-P0-G12.2 | Manga 集成 (V0.1) | Game | G12 | GAMIFY G12.2 |
| F-P0-A11.1 | 10 类关系边渲染 (4 核心 + 6 扩展, 颜色区分) | Agent | A11 | AGENT A11.1 |
| F-P0-A11.2 | 关系编辑 (UI 拖拽 + type 选择 + weight 滑块) | Agent | A11 | AGENT A11.2 |
| F-P0-A11.3 | 4 维度协作影响 UI 指示器 (dispatch / context / trust / review) | Agent | A11 | AGENT A11.3 |
| F-P0-A11.5 | 关系 audit log (per 守门 #13 W/T/M Transaction append-only + SCD Type 2) | Agent | A11 | AGENT A11.5 |
| F-P0-A12.1 | 多人同时编辑同一 canvas (实时同步, max 200ms 延迟, max 10 并发) | Agent | A12 | AGENT A12.1 |
| F-P0-A12.2 | 实时 cursor 同步 (其他用户光标 + 名字 + 当前 viewport) | Agent | A12 | AGENT A12.2 |
| F-P0-A12.3 | 元素增删改实时同步 (Realtime WS 通道, 跟 V0.1 §4.1 模式 A 扩展) | Agent | A12 | AGENT A12.3 |
| F-P0-A12.5 | 多人评论线程 + @ 提醒 (V0.1 comment_pin 已实装, 升级 thread + @ + notification 域对接) | Agent | A12 | AGENT A12.5 |
| F-P0-A12.6 | 冲突解决 (CRDT 选型, Yjs vs Automerge vs LWW, 跟 A11 同步桥同源) | Agent | A12 | AGENT A12.6 |
| F-P0-A12.8 | audit log 多人操作 (per 守门 #13 W/T/M Transaction append-only, 新增 `canvas_multi_user_audit` 表) | Agent | A12 | AGENT A12.8 |

### 4.5 跨块接口 (Cross-Block Interface)

#### 4.5.1 2 专题 SRS 互引用矩阵

| 引用方 | 引用内容 | 被引用方 |
|---|---|---|
| AGENT A1 agent_node | 共用 V0.1 sticky_note (G6 投票节点) | GAMIFY G6.1 |
| AGENT A6 操作菜单 | reward 解锁 + badge 授予 | GAMIFY G2.1-2.4 + G1.6 |
| AGENT A7 监控 | token 累计 → score 累计算法 | GAMIFY G3.2 |
| AGENT A8 聚类 | 跟 GAMIFY G1.5 class 协同 (按职业聚类) | GAMIFY G1.5 |
| GAMIFY G2.1 reward 解锁 | agent 完成 task → reward 触发 | AGENT A6 + A7 |
| GAMIFY G5 sticky note 聚类 | agent 关联 work-item 聚类 (1:N → 1 主题) | AGENT A5.1 |
| GAMIFY G9 leaderboard | per workspace / per tenant 跨域 | 25 module (tenant / workspace) |
| GAMIFY G11 power-up / inventory | DB W/T/M 三類横展 | per 守门 #13 |
| **AGENT A11 ARG 10 类关系** | delegates_to / consults / collaborates_with / reports_to + mentors / peer_reviews / stand_in_for / shadows / challenges / trusts, 画布渲染 + 编辑 | **SRS-AGENT-RELATIONSHIP-001 §4.1** |
| **AGENT A11 4 维度协作影响** | dispatch 路由 / 上下文共享 / 信任度加权 / 产出评估, UI 指示器 | **SRS-AGENT-RELATIONSHIP-001 §4.3** |
| **AGENT A11 5 团队模板** | Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council, 1-click 部署 | **SRS-AGENT-RELATIONSHIP-001 §4.5** |
| **AGENT A11 同步桥** | Memgraph ↔ LangGraph StateGraph 双向, UI sync status | **SRS-AGENT-RELATIONSHIP-001 §4.4** |
| **AGENT A11 4 表 W/T/M** | agents = Master / agent_relationship_edges = Master / agent_relationship_edges_audit = Transaction / team_template_instances = Work, 100% 表覆盖 | **per 守门 #13** |
| **AGENT A12 多人编辑** (per 17:34 JST v0.63 反转, 撤回 17:08 JST 砍多人编辑决定) | 多人同时编辑 + 实时 cursor + 元素增删改 + Follow mode + 评论线程 + @ + 冲突解决 + view-comment-edit 权限 + audit log (新增 `canvas_multi_user_audit` 表 100% RLS 13 类) | **per `frontend-canvas-design.md` §4.1 模式 A Realtime 通道** |

#### 4.5.2 25 module 联动 (per frontend-canvas-design §4 联动矩阵 + 双核心扩展)

| 25 module | 画布表现 (V0.1) | 双核心扩展 | 反向链接 | 引用 |
|---|---|---|---|---|
| work-item | 拖 WorkItem → 画布变 element (sticky / card) | AGENT A5 1:N 关联 | 点 element → 跳 /work-item?selected=wi-XXX | per V0.1 §4.2-4.3 + AGENT A5 |
| worktree | 自动在画布生成 node, 显示 status 颜色 | AGENT A4 1:N 关联 | 点 node → 跳 /worktree?selected=wt-XXX | per V0.1 §4.4 + AGENT A4 |
| agent | 画布上画 cursor (PresenceCursor 升级) | AGENT A1 agent_node + A2 拓扑 + A3 状态 + A8 聚类 | 点 cursor → 跳 /agent?selected=ag-XXX | per V0.1 §4.6 + AGENT A1-A8 |
| relation | 5 种关系渲染 connector | AGENT A2 handoff + 父子关系 | 点 connector → 跳 /relation 详情 | per V0.1 §4.5 |
| comment | element 上挂载 comment | GAMIFY G7 reaction 复用 comment_pin | 点 comment → 跳 comment 详情 | per V0.1 + GAMIFY G7 |
| automation | 画布事件触发 rule | GAMIFY G2 reward rule 触发 | rule trigger_kind 加 canvas_event | per V0.1 §4.7 + GAMIFY G2 |
| audit | 画布操作写 audit (action: canvas.element.move) | AGENT A3.2 状态变化 audit | — | per V0.1 §4.1 + AGENT A3.2 |
| search | 搜 work-item 跳 canvas 对应 element | GAMIFY G5 聚类 (AI mock) | — | per V0.1 §4.8 + GAMIFY G5 |
| notification | 画布"被 @ 提及时"通知 | GAMIFY G2.2 reward 通知 | — | per V0.1 §4.1 + GAMIFY G2.2 |
| agent-game (V0.1) | Roguelike / Manga / Theme / Settings | GAMIFY G12 集成 | — | per V0.1 game 4 份 PHASE + GAMIFY G12 |

---

## §5 非功能需求 (跨 2 专题共享基线)

### 5.1 性能 (NFR-PERF)

| 编号 | 指标 | 目标 | 引用 |
|---|---|---|---|
| NFR-PERF-1 | viewport pan/zoom 帧率 | ≥ 60 FPS (中端笔记本 Chrome 1280x720) | 跨 2 专题 + V0.1 |
| NFR-PERF-2 | element 渲染数量 | ≥ 1000 element 不掉帧 (per design §9 CANVAS-OI-09 V1 候选) | V0.1 |
| NFR-PERF-3 | 实时状态同步延迟 (agent 14 状态机) | < 200ms (P95) | AGENT A3.1 |
| NFR-PERF-4 | 导出 PNG (1920x1080) | < 5s | V0.1 |
| NFR-PERF-5 | sticky note AI 聚类 (N=50) | < 3s (mock 接口) | GAMIFY G5.1 |
| NFR-PERF-6 | confetti 动画帧率 | ≥ 60 FPS | GAMIFY G8.1 |
| NFR-PERF-7 | dot voting 实时显示 | < 100ms (P95) | GAMIFY G6.2 |

### 5.2 可访问性 (NFR-A11Y)

| 编号 | 指标 | 目标 | 引用 |
|---|---|---|---|
| NFR-A11Y-1 | 键盘可访问 | 全功能可键盘操作 (Tab / Enter / Esc / 方向键) | V0.1 (部分) |
| NFR-A11Y-2 | 颜色对比度 | ≥ 4.5:1 (正文) / ≥ 3:1 (大字体) | V0.1 |
| NFR-A11Y-3 | 焦点指示 | 明显焦点环 (高对比度, 不依赖颜色) | V0.1 |
| NFR-A11Y-4 | 完整 WCAG 2.1 AA 合规 | **砍掉, 留 P3+** (per 17:08 JST 避免过度冗余) | — |

### 5.3 安全 (NFR-SEC, per 守门 #5 + 守门 #1 v1a)

| 编号 | 指标 | 目标 | 引用 |
|---|---|---|---|
| NFR-SEC-1 | 凭据管理 | stdin pipe 模式, 密码不上命令行, 不打印 (per 守门 #5 hard ban) | AGENT A6 操作权限 |
| NFR-SEC-2 | 401 Authentication failed | 跨 session 续, **不算 timeout, max 2 retries** (per 守门 #1 v1a 2026-09-03 11:07 JST 实证) | AGENT A6 + GAMIFY G2.2 |
| NFR-SEC-3 | 网络错误 (Recv failure / Connect failed / timeout) | max 2 retries, 30s-2min 后常恢复, 不连续 retry | per 守门 #1 v1a |
| NFR-SEC-4 | agent 启停权限 | BFF API 层强制, 前端不可绕过 (per 守门 #3 5 域独立 Lead) | AGENT A6.1-6.2 |
| NFR-SEC-5 | **第三方集成 OAuth** | **砍掉, 留 P3+** (per 17:08 JST, Miro 集成超出核心) | — |

### 5.4 扩展性 (NFR-EXT)

| 编号 | 指标 | 目标 | 引用 |
|---|---|---|---|
| NFR-EXT-1 | 新增 agent 节点 kind | 接入约定 type + content, ≤ 1 PR | AGENT A1 |
| NFR-EXT-2 | 新增 gamification 节点 kind | 同上, 8 kind V1 | GAMIFY G1 |
| NFR-EXT-3 | 新增 reward rule | 复用 V0.1 automation rule, ≤ 1 PR | GAMIFY G2.1 |
| NFR-EXT-4 | **新增 AI 能力** | mock 接口 → 真实 LLM 平滑切换 (per 守门 #23 v2) | GAMIFY G5 |

### 5.5 国际化 (NFR-I18N, per STAR-I18N 拍板)

| 编号 | 指标 | 目标 | 引用 |
|---|---|---|---|
| NFR-I18N-1 | 支持语言 | zh-CN (default) / en / ja (per STAR-I18N 拍板) | per `frontend/src/lib/i18n/` |
| NFR-I18N-2 | 文案提取 | i18n key 化, 不硬编码 | per `docs/reports/STAR-I18N-TAKEOVER-REPORT.md` |
| NFR-I18N-3 | 日期 / 数字 | locale-appropriate 格式 | per i18n §3 |

### 5.6 兼容性 (NFR-COMPAT)

| 编号 | 指标 | 目标 |
|---|---|---|
| NFR-COMPAT-1 | 浏览器 | Chrome 120+ / Edge 120+ / Safari 17+ / Firefox 120+ (per V0.1) |
| NFR-COMPAT-2 | 设备 | Desktop (1280x720+) primary, Tablet (iPad) secondary, **Mobile 砍掉, 留 P3+** |
| NFR-COMPAT-3 | OS | Windows 10+ / macOS 12+ / iPadOS 16+ (per 25 module e2e 实测) |

---

## §6 接口需求 (跨 2 专题 + 25 module + V0.1 game)

### 6.1 3 份 SRS 内部接口 (本总册与 2 专题)

#### 6.1.1 总册 → 2 专题

| 总册章节 | 2 专题引用 |
|---|---|
| §4.1 双核心 60 项 总览 | AGENT §1.3 + GAMIFY §1.3 |
| §4.2-4.3 子能力索引 | AGENT §4 + GAMIFY §4 必引 |
| §4.4 P0 清单 | 2 专题 §1.3 包含范围 必引 |
| §4.5 跨块接口矩阵 | 2 专题 §7 跨专题依赖 必引 |
| §5 NFR 共享 | 2 专题 §5 NFR 不重复, 引用总册 |
| §7 约束 | 2 专题 §7 约束 必引 |

#### 6.1.2 2 专题 → 总册

| 2 专题章节 | 总册引用 |
|---|---|
| 详细 FR / NFR / AC / US | 2 专题自含, 总册仅索引 |
| §6 接口 (BFF / WS / V0.1 game) | 总册 §6.2 跨块接口 汇总 |
| §7 已知缺口 | 2 专题自含, 总册 §7 跨块缺口 汇总 |

### 6.2 画布 ↔ BFF API (per V0.1 9 API + 双核心扩展)

#### 6.2.1 V0.1 已实装 API (9 个)

| API | 方法 | 路径 | 引用 |
|---|---|---|---|
| Canvas CRUD | GET/POST/PATCH/DELETE | `/v1/collaboration/canvases` | per `frontend-canvas-design.md` §2.1 |
| Element CRUD | GET/POST/PATCH/DELETE | `/v1/collaboration/canvases/[id]/elements` | per design §2.2 |
| Connector CRUD | GET/POST/PATCH/DELETE | `/v1/collaboration/canvases/[id]/connectors` | per design §2.3 |
| Viewport sync | WS (planned) / Polling (current) | `/v1/collaboration/canvases/[id]/viewport` | per design §4.1-4.2 |
| Share (URL) | GET | `/v1/collaboration/canvases/[id]/share-link` | V0.1 |
| Export PNG | POST | `/v1/collaboration/canvases/[id]/export/png` | per `canvas-share-export-001.md` |
| Audit | GET | `/v1/collaboration/canvases/[id]/audit` | per design §4.1 |
| Search | GET | `/v1/search?q=&type=canvas` | per design §4.8 |
| Comment pin | GET/POST | `/v1/collaboration/canvases/[id]/comments` | per design §1.3 |

#### 6.2.2 双核心扩展 API (≥ 10 个新增, 砍掉 Miro 集成 ~17 个)

| API | 方法 | 路径 | 引用 |
|---|---|---|---|
| Agent status sync | WS | `wss://agent-status/canvases/[id]` | AGENT A3.1 |
| Agent operation (启停/重启) | POST | `/v1/agent-sessions/[id]/start` `/stop` `/restart` | AGENT A6.1-6.2 |
| Agent token usage | GET | `/v1/agent-sessions/[id]/token-usage` | AGENT A7.1-7.2 |
| Gamification reward | POST | `/v1/gamify/rewards` | GAMIFY G2.1-2.4 |
| **ARG edges CRUD** | GET/POST/PATCH/DELETE | `/v1/arg/edges` (per `SRS-AGENT-RELATIONSHIP-001` §6 UC-01) | AGENT A11.1-11.2 |
| **ARG template instantiate** | POST | `/v1/arg/templates/instantiate` (per UC-05) | AGENT A11.4 |
| **ARG sync status** | WS | `wss://arg-sync/canvases/[id]` (per §4.4) | AGENT A11.9 |
| **ARG audit log** | GET | `/v1/arg/edges/audit?edge_id=` (per 守门 #13 Transaction) | AGENT A11.5 |
| **ARG relationship view (Agent View tab)** | GET | `/v1/arg/view?agent_session_id=` (per §1.3) | AGENT A11.10 |
| **多人编辑 Realtime 通道 (per 17:34 JST v0.63 反转)** | WS | `wss://canvas-collab/canvases/[id]` (扩展 V0.1 §4.1 模式 A Realtime 通道) | AGENT A12.1 + A12.3 |
| **多人编辑 presence (实时 cursor)** | WS | `wss://canvas-presence/canvases/[id]` (扩展 V0.1 §4.6 PresenceCursor 升级) | AGENT A12.2 |
| **多人编辑 follow mode** | POST | `/v1/collaboration/canvases/[id]/follow?user_id=` (新) | AGENT A12.4 |
| **多人编辑 comment thread + @ 提醒** | POST | `/v1/collaboration/canvases/[id]/comments/[cid]/thread` + `/v1/notifications/@` (扩展 V0.1) | AGENT A12.5 |
| **多人编辑 audit log (新增表 `canvas_multi_user_audit`)** | GET | `/v1/collaboration/canvases/[id]/multi-user-audit` (新, per 守门 #13 Transaction append-only) | AGENT A12.8 |
| Score points | GET/POST | `/v1/gamify/scores` | GAMIFY G3.1-3.2 |
| Level up | WS | `wss://gamify-level/canvases/[id]` | GAMIFY G4.1-4.2 |
| Sticky note 聚类 (AI, mock) | POST | `/v1/gamify/sticky-notes/cluster` | GAMIFY G5.1 |
| Dot voting | POST | `/v1/gamify/votes` | GAMIFY G6.1-6.2 |
| Reaction | POST | `/v1/gamify/reactions` | GAMIFY G7.1 |
| Confetti trigger | WS | `wss://gamify-confetti/canvases/[id]` | GAMIFY G8.1 |
| Leaderboard | GET | `/v1/gamify/leaderboard?scope=workspace&metric=token` | GAMIFY G9.1-9.2 |
| Daily challenge | GET | `/v1/gamify/daily-challenge` | GAMIFY G10.1-10.2 |
| Power-up / Inventory | GET/POST | `/v1/gamify/powerups` `/v1/gamify/inventory` | GAMIFY G11.1-11.2 |

#### 6.2.3 砍掉 API (Miro 集成, per 17:08 JST 避免过度冗余)

| 砍掉 API | 砍掉理由 |
|---|---|
| ~~Realtime sync (多人编辑)~~ | Miro 通用, 砍掉 |
| ~~Presence cursor (多人)~~ | Miro 通用, 砍掉 |
| ~~Vote (workshop) / Timer (workshop)~~ | Miro 通用, 砍掉 (留 dot voting per user) |
| ~~Export PDF / Word / Excel / CSV~~ | Miro 通用, 砍掉 (留 PNG) |
| ~~Share with permission / Link with password / Expire~~ | Miro 通用, 砍掉 (留 URL copy) |
| ~~Version history / Branching / Restore~~ | Miro 通用, 砍掉 (用 Git worktree) |
| ~~Apps OAuth (Slack / Jira / Asana / Figma / Notion / GitHub / Zoom)~~ | Miro 通用, 砍掉 |
| ~~Webhook (canvas event → 3rd-party)~~ | Miro 通用, 砍掉 |
| ~~Touch / Stylus / Offline~~ | Miro 通用, 砍掉 |

### 6.3 25 module 联动接口 (per V0.1 §4 + 双核心扩展)

| 25 module | 接口 | 画布对接 | 双核心扩展 | 引用 |
|---|---|---|---|---|
| work-item | `/v1/work-items/[id]` | drag → element / dblclick → detail | AGENT A5 1:N 关联 | V0.1 §4.2-4.3 + AGENT A5 |
| worktree | `/v1/worktrees/[id]` | node 颜色同步 | AGENT A4 1:N 关联 | V0.1 §4.4 + AGENT A4 |
| agent | `/v1/agent-sessions/[id]` | PresenceCursor 升级 | AGENT A1-A8 | V0.1 §4.6 + AGENT |
| relation | `/v1/relations/[id]` | connector 渲染 | AGENT A2 handoff | V0.1 §4.5 + AGENT A2 |
| comment | `/v1/comments/[id]` | comment_pin 挂载 | GAMIFY G7 reaction 复用 | V0.1 + GAMIFY G7 |
| automation | `/v1/automations/[id]` | rule 触发 | GAMIFY G2 reward rule | V0.1 §4.7 + GAMIFY G2 |
| audit | `/v1/audit?type=canvas` | 操作日志 | AGENT A3.2 状态 audit | V0.1 §4.1 + AGENT A3.2 |
| search | `/v1/search?q=&type=canvas` | 跳 canvas | GAMIFY G5 聚类 (AI mock) | V0.1 §4.8 + GAMIFY G5 |
| notification | `/v1/notifications` | @ 提醒 | GAMIFY G2.2 reward 通知 | V0.1 + GAMIFY G2.2 |
| agent-game (V0.1) | Roguelike / Manga / Theme / Settings | — | GAMIFY G12 集成 | per V0.1 game 4 份 PHASE + GAMIFY G12 |

### 6.4 V0.1 game 集成接口 (per GAMIFY G12, 双核心之 2 关键)

| V0.1 组件 | 接口 | 画布对接 | 引用 |
|---|---|---|---|
| RoguelikeCanvas | `/v1/agent-game/roguelike` | 画布节点 = 角色 / 怪物 / 道具 | GAMIFY G12.1 + `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` v0.1 |
| AgentManga | `/v1/agent-game/manga` | 画布节点 = 漫画风格主题 | GAMIFY G12.2 + `PHASE-AGENT-MANGA-IMPL-REPORT.md` v0.1 |
| AgentGame | `/v1/agent-game/core` | RPG 核心 (avatar / level / xp) | GAMIFY G1 + `PHASE-AGENT-GAME-IMPL-REPORT.md` v0.1 |
| AgentTheme | `/v1/agent-game/theme` | 主题切换 | per `PHASE-AGENT-THEME-IMPL-REPORT.md` v0.1 |
| AgentSettingsTab | `/v1/agent-game/settings` | 玩家设置 | AGENT A10 + `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1 |

### 6.5 错误处理 (per 守门 #1 v1a + 守门 #5)

| 错误类型 | 处理 | 引用 |
|---|---|---|
| 401 Authentication failed | 跨 session 续, **不算 timeout, max 2 retries**, Ulysses 验证 $env:GHCR_PAT | per 守门 #1 v1a 2026-09-03 11:07 JST 实证 |
| Recv failure / Connect failed | max 2 retries, 30s-2min 后常恢复, 不连续 retry | per 守门 #1 v1a |
| Timeout (5s+ ) | max 2 retries | per 守门 #1 v1a |
| 5xx Server Error | 1 retry, 5min 后, 失败给用户提示 | per 守门 #1 v1a |
| 4xx Client Error (非 401) | 不 retry, 直接给用户错误 | per 守门 #1 v1a |
| 凭据 (agent 启停 / reward 解锁) | stdin pipe 模式 (`$env:VAR | sudo -S ...`), 密码不上命令行, 不打印 | per 守门 #5 hard ban |

---

## §7 约束 / 依赖 / 风险

### 7.1 约束 (per 守门 #11 缺标比错标)

| 约束 | 说明 | 引用 |
|---|---|---|
| **核心 = 管理 agent + 游戏化** | 17:08 JST Ulysses 拍板, 不可偏离 | per 17:08 JST 用户发令 |
| **不破坏 25 module 1:1** | 不新增第 26 module, canvas 是 collaboration 域强化 | per `frontend-canvas-design.md` §1.2 + 1.3 |
| **不破坏 V0.1 MVP** | 14 element + 4 frame + 8 connector + 9 e2e 守门 不重构 | per V0.1 design + code |
| **不写 Miro 通用功能** | 12 大类中 11 类 (除 #1 + #2 部分) 全部砍掉, 留 P3+ | per 17:08 JST + §1.4 |
| **0 不安全代码** | per 守门 #7 hard ban | per AGENTS.md §4 #7 |
| **PowerShell only** | 不使用 bash / sh, Windows 平台硬约束 | per 系统约束 |
| **环境变量安全** | 不打印 env 值, 仅 invoke, 凭据走 stdin pipe | per 守门 #5 |
| **代签规则** | Mavis 接手代签 Ulysses (per 守门 #14 v3 + 9/8 15:19 第 6 次强化) | per AGENTS.md §4 #10 + #14 v3 |
| **5 域独立 Lead ≠ Star 22 DDD bounded context** | per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer | per AGENTS.md §4 #3 + #4 #1 |
| **Mavis 临时代签, 真人到位后追溯签字** | per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** (per 守门 #1 禁回溯叙事) | per AGENTS.md §4 #3 + #14 v2 |
| **DB W/T/M 三類横展** | GAMIFY G11 必含 Work/Transaction/Master, 100% 表覆盖 | per 守门 #13 |
| **AI mock 接口** | 不引入第三方 API 凭据 (per 守门 #23 v2 ai-edit-mode=本地 mock) | per GAMIFY G5 |

### 7.2 依赖 (跨 2 专题 + 25 module + V0.1 game)

| 依赖 | 说明 | 引用 |
|---|---|---|
| V0.1 canvas 实装 | 14 element + 4 frame + 8 connector + 9 e2e 守门 | V0.1 design + code + 2 e2e spec |
| V0.1 5 联动 (WorkItem/Worktree/Relation/Comment/Search URL) | 跨块接口基线 | per `frontend-canvas-design.md` §4 |
| V0.1 StatusPill 60+ 色码 | status 同步基线 | per design §3.4 + ADR-FE-013 |
| V0.1 StateMachineDiagram 5×4 grid + bezier | layout 算法复用 | per `frontend/src/lib/agent-view/layout.ts` |
| 25 module 9 类 (work-item / worktree / agent / relation / comment / automation / audit / search / notification) | 联动基础 | per `frontend-canvas-design.md` §4 联动矩阵 |
| html2canvas 1.4.1 | V0.1 PNG export 依赖 (已知 foreignObject 限制) | per `canvas-share-export-001.md` V0.1 |
| react-hot-toast 2.4.1 | V0.1 toast 依赖 | per V0.1 |
| Next.js 14 + React 18 + zustand + lucide-react | 前端栈 | per `frontend/package.json` |
| V0.1 game 4 份 (Roguelike + Manga + Theme + Settings) | GAMIFY G12 集成 | per V0.1 game 4 份 PHASE |
| **AGENT A3 实时状态同步 WebSocket** 选型 | P0 阻塞 | AGENT §7 风险 |
| **AGENT A4 agent ↔ worktree 1:N 关系 schema** | P0 阻塞 (per `SRS-AGENT-VIEW-001.md` §3 缺口 #4) | AGENT §7 风险 |
| **GAMIFY G5 sticky note 聚类 AI** mock 接口 | per 守门 #23 v2, 真实 LLM 留 P2 | GAMIFY §7 风险 |
| **GAMIFY G11 power-up / inventory** DB W/T/M | per 守门 #13 | GAMIFY §7 风险 |
| **OAuth 第三方** | **砍掉, 留 P3+** (per 17:08 JST 避免过度冗余) | — |

### 7.3 风险 (per 守门 #11 缺标比错标, 跨 2 专题)

| 编号 | 风险 | 影响 | 缓解 | 引用 |
|---|---|---|---|---|
| R-1 | AGENT A3 实时状态同步 WebSocket 选型错误 | P0 阻塞, 整个 agent 管理能力延期 | 拍板前 1 周内做 P0-1 概念验证 (WS 集成 + 100 agent 同步测试) | AGENT §7 风险 |
| R-2 | AGENT A4 agent ↔ worktree 1:N 关系 schema 缺 (per SRS-AGENT-VIEW §3 缺口 #4) | P0 阻塞, 关联关系延期 | 拍板前 schema 补全, 跨 25 module 协调 | AGENT §7 风险 |
| R-3 | GAMIFY G5 sticky note 聚类 AI 走 mock 接口, 真实 LLM 引入第三方凭据 (per 守门 #23 v2 禁止) | 游戏化 AI 能力延期 | 短期 mock, 中期评估本地模型 (Ollama) | GAMIFY §7 风险 |
| R-4 | GAMIFY G8 confetti 动画库选型错误 (CSS / Lottie) | 用户体验差 | 短期 CSS, 中期 Lottie 评估 | GAMIFY §7 风险 |
| R-5 | GAMIFY G11 power-up / inventory 持久化 DB W/T/M 分类错误 | 数据合规风险 | 拍板前 1 周内做 P1-1 概念验证, 跨守门 #13 派 DDD Review Lead 确认 | GAMIFY §7 风险 |
| R-6 | 5 域 Lead 真人未到位, 跨域编排决策延后 (AGENT A2.2 跨 5 域拓扑) | P0 跨域决策需 Mavis 临时代签 | per 守门 #14 v2 + 9/3 11:35 JST 拍板 B, 真人到位后追溯签字 | per AGENTS.md §4 #3 + #14 v2 |
| R-7 | V0.1 game 4 份 (Roguelike / Manga / Theme / Settings) 跟 GAMIFY G12 集成接口设计 | P0 集成延期 | 拍板前 1 周内做 P0-1 概念验证 (Roguelike 集成 + 5 节点画布测试) | GAMIFY G12 |
| R-8 | 401 Authentication failed 跨 session 续 (per 守门 #1 v1a 实证) | agent 启停 + reward 通知 中断 | max 2 retries, 跨 session 续, Ulysses 验证 $env:GHCR_PAT | per 守门 #1 v1a |
| R-9 | 双核心 60 项 落地需 ~6-12 SRE·周 (per STAR-OLU-001) | Token OLU 超预算 | 12 个月分阶段落地, 优先级 P0 → P1 → P2 → P3 | per §4.4 P0 清单 |
| R-10 | 2 份 SRS 撰写 2 子代理并行, 跨块接口一致性风险 | 文档割裂 | 本总册 §4.5 跨块接口矩阵 + root 校对 (per 守门 #9 v27 collect_output) | per 守门 #9 v27 |
| R-11 | 旧方向 4 文件 (SRS-CANVAS-001 v0.1 + 3 brief) 撤回后, 工作树残留 → commit 污染 | 1 commit 含撤回 + 新写 3 文件 | 撤回 + 新写在 1 commit 内 (per 守门 #1 v15 docs 同步饱和) | per 守门 #1 v15 |
| R-12 | Miro 80% 能力被砍, 团队可能反弹 (workshop / 集成 / 移动) | 范围争议 | 17:08 JST 拍板是"避免过度冗余", 砍掉已确认; P3+ 评估补齐 | per 17:08 JST 拍板 |

---

## §8 验收标准 (跨 2 专题 + 本总册)

### 8.1 已有 V0.1 MVP 验收基线 (per e2e 实测 9/9 全过)

| 编号 | 验收项 | 当前状态 | 引用 |
|---|---|---|---|
| AC-V0.1-1 | 6 e2e 守门 (pan / zoom / fit / highlight / delete / minimap) | 6/6 全过 | per `canvas-view.spec.ts` (9/4 拍板) |
| AC-V0.1-2 | 3 e2e 守门 (share clipboard / export PNG / toast) | 3/3 全过 | per `canvas-share-export.spec.ts` (9/4 拍板) |
| AC-V0.1-3 | 14 element + 4 frame + 8 connector mock 渲染 | 9/4 落档 | per `seed.ts:418` |
| AC-V0.1-4 | 5 联动 (WorkItem/Worktree/Relation/Comment/Search URL) | V0.1 落档 | per `frontend-canvas-design.md` §4 |
| AC-V0.1-5 | StatusPill 60+ 色码同步 | V0.1 落档 | per design §3.4 + ADR-FE-013 |

### 8.2 3 份 SRS 撰写验收 (per 守门 #9 v27)

| 编号 | 验收项 | 当前状态 |
|---|---|---|
| AC-SRS-1 | `SRS-CANVAS-AGENT-001.md` 9-13 段齐全, **46 项**展开 (A1-A12), 字节数 ≥ **40K**, **A11 必含 4 表 W/T/M 三類横展 + A12.8 多人 audit log W/T/M + 不写 Miro 通用功能** (per 守门 #13, 撤回 17:08 JST 砍多人编辑决定 per 17:34 JST v0.63 反转) | worker 子代理 1 v1.2 重写 (bg_eb11f9e3 重派, 17:34 JST) |
| AC-SRS-2 | `SRS-CANVAS-GAMIFY-001.md` 9 段齐全, 32 项展开, 字节数 ≥ 30K, **G11 W/T/M 完整 + G5 AI mock 标注** + **不写 Miro 通用功能** | worker 子代理 2 撰写中 (bg_84cf0613, 17:20 JST 派, 不变) |
| AC-SRS-3 | `SRS-CANVAS-001.md` (本总册 v1.1) 9 段齐全, 双核心 **78 项**索引 (38+8+32) + 跨块接口 + 共享约束, 字节数 ≥ 50K | root 撰写完成 (v1.0 + 17:22 JST A11 二次更新 + 17:34 JST A12 三次更新 v0.63 反转) |

### 8.3 后续 P0 阶段验收 (3 个月内, 拍板后启动)

| 编号 | 验收项 | 目标 | 引用 |
|---|---|---|---|
| AC-P0-A1 | agent_node 完整卡渲染 | 14 element → agent_node 升级, StatusPill 色码同步 | AGENT A1 |
| AC-P0-A2 | 5 域拓扑图 + 1:N handoff connector | 5 Frame 分组 + handoff connector 实时 | AGENT A2 |
| AC-P0-A3 | 14 状态机实时色码同步 | 延迟 < 200ms (P95), 0 数据丢失 | AGENT A3 |
| AC-P0-A4 | agent 启停 / 重启 操作 | 权限: 5 域 Lead / SRE Lead, 401 retry 跨 session 续 | AGENT A6 |
| AC-P0-G1 | avatar / level / xp / skill_tree / class / badge 6 节点 + 积分 + 升级 | 节点渲染 + score 累计 + xp → level 公式 | GAMIFY G1 + G3 + G4 |
| AC-P0-G2 | dot voting + confetti | 每用户 5 票, 实时显示, confetti 触发 | GAMIFY G6 + G8 |
| AC-P0-G3 | V0.1 Roguelike / Manga 集成 | 画布节点 = 角色 / 怪物 / 道具 / 漫画主题 | GAMIFY G12 |

### 8.4 后续 P1 阶段验收 (6 个月内)

(略, 2 专题各自 §8 详细列出)

---

## §9 修订履历

### 9.1 本总册 (SRS-CANVAS-001) 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 (撤回) | 2026-09-10 17:10 JST | Ulysses — Mavis 接手 | 旧方向: Miro 全功能 12 大类 50 项, 3 专题拆分 (47.5KB) | Ulysses 17:00 JST 拍板 (后撤回) |
| **v1.0 (方向重置版)** | **2026-09-10 17:20 JST** | **Ulysses — Mavis 接手 (per 守门 #14 v3)** | **新方向: 双核心 = 管理 agent + 游戏化, 2 专题拆分, 砍掉 Miro 通用功能, 字节数目标 ~20K (vs v0.1 47.5K 砍 58%)** | **Ulysses 17:08 JST 拍板"管理 agent 和游戏化, 避免过度冗余"** |
| **v1.1 (当前, 三次更新版)** | **2026-09-10 17:34 JST** | **Ulysses — Mavis 接手 (per 守门 #14 v3)** | **v0.63 反转: 撤回 17:08 JST 砍多人编辑决定, A12 多人编辑 8 项新增, AGENT 38 → 46 项, 双核心 70 → 78 项, 31 P0 → 36 P0** | **Ulysses 17:34 JST 拍板"多人编辑是要的"** |

### 9.2 平行 2 专题 SRS 修订履历 (独立跟踪, 待 2 子代理返回后填)

| 文书 | 版本 | 修订人 | 修订内容 | 触发 | 状态 |
|---|---|---|---|---|---|
| `SRS-CANVAS-AGENT-001` | v1.0 (待) | Ulysses — Mavis 接手 | 28 项展开 (agent 管理) | 本拍板派生 (per 17:08 JST 方向重置) | 撰写中 (worker 子代理 1, bg_fc33dfea) |
| `SRS-CANVAS-GAMIFY-001` | v1.0 (待) | Ulysses — Mavis 接手 | 32 项展开 (游戏化) | 本拍板派生 (per 17:08 JST 方向重置) | 撰写中 (worker 子代理 2, bg_84cf0613) |
| `SRS-CANVAS-INTEGRATION-001` (撤回) | n/a | n/a | 撤回理由: Miro 集成超出核心 | 17:08 JST 方向重置 | 改写为 deprecated placeholder (`docs/briefs/srs-canvas-integration-001.md` 1.1KB) |

### 9.3 跨拍板派生 (per 守门 #14 v2 跨域)

| 派生触发 | 文档 | 修订人 | 修订内容 |
|---|---|---|---|
| 2026-09-10 17:00 JST Ulysses 拍板"把这些归纳进需求文档, 子代理同步撰写" | 旧 v0.1 (撤回) | Mavis 接手 | 12 大类 50 项 Miro 差距归并 (撤回) |
| **2026-09-10 17:08 JST Ulysses 拍板"管理 agent 和游戏化, 避免过度冗余"** | **本总册 v1.0** | **Mavis 接手** | **双核心重置, 2 专题拆分, 砍 11 大类 Miro 通用功能** |
| **2026-09-10 17:21 JST Ulysses 拍板"画布内体现 agent 之间关系的图论构造"** | **本总册 v1.0 (二次更新) + AGENT brief v1.1** | **Mavis 接手** | **A11 ARG 子能力新增 (10 项), AGENT 28 → 38 项, 总 60 → 70 项; 引用 `SRS-AGENT-RELATIONSHIP-001.md` v0.1 (9/8 落档) 作为 A11 主源** |
| **2026-09-10 17:34 JST Ulysses 拍板"多人编辑是要的" (v0.63 反转)** | **本总册 v1.0 → v1.1 (三次更新) + AGENT brief v1.1 → v1.2** | **Mavis 接手** | **A12 多人编辑子能力新增 (8 项), AGENT 38 → 46 项, 总 70 → 78 项; 撤回 17:08 JST 砍多人编辑决定 (per 守门 #1 禁回溯叙事, v0.63 反转行显式标); 引用 `frontend-canvas-design.md` §4.1 模式 A Realtime 通道 + §4.6 PresenceCursor 升级** |
| 2026-08-27 19:39 JST 用户授权 (per 守门 #10 + #14 v3) | 本总册 v1.0 元数据 | Mavis 接手 | 修订人/审批者 代签规则 |
| 2026-08-31 22:45 JST Q1-D 拍板 | 本总册 §1.4 + §7.1 | Mavis 接手 | 5 域 Lead ≠ Star 22 DDD disclaimer |
| 2026-09-03 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D | 本总册 §7.1 | Mavis 接手 | Mavis 临时代签, 真人到位后追溯签字 |
| 2026-09-10 12:45 JST v0.62 反转 | 本总册 §2.4 | Mavis 接手 | 真人代签流程全部取消, 改为 Mavis 审核 决定 author=Ulysses |
| 2026-09-03 11:07 JST 401 实证 (per 守门 #1 v1a) | 本总册 §5.3 + §6.5 + §7.3 R-8 | Mavis 接手 | 401 retry 跨 session 续 规则 |
| 2026-09-01 18:30 JST 守门 #13 DB W/T/M | 本总册 §3.2 + §7.1 + GAMIFY §4.11 | Mavis 接手 | GAMIFY G11 power-up / inventory W/T/M 三類横展 |
| 2026-09-02 09:01 JST 守门 #23 v2 (per ai_edit_mock.py) | 本总册 §1.4 + §7.1 + GAMIFY §4.5 | Mavis 接手 | GAMIFY G5 sticky note 聚类 AI 走 mock, 真实 LLM 留 P2 |
| 2026-09-01 13:03 JST 守门 (envoy 偏好) | 不适用 | n/a | n/a (本次 SRS 撰写不涉及部署) |
| 2026-09-01 13:05 JST 守门 (envoy 独立 deployment) | 不适用 | n/a | n/a (本次 SRS 撰写不涉及部署) |

### 9.4 守门硬约束交叉引用 (本 SRS 撰写期)

| 守门 | 应用 | 引用 |
|---|---|---|
| 守门 #1 v15 docs 同步饱和 | 1 commit 4 文件, 不写空 docs commit | per AGENTS.md §4 #1 v15 |
| 守门 #1 v19 agent 交互 Python 化 | 0 子代理调用, 0 散落子代理产出 | per AGENTS.md §4 #1 v19 |
| 守门 #5 env 安全 hard ban | §6.5 接口 stdin pipe 模式 | per AGENTS.md §4 #5 |
| 守门 #9 #3 子代理 RPC 不可靠 | 0 子代理调用 (实装期), 仅用 task tool 派 2 worker | per AGENTS.md §4 #9 #3 |
| 守门 #9 v20 子代理 dispatch 必先 brief | 2 份 brief 落 `docs/briefs/srs-canvas-{agent,gamify}-001.md` | per AGENTS.md §4 #9 v20 |
| 守门 #9 v27 RPC 失败 fallback 3 段 | 子代理 invoke → verify → collect_output | per AGENTS.md §4.1.1 v27 |
| 守门 #10 代签规则 | author=Ulysses (per 8/27 19:39 JST 授权) | per AGENTS.md §4 #10 |
| 守门 #11 缺标比错标 | 2 专题 SRS §3 / §7 / §8 显式列已知缺口 | per AGENTS.md §4 #11 |
| 守门 #12 AI 文档治理 | BAS 引用必 git log --follow 实证, 禁回溯叙事 | per AGENTS.md §4 #12 |
| 守门 #12 v21 docs 同步必更新 §4 + registry | 1 任务卡 + 1 索引追加 (root 统一) | per AGENTS.md §4 #12 v21 |
| 守门 #13 DB W/T/M | GAMIFY G11 必含 三類横展 | per AGENTS.md §4 #13 |
| 守门 #14 v3 Mavis 永久代签 | 修订人/审批者代签规则 | per AGENTS.md §4 #14 v3 |
| 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D | Mavis 临时代签, 真人到位后追溯签字 | per AGENTS.md §4 #14 v2 |
| 守门 #15 docs 同步饱和 | 1 commit 落 3 文件 (1 总册 + 2 专题) | per AGENTS.md §4 #15 |
| 守门 #23 v2 AI 第三方 API 禁止 | GAMIFY G5 走 mock, 真实 LLM 留 P2 | per AGENTS.md §4 #23 v2 |

### 9.5 token OLU 估算 (per 守门 #4 + STAR-OLU-001 v0.1)

| 阶段 | Token 估算 | 引用 |
|---|---|---|
| 撤回 4 旧文件 + 落 2 新 brief | ~0.05M | root 撤回 (本 turn) |
| 2 专题 SRS 撰写 (2 子代理并行) | ~0.7M (AGENT 0.4M + GAMIFY 0.3M) | per brief §0 token 预算 |
| 总册 SRS 撰写 (root) | ~0.05M | root (本 turn) |
| 跨块校对 + 修订历史 + 1 commit | ~0.05M | root |
| **本次撰写合计** | **~0.75M** (1 SRE·周 ≈ 1.2M, 在预算内) | per STAR-OLU-001 v0.1 |
| 后续 P0 阶段 (3 个月内, 36 项 P0) | ~3-5M | per §4.4 P0 清单 |
| 双核心 78 项 全部落地 (12 个月+) | ~13-19M (13-19 SRE·周, per STAR-OLU-001) | per §4.1 双核心 78 项 |

### 9.6 撤回 → 重写 工作流 (per 守门 #1 v15 docs 同步饱和)

| 阶段 | 时间 | 操作 |
|---|---|---|
| 1 | 17:00 JST | Ulysses 拍板 v1: 12 大类 50 项 Miro 全面对标 (旧方向) |
| 2 | 17:00-17:08 JST | root 落 3 brief + 派 3 子代理 + 写 47.5K 总册 |
| 3 | 17:08 JST | Ulysses 拍板 v2: 双核心 = agent + 游戏化 (新方向, 撤回 v1) |
| 4 | 17:08-17:17 JST | root 立即停 3 子代理 (避免浪费) + ask_user 拍板 (推荐 1 总册 + 2 专题) |
| 5 | 17:17 JST | Ulysses 选 A: 1 总册 + 2 专题, 立即执行 |
| 6 | 17:17-17:20 JST | root 撤回 4 旧文件 + 落 2 新 brief + 派 2 新子代理 + 重写总册 v1.0 (本 turn) |
| 7 | (待 2 子代理返回) | root 校对 + verify + 1 commit 落 3 文件 (per 守门 #1 v15 + 守门 #9 v27) |

---

> **撰写完成**: 2026-09-10 17:20 JST, root session mvs_942987595a124037901d37205a548e6f
> **2 专题 SRS 状态**: 撰写中, 等待 2 worker 子代理返回后 root 校对 + 1 commit 3 文件落档 (per 守门 #1 v15 + 守门 #9 v27)
> **下次拍板触发**: 2 专题返回后, root 自动 commit 落档 (per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
