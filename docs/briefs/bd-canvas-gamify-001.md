# Brief: bd-canvas-gamify-001

**Agent**: worker (root 派发, 2 子代理并行之 2/2 per 守门 #9 v20 + v27 3 段 fallback)
**Phase**: P3-D.5 BD 基本设计 (18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档")
**Created**: 2026-09-10 18:02 JST
**Token 预算**: ~0.3M (守门 #4 / #19 估算, 1 SRE·周 = 1.2M 留 4x 缓冲)
**Worktree**: 在 root 当前 main worktree 直实装 (per 守门 #9 #3 实证 5/5 RPC 不可靠, 不派二级子代理)

---

## 0. 触发

2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" — 基于 SRS-CANVAS-GAMIFY-001.md v1.0 (92KB, 32 项 G1-G12, 73 AC + 23 US + 12 已知缺口 + 15 张表 W/T/M 100% 覆盖) 制作游戏化域 BD.

17:08 JST 拍板"避免过度冗余" + 守门 #23 v2 AI 第三方 API 禁止 (G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2) + 守门 #13 DB W/T/M 三類横展.

## 1. 范围 (in-scope)

### 1.1 本专题 BD 覆盖 — 游戏化域 (双核心之 2)

**核心**: G1-G12 (12 子能力, 32 项) 的基本设计, 包括 gamification 节点 + reward/achievement + score/points + leveling/skill tree + sticky note 聚类 (AI mock) + dot voting + reaction + confetti + leaderboard + daily challenge/streak + power-up/inventory + V0.1 game 集成.

| 子能力 | 来源 SRS | 内容 | 项数 |
|---|---|---|---|
| G1 gamification 节点 | SRS §4.1 | avatar / level / xp / skill_tree / class / badge / quest / inventory | 8 |
| G2 reward/achievement | SRS §4.2 | 解锁条件 + 通知 + 画布特效 + 自动授予 | 4 |
| G3 score/points | SRS §4.3 | 操作积分规则 + 累计 + 排行榜入口 | 3 |
| G4 leveling/skill tree | SRS §4.4 | xp → level 公式 + 升级动画 + skill tree 解锁 | 3 |
| G5 sticky note 聚类 (AI, mock) | SRS §4.5 | 选中 N 张 → AI 聚类 K 主题 (走 mock, per 守门 #23 v2) | 2 |
| G6 dot voting | SRS §4.6 | 每用户 N 票 + 实时显示 | 2 |
| G7 reaction | SRS §4.7 | emoji 表情回应 (8-12 种) | 1 |
| G8 confetti | SRS §4.8 | 庆祝特效 (CSS / Lottie) | 1 |
| G9 leaderboard | SRS §4.9 | per workspace / per tenant | 2 |
| G10 daily challenge / streak | SRS §4.10 | 每日 3 任务 + 连续天数 | 2 |
| G11 power-up / inventory | SRS §4.11 | 道具 + 物品栏 (**W/T/M 三類横展**) | 2 |
| G12 V0.1 game 集成 | SRS §4.12 | Roguelike + Manga 集成 (V0.1 已实装) | 2 |
| **合计** | | | **32** |

### 1.2 引用 baseline (必读, 不能编造)

| 文档 | 用途 | 路径 |
|---|---|---|
| **专题 SRS (本 BD 派生源)** | G1-G12 32 项 + 15 张表 W/T/M 需求 | `D:\Star\docs\requirements\SRS-CANVAS-GAMIFY-001.md` v1.0 (92KB) |
| **BD 模板 (10 段)** | 严格按 10 段 IPA SEC 模板 | `D:\Star\docs\design\BD-AGENT-RELATIONSHIP-001.md` v0.1 (55KB) |
| 平行 BD 参照 (agent view 画布) | 5 view + 跨块接口 + 数据模型 | `D:\Star\docs\design\BD-AGENT-VIEW-001.md` v0.1 (44KB) |
| 总册 SRS + 总册 BD (root 写) | 跨域接口 + 共享约束 | `D:\Star\docs/requirements/SRS-CANVAS-001.md` v1.1 + `D:\Star\docs/design/BD-CANVAS-001.md` (本批 root 写) |
| V0.1 canvas design | 14 element + 4 frame + 8 connector + 9 e2e 守门 | `D:\Star\docs/frontend-canvas-design.md` v0.1 |
| V0.1 实装代码 | CanvasView.tsx 11 处 element + tool + minimap | `D:\Star\frontend/src/components/CanvasView.tsx` |
| **V0.1 agent-game 4 份 PHASE** (G12 集成) | Game + Roguelike + Manga + Theme + Settings 5 份实装报告 | `D:\Star\docs/reports/PHASE-AGENT-{GAME,ROGUELIKE,MANGA,THEME,SETTINGS}-IMPL-REPORT.md` |
| V0.1 game 组件 | RoguelikeCanvas + AgentSettingsTab + theme-tokens | `D:\Star\frontend/src/components/agent-game/` |
| 25 module 联动 | work-item / worktree / agent / relation / comment / search / notification | per 总册 §6.3 |
| 守门交叉引用 | 16 守门 + 26 派生规 | `D:\Star\AGENTS.md` §4 |
| 守门 #23 v2 AI mock | 跨域硬约束, GAMIFY G5 走 mock, 真实 LLM 留 P2 | per `D:\Star\scripts/automation/ai_edit_mock.py` v0.1 |
| 守门 #13 W/T/M | GAMIFY G11.2 15 张表 100% 覆盖 (Work 6 + Transaction 4 + Master 5) | per 总册 §7 + GAMIFY §7.2 |

### 1.3 文档结构 (per BD-AGENT-RELATIONSHIP-001 v0.1 模板, 10 段)

```
§0 目的 (Purpose)
§1 适用范围 (Scope) — 1.1 In-Scope (G1-G12 12 子能力) + 1.2 Out-of-Scope (Miro 通用 12 类砍掉 + 详细设计 DD 后续 + AI 真实接入 P2 砍掉)
§2 系统架构 (System Architecture) — 5 view (機能/データ/動作/モジュール/ネットワーク) 跨 G1-G12
§3 组件一览 (Components) — G1-G12 子能力组件 + 新增 8 module / 复用 V0.1 game 组件
§4 数据模型 (Data Model) — 15 张表 (Work 6 + Transaction 4 + Master 5) + 5 domain-* Rust 数据结构 + zustand store 扩展 + 14 张表跨域汇总 (G11 power-up/inventory W/T/M 完整)
§5 接口设计 (Interface Design) — BFF REST API (双核心 13 + GAMIFY 9 = 22 端点) + WebSocket 2 端点 (G4 level up + G8 confetti) + 内部 4 协议 (G5 mock 协议)
§6 5 view 詳細 (5 views) — 機能 view (32 项) + データ view (15 表) + 動作 view (3 维: score/聚类/投票) + モジュール view (3 文件 + 2 复用 V0.1) + ネットワーク view (e2e 9)
§7 NFR (Non-Functional Requirements) — 6 类 (性能/可靠性/安全/易用/可观测/AI mock)
§8 守门 (Guards) + 子代理失败接手 + 已知缺口 (12 个 per SRS GAMIFY + G11 retention 默认值 DDD Review)
§9 签字栏 (5 角色 per AGENTS.md §3)
§10 修订履历 (v0.1 + 修订人 + 触发)
```

### 1.4 输出文件

| 文件 | 内容 | 预估大小 |
|---|---|---|
| `D:\Star\docs\design\BD-CANVAS-GAMIFY-001.md` | 专题 BD 完整 10 段, G1-G12 32 项 | ~40-60K 字 |

**仅输出 1 份文件**, 不拆 commit, 不写 report, 不动 implementation.

## 2. 范围外 (out-of-scope, 由其他子代理 / 总册 BD / root 处理)

| 类别 | 处理方 |
|---|---|
| 双核心之 1: agent 管理 46 项 详细 BD | 专题 BD 子代理 1 (bd-canvas-agent-001) |
| 总册 BD (跨域共享部分) | root 写 (bd-canvas-total-001) |
| 详细设计 (DD) 文档 | 后续 P3-D 阶段, 待 SRS + BD 落档后启动 |
| 实现 (PHASE-* 报告) | 后续 P3-D.6 阶段, 待 DD 落档后启动 |
| **AI 真实 LLM 接入 (G5 sticky note 聚类)** | ❌ 砍掉, 走 mock 接口 (per 守门 #23 v2), 真实 LLM 留 P2 |
| Miro 通用 12 类 | ❌ 砍掉, 留 P3+ 评估 |
| 25 module 实体实现 | 25 module 各自 docs, 画布只联动不实装 |
| 5 域 Lead 真人到位 | per 守门 #14 v2, Mavis 临时代签, 真人到位后追溯签字 |

## 3. 已知缺口 (per 守门 #11 缺标比错标)

本 BD 须在 §8 显式列已知缺口 (≥ 12 个, per SRS GAMIFY v1.0 附录 A 12 缺口), 不得隐藏:
- **#1 真实 user 上传头像 (G1.1)** — V2 候选
- **#2 G3.2 score 累计 TZ 用户配置** — P2
- **#3 G6.1 dot voting 跨 session 持久化** — P2
- **#4 G4.1 公式用户可配** — P2
- **#5 G4.2 升级跳过 level 动画合并** — P2
- **#6 G4.3 skill 复合解锁** — P2
- **#7 G7 reaction 撤回** — P2
- **#8 G5 真实 LLM 接入** — P2 (需先破守门 #23 v2, 走 mock)
- **#9 G8 confetti 音效** — P2
- **#10 G2.3 confetti 跟 G7.1 reaction 互斥** — 已显式
- **#11 G11.2 W/T/M retention 默认值** — DDD Review 必查
- **#12 G9.2 admin 13 租户权限** — DDD Review 必查

**DDD Review 必查**: #11 + #12 + G12 V0.1 Roguelike 共享 state + G5 真实 LLM + 守门 #23 v2 跨域硬约束

## 4. 守门硬约束 (per 守门 #1 + 守门 #9 #3 + 守门 #13 + 守门 #14 + 守门 #15 + 守门 #23 v2)

- 文档结构严格 10 段 (per BD-AGENT-RELATIONSHIP-001 模板), 不增不减
- 32 项每项 5 view 跨域覆盖 (不重写总册 BD 跨域部分)
- 已知缺口 ≥ 12 个 (含 DDD Review 必查 #11 + #12)
- NFR 6 类 (性能/可靠性/安全/易用/可观测/AI mock)
- 守门 19 项 + 26 派生规 跨域 (含 #23 v2 AI 第三方 API 禁止)
- 5 角色签字栏 per AGENTS.md §3
- **5 域 Lead ≠ Star 22 DDD bounded context** disclaimer 显式 (per 2026-08-31 22:45 JST Q1-D 拍板)
- **5 域 Lead 真人未到位前 Mavis 临时代签, 真人到位后追溯签字** (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
- **AI mock 接口** (per 守门 #23 v2, GAMIFY G5 走 mock, 真实 LLM 留 P2, 跨域硬约束)
- **DB W/T/M 三類横展** (per 守门 #13, GAMIFY G11.2 15 张表 100% 覆盖, 禁止混在)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 v0.62 反转 Mavis 审核 author=Ulysses)
- 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
- 0 文件改动除输出 BD
- 0 commit, 仅产出 markdown (root 统一 commit per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
- **不重写 4 SRS commit** (per 守门 #1 禁回溯叙事, BD 是新方向, 不回写 SRS)
- **G12 必须派生自 V0.1 game 4 份 PHASE 报告** (RoguelikeCanvas + AgentSettingsTab + theme-tokens 实装已落档, 仅画布集成引用, 不重写功能)
- **G5 必须派生自 mock 接口** (per 守门 #23 v2, 真实 LLM 不引入第三方 API 凭据, 走 ai_edit_mock.py 模式)

## 5. 落地清单

| # | 文件 | 内容 | 行数预估 |
|---|---|---|---|
| 1 | `D:\Star\docs\design\BD-CANVAS-GAMIFY-001.md` | 10 段 BD, G1-G12 32 项 | ~1000-1500 行 |

预估 0 commit (root 统一 commit), 1 文件, ~40-60K 字。

## 6. 返报告知 (per 守门 #9 v27 collect_output)

子代理返回时, 报告必须含:
1. 实际写入文件路径 + 字节数
2. 10 段是否齐全 (§0~§10)
3. 5 view 覆盖检查 (機能/データ/動作/モジュール/ネットワーク) 跨 G1-G12
4. **15 张表 W/T/M 100% 覆盖** (GAMIFY 15 张 = Work 6 + Transaction 4 + Master 5, 列出每张表归到 W/T/M 哪類)
5. 22 API 端点 + 2 WebSocket 跨域汇总 (BFF REST + WebSocket, 列表)
6. 已知缺口清单 (≥ 12 个, 含 DDD Review 必查 #11 + #12)
7. 跨专题引用 (引用 SRS-CANVAS-{GAMIFY v1.0, 001 v1.1} + 现有 canvas design + V0.1 game 4 份 PHASE + 25 module + BD-AGENT-VIEW-001 v0.1 + 守门 19 项 + 拍板 4 阶段)
8. **G12 必须派生自 V0.1 game 4 份 PHASE 报告** (Roguelike + Manga + Theme + Settings 5 份实装, 仅画布集成引用)
9. **G5 sticky note 聚类 AI 必走 mock 接口** (per 守门 #23 v2, 真实 LLM 留 P2, 跟 ai_edit_mock.py 一致)
10. 守门 19/19 跨域覆盖 (含 #1 v15 / #9 #3 / #9 v19 / #10 / #11 / #13 / #14 v2/v3/v4 / #15 / #23 v2)
11. 任何意外 / 偏离 / 简化 / 跳过 项, 显式标注

不要只回 "done" — 必须给可验证证据.

## 7. 元数据 (per AGENTS.md §3 守门 #10 + 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 + 9/10 12:45 JST v0.62 反转 + 17:08 JST 双核心 + 守门 #23 v2)

- 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手`
- 审批: `架构师 (Mavis 接手 agent per DEC-008)` (5 角色签字栏 per AGENTS.md §3)
- 日期: 2026-09-10 JST
- 关联 commit: 留空 (root 统一 commit 时填)
- 关联文档: SRS-CANVAS-GAMIFY-001.md v1.0 (本 BD 派生源) + 2 份平行 SRS + V0.1 game 4 份 PHASE + 现有 canvas design + BD-AGENT-VIEW-001 v0.1 (平行 BD)
- 拍板来源: 2026-09-10 18:00 JST Ulysses 拍板"基于需求文档制作基本设计文档" + 17:08 JST 双核心 + 守门 #23 v2 AI 第三方 API 禁止 (G5 走 mock)

## 8. 起点

读完 13 份必读后, 用 Write 工具写 `D:\Star\docs\design\BD-CANVAS-GAMIFY-001.md` (10 段, 全量覆盖).
