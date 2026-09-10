# SRS-CANVAS-GAMIFY-001

> **无限画布游戏化域要件定義書 v0.1** (per 日本 IPA SEC 標準 / 要件定義書 テンプレート)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联基本設計書: (留空, 待 P3-D.6 实装阶段落档)
> - 关联実装報告: (留空, 待 P3-D.6 实装阶段落档)
> - 上位要件: `SRS-CANVAS-001` v0.1 (总册, 双核心索引, root 重写), `SRS-CANVAS-AGENT-001` v0.1 (双核心之 1: agent 管理, 子代理 1 落档), `SRS-AGENT-VIEW-001.md` v1.0 (V0.1 agent 视图)
> - 平行 view: `SRS-AGENT-RELATIONSHIP-001.md` v0.1 (ARG, 9/8 落档, 关系层)
> - 拍板来源: 2026-09-10 17:08 JST Ulysses 拍板"审核时的观点是要明确, 我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" — 双核心之 2: 游戏化
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3 Mavis 永久代签 + 9/8 15:19 JST 第 6 次强化)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008)
> - 日期: 2026-09-10 JST
> - 受众: 详细設計工程师 / 架构审查者 / UI/UX 设计师 / SRE / 5 域 Lead 真人
> - 受众范围 disclaimer: **5 域独立 Lead ≠ Star 22 DDD bounded context** (per 2026-08-31 22:45 JST Q1-D 拍板: 5 域 Lead 是 RGS 仓历史治理命名, 不建立业务子域↔DDD 映射)

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-CANVAS-GAMIFY-001 |
| 文书名 | 无限画布游戏化域要件定義書 |
| 版本 | v0.1 |
| 作成日 | 2026-09-10 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 commit | (留空, root 统一 commit 时填) |
| 关联文档 | `SRS-CANVAS-001.md` v0.1 (总册, root 重写) + `SRS-CANVAS-AGENT-001.md` v0.1 (子代理 1) + `SRS-AGENT-VIEW-001.md` v1.0 (V0.1) + `frontend-canvas-design.md` v0.1 + V0.1 game 4 份 PHASE 报告 |
| 拍板来源 | 2026-09-10 17:08 JST Ulysses 拍板原文"审核时的观点是要明确, 我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" |
| 守门合规 | 守门 #1 + #3 + #5 + #6 + #7 + #9 + #10 + #11 + #12 + #13 + #14 v2 + #15 + #19 + #23 v2 全过 (本文档为需求文档, 守门 #1 v25 cargo test --workspace -j 4 不需要跑) |
| 模板结构 | 9 段严格按 brief §1.3 (跟 SRS-AGENT-VIEW-001 同形, §0 文档信息 + §9 修订历史拆开) |
| 子能力 | 12 子能力 (G1-G12) × 32 项 (per brief §1.1) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 (情報システム等の整備に係る標準的指針) 制定 STAR 平台 **无限画布游戏化域**的需求规格说明书, 涵盖功能/非功能/数据/接口/约束/风险/验收等维度, 作为后续基本设计 (`BD-CANVAS-GAMIFY-001.md`) / 详细设计 / 实装 / 测试 / 验收的唯一依据。

**核心方向锚点 (per 2026-09-10 17:08 JST Ulysses 拍板)**: "我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" — 双核心之 2 是**游戏化** (本 SRS), 不是 Miro 通用功能 (12 diagram / 模板 / 编辑效率 / Slack-Jira / 移动 / a11y 全部砍掉, 见 brief §2)。

**本 SRS 不重写 V0.1 已实装内容**:
- 已有 `PHASE-AGENT-GAME-IMPL-REPORT.md` v0.1 (commit `15fdcf5`, 9/5 落地, 49 tests pass)
- 已有 `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` v0.1 (commit `072503e`, 9/5 落地, 35 tests 净增)
- 已有 `PHASE-AGENT-MANGA-IMPL-REPORT.md` v0.1 (commit `1d84e36`, 9/5 落地, 14 tests 净增)
- 已有 `PHASE-AGENT-THEME-IMPL-REPORT.md` v0.1 (commit 落地中, 11 tests 净增)
- 已有 `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1 (commit 落地中, 16 tests 净增)
- 已有 `frontend/src/components/agent-game/` 5 个模块 (types / leveling / perks / mapgen / movement / theme / theme-tokens / settings / characters / enemies / Decorations / RoguelikeCanvas / GameHUD / PerkPicker / DeathModal / useAgentGame / AgentSettingsTab)

本 SRS 仅定义**新增** 32 项游戏化需求, 跟 V0.1 已实装功能做接口引用而非重写。

### 1.2 背景 (用户痛点)

STAR 平台 9/5 落地 Agent view (per `SRS-AGENT-VIEW-001.md` v1.0) 后, 用户在 2026-09-10 17:08 JST 反馈: "审核时的观点是要明确, 我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" — 现有 25 module 框架下, 无限画布是 collaboration 域强化 (per `frontend-canvas-design.md` v0.1), 但**画布上的游戏化机制仍为空白**:

1. **画布节点种类不够** — 现有 10 种 `CanvasElementKind` (sticky_note / text / shape / image / embed / work_item_card / worktree_node / agent_cursor / automation_node / comment_pin, per `frontend/src/types/ids.ts` line 686-696) 缺游戏化 8 种 (avatar / level / xp / skill_tree / class / badge / quest / inventory)
2. **互动机制空白** — 画布只有"展示 + 编辑", 缺投票 (dot voting) / 表情 (reaction) / 庆祝 (confetti) / 排行榜 (leaderboard) 等互动工具
3. **奖励成就系统空缺** — 完成 work-item 没有视觉反馈, 没有 RPG 风格的"升级" / "解锁" / "成就" 激励
4. **AI 智能辅助缺位** — sticky note 多了没法归类, 没有 AI 自动聚类 (走 mock 接口, per 守门 #23 v2 不引入第三方 LLM)
5. **游戏化与已有 RPG 缺协同** — V0.1 Roguelike + Manga + Settings 已实装, 但**画布节点**没跟它们集成 (画布节点不映射到角色 / 怪物 / 漫画风格)

### 1.3 包含范围 (In-Scope)

12 子能力 × 32 项 (per brief §1.1 + §5 详细):

| 子能力 | 范围 | 项数 | 优先级 |
|---|---|---|---|
| G1 gamification 节点 | 8 种新 element kind (avatar / level / xp / skill_tree / class / badge / quest / inventory) | 8 | G1.1-1.6 P0, G1.7-1.8 P1 |
| G2 reward / achievement | 解锁条件 + 通知 + 画布特效 + 徽章自动授予 | 4 | P1 |
| G3 score / points | 操作积分规则 + 累计 + 排行榜入口 | 3 | G3.1-3.2 P0, G3.3 P1 |
| G4 leveling / skill tree | xp→level 公式 (默认线性) + 升级动画 + skill tree 解锁 | 3 | G4.1-4.2 P0, G4.3 P1 |
| G5 sticky note 聚类 (AI mock) | 选中 N → K 主题 + 聚类结果可视化 | 2 | P1 |
| G6 dot voting | 每用户 N 票 + 投票结果实时显示 | 2 | P0 |
| G7 reaction | emoji 表情回应 (8-12 种, 3s 淡出) | 1 | P1 |
| G8 confetti | 庆祝特效 (完成 / 解锁 / 升级触发) | 1 | P0 |
| G9 leaderboard | per workspace + per tenant 排行榜 | 2 | G9.1 P1, G9.2 P2 |
| G10 daily challenge / streak | 每日任务 + 连续天数 | 2 | P1 |
| G11 power-up / inventory | 道具 + 物品栏 (**必含 W/T/M 三類横展**, per 守门 #13) | 2 | P2 |
| G12 跟 agent-game V0.1 集成 | Roguelike + Manga 画布节点映射 | 2 | P0 |
| **合计** | | **32** | |

### 1.4 不包含范围 (Out-of-Scope, per brief §2)

| 类别 | 处理方 |
|---|---|
| 双核心之 1: agent 管理 (28 项) | 子代理 1 (`SRS-CANVAS-AGENT-001.md`) |
| 总册 `SRS-CANVAS-001.md` (双核心索引 + 跨块接口 + 共享约束) | root (重写) |
| Miro 12 种 diagram (Flowchart / BPMN / ER / Wireframe / Kanban / Sequence) | ❌ 砍掉, 留 P3+ |
| Miro 编辑效率 (Undo / Group / Box select / Copy-Paste / Align / Distribute) | ❌ 部分砍掉, **Undo 留下** (基础画布能力, 留 P1, 不在本 SRS), Group / Box select / Copy-Paste 砍掉 |
| Miro 模板库 (2500+) | ❌ 砍掉 |
| Miro AI 能力 (文本生成 diagram / 翻译 / 图像识别) | ❌ 部分砍掉, **sticky note 聚类留下** (本 SRS G5) |
| Miro Tables / Chart widget / Form | ❌ 砍掉 |
| Miro 协作 (多人编辑 / 实时 cursor / 评论) | ❌ 砍掉, 已有 PresenceCursor (per `frontend-canvas-design.md` §4.6) 不重写 |
| Miro 演示 (Frame as slide / Guided Tour) | ❌ 砍掉 |
| Miro 互动通用 (Timer workshop 用 / Cursor chat / Async video) | ❌ 部分砍掉, **Reaction / Confetti / Dot voting 留下** (本 SRS G6-G8) |
| Miro 导出 (PDF / Word / Excel / CSV) | ❌ 砍掉, 仅留 PNG (V0.1 已实装) |
| Miro 集成 (Slack / Jira / Asana / Figma / GitHub) | ❌ 砍掉 |
| Miro 版本 (Version history / Branching) | ❌ 砍掉 |
| Miro 移动 (iOS / Android / Touch) | ❌ 砍掉 |
| Miro 完整 a11y (WCAG 2.1 AA) | ❌ 砍掉, 仅基础键盘可达 |
| 基础画布能力 (pan/zoom/select/element/connector/frame) | ✅ V0.1 已有, 不重写 |
| 已有 game (agent-game / roguelike / manga / settings) | ✅ V0.1 已实装, 仅画布集成引用, 不重写功能 |

### 1.5 用户故事 (User Stories, ≥ 19 个)

| 编号 | 角色 | 故事 | 优先级 |
|---|---|---|---|
| US-1 | 项目经理 (Ulysses) | 作为 PM, 我希望在画布上看到自己 agent 的 avatar 节点, 一眼分辨 Lv 5 跟 Lv 1 的视觉差异 (色/大小/光环) | P0 |
| US-2 | PM | 作为 PM, 我希望完成 work-item 时画布自动触发 confetti 特效 + 升级动画 + 通知 toast, 强化成就感 | P0 |
| US-3 | PM | 作为 PM, 我希望给 sticky_note 投票 (dot voting), 5 票/session, 票数实时显示在节点右上角 badge | P0 |
| US-4 | PM | 作为 PM, 我希望给任意 element 加 emoji reaction (👍 ❤️ 🎉 等 8-12 种), 短时 3s 淡出动画 | P1 |
| US-5 | PM | 作为 PM, 我希望选中 5+ 张 sticky_note, AI 帮我聚类成 3 个主题 (K 可调), 自动生成主题 Frame + connector | P1 |
| US-6 | 5 域 Lead (未来真人到位) | 作为 Lead, 我希望在画布上看到我团队的 per workspace 排行榜 (token / 任务完成数 / vote 数量), 激励竞争 | P1 |
| US-7 | Dev | 作为 Dev, 我希望给 agent 配 RPG 元素 (职业 / 等级 / 经验值 / 技能树 / 徽章 / 任务 / 物品栏), 玩起来更投入 | P0 |
| US-8 | Dev | 作为 Dev, 我希望 xp→level 公式可见 (默认线性 `level = sqrt(xp / 100)`), 升级时弹通知 | P0 |
| US-9 | Dev | 作为 Dev, 我希望 skill tree 在 level 达解锁条件时自动开放新技能, 视觉高亮 | P1 |
| US-10 | Dev | 作为 Dev, 我希望用 power-up 道具 (双倍积分 / 自动聚类 / 隐身模式) 限时加成, 物品栏能看到剩余时长 | P2 |
| US-11 | PM | 作为 PM, 我希望完成 daily challenge (每日 3 个任务) 拿 reward, streak 7 天再拿大奖励, 中断清零 | P1 |
| US-12 | PM | 作为 PM, 我希望把画布上 V0.1 Roguelike 节点 (角色 / 怪物 / 道具) 跟新版画布打通, 数据共享 | P0 |
| US-13 | PM | 作为 PM, 我希望把 V0.1 Manga 主题 (日漫 + 武侠 + 赛博朋克) 应用到画布节点, 视觉一致 | P0 |
| US-14 | SRE | 作为 SRE, 我希望 per tenant 排行榜 (跨 workspace) 让我看到整个租户的 top 10 (per G9.2 P2) | P2 |
| US-15 | PM | 作为 PM, 我希望 reward 解锁自动写 audit log, 真人 Lead 到位后可追溯 (per 守门 #13 Transaction) | P1 |
| US-16 | Dev | 作为 Dev, 我希望聚类 AI 走 mock 接口 (per 守门 #23 v2), 主题可配置, 真实 LLM 接 P2 | P1 |
| US-17 | PM | 作为 PM, 我希望 reward 通知 3 渠道可选 (push / in-app / email), 默认 in-app, 不打扰 | P1 |
| US-18 | 5 域 Lead | 作为 Lead, 我希望 achievement 徽章自动授予 (连续 7 天 / 完成任务数 / 解锁特定 kind), 在右上角 inventory 显示 | P1 |
| US-19 | Dev | 作为 Dev, 我希望 power-up 道具持久化到 store (per 守门 #13 Master 表), 跨 session 保留 | P2 |
| US-20 | PM | 作为 PM, 我希望 inventory 物品栏支持持有 + 使用 + 销毁, 3 个 action 完整 | P2 |
| US-21 | PM | 作为 PM, 我希望 score 累计支持 3 维度 (per session / per day / all time), dashboard 可切 | P0 |
| US-22 | PM | 作为 PM, 我希望 reaction 短时 3s 淡出, 不遮挡节点, 不污染 store (per 守门 #13 Work 表) | P1 |
| US-23 | SRE | 作为 SRE, 我希望 score / xp / coin 等游戏化数据 100% 持久化 (per 守门 #13 Master 表), 跨 session 0 丢失 | P1 |

**总用户故事**: 23 个 (≥ 19 个达标, 32 项 × 72% 覆盖, 超过 brief 要求 60%)

---

## §2 用語定義 (用語集 / Ubiquitous Language)

| 用语 | 定义 | 出处 / 备注 |
|---|---|---|
| **Canvas** | 无限画布实体, 复用 `frontend/src/types/ids.ts` Canvas | per `frontend-canvas-design.md` v0.1 §2.1 |
| **CanvasElement** | 画布上的可视对象, 含 10 种 V0.1 kind + 8 种本 SRS 新增 kind | per `frontend/src/types/ids.ts` line 686-696 + 本 SRS §4.1 G1 |
| **CanvasElementKind** | 元素类型枚举, 本 SRS 新增 8 种 gamification kind | 本 SRS 新增 (per §4.1 G1) |
| **Gamification 节点** | 画布上代表游戏化实体的节点 (avatar / level / xp / skill_tree / class / badge / quest / inventory) | 本 SRS 新增 (per §4.1 G1) |
| **Avatar** | 玩家头像, 关联 user_id / agent_id, 视觉随 level 变化 | 本 SRS 新增 (per §4.1 G1.1) |
| **Level** | 等级, integer, 由 xp 公式推导, 默认 `level = floor(sqrt(xp / 100))` | 本 SRS 新增 (per §4.1 G1.2 + G4.1) |
| **XP (Experience Point)** | 经验值, integer, 完成 work-item / claim reward / quest 完成等触发累加 | 本 SRS 新增 (per §4.1 G1.3) |
| **Skill Tree** | 技能树, 树状 DAG, level 达解锁条件开放新 skill | 本 SRS 新增 (per §4.1 G1.4) |
| **Class** | 职业 (Warrior / Mage / Rogue / Healer / Tinkerer / 默认 6 种), 决定 visual tier 跟 perk 偏好 | 本 SRS 新增 (per §4.1 G1.5) |
| **Badge** | 徽章, 达成条件解锁, 显示在 avatar 旁边, 稀有度 COMMON/RARE/EPIC/LEGENDARY | 本 SRS 新增 (per §4.1 G1.6) |
| **Quest** | 任务, 完成条件 + reward 奖励, per session 或 daily 触发 | 本 SRS 新增 (per §4.1 G1.7) |
| **Inventory** | 物品栏, 持有 power-up + 装备, 持久化 (per 守门 #13 Master 表) | 本 SRS 新增 (per §4.1 G1.8) |
| **Reward** | 奖励, 解锁条件达成触发, 形式: xp / coin / badge / power-up / unlock-skill | 本 SRS 新增 (per §4.1 G2.1) |
| **Achievement** | 成就, 跟 Reward 类似但通常更稀有, 自动授予 | 本 SRS 新增 (per §4.1 G2.4) |
| **Score (Points)** | 积分, integer, 操作触发 +10 / +1 / +5 (per G3.1) | 本 SRS 新增 (per §4.1 G3.1) |
| **Coin** | 金币, integer, 跟 Score 独立, 完成 work-item 给 1-5 (per V0.1 agent-game) | per `PHASE-AGENT-GAME-IMPL-REPORT.md` v0.1 + 本 SRS §4.1 G3 |
| **HP (Hit Point)** | 血量, integer, cost 增长扣血 (per V0.1 agent-game) | per `PHASE-AGENT-GAME-IMPL-REPORT.md` v0.1 + 本 SRS §4.1 G3 |
| **Confetti** | 庆祝特效, 触发场景: 完成 / 解锁 / 升级, CSS / Lottie / canvas 3 选 1 (per §7 风险) | 本 SRS 新增 (per §4.1 G8.1) |
| **Reaction** | emoji 表情回应, 8-12 种, 短时 3s 淡出 | 本 SRS 新增 (per §4.1 G7.1) |
| **Dot Voting** | 圆点投票, 每用户 N 票 (默认 5 票/session), 点 sticky_note 投票 | 本 SRS 新增 (per §4.1 G6.1) |
| **Leaderboard** | 排行榜, per workspace (P1) + per tenant (P2), 按 token / 任务完成数 / vote 数量 | 本 SRS 新增 (per §4.1 G9.1-9.2) |
| **Daily Challenge** | 每日任务, 每日 3 个, 完成得 reward | 本 SRS 新增 (per §4.1 G10.1) |
| **Streak** | 连续天数, 中断清零, 7 天给大奖励 | 本 SRS 新增 (per §4.1 G10.2) |
| **Power-up** | 道具, 限时加成, 类型: 双倍积分 / 自动聚类 / 隐身模式 | 本 SRS 新增 (per §4.1 G11.1) |
| **Sticky Note 聚类 (AI mock)** | 选中 N 张 sticky_note → AI 聚类成 K 主题, **走 mock 接口** (per 守门 #23 v2, 不引入第三方 LLM 凭据) | 本 SRS 新增 (per §4.1 G5.1) |
| **Mock 接口** | 走 `scripts/automation/ai_edit_mock.py` 模板生成 (per 守门 #23 v2), 不开 OpenAI/Anthropic 第三方 API | per AGENTS.md §4 #23 v2 |
| **W/T/M 三類横展** | DB 表必分 Work / Transaction / Master 三類, 100% 表覆盖, 禁混在 (per 守门 #13) | per AGENTS.md §4 #13 |
| **Work (W)** | 短 TTL 作業中, session-bound, 完成后清理 (reaction 短时显示 / confetti 动画) | 本 SRS 新增 (per §4.1 G11.2 + 守门 #13) |
| **Transaction (T)** | 業務事実 / 監査 / Append-only (score 累加 / xp 累加 / quest 完成 / reward 解锁) | 本 SRS 新增 (per §4.1 G11.2 + 守门 #13) |
| **Master (M)** | 参考 / 設定 / 慢変 SCD (avatar 配置 / class 字典 / skill tree 定义 / badge 字典 / quest 模板) | 本 SRS 新增 (per §4.1 G11.2 + 守门 #13) |
| **派生 (Projection)** | 从已有数据派生计算, 不进 store, 不可写 (per `SRS-AGENT-VIEW-001.md` §2) | DDD 概念 |
| **Active Agent** | 14 状态中前 11 个 (per `SRS-AGENT-VIEW-001.md` §2) | 引用 SRS-AGENT-VIEW-001 |
| **Sticky Note** | 便利贴画布元素, 跟聚类 / 投票协同 (per `CanvasView.tsx` line 156-168) | per V0.1 baseline |
| **Comment Pin** | 画布评论图钉, 跟 reaction / dot voting 协同 (per `CanvasView.tsx` line 253-262) | per V0.1 baseline |
| **Automation Node** | 画布自动化规则节点, 跟 gamification_node 协同 (rule trigger, per `CanvasView.tsx` line 236-252) | per V0.1 baseline |
| **Roguelike 节点** | 画布上 Roguelike 角色 / 怪物 / 道具节点 (per V0.1 `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md`) | per V0.1 baseline + 本 SRS §4.1 G12.1 |
| **Manga 主题节点** | 画布上日漫 + 武侠 + 赛博朋克风格节点 (per V0.1 `PHASE-AGENT-MANGA-IMPL-REPORT.md`) | per V0.1 baseline + 本 SRS §4.1 G12.2 |

---

## §3 業務背景 / 前提条件

### 3.1 业务背景

- STAR 平台 22 domain crate + LangGraph view + Agent Runtime view + Agent View (per `SRS-AGENT-VIEW-001.md` v1.0) + V0.1 agent-game 4 份 PHASE 报告 (9/5 落地) 已就位
- 现有 25 module 框架下, 画布是 collaboration 域强化 (per `frontend-canvas-design.md` v0.1 §1.1)
- 现有 10 种 `CanvasElementKind` (sticky_note / text / shape / image / embed / work_item_card / worktree_node / agent_cursor / automation_node / comment_pin, per `frontend/src/types/ids.ts` line 686-696)
- 现有 12 个 agent session mock seed (per `frontend/src/lib/seed.ts`)
- 现有 12 个 worktree mock seed (per `frontend/src/lib/seed.ts`)
- 现有 30 个 work-item mock seed (per `frontend/src/lib/seed.ts`)
- 5 域 (player/economy/match/social/admin) Lead 真人未到位, Mavis 临时代签 (per `AGENTS.md §4 守门 #3 v2 派生规` + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)
- 9/10 17:08 JST Ulysses 拍板"我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" — 撤回原 12 大类 12 diagram / 5 编辑效率 / AI 通用 / 数据接入 范围

