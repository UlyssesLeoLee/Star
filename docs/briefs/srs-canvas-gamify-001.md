# Brief: srs-canvas-gamify-001

**Agent**: worker (root 派发, 2 子代理并行之 2/2 per 守门 #9 v20 + v27 3 段 fallback)
**Phase**: P3-D.5 无限画布需求文档化 (Ulysses 17:08 JST 拍板: agent 管理 + 游戏化, 避免过度冗余)
**Created**: 2026-09-10 17:17 JST (重写自原 srs-canvas-content-001, 方向重置)
**Token 预算**: ~0.3M (守门 #4 / #19 估算, 1 SRE·周 = 1.2M 留 4x 缓冲)
**Worktree**: 在 root 当前 main worktree 直实装 (per 守门 #9 #3 实证 5/5 RPC 不可靠)

---

## 0. 触发

2026-09-10 17:08 JST Ulysses 拍板"审核时的观点是要明确, 我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" — 双核心之 2: 游戏化。

撤回范围: 12 大类 #3 图表 (12 种 diagram) / #4 模板 / #5 编辑效率 (12 项 Miro 通用) / #10 AI / #12 数据接入 全部砍掉, **仅保留游戏化** 相关:
- 已有: agent-game / agent-roguelike / agent-manga / agent-settings (V0.1 已实装, 不重写)
- 新增: 画布上的 gamification 节点 (sticky note 聚类 + dot voting + reaction + confetti + 奖励成就 + 积分等级 + 排行榜 + RPG 元素)

## 1. 范围 (in-scope)

### 1.1 本专题覆盖 — 游戏化域 (双核心之 2)

**核心**: 画布上**游戏化机制** + RPG 元素 + 奖励成就 + 互动工具 (投票 / 表情 / 聚类 / confetti)。

| 子能力 | 范围 | 项数 |
|---|---|---|
| G1 gamification 节点 | 新 element kind (avatar / level / xp / skill tree / class / badge / quest / inventory) | 8 项 |
| G2 reward / achievement | 解锁条件 + 通知 + 画布特效 | 4 项 |
| G3 score / points | 操作积分 + 排行榜 | 3 项 |
| G4 leveling / skill tree | 升级 + 技能解锁 | 3 项 |
| G5 sticky note 聚类 (AI) | 跟 SRS-AGENT 协同, AI 自动归类成主题 | 2 项 |
| G6 dot voting | 圆点投票 + 决策 | 2 项 |
| G7 reaction | emoji 表情回应 (短时显示) | 1 项 |
| G8 confetti | 庆祝特效 (完成 / 解锁 / 升级) | 1 项 |
| G9 leaderboard | 排行榜 (按 token / 任务完成数 / vote 数量) | 2 项 |
| G10 daily challenge / streak | 每日任务 + 连续天数 | 2 项 |
| G11 power-up / inventory | 道具 + 物品 | 2 项 |
| G12 跟 agent-game 集成 (V0.1) | Roguelike + Manga + Settings 4 份 PHASE 报告 引用 | 2 项 |
| **合计** | | **32 项** |

子代理必须**逐项展开**为 SRS 需求条目, 不得合并 / 跳过 / 简写。每项含:
- ID (e.g. `F-GAMIFY-G1.1` avatar 节点)
- 标题 + 1 句描述
- 优先级 (P0/P1/P2/P3)
- 用户故事 (US-x)
- 功能需求 (FR-x.y)
- 非功能需求 (NFR-x.y, 性能/可访问性/安全)
- 数据字段 (如有, 列出 schema 增项)
- 接口依赖 (BFF API / WS)
- 验收标准 (AC-x.y)
- 已知缺口 (per 守门 #11 缺标比错标)

### 1.2 引用 baseline (必读, 不能编造)

| 文档 | 用途 | 路径 |
|---|---|---|
| Ulysses 17:08 JST 拍板 | **本 SRS 的核心方向锚点** | 本 brief §0 |
| V0.1 agent-game 实现 | Roguelike 游戏化基础 | `frontend/src/components/agent-game/RoguelikeCanvas.tsx` + `PHASE-AGENT-ROGUELIKE-IMPL-REPORT.md` v0.1 (8.4KB) |
| V0.1 agent-manga 实现 | 漫画风格主题 | `PHASE-AGENT-MANGA-IMPL-REPORT.md` v0.1 (8.4KB) |
| V0.1 agent-game 实现 | 游戏核心 (RPG 元素) | `PHASE-AGENT-GAME-IMPL-REPORT.md` v0.1 (10KB) |
| V0.1 agent-theme 实现 | 主题 (跟游戏化相关) | `PHASE-AGENT-THEME-IMPL-REPORT.md` v0.1 (5.8KB) |
| V0.1 agent-settings 实现 | 玩家设置 | `frontend/src/components/agent-game/AgentSettingsTab.tsx` + `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1 (7.1KB) |
| V0.1 agent-view SRS | agent 视图协同 | `docs/requirements/SRS-AGENT-VIEW-001.md` v1.0 |
| V0.1 agent 联动 (canvas) | §4.6 PresenceCursor 升级 | `docs/frontend-canvas-design.md` §4.6 |
| V0.1 StatusPill 60+ 色码 | 状态色码同步基线 | per `frontend-canvas-design.md` §3.4 + ADR-FE-013 |
| V0.1 automation_node | 跟 gamification_node 协同 (rule trigger) | `frontend/src/components/CanvasView.tsx` line 236-252 |
| V0.1 comment_pin | 跟 reaction / dot voting 协同 | `frontend/src/components/CanvasView.tsx` line 253-262 |
| V0.1 sticky_note | 跟聚类 / 投票 协同 | `frontend/src/components/CanvasView.tsx` line 156-168 |
| SRS 模板 (9 段 IPA SEC 结构) | 严格按 §0~§9 | `docs/requirements/SRS-AGENT-VIEW-001.md` (31KB) |
| 平行 SRS 参照 | `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` (37KB) | — |
| 守门 #13 DB W/T/M | 涉及数据存储必含三類横展 | per AGENTS.md §4 #13 |
| 守门 #23 v2 AI 第三方 API 禁止 | AI 聚类走 mock 接口, 不引入第三方 LLM | per AGENTS.md §4 #23 v2 |

### 1.3 文档结构 (per SRS-AGENT-VIEW-001 9 段模板, 不得改结构)

```
§0 文档信息 / 修订履历
§1 文档目的 / 适用范围
§2 用語定義
§3 業務背景 / 前提条件
§4 功能需求
§5 非功能需求
§6 接口需求
§7 约束 / 依赖 / 风险
§8 验收标准
§9 修订履历
```

### 1.4 输出文件

| 文件 | 内容 | 预估大小 |
|---|---|---|
| `docs/requirements/SRS-CANVAS-GAMIFY-001.md` | 本专题 SRS 完整 9 段, 32 项展开 | ~30-40K 字 |

**仅输出 1 份文件**, 不拆 commit, 不写 report, 不动 implementation。

## 2. 范围外 (out-of-scope, 由其他子代理 / root 处理)

| 类别 | 处理方 |
|---|---|
| 双核心之 1: agent 管理 (28 项) | 子代理 1 (SRS-CANVAS-AGENT-001) |
| 总册 SRS-CANVAS-001 (双核心索引 + 跨块接口 + 共享约束) | root (重写) |
| Miro 12 种 diagram (Flowchart / BPMN / ER / Wireframe / Kanban / Sequence) | ❌ 砍掉, 留 P3+ |
| Miro 编辑效率 (Undo / Group / Box select / Copy-Paste / Align / Distribute) | ❌ 部分砍掉, **Undo 留下** (基础画布能力, 留 P1), Group / Box select / Copy-Paste 砍掉 |
| Miro 模板库 (2500+) | ❌ 砍掉 |
| Miro AI 能力 (文本生成 diagram / 翻译 / 图像识别) | ❌ 部分砍掉, **sticky note 聚类留下** (本专题 G5) |
| Miro Tables / Chart widget / Form | ❌ 砍掉 |
| Miro 协作 (多人编辑 / 实时 cursor / 评论) | ❌ 砍掉 |
| Miro 演示 (Frame as slide / Guided Tour) | ❌ 砍掉 |
| Miro 互动通用 (Timer workshop 用 / Cursor chat / Async video) | ❌ 部分砍掉, **Reaction / Confetti / Dot voting 留下** (本专题 G6-G8) |
| Miro 导出 (PDF / Word / Excel / CSV) | ❌ 砍掉, 仅留 PNG (V0.1 已实装) |
| Miro 集成 (Slack / Jira / Asana / Figma / GitHub) | ❌ 砍掉 |
| Miro 版本 (Version history / Branching) | ❌ 砍掉 |
| Miro 移动 (iOS / Android / Touch) | ❌ 砍掉 |
| Miro 完整 a11y (WCAG 2.1 AA) | ❌ 砍掉 |
| 基础画布能力 (pan/zoom/select/element/connector/frame) | ✅ V0.1 已有, 不重写 |
| 已有 game (agent-game / roguelike / manga / settings) | ✅ V0.1 已实装, 仅画布集成引用, 不重写功能 |

## 3. 已知缺口 (per 守门 #11 缺标比错标)

子代理须在 SRS §3 / §7 / §8 显式列已知缺口, 不得隐藏:
- G1 gamification 节点 (8 种 kind) 跟 V0.1 element kind 扩展关系 → §3 业务背景明确
- G2 reward / achievement 解锁条件 (rule engine) 跟 V0.1 automation 域对接 → §6 接口必含
- G3 score / points 累计算法 (per session / per day / all time) → §4 FR 必含
- G4 leveling 公式 (xp → level 曲线) → §4 FR 必含, 默认线性
- G5 sticky note 聚类 (AI) 走 mock 接口 (per 守门 #23 v2), 真实 LLM 接 P2 → §7 风险
- G6 dot voting 票数限制 (per user N 票) → §4 FR 必含
- G7 reaction emoji 库 (8-12 种) → §4 FR 必含
- G8 confetti 动画 (CSS / Lottie / 库选型) → §7 风险
- G9 leaderboard 跨域 (per workspace / per tenant) → §4 FR 必含
- G10 daily challenge / streak 时间窗口 (per user TZ) → §4 FR 必含
- G11 power-up / inventory 持久化 (per 守门 #13 DB W/T/M) → §4 FR 必含三類横展
- G12 跟 agent-game V0.1 集成 (4 份 PHASE 报告) → §1.4 不包含范围明确
- 5 域 Lead 真人未到位, 跨域编排决策延后 → §7 约束
- Star 25 module DDD bounded context 不建立业务子域↔DDD映射 (per 2026-08-31 22:45 JST Q1-D 拍板) → §7 约束

## 4. 守门硬约束 (per 守门 #1 + 守门 #13 + 守门 #23 v2)

- 文档结构严格 9 段, 不增不减
- 32 项每项**全部展开**, 不合并, 不简写
- 用户故事 ≥ 19 个 (32 项 × 60% 覆盖)
- 验收标准 ≥ 32 个 (每项 1-2 个 AC)
- 已知缺口 ≥ 8 个
- **G11 power-up / inventory 必含 W/T/M 三類横展** (per 守门 #13)
- **G5 sticky note 聚类 AI 必走 mock 接口**, 真实 LLM 留 P2 (per 守门 #23 v2)
- **不写 Miro 通用功能** (本专题仅游戏化, 跟 agent 管理交叉部分在总册 §4.4 跨块接口)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权)
- 修订人 / 审批者 = `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 守门 #14 v3 Mavis 永久代签 + 9/8 15:19 JST 第 6 次强化)
- 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
- 0 文件改动除输出 SRS
- 0 commit, 仅产出 markdown (root 统一 commit per 守门 #1 v15)

## 5. 子能力 32 项 详细 (root 输入)

### G1 gamification 节点 (8 项)
- G1.1 `avatar_node` (玩家头像, 关联 user_id)
- G1.2 `level_node` (等级, 关联 xp)
- G1.3 `xp_node` (经验值, 实时更新)
- G1.4 `skill_tree_node` (技能树, 树状结构)
- G1.5 `class_node` (职业, e.g. Warrior / Mage / Rogue)
- G1.6 `badge_node` (徽章, 解锁条件)
- G1.7 `quest_node` (任务, 完成条件 + 奖励)
- G1.8 `inventory_node` (物品栏, 道具 + 装备)

### G2 reward / achievement (4 项)
- G2.1 reward 解锁条件 (rule engine, 跟 V0.1 automation 域对接)
- G2.2 reward 通知 (push / in-app / email)
- G2.3 reward 画布特效 (confetti 触发)
- G2.4 achievement 徽章自动授予

### G3 score / points (3 项)
- G3.1 操作积分规则 (完成任务 +10, 投票 +1, 分享 +5)
- G3.2 score 累计 (per session / per day / all time)
- G3.3 score 排行榜入口

### G4 leveling / skill tree (3 项)
- G4.1 xp → level 公式 (默认线性: level = sqrt(xp / 100))
- G4.2 升级动画 + 通知
- G4.3 skill tree 解锁 (level 达到解锁条件)

### G5 sticky note 聚类 (AI, 走 mock) (2 项)
- G5.1 选中 N 张 sticky_note → AI 聚类成 K 个主题 (K 用户可调)
- G5.2 聚类结果可视化 (主题 Frame + connector)

### G6 dot voting (2 项)
- G6.1 每用户 N 票 (默认 5 票/session), 点 sticky_note 投票
- G6.2 投票结果实时显示 (票数 badge + 排名)

### G7 reaction (1 项)
- G7.1 emoji 表情回应 (8-12 种: 👍 ❤️ 🎉 😄 🤔 👀 🔥 ⭐), 短时显示 (3s 淡出)

### G8 confetti (1 项)
- G8.1 confetti 动画 (CSS / Lottie / 库选型, 触发: 完成 / 解锁 / 升级)

### G9 leaderboard (2 项)
- G9.1 per workspace 排行榜 (token 用量 / 任务完成数 / vote 数量)
- G9.2 per tenant 排行榜 (跨 workspace, 管理员可见)

### G10 daily challenge / streak (2 项)
- G10.1 daily challenge (每日 3 个任务, 完成得 reward)
- G10.2 streak 连续天数 (中断清零, 7 天奖励)

### G11 power-up / inventory (2 项)
- G11.1 power-up 道具 (双倍积分 / 自动聚类 / 隐身模式, 限时)
- G11.2 inventory 物品栏 (持有 + 使用, per 守门 #13 W/T/M 三類横展)

### G12 跟 agent-game 集成 (V0.1 已实装) (2 项)
- G12.1 Roguelike 集成 (画布节点 = 角色 / 怪物 / 道具)
- G12.2 Manga 集成 (画布节点 = 漫画风格主题)

### 优先级建议
- P0: G1.1-1.6 (核心 RPG 元素), G3.1-3.2, G4.1-4.2, G6.1-6.2, G8.1, G12.1-12.2 (V0.1 集成)
- P1: G1.7-1.8, G2.1-2.4, G3.3, G4.3, G5.1-5.2, G7.1, G9.1, G10.1-10.2
- P2: G9.2, G11.1-11.2

## 6. 落地清单

| # | 文件 | 内容 | 行数预估 |
|---|---|---|---|
| 1 | `docs/requirements/SRS-CANVAS-GAMIFY-001.md` | 9 段 SRS, 32 项展开 | ~1000-1300 行 |

预估 0 commit (root 统一 commit), 1 文件, ~30-40K 字。

## 7. 返报告知 (per 守门 #9 v27 collect_output)

子代理返回时, 报告必须含:
1. 实际写入文件路径 + 字节数
2. 9 段是否齐全
3. 32 项展开计数: FR 数量 + NFR 数量 + AC 数量 + US 数量
4. 已知缺口清单 (≥ 8 个)
5. **G11 power-up / inventory W/T/M 三類横展 表格是否完整** (per 守门 #13)
6. **G5 sticky note 聚类 AI mock 接口是否标注** (per 守门 #23 v2)
7. 跨专题引用清单 (引用了 SRS-CANVAS-AGENT-001 / SRS-CANVAS-001 / 现有 V0.1 game 4 份 PHASE 报告 / SRS-AGENT-VIEW-001 的具体 §)
8. **不写 Miro 通用功能** 清单 (本专题明确砍掉的, 防止 scope creep)
9. 任何意外 / 偏离 / 简化 / 跳过 项, 显式标注

不要只回 "done" — 必须给可验证证据.