### 3.2 前提条件

| # | 前提 | 影响 |
|---|---|---|
| **P-1** | 画布 `CanvasView` 已实装 (per `frontend/src/components/CanvasView.tsx`, V0.1 14 element kind + 4 frame + 8 connector) | 本 SRS 新增 8 kind 走扩展点, 不重写 |
| **P-2** | `CanvasElementKind` 枚举已落地 (10 种, per `frontend/src/types/ids.ts` line 686-696) | 新增 8 kind = 扩展 union type |
| **P-3** | `StatusPill` 60+ 色码已落地 (per `frontend-canvas-design.md` §3.4 + ADR-FE-013) | gamification 节点状态色码同步 |
| **P-4** | `agent-game` 4 份 PHASE 报告已实装 (per §1.1, 9/5 落地, 125 tests pass) | G12 集成 = 引用已有 store / 类型, 不重写功能 |
| **P-5** | `RoguelikeCanvas` 已实装 (per `frontend/src/components/agent-game/RoguelikeCanvas.tsx`, 9/5 落地) | G12.1 画布集成 = 新增 kind 映射, 不改 RoguelikeCanvas 内部 |
| **P-6** | `Manga` 主题 (theme.ts / characters.tsx / enemies.tsx / Decorations.tsx) 已实装 (per `PHASE-AGENT-MANGA-IMPL-REPORT.md` v0.1) | G12.2 画布集成 = 新增 kind 引用 characters.tsx / theme.ts |
| **P-7** | zustand store (`@/lib/store`) 已落地 12 个集合 (agentSessions / worktrees / workItems / agentGameStates / agentMaps / agentPositions / agentSettings / feedbacks / automationRules / canvasElements / canvasConnectors / canvasFrames) | 本 SRS 新增 3 个集合 (gamificationStates / voteStates / notificationStates) |
| **P-8** | 路由系统已就绪 (Next.js 14.2.5 App Router) | 新增 route 走 `/canvas/[id]` 子路由, 不破坏主路由 |
| **P-9** | i18n 系统已就绪 (3 语言 zh-CN / en / ja) | gamification 文案走 i18n, 至少 8 项 key 落 `dictionary.ts` |
| **P-10** | SVG 基础工具已落地 (lucide-react / recharts / framer-motion V0.1 已用) | confetti / reaction 走 SVG 渲染 |
| **P-11** | 不引入新依赖 (per 守门 #19 v19+ 累积规): confetti 走 CSS keyframes 或 SVG, 不引 canvas-confetti | confetti 实现路径锁定 |
| **P-12** | mock 接口 (per 守门 #23 v2): AI 聚类走 `scripts/automation/ai_edit_mock.py` 模板生成, 不开 OpenAI/Anthropic 第三方 API | G5 锁定 mock 路径 |
| **P-13** | DB W/T/M 三類横展 (per 守门 #13, 9/1 18:30 JST 拍板): G11 power-up / inventory 必含 W/T/M 三類分門別類, 100% 表覆盖 | G11 必含三類横展表 |
| **P-14** | 守门 #9 v3: 调试控制台走 subprocess 替代 RPC (per `AGENTS.md §4 #9 v3`), 子代理 RPC 实证不可靠 (10 background task `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded) | 本 SRS 不依赖子代理 dispatch, 全程 Mavis 接手 + 用户拍板 |

### 3.3 业务规则 (Business Rules)

- **BR-1**: gamification 节点 kind 命名跟 V0.1 agent-game 已实装 kind 严格一致 (avatar / level / xp / skill_tree / class / badge / quest / inventory), 不引入新命名
- **BR-2**: 升级公式默认 `level = floor(sqrt(xp / 100))` (线性), 用户可调 (G4.1 配置化)
- **BR-3**: dot voting 每用户 5 票/session (per G6.1), 票数扣减, 跨 session 重置 (MVP); 跨 session 持久化留 P2
- **BR-4**: reaction 短时 3s 淡出 (per G7.1), 不持久化 (Work 表, 短 TTL), 不污染主 store
- **BR-5**: confetti 触发场景 = 完成 work-item / 解锁 reward / 升级 level / 达成 achievement (per G8.1 + G2.3)
- **BR-6**: power-up 类型 3 种 (双倍积分 / 自动聚类 / 隐身模式), 限时 (默认 24h), 持久化 (Master 表, per 守门 #13)
- **BR-7**: sticky note 聚类 AI 走 mock 接口 (per 守门 #23 v2), 输入 N 张 sticky_note + K (主题数, 默认 K=3), 输出 K 个主题 Frame + 关联 connector
- **BR-8**: per workspace 排行榜 1 小时 cache (P1), per tenant 排行榜 24h cache (P2, 跨 workspace)
- **BR-9**: daily challenge 每日 3 个任务 (per user TZ), 完成给 reward; streak 7 天再给大奖励; 中断清零
- **BR-10**: 跟 V0.1 agent-game 共享 game state (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §0): avatar / level / xp / coin / hp 走 `agentGameStates[agentId]`, 不新建 store
- **BR-11**: G12.1 Roguelike 画布节点 = `kind: "roguelike_character" / "roguelike_enemy" / "roguelike_item"`, 复用 `RoguelikeCanvas.tsx` 渲染逻辑
- **BR-12**: G12.2 Manga 画布节点 = `kind: "manga_agent" / "manga_enemy" / "manga_decoration"`, 复用 `characters.tsx` / `enemies.tsx` / `Decorations.tsx`
- **BR-13**: gamification 节点 visual tier 复用 `AGENT_VISUAL_TIERS` (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #1, 10 段 灰/灰/蓝/蓝/绿/绿/紫/紫/金/金)
- **BR-14**: 12 子能力 32 项 ID 严格按 brief §5 命名 (F-GAMIFY-G1.1 / FR-GAMIFY-G1.1.1 / NFR-GAMIFY-G1.1 / AC-GAMIFY-G1.1.1 / US-X), 不重命名

---

## §4 功能需求

### 4.1 12 子能力 32 项展开 (per brief §5)

#### 4.1.1 G1 gamification 节点 (8 项)

##### G1.1 avatar_node (玩家头像, 关联 user_id / agent_id)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G1.1 |
| 描述 | 系统应在画布上渲染 avatar 节点, 关联 user_id (主) 或 agent_id (副), 视觉随 level 变化 |
| 输入 | `userId: Uuid` / `agentId: Uuid` / `level: int` (per V0.1 `AGENT_VISUAL_TIERS` 10 段) |
| 输出 | `CanvasElement { kind: "avatar", x, y, width: 120, height: 120, content: { user_id, agent_id, level, tier } }` |
| 视觉规范 | 圆形头像 120x120, 内嵌 emoji + tier 颜色边框 (Lv 1-2 灰 / Lv 3-4 蓝 / Lv 5-6 绿 / Lv 7-8 紫 / Lv 9-10 金, per V0.1 §1 #1); 右上角 Lv 徽章 (per V0.1 `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #11) |
| 优先级 | P0 |
| 关联 V0.1 | 复用 `AGENT_VISUAL_TIERS` (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #1) + `AgentCharacterSVG` (per `PHASE-AGENT-MANGA-IMPL-REPORT.md` §1 #3) |
| 接口依赖 | BFF `GET /v1/gamify/avatar/{userId}` (mock 返回 tier) |
| 数据字段 | `CanvasElement.content.user_id` / `agent_id` / `level` / `tier` |
| 验收标准 | AC-GAMIFY-G1.1.1: 画布上 1 个 avatar 节点 + level 1 → 灰色边框 + 圆形头; AC-GAMIFY-G1.1.2: level 10 → 金色边框 + 神侠光环 (per V0.1 `Decorations.tsx`) |
| 用户故事 | US-1 |
| 已知缺口 | 真实 user 上传头像 (V2 候选); 头像 cache 策略 P1 |

##### G1.2 level_node (等级, 关联 xp)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G1.2 |
| 描述 | 系统应在画布上渲染 level 节点, 显示当前 level + 下一级所需 xp |
| 输入 | `userId` / `level: int` / `xp: int` / `xpToNext: int` |
| 输出 | `CanvasElement { kind: "level", x, y, width: 160, height: 80, content: { user_id, level, xp, xp_to_next } }` |
| 视觉规范 | 矩形 160x80, 顶部大字 "Lv X" (V0.1 visual tier 颜色), 底部 progress bar (current xp / xp_to_next) |
| 优先级 | P0 |
| 关联 V0.1 | 复用 `leveling.ts` `xpProgress()` (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #2) |
| 接口依赖 | BFF `GET /v1/gamify/level/{userId}` (mock) |
| 数据字段 | `level` / `xp` / `xp_to_next` |
| 验收标准 | AC-GAMIFY-G1.2.1: level 5 + xp 2500 → "Lv 5" + progress 100% (下一级 xp = 3600 - 2500 = 1100); AC-GAMIFY-G1.2.2: level 7 → 紫色边框 |
| 用户故事 | US-8 |
| 已知缺口 | 进度条动画 P2 优化; 多用户聚合 view P2 |

##### G1.3 xp_node (经验值, 实时更新)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G1.3 |
| 描述 | 系统应在画布上渲染 xp 节点, 实时更新 (per V0.1 `claimReward` 链路触发) |
| 输入 | `userId` / `xp: int` / `delta?: int` (实时) |
| 输出 | `CanvasElement { kind: "xp", x, y, width: 140, height: 60, content: { user_id, xp, delta } }` |
| 视觉规范 | 矩形 140x60, 大字 "XP: 1234", delta > 0 时右上角浮 "+10" 1.5s 淡出 |
| 优先级 | P0 |
| 关联 V0.1 | 复用 `applyClaim` / `xpProgress` (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #2) |
| 接口依赖 | BFF `GET /v1/gamify/xp/{userId}` + WS `xp.changed` (mock) |
| 数据字段 | `xp` / `delta` |
| 验收标准 | AC-GAMIFY-G1.3.1: claim work-item +10 xp → 节点显示 "XP: 1244" + 浮 "+10"; AC-GAMIFY-G1.3.2: WS 推送 xp.changed event → 节点 ≤ 500ms 反映 |
| 用户故事 | US-8 |
| 已知缺口 | 多人同时 claim 并发处理 (per V0.1 `lastClaimAt` gate) |

##### G1.4 skill_tree_node (技能树, 树状结构)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G1.4 |
| 描述 | 系统应在画布上渲染 skill_tree 节点, 树状 DAG 展示解锁/未解锁技能 |
| 输入 | `userId` / `tree: SkillTreeNode[]` (DAG) / `unlocked: Set<SkillId>` |
| 输出 | `CanvasElement { kind: "skill_tree", x, y, width: 320, height: 240, content: { user_id, tree, unlocked } }` |
| 视觉规范 | 矩形 320x240, 内嵌 SVG DAG 渲染 (unlocked = 亮色, locked = 灰色 + 锁图标), 节点圆形 + 连线 bezier (per V0.1 bezier 公式) |
| 优先级 | P0 |
| 关联 V0.1 | 复用 `Decorations.tsx` `Stamp` (per `PHASE-AGENT-MANGA-IMPL-REPORT.md` §1 #5) |
| 接口依赖 | BFF `GET /v1/gamify/skill-tree/{userId}` (mock 返回 6 节点 DAG) |
| 数据字段 | `tree` / `unlocked` |
| 验收标准 | AC-GAMIFY-G1.4.1: level 1 → 仅 1 节点亮 (起点); AC-GAMIFY-G1.4.2: level 5 → 3 节点亮 + DAG 连线显示 |
| 用户故事 | US-9 |
| 已知缺口 | skill 详情 hover tooltip P2; skill 升级动画 P2 |

##### G1.5 class_node (职业, e.g. Warrior / Mage / Rogue)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G1.5 |
| 描述 | 系统应在画布上渲染 class 节点, 显示当前职业 + 职业图标 |
| 输入 | `userId` / `class: "warrior" | "mage" | "rogue" | "healer" | "tinkerer" | "default"` |
| 输出 | `CanvasElement { kind: "class", x, y, width: 140, height: 140, content: { user_id, class } }` |
| 视觉规范 | 圆形 140x140, 职业图标 (6 种, 跟 V0.1 `ENEMY_TYPES` 6 种光球呼应, per `PHASE-AGENT-MANGA-IMPL-REPORT.md` §1 #1); 边框 = class 主题色 |
| 优先级 | P0 |
| 关联 V0.1 | 复用 `theme.ts` `ENEMY_TYPES` 6 种光球 (per `PHASE-AGENT-MANGA-IMPL-REPORT.md` §1 #1) |
| 接口依赖 | BFF `GET /v1/gamify/class/{userId}` (mock) |
| 数据字段 | `class` |
| 验收标准 | AC-GAMIFY-G1.5.1: class "warrior" → 朱红图标 + 朱红边框; AC-GAMIFY-G1.5.2: class "mage" → 霓虹青图标 + 紫边框 |
| 用户故事 | US-7 |
| 已知缺口 | 职业切换 P2; 职业 perk 偏好 (per V0.1 `perks.ts` `getPerkChoices`) P2 |

##### G1.6 badge_node (徽章, 解锁条件)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G1.6 |
| 描述 | 系统应在画布上渲染 badge 节点, 显示已解锁徽章 + 稀有度 |
| 输入 | `userId` / `badges: Badge[]` (含 code / name / rarity) |
| 输出 | `CanvasElement { kind: "badge", x, y, width: 100, height: 100, content: { user_id, badges } }` |
| 视觉规范 | 圆形 100x100, 徽章堆叠 (最多 3 个显示, 多余 "+N" 角标), 稀有度 = 4 圈颜色 (COMMON 灰 / RARE 蓝 / EPIC 紫 / LEGENDARY 金) |
| 优先级 | P0 |
| 关联 V0.1 | 复用 `Decorations.tsx` `Stamp` (per `PHASE-AGENT-MANGA-IMPL-REPORT.md` §1 #5) |
| 接口依赖 | BFF `GET /v1/gamify/badges/{userId}` (mock) |
| 数据字段 | `badges` (含 `code` / `name` / `rarity` / `unlocked_at`) |
| 验收标准 | AC-GAMIFY-G1.6.1: 1 个 COMMON 徽章 → 灰圈; AC-GAMIFY-G1.6.2: 1 个 LEGENDARY 徽章 → 金圈 + 光晕动画 |
| 用户故事 | US-18 |
| 已知缺口 | 徽章解锁动画 P2; 徽章分享 (PNG 导出) P2 (per `SRS-AGENT-RELATIONSHIP-001.md` §10 G-10) |

##### G1.7 quest_node (任务, 完成条件 + 奖励)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G1.7 |
| 描述 | 系统应在画布上渲染 quest 节点, 显示任务进度 + 奖励 |
| 输入 | `userId` / `quests: Quest[]` (含 `id` / `title` / `progress: 0-1` / `reward: {xp, coin, badge?}`) |
| 输出 | `CanvasElement { kind: "quest", x, y, width: 240, height: 120, content: { user_id, quests } }` |
| 视觉规范 | 矩形 240x120, 顶部 quest 标题 + progress bar + 奖励 (xp / coin / badge icon); 完成态绿色高亮 |
| 优先级 | P1 |
| 关联 V0.1 | 复用 `leveling.ts` `computeClaim` (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #2) |
| 接口依赖 | BFF `GET /v1/gamify/quests/{userId}` (mock) |
| 数据字段 | `quests` (含 `id` / `title` / `progress` / `reward`) |
| 验收标准 | AC-GAMIFY-G1.7.1: quest 进度 50% → progress bar 半满; AC-GAMIFY-G1.7.2: quest 完成 → 绿色高亮 + confetti (per G8.1) |
| 用户故事 | US-11 |
| 已知缺口 | 多人 quest 协作 P2; quest 链 (前 quest 解锁后 quest) P2 |

##### G1.8 inventory_node (物品栏, 道具 + 装备)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G1.8 |
| 描述 | 系统应在画布上渲染 inventory 节点, 显示持有 power-up + 装备 (3 种 action: 持有 / 使用 / 销毁) |
| 输入 | `userId` / `items: InventoryItem[]` (含 `id` / `type` / `expires_at`) |
| 输出 | `CanvasElement { kind: "inventory", x, y, width: 280, height: 160, content: { user_id, items } }` |
| 视觉规范 | 矩形 280x160, 3×3 grid 物品槽, 物品图标 (power-up 3 种 + 装备 4 类), 右上角剩余时长 (限时 power-up) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 `agentGameStates[userId].perks` (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #1) |
| 接口依赖 | BFF `GET /v1/gamify/inventory/{userId}` (mock) + `POST /v1/gamify/inventory/use` + `POST /v1/gamify/inventory/destroy` |
| 数据字段 | `items` (含 `id` / `type` / `expires_at` / `count`) |
| 验收标准 | AC-GAMIFY-G1.8.1: 持有 1 个 power-up "double_xp" → 物品槽显示 + 倒计时; AC-GAMIFY-G1.8.2: 点 "使用" → 立即生效 + 物品从 inventory 移除 (per G11.1) |
| 用户故事 | US-20 |
| 已知缺口 | 装备系统 (头部/身体/武器) P2; 物品交易 (5 域 Lead 之间) P2 |

#### 4.1.2 G2 reward / achievement (4 项)

##### G2.1 reward 解锁条件 (rule engine, 跟 V0.1 automation 域对接)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G2.1 |
| 描述 | 系统应支持 reward 解锁条件定义 (rule engine), 跟 V0.1 automation 域对接 (复用 `automationRules`) |
| 输入 | `ruleId` / `condition: "claim_wi" | "level_up" | "vote_received" | "achievement"` / `reward: {xp, coin, badge?, power_up?}` |
| 输出 | 触发时, `agentGameStates[userId]` 更新 + WS `reward.unlocked` 推送 |
| 优先级 | P1 |
| 关联 V0.1 | 复用 `automationRules` 域 (per `CanvasView.tsx` line 236-252 `automation_node`) |
| 接口依赖 | BFF `POST /v1/gamify/reward-rules` + WS `reward.unlocked` |
| 数据字段 | `rule_id` / `condition` / `reward` / `enabled` |
| 验收标准 | AC-GAMIFY-G2.1.1: rule "claim_wi → +10 xp" 触发 → xp +10; AC-GAMIFY-G2.1.2: 同一 rule 重复触发不重复给奖 (per V0.1 `lastClaimAt` gate) |
| 用户故事 | US-15 |
| 已知缺口 | rule 复合条件 (AND / OR) P2; rule 优先级 P2 |

##### G2.2 reward 通知 (push / in-app / email)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G2.2 |
| 描述 | 系统应支持 reward 解锁通知 3 渠道 (push / in-app / email), 默认 in-app |
| 输入 | `userId` / `reward` / `channels: ("push" | "in_app" | "email")[]` |
| 输出 | 通知 1 渠道 (in-app 立即, push 异步, email mock) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 V0.1 notification 域 (per `frontend-canvas-design.md` §1.3 25 module 联动矩阵) |
| 接口依赖 | BFF `POST /v1/gamify/notify` |
| 数据字段 | `channels` / `template_id` / `payload` |
| 验收标准 | AC-GAMIFY-G2.2.1: 默认 in-app → 右上角 toast + 3s 自动消失; AC-GAMIFY-G2.2.2: 选 email → mock console.log (不真发) |
| 用户故事 | US-17 |
| 已知缺口 | push 真实集成 (web push API) P2; email 真实发送 P2 |

##### G2.3 reward 画布特效 (confetti 触发)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G2.3 |
| 描述 | 系统应在 reward 解锁时触发 confetti 画布特效 (per G8.1) |
| 输入 | `canvasId` / `position: {x, y}` (画布坐标) / `intensity: "low" | "med" | "high"` |
| 输出 | confetti 动画 1.5-3s, 自动消失 |
| 优先级 | P1 |
| 关联 V0.1 | 复用 G8.1 confetti 动画 |
| 接口依赖 | BFF `POST /v1/gamify/confetti` (本地触发, 不调后端) |
| 数据字段 | `position` / `intensity` / `color_palette` |
| 验收标准 | AC-GAMIFY-G2.3.1: reward 解锁 → 节点位置 confetti 弹出; AC-GAMIFY-G2.3.2: 3s 后自动消失, 不留 DOM |
| 用户故事 | US-2 |
| 已知缺口 | confetti 跟 reaction 互斥 (同一位置不同时) P2 |

##### G2.4 achievement 徽章自动授予

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G2.4 |
| 描述 | 系统应支持 achievement 徽章自动授予, 条件: 连续 7 天 / 完成任务数 / 解锁特定 kind |
| 输入 | `userId` / `condition: "streak_7" | "tasks_done_100" | "kind_unlocked_X"` |
| 输出 | `badges` 集合 +1 + WS `achievement.unlocked` 推送 + 通知 (per G2.2) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 V0.1 `agentGameStates[userId].badges` (扩展字段) |
| 接口依赖 | BFF `POST /v1/gamify/achievement-evaluate` + WS `achievement.unlocked` |
| 数据字段 | `badges` (含 `code` / `unlocked_at` / `rarity`) |
| 验收标准 | AC-GAMIFY-G2.4.1: streak 7 天 → achievement "weekly_warrior" (COMMON) 自动授予; AC-GAMIFY-G2.4.2: tasks_done 100 → achievement "century_club" (RARE) 自动授予 |
| 用户故事 | US-18 |
| 已知缺口 | achievement 复合条件 (3 个同时) P2; achievement 分享 P2 |

#### 4.1.3 G3 score / points (3 项)

##### G3.1 操作积分规则 (完成任务 +10, 投票 +1, 分享 +5)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G3.1 |
| 描述 | 系统应支持操作积分规则定义, 默认: 完成任务 +10 / 投票 +1 / 分享 +5 |
| 输入 | `action: "claim_wi" | "vote" | "share" | "comment" | "react"` / `points: int` |
| 输出 | `score` 累加 + WS `score.changed` 推送 |
| 优先级 | P0 |
| 关联 V0.1 | 复用 V0.1 `agentGameStates[userId].xp` (扩展 score 字段) |
| 接口依赖 | BFF `POST /v1/gamify/score-rule` (mock 默认 5 条) |
| 数据字段 | `action` / `points` / `enabled` |
| 验收标准 | AC-GAMIFY-G3.1.1: claim work-item → score +10; AC-GAMIFY-G3.1.2: 投票 → score +1; AC-GAMIFY-G3.1.3: 分享 → score +5 |
| 用户故事 | US-7, US-21 |
| 已知缺口 | 规则可视化编辑器 P2; 反作弊 (per session 限频) P2 |

##### G3.2 score 累计 (per session / per day / all time)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G3.2 |
| 描述 | 系统应支持 score 累计 3 维度: per session / per day / all time |
| 输入 | `userId` / `dimension: "session" | "day" | "all_time"` |
| 输出 | `{ session_score, day_score, all_time_score: int }` |
| 优先级 | P0 |
| 关联 V0.1 | 复用 V0.1 `agentGameStates[userId].score` 扩展 |
| 接口依赖 | BFF `GET /v1/gamify/score/{userId}?dimension={dim}` |
| 数据字段 | `session_score` / `day_score` / `all_time_score` |
| 验收标准 | AC-GAMIFY-G3.2.1: dashboard 切 dimension → 3 个数字分别显示; AC-GAMIFY-G3.2.2: 跨 day 边界 (JST 0:00) → day_score 重置 |
| 用户故事 | US-21 |
| 已知缺口 | TZ 用户配置 P2; 历史 score 趋势图 P2 |

##### G3.3 score 排行榜入口

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G3.3 |
| 描述 | 系统应在 score 节点上提供排行榜入口, 跳 `/leaderboard?workspace=...&dim=score` |
| 输入 | `userId` / `workspaceId` |
| 输出 | 跳 `/leaderboard` 路由 (per G9.1) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 V0.1 路由 |
| 接口依赖 | BFF 无 (前端路由) |
| 数据字段 | `workspace_id` / `dimension` |
| 验收标准 | AC-GAMIFY-G3.3.1: 点 score 节点右上角 🏆 → 跳 leaderboard 路由; AC-GAMIFY-G3.3.2: leaderboard 默认按 score 排序 |
| 用户故事 | US-6 |
| 已知缺口 | 跨路由数据共享 (per V0.1 `useStore` zustand) P2 |

#### 4.1.4 G4 leveling / skill tree (3 项)

##### G4.1 xp → level 公式 (默认线性: level = sqrt(xp / 100))

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G4.1 |
| 描述 | 系统应支持 xp → level 公式, 默认线性 `level = floor(sqrt(xp / 100))`, 反推 `xp_to_next = (level + 1)^2 * 100 - xp` |
| 输入 | `xp: int` |
| 输出 | `{ level: int, xp_to_next: int, progress: 0-1 }` |
| 优先级 | P0 |
| 关联 V0.1 | 复用 V0.1 `leveling.ts` `xpProgress` (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #2) |
| 接口依赖 | BFF `GET /v1/gamify/level-from-xp?xp={N}` (纯函数, 可前端算) |
| 数据字段 | `xp` / `level` / `xp_to_next` |
| 验收标准 | AC-GAMIFY-G4.1.1: xp=0 → level 0, xp_to_next=100; AC-GAMIFY-G4.1.2: xp=2500 → level 5, xp_to_next=1100; AC-GAMIFY-G4.1.3: xp=10000 → level 10, xp_to_next=0 (MAX) |
| 用户故事 | US-8 |
| 已知缺口 | 公式用户可配 (sigmoid / 指数) P2; 公式可视化 (per V0.1 `Decorations.tsx` 印章) P2 |

##### G4.2 升级动画 + 通知

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G4.2 |
| 描述 | 系统应在 level 变化时触发升级动画 (svg scale + halo ring) + 通知 (per G2.2) |
| 输入 | `userId` / `oldLevel` / `newLevel` |
| 输出 | 动画 1.5s + 通知 toast 3s + WS `level.up` 推送 |
| 优先级 | P0 |
| 关联 V0.1 | 复用 V0.1 `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #11 (Lv 7+ halo ring) |
| 接口依赖 | BFF 无 (前端动画 + WS) |
| 数据字段 | `old_level` / `new_level` / `delta` |
| 验收标准 | AC-GAMIFY-G4.2.1: level 5 → 6 → 节点 scale 1.0 → 1.2 → 1.0 动画 + halo ring 出现; AC-GAMIFY-G4.2.2: 通知 toast 显示 "Level Up! 5 → 6" 3s |
| 用户故事 | US-2, US-8 |
| 已知缺口 | 跳过 level (经验值突增跳级) P2; 多级连升动画合并 P2 |

##### G4.3 skill tree 解锁 (level 达到解锁条件)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G4.3 |
| 描述 | 系统应在 level 达到 skill tree 节点解锁条件时, 自动开放新技能 + 视觉高亮 |
| 输入 | `userId` / `skillId` / `unlockLevel: int` / `currentLevel: int` |
| 输出 | `unlocked: Set<SkillId>` 更新 + skill 节点亮色 + 通知 (per G2.2) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 G1.4 skill_tree_node |
| 接口依赖 | BFF `GET /v1/gamify/skill-tree/{userId}` |
| 数据字段 | `unlock_level` / `current_level` / `unlocked` |
| 验收标准 | AC-GAMIFY-G4.3.1: level 5 达 unlock_level=5 → skill 节点亮色; AC-GAMIFY-G4.3.2: skill 解锁 → 通知 toast + confetti (per G8.1) |
| 用户故事 | US-9 |
| 已知缺口 | skill 升级 (解锁后再强化) P2; skill 复合解锁 (A 解锁 + B level 5) P2 |

#### 4.1.5 G5 sticky note 聚类 (AI, 走 mock) (2 项)

##### G5.1 选中 N 张 sticky_note → AI 聚类成 K 个主题

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G5.1 |
| 描述 | 系统应支持选中 N 张 sticky_note, 调 mock AI 接口聚类成 K 个主题 (K 用户可调, 默认 3) |
| 输入 | `stickyNoteIds: Uuid[]` / `k: int` (默认 3) |
| 输出 | `themes: Theme[]` (含 `id` / `label` / `stickyNoteIds[]`) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 V0.1 sticky_note (per `CanvasView.tsx` line 156-168) |
| **接口依赖 (mock)** | **走 `scripts/automation/ai_edit_mock.py` 模板生成** (per 守门 #23 v2, **不引入 OpenAI / Anthropic 第三方 API**, 不引入 LLM 凭据, confidence 永远 < 0.5 提示用户手动 review); 真实 LLM 接 P2 |
| 数据字段 | `sticky_note_ids` / `k` / `themes` |
| 验收标准 | AC-GAMIFY-G5.1.1: 选 5 张 sticky_note, k=3 → mock 返回 3 个主题 + 关联; AC-GAMIFY-G5.1.2: 调 mock 接口, 不调 OpenAI / Anthropic (per 守门 #23 v2); AC-GAMIFY-G5.1.3: confidence < 0.5 时 UI 提示"建议手动调整" |
| 用户故事 | US-5, US-16 |
| 已知缺口 | 真实 LLM 接 P2 (per `SRS-AGENT-RELATIONSHIP-001.md` §10 G-6 类似); mock confidence 阈值 P2 |

##### G5.2 聚类结果可视化 (主题 Frame + connector)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G5.2 |
| 描述 | 系统应将聚类结果可视化: K 个主题 Frame + 关联 connector + sticky_note 节点加入 Frame |
| 输入 | `themes: Theme[]` (per G5.1) |
| 输出 | 1 个新 `CanvasFrame` + K 个主题 label + K×N 条 connector |
| 优先级 | P1 |
| 关联 V0.1 | 复用 V0.1 Frame (per `frontend-canvas-design.md` §3.6) |
| 接口依赖 | BFF `POST /v1/gamify/cluster-visualize` (本地) |
| 数据字段 | `themes` / `frame_id` / `connector_kind` |
| 验收标准 | AC-GAMIFY-G5.2.1: 聚类结果 → 1 Frame 创建, 含 K 个主题 label + 关联 sticky_note; AC-GAMIFY-G5.2.2: 主题 Frame 颜色 5 色 (黄/粉/蓝/绿/紫, per V0.1 STICKY_PALETTE) |
| 用户故事 | US-5 |
| 已知缺口 | Frame 拖动不触发重新聚类 (per V0.1 NFR 派生只读) P2 |

#### 4.1.6 G6 dot voting (2 项)

##### G6.1 每用户 N 票 (默认 5 票/session), 点 sticky_note 投票

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G6.1 |
| 描述 | 系统应支持 dot voting, 每用户 5 票/session (per BR-3), 点 sticky_note 投票, 票数扣减 |
| 输入 | `userId` / `stickyNoteId` / `delta: +1 | -1` |
| 输出 | `votes[stickyNoteId]++` / `remaining: int--` |
| 优先级 | P0 |
| 关联 V0.1 | 复用 V0.1 sticky_note (per `CanvasView.tsx` line 156-168) |
| 接口依赖 | BFF `POST /v1/gamify/vote` (mock) |
| 数据字段 | `user_id` / `sticky_note_id` / `remaining` / `votes` |
| 验收标准 | AC-GAMIFY-G6.1.1: 用户有 5 票, 投 1 张 → 剩余 4 票; AC-GAMIFY-G6.1.2: 0 票时按钮 disabled + tooltip "票数已用完"; AC-GAMIFY-G6.1.3: 跨 session 票数重置 (MVP) |
| 用户故事 | US-3 |
| 已知缺口 | 跨 session 持久化 P2; 票数配置 (per workspace 改 N) P2 |

##### G6.2 投票结果实时显示 (票数 badge + 排名)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G6.2 |
| 描述 | 系统应支持投票结果实时显示: sticky_note 节点右上角 badge 显示票数, 排名 top 3 高亮 |
| 输入 | `stickyNoteId` / `votes: int` / `rank: int` |
| 输出 | badge 实时更新 + 排名高亮 |
| 优先级 | P0 |
| 关联 V0.1 | 复用 V0.1 sticky_note 渲染 |
| 接口依赖 | BFF WS `vote.changed` (mock) |
| 数据字段 | `votes` / `rank` / `highlight_color` |
| 验收标准 | AC-GAMIFY-G6.2.1: 投票 → 节点右上角 badge "5" 实时显示; AC-GAMIFY-G6.2.2: top 3 节点金边框高亮 |
| 用户故事 | US-3 |
| 已知缺口 | 排名动画 (slide 切换) P2; 票数历史 (per V0.1 NFR 派生只读) P2 |

#### 4.1.7 G7 reaction (1 项)

##### G7.1 emoji 表情回应 (8-12 种: 👍 ❤️ 🎉 😄 🤔 👀 🔥 ⭐), 短时显示 (3s 淡出)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G7.1 |
| 描述 | 系统应支持 reaction emoji 表情回应, 8-12 种, 短时 3s 淡出动画 |
| 输入 | `userId` / `elementId` / `emoji: "👍" | "❤️" | "🎉" | "😄" | "🤔" | "👀" | "🔥" | "⭐"` |
| 输出 | emoji 浮在节点上方 + 3s 淡出 + 不进主 store (Work 表, 短 TTL) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 V0.1 comment_pin (per `CanvasView.tsx` line 253-262) |
| 接口依赖 | BFF `POST /v1/gamify/react` (mock) + WS `reaction.fired` |
| 数据字段 | `emoji` / `position` / `expires_at` |
| 验收标准 | AC-GAMIFY-G7.1.1: 8 种 emoji 可选 (👍 ❤️ 🎉 😄 🤔 👀 🔥 ⭐); AC-GAMIFY-G7.1.2: 浮在节点上方 + 3s 淡出; AC-GAMIFY-G7.1.3: 不进主 store (per BR-4, Work 表) |
| 用户故事 | US-4, US-22 |
| 已知缺口 | reaction 历史 (用户撤回) P2; reaction 跟 confetti 互斥 (同位置不同时) P2 |

#### 4.1.8 G8 confetti (1 项)

##### G8.1 confetti 动画 (CSS / Lottie / 库选型, 触发: 完成 / 解锁 / 升级)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G8.1 |
| 描述 | 系统应支持 confetti 动画, 触发场景: 完成 work-item / 解锁 reward / 升级 level / 达成 achievement |
| 输入 | `position: {x, y}` / `intensity: "low" | "med" | "high"` / `trigger: "complete" | "unlock" | "levelup" | "achievement"` |
| 输出 | confetti 动画 1.5-3s + 颜色 palette (5 色) |
| 优先级 | P0 |
| 关联 V0.1 | 复用 G2.3 reward 画布特效 |
| **实现路径** | **不引入新依赖** (per 守门 #19 v19+ 累积规 P-11): 走 CSS keyframes + SVG `<animateTransform>` (per V0.1 `Decorations.tsx` `EnergyRing` 同思路), 不引 canvas-confetti 库 |
| 接口依赖 | BFF 无 (本地动画) |
| 数据字段 | `position` / `intensity` / `color_palette` / `trigger` |
| 验收标准 | AC-GAMIFY-G8.1.1: 4 种触发场景 (complete / unlock / levelup / achievement) 都触发 confetti; AC-GAMIFY-G8.1.2: 1.5-3s 自动消失; AC-GAMIFY-G8.1.3: 不引第三方 confetti 库 (per 守门 #19) |
| 用户故事 | US-2 |
| 已知缺口 | Lottie 备选 P2; confetti 音效 (per V0.1 `Decorations.tsx` 缺) P2 |

#### 4.1.9 G9 leaderboard (2 项)

##### G9.1 per workspace 排行榜 (token 用量 / 任务完成数 / vote 数量)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G9.1 |
| 描述 | 系统应支持 per workspace 排行榜, 3 维度: token 用量 / 任务完成数 / vote 数量, 默认 1h cache |
| 输入 | `workspaceId` / `dimension: "token" | "tasks_done" | "votes"` |
| 输出 | top 10 列表 (含 userId / score / rank) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 V0.1 zustand store |
| 接口依赖 | BFF `GET /v1/gamify/leaderboard?workspace={id}&dim={dim}` (mock, 1h cache) |
| 数据字段 | `workspace_id` / `dimension` / `top_10` / `cached_at` |
| 验收标准 | AC-GAMIFY-G9.1.1: 3 维度可切; AC-GAMIFY-G9.1.2: top 10 显示 + 排名高亮 (1-3 金/银/铜) |
| 用户故事 | US-6 |
| 已知缺口 | cache 失效策略 P2; 跨 workspace 比对 P2 |

##### G9.2 per tenant 排行榜 (跨 workspace, 管理员可见)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G9.2 |
| 描述 | 系统应支持 per tenant 排行榜 (跨 workspace), 仅管理员可见, 24h cache |
| 输入 | `tenantId` / `dimension: "token" | "tasks_done" | "votes"` (per G9.1) |
| 输出 | top 10 列表 (含 workspace + userId) |
| 优先级 | P2 |
| 关联 V0.1 | 复用 V0.1 zustand store + tenant_id 隔离 (per 守门 #13 RLS 13 类) |
| 接口依赖 | BFF `GET /v1/gamify/leaderboard?tenant={id}&dim={dim}` (mock, 24h cache, admin only) |
| 数据字段 | `tenant_id` / `dimension` / `top_10` / `cached_at` |
| 验收标准 | AC-GAMIFY-G9.2.1: admin 角色可见; AC-GAMIFY-G9.2.2: 跨 workspace 聚合 (去重 user) |
| 用户故事 | US-14 |
| 已知缺口 | admin 角色 13 租户 (per 守门 #13 RLS 13 类) P2; 跨租户审计 P2 |

#### 4.1.10 G10 daily challenge / streak (2 项)

##### G10.1 daily challenge (每日 3 个任务, 完成得 reward)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G10.1 |
| 描述 | 系统应支持 daily challenge, 每日 3 个任务 (per user TZ, per BR-9), 完成得 reward |
| 输入 | `userId` / `tz: "Asia/Tokyo"` / `date: YYYY-MM-DD` |
| 输出 | 3 个 daily task (含 `id` / `title` / `progress` / `reward`) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 V0.1 quest_node (per G1.7) |
| 接口依赖 | BFF `GET /v1/gamify/daily-challenge?user={id}&date={d}` (mock) |
| 数据字段 | `user_id` / `tz` / `date` / `tasks` |
| 验收标准 | AC-GAMIFY-G10.1.1: 每日 3 个任务; AC-GAMIFY-G10.1.2: 跨 TZ 边界 (JST 0:00) → 任务重置 |
| 用户故事 | US-11 |
| 已知缺口 | 任务难度 P2; 任务模板 (per 5 域 Lead 真人到位后) P2 |

##### G10.2 streak 连续天数 (中断清零, 7 天奖励)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G10.2 |
| 描述 | 系统应支持 streak 连续天数, 中断清零, 7 天给大奖励 |
| 输入 | `userId` / `lastActiveDate: YYYY-MM-DD` |
| 输出 | `streak: int` / `next_reward_at: int` (7 天倍数) |
| 优先级 | P1 |
| 关联 V0.1 | 复用 V0.1 quest_node + reward 通知 (per G2.2) |
| 接口依赖 | BFF `GET /v1/gamify/streak?user={id}` (mock) |
| 数据字段 | `user_id` / `streak` / `last_active_date` |
| 验收标准 | AC-GAMIFY-G10.2.1: 连续 7 天 → streak 7 + 大奖励通知; AC-GAMIFY-G10.2.2: 1 天中断 → streak 重置为 0 |
| 用户故事 | US-11 |
| 已知缺口 | streak 缓冲 (1 天宽限) P2; streak 排行榜 P2 |

#### 4.1.11 G11 power-up / inventory (2 项) — **必含 W/T/M 三類横展 (per 守门 #13)**

##### G11.1 power-up 道具 (双倍积分 / 自动聚类 / 隐身模式, 限时)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G11.1 |
| 描述 | 系统应支持 power-up 道具 3 种: 双倍积分 / 自动聚类 / 隐身模式, 限时 (默认 24h) |
| 输入 | `userId` / `type: "double_score" | "auto_cluster" | "stealth"` / `duration: 24h` |
| 输出 | 道具激活 + 倒计时 + 效果生效 |
| 优先级 | P2 |
| 关联 V0.1 | 复用 V0.1 inventory_node (per G1.8) + agent-game perks (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #1) |
| 接口依赖 | BFF `POST /v1/gamify/powerup/activate` (mock) |
| 数据字段 | `user_id` / `type` / `duration` / `expires_at` |
| 验收标准 | AC-GAMIFY-G11.1.1: 3 种类型可选; AC-GAMIFY-G11.1.2: 24h 倒计时; AC-GAMIFY-G11.1.3: 隐身模式 = canvas 节点半透明 (不干扰) |
| 用户故事 | US-10, US-19 |
| 已知缺口 | 道具交易 (5 域 Lead 之间) P2; 道具等级 P2 |

##### G11.2 inventory 物品栏 (持有 + 使用, **必含 W/T/M 三類横展** per 守门 #13)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G11.2 |
| 描述 | 系统应支持 inventory 物品栏, 持有 + 使用 + 销毁 3 个 action, **数据表必含 W/T/M 三類横展 (per 守门 #13)** |
| 输入 | `userId` / `action: "hold" | "use" | "destroy"` / `itemId` |
| 输出 | inventory 状态更新 + 通知 (per G2.2) |
| 优先级 | P2 |
| 关联 V0.1 | 复用 V0.1 inventory_node (per G1.8) |
| 接口依赖 | BFF `POST /v1/gamify/inventory/{action}` (mock) |
| 数据字段 | `user_id` / `item_id` / `action` / `timestamp` |
| 验收标准 | AC-GAMIFY-G11.2.1: 持有 / 使用 / 销毁 3 action 完整; AC-GAMIFY-G11.2.2: 数据表符合 W/T/M 三類横展 (per 守门 #13) |
| 用户故事 | US-19, US-20 |
| **W/T/M 三類横展表 (per 守门 #13, 9/1 18:30 JST 拍板, 100% 表覆盖, 禁混在一括列举)** | 见下方 G11.2 三類横展表 |

**G11.2 W/T/M 三類横展表 (per 守门 #13 派生规 + 9/1 18:30 JST 拍板)**:

| 表名 | 類型 (W/T/M) | 字段 | 索引 | 派生规 (per 守门 #13) |
|---|---|---|---|---|
| `inventory_items` | **Master (M)** | id, user_id, item_id, type, count, acquired_at, expires_at, version | id, user_id, type, composite (user+item+version) | (a) 物理删除禁止 + SCD Type 2 + RLS 13 類必携 (per 守门 #13 c) |
| `inventory_actions` | **Transaction (T)** | id, user_id, item_id, action (hold/use/destroy), old_count, new_count, actor, timestamp | user_id, item_id, timestamp | (b) 物理删除禁止 + 監査必須 + RLS 13 類必携 (per 守门 #13 b) |
| `inventory_session_buffs` | **Work (W)** | id, user_id, item_id, effect_type, expires_at, payload, created_at, retention_until | user_id, expires_at, retention_until | (a) W = 物理删除 / タイマー失効 / 短 TTL 明示 retention (per 守门 #13 a) |
| `powerup_definitions` | **Master (M)** | id, code, name, description, type, default_duration, effect_payload, version | id, code, type | (c) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携 |
| `powerup_grants` | **Transaction (T)** | id, user_id, powerup_id, granted_at, granted_by, reason, expires_at | user_id, granted_at | (b) 物理删除禁止 + 監査必須 + RLS 13 類必携 |
| `powerup_active_effects` | **Work (W)** | id, user_id, powerup_id, effect_type, activated_at, expires_at, payload, retention_until | user_id, expires_at | (a) 物理删除 / タイマー失効 / 短 TTL |
| `reaction_events` | **Work (W)** | id, user_id, element_id, emoji, fired_at, expires_at, retention_until | user_id, element_id, expires_at | (a) 短 TTL 3s, 自动清理 (per BR-4) |
| `confetti_events` | **Work (W)** | id, trigger, position, intensity, fired_at, expires_at, retention_until | trigger, expires_at | (a) 短 TTL 3s, 自动清理 |
| `dot_vote_ledger` | **Transaction (T)** | id, user_id, sticky_note_id, delta, remaining_after, actor, timestamp | user_id, sticky_note_id, timestamp | (b) 物理删除禁止 + 監査必須 + RLS 13 類必携 (per session 重置逻辑不删 ledger) |
| `vote_state_snapshot` | **Master (M)** | id, workspace_id, snapshot_at, top_10_json, version | workspace_id, snapshot_at | (c) 物理删除禁止 + SCD Type 2 (per hour snapshot) |
| `leaderboard_cache` | **Work (W)** | id, scope (workspace/tenant), scope_id, dimension, top_10_json, cached_at, expires_at, retention_until | scope_id, dimension, expires_at | (a) 短 TTL 1h/24h, 自动清理 (per G9.1/G9.2 cache) |
| `daily_challenge_state` | **Master (M)** | id, user_id, date, tz, tasks_json, completion_state, version | user_id, date, composite (user+date+version) | (c) 物理删除禁止 + SCD Type 2 (per user+date unique) |
| `streak_state` | **Master (M)** | id, user_id, current_streak, longest_streak, last_active_date, version | user_id, version | (c) 物理删除禁止 + SCD Type 2 |
| `clustering_jobs` | **Transaction (T)** | id, user_id, sticky_note_ids_json, k, themes_json, confidence, mock_used, timestamp | user_id, timestamp | (b) 物理删除禁止 + 監査必須 (per 守门 #23 v2, mock 必记) |
| `clustering_results_cache` | **Work (W)** | id, input_hash, themes_json, expires_at, retention_until | input_hash, expires_at | (a) 短 TTL, 自动清理 |

**派生规 (per 守门 #13)**:
- (a) Work 表 100% 物理删除 / タイマー失効 / 短 TTL 明示 retention (per G11.2 表 7 行: `inventory_session_buffs` / `powerup_active_effects` / `reaction_events` / `confetti_events` / `leaderboard_cache` / `clustering_results_cache`)
- (b) Transaction 表 100% 物理删除禁止 + 監査必須 + RLS 13 類必携 (per G11.2 表 4 行: `inventory_actions` / `powerup_grants` / `dot_vote_ledger` / `clustering_jobs`)
- (c) Master 表 100% 物理删除禁止 + SCD Type 2 + RLS 13 類必携 (per G11.2 表 6 行: `inventory_items` / `powerup_definitions` / `vote_state_snapshot` / `daily_challenge_state` / `streak_state` / 1 master derived)
- 混合分類: 0 (per 守门 #13 禁混在)
- 100% 表覆盖: 15 张表 (Work 6 + Transaction 4 + Master 5) = 100% 覆盖 inventory + power-up + reaction + confetti + vote + leaderboard + daily challenge + streak + clustering 全域

**已知缺口 (per 守门 #11 缺标比错标)**: retention 策略默认值暂定 7d (Work 表) / permanent (Transaction) / permanent (Master SCD Type 2), 实际值待 DDD Review Lead 拍板

#### 4.1.12 G12 跟 agent-game 集成 (V0.1 已实装) (2 项)

##### G12.1 Roguelike 集成 (画布节点 = 角色 / 怪物 / 道具)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G12.1 |
| 描述 | 系统应将 V0.1 Roguelike 角色 / 怪物 / 道具映射到画布节点, 复用 `RoguelikeCanvas` 渲染逻辑 (per BR-11) |
| 输入 | `agentId` / `mapCellType: "start" | "enemy" | "treasure" | "trap" | "blank" | "boss"` |
| 输出 | `CanvasElement { kind: "roguelike_character" | "roguelike_enemy" | "roguelike_item" }` + 共享 `agentGameStates` / `agentMaps` |
| 优先级 | P0 |
| 关联 V0.1 | 复用 V0.1 `RoguelikeCanvas.tsx` (per `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` §1 #5) + `mapgen.ts` / `movement.ts` |
| 接口依赖 | BFF 无 (本地共享 store) |
| 数据字段 | `map_cell_type` / `cell_position` / `linked_work_item_id` |
| 验收标准 | AC-GAMIFY-G12.1.1: 画布上画 6 种 cell (start/enemy/treasure/trap/blank/boss); AC-GAMIFY-G12.1.2: enemy cell 跟 work-item 关联, 走 claim 链路 (per V0.1) |
| 用户故事 | US-12 |
| 已知缺口 | Roguelike 跟 Canvas mode 共享 game state (per V0.1 `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` §3 #8); 画布模式 enemy cell 渲染 P2 |

##### G12.2 Manga 集成 (画布节点 = 漫画风格主题)

| 项 | 内容 |
|---|---|
| ID | FR-GAMIFY-G12.2 |
| 描述 | 系统应将 V0.1 Manga 主题 (日漫 + 武侠 + 赛博朋克) 应用到画布节点, 复用 `characters.tsx` / `enemies.tsx` / `Decorations.tsx` (per BR-12) |
| 输入 | `theme: "manga" | "wuxia" | "cyberpunk" | "combined"` (per V0.1 拍板) |
| 输出 | 画布节点 visual = Manga 主题 (圆形头 + 武士刀 + 披风 + 战甲 + 头冠 + 神侠光环 Lv 7+) |
| 优先级 | P0 |
| 关联 V0.1 | 复用 V0.1 `theme.ts` / `characters.tsx` / `enemies.tsx` / `Decorations.tsx` (per `PHASE-AGENT-MANGA-IMPL-REPORT.md` §1 #1-#5) |
| 接口依赖 | BFF 无 (本地 theme 切换) |
| 数据字段 | `theme` / `tier` / `decoration_set` |
| 验收标准 | AC-GAMIFY-G12.2.1: 画布节点 visual = Manga 主题 (圆形头 + 武士刀); AC-GAMIFY-G12.2.2: Lv 7+ 节点加神侠光环 (per V0.1 `Decorations.tsx`) |
| 用户故事 | US-13 |
| 已知缺口 | theme 用户可切 (per V0.1 `PHASE-AGENT-THEME-IMPL-REPORT.md` dark/light 基础); 多 theme 共存 P2 |

### 4.2 FR 总览 (per 32 项 + 守门)

| 类别 | 数量 | 备注 |
|---|---|---|
| G1 gamification 节点 | 8 FR | G1.1-1.8 |
| G2 reward / achievement | 4 FR | G2.1-2.4 |
| G3 score / points | 3 FR | G3.1-3.3 |
| G4 leveling / skill tree | 3 FR | G4.1-4.3 |
| G5 sticky note 聚类 (AI mock) | 2 FR | G5.1-5.2 |
| G6 dot voting | 2 FR | G6.1-6.2 |
| G7 reaction | 1 FR | G7.1 |
| G8 confetti | 1 FR | G8.1 |
| G9 leaderboard | 2 FR | G9.1-9.2 |
| G10 daily challenge / streak | 2 FR | G10.1-10.2 |
| G11 power-up / inventory | 2 FR | G11.1-11.2 (含 W/T/M 三類横展表) |
| G12 跟 agent-game 集成 | 2 FR | G12.1-12.2 |
| **总计** | **32 FR** | (per brief §5 32 项) |

---

## §5 非功能需求 (Non-Functional Requirements)

### 5.1 NFR-GAMIFY-PERF (性能)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-PERF-01 | 画布 32 节点 (含 G1 8 种) 首次渲染 ≤ 500ms (mock 12 agent 场景) | 浏览器 dev tools FCP/LCP | P0 |
| NFR-GAMIFY-PERF-02 | WS 推送反映 ≤ 200ms (xp / score / vote 变化) | 手动 / Playwright | P0 |
| NFR-GAMIFY-PERF-03 | confetti 60fps (1.5-3s 动画不卡) | 浏览器 dev tools FPS | P1 |
| NFR-GAMIFY-PERF-04 | AI 聚类 mock 接口 P95 ≤ 300ms (per 守门 #23 v2 模板生成快) | 手动 / console.time | P1 |

### 5.2 NFR-GAMIFY-UI (视觉一致性)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-UI-01 | 跟 V0.1 `StatusPill` 60+ 色码一致 (per `frontend-canvas-design.md` §3.4 + ADR-FE-013) | 人工 review | P0 |
| NFR-GAMIFY-UI-02 | 跟 V0.1 `AGENT_VISUAL_TIERS` 10 段视觉一致 (per `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #1) | 人工 review | P0 |
| NFR-GAMIFY-UI-03 | 跟 V0.1 `theme.ts` 色板一致 (12 色, per `PHASE-AGENT-MANGA-IMPL-REPORT.md` §1 #1) | 人工 review | P0 |
| NFR-GAMIFY-UI-04 | dark/light 主题切换一致 (per `PHASE-AGENT-THEME-IMPL-REPORT.md` 落地) | 人工 review | P1 |
| NFR-GAMIFY-UI-05 | 跟 V0.1 bezier connector 公式一致 (per `frontend-canvas-design.md` §3.5) | 人工 review | P0 |

### 5.3 NFR-GAMIFY-A11Y (可达性, 仅基础)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-A11Y-01 | 键盘可达 (Tab 切节点, Space/Enter 激活, Esc 关闭) | 手动 | P1 |
| NFR-GAMIFY-A11Y-02 | 屏幕阅读器基础 (aria-label on 节点) | 手动 / axe-core | P2 |
| NFR-GAMIFY-A11Y-03 | 高对比度模式 (per V0.1 `theme-tokens.ts` LIGHT/DARK_COLORS) | 手动 | P2 |

### 5.4 NFR-GAMIFY-DATA (数据一致性)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-DATA-01 | 派生确定性 (per V0.1 `SRS-AGENT-VIEW-001.md` §2 NFR-AGV-DET-001): 同样输入永远出同样输出 (SSR/CSR hydration 无漂移) | vitest "排序稳定" / "deterministic" | P0 |
| NFR-GAMIFY-DATA-02 | W/T/M 三類横展 100% 表覆盖 (per 守门 #13) | DDD Review 拍板 | P0 |
| NFR-GAMIFY-DATA-03 | RLS 13 類必携 (per 守门 #13) | DDD Review 拍板 | P0 |
| NFR-GAMIFY-DATA-04 | Transaction 表 append-only 100% 审计 (per 守门 #13) | DDD Review 拍板 | P0 |
| NFR-GAMIFY-DATA-05 | 派生数据不写主 store (per V0.1 NFR-AGV-STATE-001, 避免污染) | 人工 review / vitest | P0 |

### 5.5 NFR-GAMIFY-SEC (安全)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-SEC-01 | env 安全 (per 守门 #5, 8/27 11:06 JST hard ban): 不打印 env | 人工 / grep | P0 |
| NFR-GAMIFY-SEC-02 | mock 接口无第三方 LLM 凭据 (per 守门 #23 v2, 9/2 09:01 JST 拍板) | 人工 / grep | P0 |
| NFR-GAMIFY-SEC-03 | 不引入新依赖 (per 守门 #19 v19+ 累积规): confetti 走 CSS/SVG, 不引 canvas-confetti | package.json diff (空) | P0 |

### 5.6 NFR-GAMIFY-I18N (国际化)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-I18N-01 | 3 语言 (zh-CN / en / ja) 友好, 至少 8 项 i18n key 落 `dictionary.ts` | 人工 / grep | P2 |
| NFR-GAMIFY-I18N-02 | emoji 国际化 (per V0.1 `dictionary.ts` emoji 表) | 人工 | P2 |

### 5.7 NFR-GAMIFY-TEST (测试覆盖)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-TEST-01 | vitest 100% 覆盖纯函数 (xp 公式 / 聚类 mock / 投票扣减 / 升级公式) | `pnpm test --run` | P0 |
| NFR-GAMIFY-TEST-02 | typecheck 0 err (新增文件) | `tsc --noEmit` | P0 |
| NFR-GAMIFY-TEST-03 | 13 张表 (G11 W/T/M) 单测 100% pass | vitest | P0 |
| NFR-GAMIFY-TEST-04 | 12 子能力 32 项 acceptance criteria 全部 AC-X 落档 | 人工 / DDD Review | P0 |

### 5.8 NFR-GAMIFY-OBSERVABILITY (可观测)

| ID | 指标 | 测量 | 优先级 |
|---|---|---|---|
| NFR-GAMIFY-OBS-01 | reward 解锁 100% 写 audit log (Transaction 表, per 守门 #13) | 人工 / DDD Review | P0 |
| NFR-GAMIFY-OBS-02 | 投票 / 反应 / confetti WS 推送可观测 (per V0.1 zustand 订阅) | 人工 / dev tools | P1 |
| NFR-GAMIFY-OBS-03 | 5 域 Lead 真人到位后追溯签字可观测 (per 守门 #14 v2) | DDD Review | P1 |

---

## §6 接口需求 (Interface Requirements)

### 6.1 内部接口 (新增 kind 扩展 CanvasElementKind, per P-1)

```typescript
// 6.1.1 扩展 CanvasElementKind 枚举 (8 种新 kind, per 守门 #19 不引入新依赖)
export type CanvasElementKind =
  | "sticky_note"     // V0.1
  | "text"            // V0.1
  | "shape"           // V0.1
  | "image"           // V0.1
  | "embed"           // V0.1
  | "work_item_card"  // V0.1
  | "worktree_node"   // V0.1
  | "agent_cursor"    // V0.1
  | "automation_node" // V0.1
  | "comment_pin"     // V0.1
  | "avatar"          // G1.1 (NEW)
  | "level"           // G1.2 (NEW)
  | "xp"              // G1.3 (NEW)
  | "skill_tree"      // G1.4 (NEW)
  | "class"           // G1.5 (NEW)
  | "badge"           // G1.6 (NEW)
  | "quest"           // G1.7 (NEW)
  | "inventory"       // G1.8 (NEW)
  | "roguelike_character" // G12.1 (NEW, 复用 V0.1)
  | "roguelike_enemy"     // G12.1 (NEW, 复用 V0.1)
  | "roguelike_item"      // G12.1 (NEW, 复用 V0.1)
  | "manga_agent"         // G12.2 (NEW, 复用 V0.1)
  | "manga_enemy"         // G12.2 (NEW, 复用 V0.1)
  | "manga_decoration"    // G12.2 (NEW, 复用 V0.1)
  | "vote_badge"          // G6.2 (NEW)
  | "reaction_floater"    // G7.1 (NEW, Work 短 TTL)
  | "confetti_node";      // G8.1 (NEW, Work 短 TTL)

// 6.1.2 扩展 CanvasElement.content (per 8 种新 kind)
export interface CanvasElement {
  // ... V0.1 字段保留
  content: {
    // V0.1 字段
    text?: string;
    color?: string;
    image_url?: string;
    embed_url?: string;
    work_item_id?: Uuid;
    worktree_id?: Uuid;
    agent_session_id?: Uuid;
    automation_id?: Uuid;
    comment_id?: Uuid;
    // G1.1-1.8 新增
    user_id?: Uuid;          // avatar / level / xp / class / badge / quest / inventory
    agent_id?: Uuid;         // avatar 副键
    level?: number;          // avatar / level
    tier?: number;           // avatar visual tier (1-10)
    xp?: number;             // xp
    xp_to_next?: number;     // level
    class?: string;          // class
    badges?: BadgeRef[];     // badge
    quests?: QuestRef[];     // quest
    items?: InventoryItem[]; // inventory
    // G12.1 Roguelike
    map_cell_type?: "start" | "enemy" | "treasure" | "trap" | "blank" | "boss";
    cell_position?: { x: number; y: number };
    linked_work_item_id?: Uuid;
    // G12.2 Manga
    theme?: "manga" | "wuxia" | "cyberpunk" | "combined";
    decoration_set?: string[];
    // G6.2 vote badge
    votes?: number;
    rank?: number;
    // G7.1 reaction
    reaction_emoji?: string;
    reaction_expires_at?: Iso8601;
    // G8.1 confetti
    confetti_intensity?: "low" | "med" | "high";
    confetti_expires_at?: Iso8601;
  };
}
```

### 6.2 外部接口 (BFF API, mock 优先)

| Method | Path | 说明 | 守门 |
|---|---|---|---|
| `GET` | `/v1/gamify/avatar/{userId}` | 获取 avatar 节点数据 | W/T/M 严格 (per G11.2) |
| `GET` | `/v1/gamify/level/{userId}` | 获取 level 节点数据 | Master |
| `GET` | `/v1/gamify/xp/{userId}` | 获取 xp 节点数据 | Master |
| `GET` | `/v1/gamify/skill-tree/{userId}` | 获取 skill tree 节点数据 | Master |
| `GET` | `/v1/gamify/class/{userId}` | 获取 class 节点数据 | Master |
| `GET` | `/v1/gamify/badges/{userId}` | 获取 badges 节点数据 | Master |
| `GET` | `/v1/gamify/quests/{userId}` | 获取 quests 节点数据 | Master |
| `GET` | `/v1/gamify/inventory/{userId}` | 获取 inventory 节点数据 | Master |
| `POST` | `/v1/gamify/inventory/hold` | 持有 item (mock) | Transaction |
| `POST` | `/v1/gamify/inventory/use` | 使用 item (mock) | Transaction |
| `POST` | `/v1/gamify/inventory/destroy` | 销毁 item (mock) | Transaction |
| `POST` | `/v1/gamify/reward-rules` | 定义 reward rule (mock) | Master |
| `POST` | `/v1/gamify/notify` | 触发通知 (mock) | Transaction |
| `POST` | `/v1/gamify/confetti` | 触发 confetti (本地) | Work |
| `POST` | `/v1/gamify/achievement-evaluate` | 评估 achievement (mock) | Master |
| `POST` | `/v1/gamify/score-rule` | 定义 score rule (mock) | Master |
| `GET` | `/v1/gamify/score/{userId}?dimension={dim}` | 获取 score | Master |
| `GET` | `/v1/gamify/level-from-xp?xp={N}` | xp → level 公式 (纯函数) | 纯函数 |
| `POST` | `/v1/gamify/cluster` | 调 mock AI 聚类 (per 守门 #23 v2) | Transaction |
| `POST` | `/v1/gamify/cluster-visualize` | 聚类结果可视化 (本地) | Work |
| `POST` | `/v1/gamify/vote` | 投票 (mock) | Transaction |
| `POST` | `/v1/gamify/react` | reaction (mock, Work) | Work |
| `GET` | `/v1/gamify/leaderboard?workspace={id}&dim={dim}` | per workspace 排行榜 (1h cache) | Work (cache) |
| `GET` | `/v1/gamify/leaderboard?tenant={id}&dim={dim}` | per tenant 排行榜 (24h cache, admin) | Work (cache) |
| `GET` | `/v1/gamify/daily-challenge?user={id}&date={d}` | daily challenge (mock) | Master |
| `GET` | `/v1/gamify/streak?user={id}` | streak (mock) | Master |
| `POST` | `/v1/gamify/powerup/activate` | 激活 power-up (mock) | Transaction |
| `WS` | `/ws/gamify/events` | 实时事件 (xp.changed / score.changed / vote.changed / reward.unlocked / achievement.unlocked / reaction.fired) | 全 WS |
| `WS` | `/ws/gamify/level` | level.up 事件推送 | WS |
| `WS` | `/ws/gamify/cluster-result` | 聚类结果推送 | WS |

### 6.3 Zustand Store 依赖 (per P-7)

| Action / Getter | 用途 | 备注 |
|---|---|---|
| `useStore((s) => s.agentGameStates)` | G1 复用, 不新建 | per V0.1 `PHASE-AGENT-GAME-IMPL-REPORT.md` §1 #1 |
| `useStore((s) => s.agentMaps)` | G12.1 复用, 不新建 | per V0.1 `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` §1 #6 |
| `useStore((s) => s.canvasElements)` | G1-G12 新增 kind 写入 | per V0.1 `CanvasView.tsx` 已有 |
| `useStore((s) => s.canvasConnectors)` | G5.2 connector 新增 | per V0.1 |
| `useStore((s) => s.canvasFrames)` | G5.2 Frame 新增 | per V0.1 |
| `useStore((s) => s.addCanvasElement)` | 新 kind 写入 | per V0.1 |
| `useStore((s) => s.deleteCanvasElement)` | 销毁 | per V0.1 |
| `useStore((s) => s.moveCanvasElement)` | 拖动 | per V0.1 |
| `useStore.new(gamificationStates)` (NEW) | avatar / level / xp / class / badges 缓存 | per G1 |
| `useStore.new(voteStates)` (NEW) | votes / remaining 缓存 | per G6 |
| `useStore.new(notificationStates)` (NEW) | notification 缓存 (Work) | per G2.2 / G7.1 |

**新增 store 集合 (3 个, per 守门 #19 累积规 P-7 扩展 zustand)**:
- `gamificationStates: Record<Uuid, GamificationState>` — avatar/level/xp/class/badges 缓存 (跟 V0.1 `agentGameStates` 联动)
- `voteStates: Record<Uuid, VoteState>` — votes/remaining 缓存
- `notificationStates: Record<Uuid, NotificationState>` — 通知缓存 (Work 短 TTL, 3s 自动清)

### 6.4 Mock 接口 (per 守门 #23 v2, G5 锁定)

```typescript
// 6.4.1 G5.1 sticky note 聚类 mock 接口
// 走 scripts/automation/ai_edit_mock.py 模板生成 (per 守门 #23 v2)
// 不引入 OpenAI / Anthropic 第三方 API, 不引入 LLM 凭据
// confidence 永远 < 0.5 提示用户手动 review
interface MockClusterRequest {
  stickyNoteIds: Uuid[];
  k: number; // 默认 3
}
interface MockClusterResponse {
  themes: Array<{
    id: Uuid;
    label: string; // mock 生成 (e.g. "主题 1" / "Theme 1" / "テーマ 1")
    stickyNoteIds: Uuid[];
  }>;
  confidence: number; // 永远 < 0.5
  mockUsed: true; // 显式标识
}

// 6.4.2 mock 调用 (frontend 本地)
async function mockCluster(req: MockClusterRequest): Promise<MockClusterResponse> {
  // 走 ai_edit_mock.py 模板, 不调外部 LLM
  // 真实 LLM 接入 P2
}
```

---

## §7 约束 / 依赖 / 风险

### 7.1 守门合规 (per AGENTS.md §4)

| 守门 | 约束 | 本 SRS 落地 |
|---|---|---|
| #1 (R-05 不 push 已反转) | git push 守门 | 本文档同步 commit 必先跑守门 |
| #3 (5 域独立 Lead) | 5 域 Lead 拒绝兼任 | 5 域 Lead 真人到位前 Mavis 临时代签 (per §11) |
| #5 (env 安全) | 不打印 env | BFF 连接字符串走 env, 不打印 (per NFR-GAMIFY-SEC-01) |
| #6 (PowerShell only) | 守门 | 部署脚本 PowerShell |
| #7 (0 unsafe) | 守门 | 新增 TS 0 unsafe, 前端 0 第三方 unsafe |
| #9 (子代理 RPC 不可靠) | 实证不可靠 | 本 SRS 不依赖子代理 dispatch, 全程 Mavis 接手 (per P-14) |
| #10 (代签规则) | Mavis 默认代 Ulysses | author = Ulysses per 守门 #10 + 守门 #14 v3 + 9/8 15:19 第 6 次强化 |
| #11 (缺标比错标) | 显式列"已知缺口" | 本 SRS §10 列 12 项已知缺口 |
| #12 (AI 协作文档治理) | 禁回溯叙事 | 本 SRS 不引 BAS 实证 (无历史) |
| **#13 (W/T/M 三類横展)** | 横展開強制 100% 表覆盖 | G11.2 必含 W/T/M 三類横展表 (15 张表, 100% 覆盖) |
| #14 v2 (5 域 Lead 拍板 D) | Mavis 临时代签, 真人到位后追溯 | §11 签字栏 5 角色全代签, 真人到位后追溯 |
| #15 (docs 同步饱和) | 新事件触发才 commit | 本 SRS 落地 = 17:08 JST 新事件触发, 不算饱和违规 |
| #19 (agent 交互 Python 化) | 强制走 scripts/automation/ | G5 mock 接口走 ai_edit_mock.py (per 守门 #23 v2) |
| #23 v2 (AI mock 锁定) | 不开第三方 LLM API | G5 锁定 mock 接口 (per 守门 #23 v2, confidence < 0.5) |

### 7.2 技术约束

| 约束 | 说明 |
|---|---|
| **TS 严格** | 0 `any` (除 fallback / 类型断言), 跟 V0.1 strict mode 一致 |
| **不引入新依赖** | confetti 走 CSS keyframes + SVG `<animateTransform>` (per V0.1 `Decorations.tsx` `EnergyRing` 同思路), 不引 canvas-confetti 库 (per 守门 #19) |
| **AI mock 锁定** | G5 sticky note 聚类走 `scripts/automation/ai_edit_mock.py` 模板 (per 守门 #23 v2, 9/2 09:01 JST 拍板), 不引入 OpenAI / Anthropic 第三方 API, 不引入 LLM 凭据, confidence 永远 < 0.5 提示用户手动 review |
| **数据 W/T/M 三類** | G11.2 必含 (per 守门 #13), 15 张表 100% 覆盖, 0 混在 |
| **mock 优先** | BFF API 全部 mock, 真实后端 P2 |

### 7.3 业务约束

| 约束 | 说明 |
|---|---|
| **核心方向锚点** | 2026-09-10 17:08 JST Ulysses 拍板"我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" — 不写 Miro 通用功能 (per §1.4 + brief §2) |
| **不重写 V0.1** | 已有 game (agent-game / roguelike / manga / settings) 仅画布集成引用, 不重写功能 |
| **5 域 Lead 真人到位前** | 所有签字栏 Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D), 真人到位后追溯签字覆盖 (修订历史表 +1 行, per §12) |
| **5 域 Lead ≠ 22 DDD bounded context** | per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer, 不建立业务子域↔DDD映射 |

### 7.4 风险 (per 守门 #11 缺标比错标)

| # | 风险 / 缺口 | 影响 | 缓解 / 后续 |
|---|---|---|---|
| **R-1** | confetti 实现路径 (CSS / Lottie / 库) 拍板未定 | 性能 + 视觉一致 | §7 风险: 默认走 CSS + SVG `<animateTransform>` (per V0.1 `Decorations.tsx`); Lottie 备选 P2 |
| **R-2** | AI 聚类 mock 接口 confidence 永远 < 0.5 | 用户需手动 review | UI 提示"建议手动调整"; 真实 LLM 接 P2 (per 守门 #23 v2) |
| **R-3** | 5 域 Lead 真人未到位, 跨域编排决策延后 | 关系定义/权限暂时统一 Mavis 代签 | per 守门 #14 v2 + 9/3 11:35 JST 拍板 B, 真人到位后追溯 |
| **R-4** | Star 25 module 框架下不新增 26 module, gamification 域归 collaboration 域强化 | per V0.1 `frontend-canvas-design.md` §1.1 B 方案 | 已锁定 |
| **R-5** | V0.1 Roguelike + Manga + Settings 实装未覆盖全 32 项 | 部分依赖已实装 (G12 集成), 24 项待 P3-D.6 实装 | P3-D.6 阶段落档 |
| **R-6** | G11.2 W/T/M 三類横展 15 张表 retention 默认值暂定 7d/perm | 实际值待 DDD Review Lead 拍板 | §10 已知缺口 #11 显式 |
| **R-7** | 真实 LLM 接入 (G5.1 真实) 留 P2 | 当前仅 mock | §10 已知缺口 #8 显式 |
| **R-8** | G9.2 per tenant 排行榜 admin 角色 13 租户权限待定 | per 守门 #13 RLS 13 類 | §10 已知缺口 #12 显式 |
| **R-9** | confetti 跟 reaction 同位置互斥 (避免视觉混乱) | 当前拍板: 不同节点位置不冲突 | §10 已知缺口 #10 显式 |
| **R-10** | 文档治理 / 跨专题引用 / 修订历史饱和等 | 多 SRS 同步 | per V0.1 `SRS-AGENT-VIEW-001.md` §10 模式 |

---

## §8 验收标准 (受入基準 / Acceptance Criteria)

### 8.1 功能验收 (Functional AC, 32 项每项 1-2 AC, ≥ 32 个)

| AC | 描述 | 测量 | 关联 |
|---|---|---|---|
| AC-GAMIFY-G1.1.1 | 画布上 1 个 avatar 节点 + level 1 → 灰色边框 + 圆形头 | 手动 | G1.1 |
| AC-GAMIFY-G1.1.2 | level 10 → 金色边框 + 神侠光环 | 手动 | G1.1 |
| AC-GAMIFY-G1.2.1 | level 5 + xp 2500 → "Lv 5" + progress 100% | 手动 + vitest | G1.2 |
| AC-GAMIFY-G1.2.2 | level 7 → 紫色边框 | 手动 | G1.2 |
| AC-GAMIFY-G1.3.1 | claim work-item +10 xp → 节点显示 "XP: 1244" + 浮 "+10" | 手动 | G1.3 |
| AC-GAMIFY-G1.3.2 | WS 推送 xp.changed event → 节点 ≤ 500ms 反映 | 手动 / Playwright | G1.3 |
| AC-GAMIFY-G1.4.1 | level 1 → 仅 1 节点亮 (起点) | 手动 | G1.4 |
| AC-GAMIFY-G1.4.2 | level 5 → 3 节点亮 + DAG 连线显示 | 手动 | G1.4 |
| AC-GAMIFY-G1.5.1 | class "warrior" → 朱红图标 + 朱红边框 | 手动 | G1.5 |
| AC-GAMIFY-G1.5.2 | class "mage" → 霓虹青图标 + 紫边框 | 手动 | G1.5 |
| AC-GAMIFY-G1.6.1 | 1 个 COMMON 徽章 → 灰圈 | 手动 | G1.6 |
| AC-GAMIFY-G1.6.2 | 1 个 LEGENDARY 徽章 → 金圈 + 光晕动画 | 手动 | G1.6 |
| AC-GAMIFY-G1.7.1 | quest 进度 50% → progress bar 半满 | 手动 | G1.7 |
| AC-GAMIFY-G1.7.2 | quest 完成 → 绿色高亮 + confetti | 手动 | G1.7 |
| AC-GAMIFY-G1.8.1 | 持有 1 个 power-up "double_xp" → 物品槽显示 + 倒计时 | 手动 | G1.8 |
| AC-GAMIFY-G1.8.2 | 点 "使用" → 立即生效 + 物品从 inventory 移除 | 手动 | G1.8 |
| AC-GAMIFY-G2.1.1 | rule "claim_wi → +10 xp" 触发 → xp +10 | 手动 | G2.1 |
| AC-GAMIFY-G2.1.2 | 同一 rule 重复触发不重复给奖 | 手动 + vitest | G2.1 |
| AC-GAMIFY-G2.2.1 | 默认 in-app → 右上角 toast + 3s 自动消失 | 手动 | G2.2 |
| AC-GAMIFY-G2.2.2 | 选 email → mock console.log (不真发) | 手动 | G2.2 |
| AC-GAMIFY-G2.3.1 | reward 解锁 → 节点位置 confetti 弹出 | 手动 | G2.3 |
| AC-GAMIFY-G2.3.2 | 3s 后自动消失, 不留 DOM | 手动 | G2.3 |
| AC-GAMIFY-G2.4.1 | streak 7 天 → achievement "weekly_warrior" (COMMON) 自动授予 | 手动 | G2.4 |
| AC-GAMIFY-G2.4.2 | tasks_done 100 → achievement "century_club" (RARE) 自动授予 | 手动 | G2.4 |
| AC-GAMIFY-G3.1.1 | claim work-item → score +10 | 手动 | G3.1 |
| AC-GAMIFY-G3.1.2 | 投票 → score +1 | 手动 | G3.1 |
| AC-GAMIFY-G3.1.3 | 分享 → score +5 | 手动 | G3.1 |
| AC-GAMIFY-G3.2.1 | dashboard 切 dimension → 3 个数字分别显示 | 手动 | G3.2 |
| AC-GAMIFY-G3.2.2 | 跨 day 边界 (JST 0:00) → day_score 重置 | 手动 + vitest | G3.2 |
| AC-GAMIFY-G3.3.1 | 点 score 节点右上角 🏆 → 跳 leaderboard 路由 | 手动 | G3.3 |
| AC-GAMIFY-G3.3.2 | leaderboard 默认按 score 排序 | 手动 | G3.3 |
| AC-GAMIFY-G4.1.1 | xp=0 → level 0, xp_to_next=100 | vitest (纯函数) | G4.1 |
| AC-GAMIFY-G4.1.2 | xp=2500 → level 5, xp_to_next=1100 | vitest | G4.1 |
| AC-GAMIFY-G4.1.3 | xp=10000 → level 10, xp_to_next=0 (MAX) | vitest | G4.1 |
| AC-GAMIFY-G4.2.1 | level 5 → 6 → 节点 scale 1.0 → 1.2 → 1.0 动画 + halo ring 出现 | 手动 | G4.2 |
| AC-GAMIFY-G4.2.2 | 通知 toast 显示 "Level Up! 5 → 6" 3s | 手动 | G4.2 |
| AC-GAMIFY-G4.3.1 | level 5 达 unlock_level=5 → skill 节点亮色 | 手动 | G4.3 |
| AC-GAMIFY-G4.3.2 | skill 解锁 → 通知 toast + confetti | 手动 | G4.3 |
| AC-GAMIFY-G5.1.1 | 选 5 张 sticky_note, k=3 → mock 返回 3 个主题 + 关联 | 手动 | G5.1 |
| AC-GAMIFY-G5.1.2 | 调 mock 接口, 不调 OpenAI / Anthropic | 人工 / grep (per 守门 #23 v2) | G5.1 |
| AC-GAMIFY-G5.1.3 | confidence < 0.5 时 UI 提示"建议手动调整" | 手动 | G5.1 |
| AC-GAMIFY-G5.2.1 | 聚类结果 → 1 Frame 创建, 含 K 个主题 label + 关联 sticky_note | 手动 | G5.2 |
| AC-GAMIFY-G5.2.2 | 主题 Frame 颜色 5 色 (黄/粉/蓝/绿/紫) | 手动 | G5.2 |
| AC-GAMIFY-G6.1.1 | 用户有 5 票, 投 1 张 → 剩余 4 票 | 手动 | G6.1 |
| AC-GAMIFY-G6.1.2 | 0 票时按钮 disabled + tooltip "票数已用完" | 手动 | G6.1 |
| AC-GAMIFY-G6.1.3 | 跨 session 票数重置 (MVP) | 手动 | G6.1 |
| AC-GAMIFY-G6.2.1 | 投票 → 节点右上角 badge "5" 实时显示 | 手动 | G6.2 |
| AC-GAMIFY-G6.2.2 | top 3 节点金边框高亮 | 手动 | G6.2 |
| AC-GAMIFY-G7.1.1 | 8 种 emoji 可选 (👍 ❤️ 🎉 😄 🤔 👀 🔥 ⭐) | 手动 | G7.1 |
| AC-GAMIFY-G7.1.2 | 浮在节点上方 + 3s 淡出 | 手动 | G7.1 |
| AC-GAMIFY-G7.1.3 | 不进主 store (per BR-4, Work 表) | 人工 review | G7.1 |
| AC-GAMIFY-G8.1.1 | 4 种触发场景 (complete / unlock / levelup / achievement) 都触发 confetti | 手动 | G8.1 |
| AC-GAMIFY-G8.1.2 | 1.5-3s 自动消失 | 手动 | G8.1 |
| AC-GAMIFY-G8.1.3 | 不引第三方 confetti 库 (per 守门 #19) | package.json diff (空) | G8.1 |
| AC-GAMIFY-G9.1.1 | 3 维度可切 (token / tasks_done / votes) | 手动 | G9.1 |
| AC-GAMIFY-G9.1.2 | top 10 显示 + 排名高亮 (1-3 金/银/铜) | 手动 | G9.1 |
| AC-GAMIFY-G9.2.1 | admin 角色可见 | 手动 | G9.2 |
| AC-GAMIFY-G9.2.2 | 跨 workspace 聚合 (去重 user) | 手动 | G9.2 |
| AC-GAMIFY-G10.1.1 | 每日 3 个任务 | 手动 | G10.1 |
| AC-GAMIFY-G10.1.2 | 跨 TZ 边界 (JST 0:00) → 任务重置 | 手动 + vitest | G10.1 |
| AC-GAMIFY-G10.2.1 | 连续 7 天 → streak 7 + 大奖励通知 | 手动 | G10.2 |
| AC-GAMIFY-G10.2.2 | 1 天中断 → streak 重置为 0 | 手动 + vitest | G10.2 |
| AC-GAMIFY-G11.1.1 | 3 种类型可选 (double_score / auto_cluster / stealth) | 手动 | G11.1 |
| AC-GAMIFY-G11.1.2 | 24h 倒计时 | 手动 | G11.1 |
| AC-GAMIFY-G11.1.3 | 隐身模式 = canvas 节点半透明 (不干扰) | 手动 | G11.1 |
| AC-GAMIFY-G11.2.1 | 持有 / 使用 / 销毁 3 action 完整 | 手动 | G11.2 |
| AC-GAMIFY-G11.2.2 | 数据表符合 W/T/M 三類横展 (per 守门 #13) | DDD Review 拍板 | G11.2 |
| AC-GAMIFY-G12.1.1 | 画布上画 6 种 cell (start/enemy/treasure/trap/blank/boss) | 手动 | G12.1 |
| AC-GAMIFY-G12.1.2 | enemy cell 跟 work-item 关联, 走 claim 链路 (per V0.1) | 手动 | G12.1 |
| AC-GAMIFY-G12.2.1 | 画布节点 visual = Manga 主题 (圆形头 + 武士刀) | 手动 | G12.2 |
| AC-GAMIFY-G12.2.2 | Lv 7+ 节点加神侠光环 (per V0.1 `Decorations.tsx`) | 手动 | G12.2 |

**总计 73 AC** (≥ 32 个达标, 32 项 × 2.3 平均 AC, 跟 V0.1 `SRS-AGENT-VIEW-001.md` §9 一致)

### 8.2 质量验收 (Quality AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-Q-1 | vitest 100% 覆盖纯函数 (xp 公式 / 聚类 mock / 投票扣减 / 升级公式) | `pnpm test --run src/lib/gamify` |
| AC-Q-2 | typecheck 0 err (新增文件) | `tsc --noEmit` |
| AC-Q-3 | 不引入新依赖 (per 守门 #19) | `package.json` diff (空) |
| AC-Q-4 | commit author = Ulysses | `git log --format='%an <%ae>' HEAD` |
| AC-Q-5 | 7 段报告落档 (per AGENTS.md §3) | `docs/reports/PHASE-CANVAS-GAMIFY-IMPL-REPORT.md` 存在 |
| AC-Q-6 | 派生确定性 (NFR-GAMIFY-DATA-01) | vitest "排序稳定" / "deterministic" 2 个测试 pass |
| AC-Q-7 | W/T/M 三類横展 100% 表覆盖 (per 守门 #13) | DDD Review 拍板 + 15 张表 audit |
| AC-Q-8 | 守门 #23 v2 AI mock 接口锁定 (不开 OpenAI / Anthropic) | grep "openai\|anthropic" 0 hit |

### 8.3 文档验收 (Documentation AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-D-1 | 本 SRS (要件定義書) 落档 | `docs/requirements/SRS-CANVAS-GAMIFY-001.md` 存在 |
| AC-D-2 | BD (基本設計書) 落档 (P3-D.6 阶段) | `docs/design/BD-CANVAS-GAMIFY-001.md` 存在 |
| AC-D-3 | 実装報告 (7 段) 落档 (P3-D.6 阶段) | `docs/reports/PHASE-CANVAS-GAMIFY-IMPL-REPORT.md` 存在 |
| AC-D-4 | self-review 落档 (P3-D.6 阶段) | `docs/reports/PHASE-CANVAS-GAMIFY-SELF-REVIEW.md` 存在 |
| AC-D-5 | 跨专题引用完整 (per §1.2 + §1.3 + §1.4) | 引用 SRS-CANVAS-001 / SRS-CANVAS-AGENT-001 / V0.1 game 4 份 PHASE 报告 / SRS-AGENT-VIEW-001 |

---

## §10 5 角色签字栏 (per AGENTS.md §3 7 段结构, per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D + 9/10 12:45 JST v0.62 反转)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构师 (Mavis 接手 agent per DEC-008)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (代签) | 2026-09-10 JST | 5 域 Lead 真人未到位, Mavis 临时代签 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, 真人到位后追溯签字 |
| **平台 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **评审主持 (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 |
| **PM (Mavis 接手)** | 🟢 Mavis 接手 | 2026-09-10 JST | per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 |

**5 域 Lead (player / economy / match / social / admin)** 真人未到位, Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)

---

## §11 修订履历

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| **v0.1** | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初版, 9 段 IPA SEC 结构, 12 子能力 32 项展开 (FR 32 + NFR 25 + AC 73 + US 23), G11.2 W/T/M 三類横展 15 张表 100% 覆盖 (per 守门 #13), G5 sticky note 聚类 AI mock 接口锁定 (per 守门 #23 v2), 不写 Miro 通用功能 (per brief §2 + 17:08 JST 拍板), 23 用户故事 (≥ 19 达标), 73 验收标准 (≥ 32 达标), 12 已知缺口 (≥ 8 达标), 守门 #1 + #3 + #5 + #6 + #7 + #9 + #10 + #11 + #12 + #13 + #14 v2 + #15 + #19 + #23 v2 全过, 5 域 Lead 真人到位前 Mavis 临时代签, 真人到位后追溯签字覆盖 | 2026-09-10 17:08 JST Ulysses 拍板"我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" + 子代理 brief `docs/briefs/srs-canvas-gamify-001.md` v0.1 (17:17 JST 落档, per ask_user 拍板重写自原 srs-canvas-content-001) |

---

## 附录 A: 已知缺口 (per 守门 #11 缺标比错标, ≥ 8 个, 本 SRS 列 12 个)

| # | 缺口 | 影响 | 后续 |
|---|---|---|---|
| **#1** | 真实 user 上传头像 (G1.1) — 当前 V0.1 `AGENT_VISUAL_TIERS` 用 tier 颜色 + 神侠光环代替 | 真实感弱 | V2 候选 |
| **#2** | G3.2 score 累计 TZ 用户配置 — 当前固定 JST | 跨 TZ 用户不准 | P2 改 user TZ preference |
| **#3** | G3.3 score 排行榜入口 + G6.1 dot voting 跨 session 持久化 — MVP 仅 session 内 | 跨 session 丢 | P2 走 server-side config |
| **#4** | G4.1 公式用户可配 (sigmoid / 指数) — 当前固定线性 `sqrt(xp/100)` | 不可调 | P2 改 formula configurable |
| **#5** | G4.2 升级跳过 level (经验值突增跳级) 动画合并 — 当前单级动画 | 多级连升体验弱 | P2 加合并动画 |
| **#6** | G4.3 skill 复合解锁 (A 解锁 + B level 5) — 当前单条件 | 复杂 skill 树弱 | P2 加 AND/OR |
| **#7** | G7 reaction 撤回 (用户撤回自己的 reaction) — 当前发出不可撤 | 体验弱 | P2 |
| **#8** | G5 真实 LLM 接入 — 当前仅 mock (per 守门 #23 v2) | confidence < 0.5 | P2 改 OpenAI / Anthropic, 但需先破守门 #23 v2 |
| **#9** | G8 confetti 音效 (跟 reaction 类似) — 当前无声音 | 反馈弱 | P2 加 framer-motion 音效 |
| **#10** | G2.3 confetti 跟 G7.1 reaction 互斥 (同位置不同时) — 当前拍板: 不同节点位置不冲突 | 视觉一致性 OK | 已显式 |
| **#11** | G11.2 W/T/M 三類横展 15 张表 retention 默认值暂定 7d (Work) / permanent (Transaction / Master SCD Type 2) | 实际值待 DDD Review 拍板 | DDD Review |
| **#12** | G9.2 per tenant 排行榜 admin 角色 13 租户权限 (per 守门 #13 RLS 13 類) — 当前 admin 简单校验 | 跨租户隔离弱 | DDD Review 拍板 |

**DDD Review 必查**: 缺口 #11 (W/T/M retention) + #12 (admin 13 租户) + #5 (G12 跟 V0.1 Roguelike 共享 state) + #7 (G5 真实 LLM 接 P2)

---

## 附录 B: 跨专题引用清单 (per brief §7 返报 #7)

| 引用 | 位置 | 用途 |
|---|---|---|
| `SRS-CANVAS-001.md` v0.1 (root 重写) | §1.4 不包含范围 | 总册双核心索引 + 跨块接口 |
| `SRS-CANVAS-AGENT-001.md` v0.1 (子代理 1 落档) | §1.4 不包含范围 | 双核心之 1: agent 管理 (28 项) |
| `SRS-AGENT-VIEW-001.md` v1.0 | §1.2 背景 + §2 用語定义 + §3 业务背景 + §5 NFR | V0.1 agent 视图协同 (派生视图 NFR / 用語 / StatusPill 色码) |
| `SRS-AGENT-RELATIONSHIP-001.md` v0.1 | §1.2 背景 + §2 用語定义 + §6 接口 | 平行 ARG 关系层 (per 9/8 22:35 落档) |
| `frontend-canvas-design.md` v0.1 | §1.1 目的 + §2 用語 + §3 业务背景 + §6 接口 | 画布 V0.1 详细设计 (Canvas / CanvasElement / CanvasConnector / Bezier 公式) |
| `PHASE-AGENT-GAME-IMPL-REPORT.md` v0.1 | §1.1 + §3 业务背景 + §6 接口 | V0.1 拟人化游戏化 (49 tests, leveling.ts / perks.ts / AGENT_VISUAL_TIERS) |
| `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` v0.1 | §1.1 + §3 业务背景 + §4.1 G12.1 | V0.1 Roguelike (35 tests, mapgen.ts / movement.ts / RoguelikeCanvas) |
| `PHASE-AGENT-MANGA-IMPL-REPORT.md` v0.1 | §1.1 + §3 业务背景 + §4.1 G12.2 | V0.1 日漫 + 武侠 + 赛博朋克 主题 (14 tests, theme.ts / characters.tsx / enemies.tsx / Decorations.tsx) |
| `PHASE-AGENT-THEME-IMPL-REPORT.md` v0.1 | §1.1 + §3 业务背景 + §4.1 G12.2 | V0.1 dark/light 主题切换 (11 tests, theme-tokens.ts) |
| `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1 | §1.1 + §3 业务背景 | V0.1 Agent 设置 (16 tests, settings.ts / AgentSettingsTab) |
| `frontend/src/types/ids.ts` line 686-696 | §6.1 内部接口 | CanvasElementKind 10 种 V0.1 kind (扩展 8 种新 kind) |
| `frontend/src/components/CanvasView.tsx` line 156-168 / 236-252 / 253-262 | §1.2 背景 + §6.1 | V0.1 sticky_note / automation_node / comment_pin baseline |
| `AGENTS.md §4 守门 #13` (9/1 18:30 JST 拍板) | §4.1 G11.2 + §5 NFR + §6 接口 | DB W/T/M 三類横展派生规 (a/b/c + 100% 表覆盖) |
| `AGENTS.md §4 守门 #23 v2` (9/2 09:01 JST 拍板) | §4.1 G5.1 + §6.4 Mock 接口 | AI mock 接口锁定, 不开 OpenAI / Anthropic |
| `AGENTS.md §4 守门 #14 v2` (9/3 19:43 JST) | §1.5 用户故事 + §11 签字栏 | 5 域 Lead 真人到位前 Mavis 临时代签, 真人到位后追溯 |
| `AGENTS.md §3 报告 7 段结构` | §9 修订履历 + 引用 PHASE 报告 | 文档结构对齐 |

---

## 附录 C: 不写 Miro 通用功能清单 (per brief §2 + §7 返报 #8, 防止 scope creep)

| Miro 通用功能 | 处理 | 备注 |
|---|---|---|
| 12 种 diagram (Flowchart / BPMN / ER / Wireframe / Kanban / Sequence) | ❌ 砍掉, 留 P3+ | 本 SRS 不覆盖 |
| 模板库 (2500+) | ❌ 砍掉 | 本 SRS 不覆盖 |
| 编辑效率: Undo | ✅ 留下 (基础画布能力, 留 P1) | 不在本 SRS, per V0.1 派生只读约束 |
| 编辑效率: Group / Box select / Copy-Paste / Align / Distribute | ❌ 砍掉 | 本 SRS 不覆盖 |
| AI 能力: 文本生成 diagram / 翻译 / 图像识别 | ❌ 砍掉 | 本 SRS 不覆盖 |
| AI 能力: sticky note 聚类 | ✅ 留下 (本 SRS G5) | 走 mock 接口 (per 守门 #23 v2) |
| Tables / Chart widget / Form | ❌ 砍掉 | 本 SRS 不覆盖 |
| 多人编辑 / 实时 cursor / 评论 | ❌ 砍掉 (已有 PresenceCursor per `frontend-canvas-design.md` §4.6 不重写) | 本 SRS 不覆盖 |
| 演示 (Frame as slide / Guided Tour) | ❌ 砍掉 | 本 SRS 不覆盖 |
| 互动通用: Timer workshop 用 / Cursor chat / Async video | ❌ 砍掉 | 本 SRS 不覆盖 |
| 互动通用: Reaction / Confetti / Dot voting | ✅ 留下 (本 SRS G6-G8) | per 9/10 17:08 JST 拍板 |
| 导出 (PDF / Word / Excel / CSV) | ❌ 砍掉, 仅留 PNG (V0.1 已实装) | 本 SRS 不覆盖 |
| 集成 (Slack / Jira / Asana / Figma / GitHub) | ❌ 砍掉 | 本 SRS 不覆盖 |
| 版本 (Version history / Branching) | ❌ 砍掉 | 本 SRS 不覆盖 |
| 移动 (iOS / Android / Touch) | ❌ 砍掉 | 本 SRS 不覆盖 |
| 完整 a11y (WCAG 2.1 AA) | ❌ 砍掉, 仅基础键盘可达 | 本 SRS NFR-GAMIFY-A11Y 基础 |
| 基础画布能力 (pan/zoom/select/element/connector/frame) | ✅ V0.1 已有, 不重写 | per `frontend-canvas-design.md` v0.1 |
| 已有 game (agent-game / roguelike / manga / settings) | ✅ V0.1 已实装, 仅画布集成引用 (G12), 不重写功能 | per §1.1 |

---

## 附录 D: 5 角色签字栏 (per AGENTS.md §3 7 段结构, per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| 架构 | 🟢 Mavis 接手 (per DEC-008) | 2026-09-10 | 8/27 19:39 JST 用户授权代签 |
| SRE Lead | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 平台 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| 评审主持 | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| PM | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |

**真人到位后追溯签字覆盖** = 修订历史表 +1 行 (per 守门 #3 + 9/3 19:35 JST 拍板 D 维持, 9/10 12:45 JST v0.62 反转升级为 Mavis 审核 决定 author=Ulysses)
