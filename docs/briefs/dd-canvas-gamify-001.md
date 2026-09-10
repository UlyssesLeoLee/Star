# Brief: dd-canvas-gamify-001

**Agent**: worker (root 派发, 2 子代理并行之 2/2 per 守门 #9 v20 + v27 3 段 fallback)
**Phase**: P3-D.5 DD 详细设计 (18:25 JST Ulysses 拍板"完善详细设计文档")
**Created**: 2026-09-10 18:27 JST
**Token 预算**: ~0.4M (守门 #4 / #19 估算, 1 SRE·周 = 1.2M 留 3x 缓冲)
**Worktree**: 在 root 当前 main worktree 直实装 (per 守门 #9 #3 实证 5/5 RPC 不可靠, 不派二级子代理)

---

## 0. 触发

2026-09-10 18:25 JST Ulysses 拍板"完善详细设计文档" — 基于 BD-CANVAS-GAMIFY-001.md v0.1 (111KB, 32 项 G1-G12 详细设计) + SRS-CANVAS-GAMIFY-001.md v1.0 (92KB) 制作游戏化域详细设计 DD.

17:08 JST 拍板"避免过度冗余" + 守门 #23 v2 AI 第三方 API 禁止 (G5 sticky note 聚类走 mock 接口, 真实 LLM 留 P2) + 守门 #13 DB W/T/M 三類横展.

## 1. 范围 (in-scope)

### 1.1 本专题 DD 覆盖 — 游戏化域 (双核心之 2)

**核心**: G1-G12 (12 子能力, 32 项) 的**详细实现** (Rust struct + TS interface + SQL DDL + 状态机 + 时序图 + 测试用例).

| 子能力 | 来源 BD | 详细实现 |
|---|---|---|
| G1 gamification 节点 | BD §4.1 | 8 节点 struct (avatar/level/xp/skill_tree/class/badge/quest/inventory) + 视觉 (复用 V0.1 `AGENT_VISUAL_TIERS` 10 段) |
| G2 reward/achievement | BD §4.2 | reward rule engine (复用 V0.1 automation) + 通知 + confetti trigger |
| G3 score/points | BD §4.3 | 操作积分规则 (per user/task/day/all time 4 dim) + 累计 + 排行榜入口 |
| G4 leveling | BD §4.4 | `xp_to_level` 公式 (默认 `level = sqrt(xp/100)`) + 升级动画 (复用 V0.1 §1 #11) + skill tree unlock |
| G5 sticky note 聚类 (AI, mock) | BD §4.5 | **走 mock 接口** (per 守门 #23 v2) + `MockClusterRequest`/`MockClusterResponse` TS interface + confidence < 0.5 |
| G6 dot voting | BD §4.6 | 每用户 N 票 (默认 5) + 实时显示 + `dot_vote_ledger` Transaction |
| G7 reaction | BD §4.7 | 8-12 emoji (👍 ❤️ 🎉 😄 🤔 👀 🔥 ⭐) + 短时显示 (3s 淡出) |
| G8 confetti | BD §4.8 | CSS / Lottie 动画 (per `conftti_events` Work) + 触发: 完成 / 解锁 / 升级 |
| G9 leaderboard | BD §4.9 | per workspace / per tenant 排行榜 + admin 13 租户权限 (per 守门 #13 RLS) |
| G10 daily challenge | BD §4.10 | 每日 3 任务 + streak 连续天数 (中断清零, 7 天奖励) |
| G11 power-up/inventory | BD §4.11 | 道具 + 物品栏 + **W/T/M 三類横展 15 张表 100% 覆盖** |
| G12 V0.1 game 集成 | BD §4.12 | Roguelike + Manga 集成 (复用 V0.1 4 份 PHASE 9/5 落地 125 tests pass) |
| **合计** | | | **32 项详细实现** |

### 1.2 引用 baseline (必读, 不能编造)

| 文档 | 用途 | 路径 |
|---|---|---|
| **派生源 BD (核心)** | G1-G12 32 项基本设计 | `D:\Star\docs\design\BD-CANVAS-GAMIFY-001.md` v0.1 (111KB) |
| **派生源 SRS** | G1-G12 32 项需求 + 15 张表 W/T/M | `D:\Star\docs\requirements\SRS-CANVAS-GAMIFY-001.md` v1.0 (92KB) |
| **DD 模板 (15 章节)** | 严格按 15 章节 + 10 段 | `D:\Star\docs\design\DD-AGENT-RELATIONSHIP-001.md` v0.1 (94KB) |
| 总册 SRS + 总册 BD (root 写) | 跨域接口 + 共享约束 | `D:\Star\docs/requirements/SRS-CANVAS-001.md` v1.1 + `D:\Star\docs/design/BD-CANVAS-001.md` v0.1 |
| V0.1 canvas design | 14 element + 4 frame + 8 connector | `D:\Star\docs/frontend-canvas-design.md` v0.1 |
| V0.1 实装代码 | CanvasView.tsx 11 处 element | `D:\Star\frontend/src/components/CanvasView.tsx` |
| **V0.1 agent-game 4 份 PHASE** (G12 集成) | 5 份实装报告 | `D:\Star\docs/reports/PHASE-AGENT-{GAME,ROGUELIKE,MANGA,THEME,SETTINGS}-IMPL-REPORT.md` |
| V0.1 game 组件 | RoguelikeCanvas + AgentSettingsTab + theme-tokens | `D:\Star\frontend/src/components/agent-game/` |
| 25 module 联动 | per 总册 §6.3 25 module | per `frontend-canvas-design.md` §1.3 |
| 守门 #23 v2 AI mock 协议 | `D:\Star\scripts/automation/ai_edit_mock.py` v0.1 (G5 走 mock) | per 守门 #23 v2 |
| 守门 #13 W/T/M | GAMIFY G11.2 15 张表 100% 覆盖 (Work 6 + Transaction 4 + Master 5) | per 总册 §7 + GAMIFY §7.2 |
| **Ulysses 18:25 JST 拍板原文** | "完善详细设计文档" — 不可偏离 |

### 1.3 文档结构 (per DD-AGENT-RELATIONSHIP-001 v0.1 模板, 10 段 + 15 章节)

```
§0 文档信息 / 修订履历
§1 文档目的 / 适用范围
§2 系统架构 (System Architecture) — 5 view 跨域 + 5-tier + 4 sequence diagram
§3 概念 module 布局 (Conceptual Module Layout) — 24 组件 → 18 Rust module + 1 Python LangGraph module + 跨域组件映射
§4 关键 class (Key Classes) — 13 关键 class 完整字段 + 方法签名 + 错误处理
§5 状态机 (State Machines) — 5 状态机 Rust enum + 状态转移函数
§6 共享类型 (Shared Types) — 11 共享类型完整定义
§7 接口协议 (Interface Protocols) — 5 WebSocket 协议 + 内部 5 协议 + 32 API 端点 OpenAPI spec
§8 时序图 (Sequence Diagrams) — 4 关键时序图 (写关系 / 协作影响 / 成就评估 / 离线降级)
§9 数据持久化 (Data Persistence) — 15 张表 SQL DDL + Memgraph Cypher schema + zustand store 扩展
§10 测试用例 (Test Cases) — UT + IT + E2E + PT 4 类测试, 跨域测试 ≥ 30 个
附录 A: 跨专题引用清单
附录 B: 跨域组件映射 (4 文件 DD + 6 crate + 5 V0.1 复用 + 25 module 联动)
附录 C: 已知缺口 (12 个 per BD §8.3, 含 DDD Review 必查 #11 + #12)
附录 D: 5 角色签字栏 (per AGENTS.md §3)
附录 E: 修订履历 (v0.1 + 修订人 + 触发)
```

### 1.4 输出文件

| 文件 | 内容 | 预估大小 |
|---|---|---|
| `D:\Star\docs\design\DD-CANVAS-GAMIFY-001.md` | 专题 DD 完整 10 段 + 5 附录, G1-G12 32 项详细 | ~70-100K 字 |

**仅输出 1 份文件**, 不拆 commit, 不写 report, 不动 implementation.

## 2. 范围外 (out-of-scope, 由其他子代理 / 总册 DD / root 处理)

| 类别 | 处理方 |
|---|---|
| 双核心之 1: agent 管理 46 项 详细 DD | 专题 DD 子代理 1 (dd-canvas-agent-001) |
| 总册 DD (跨域共享部分) | root 写 (dd-canvas-total-001) |
| 实现 (PHASE-* 报告) | 后续 P3-D.6 阶段 |
| **AI 真实 LLM 接入 (G5 sticky note 聚类)** | ❌ 砍掉, 走 mock 接口 (per 守门 #23 v2), 真实 LLM 留 P2 |
| 单元测试 (UT) 代码实装 | P3-D.6 阶段 |
| Miro 通用 12 类 | ❌ 砍掉, 留 P3+ 评估 |
| 25 module 实体实现 | 25 module 各自 docs, 画布只联动不实装 |

## 3. 已知缺口 (per 守门 #11 缺标比错标)

本 DD 须在附录 C 显式列已知缺口 (≥ 12 个, per BD GAMIFY §8.3 12 缺口), 不得隐藏:
- **#1 真实 user 上传头像 (G1.1)** — V2 候选
- **#2 G3.2 score 累计 TZ 用户配置** — P2
- **#3 G6.1 dot voting 跨 session 持久化** — P2
- **#4 G4.1 公式用户可配** — P2
- **#5 G4.2 升级跳过 level 动画合并** — P2
- **#6 G4.3 skill 复合解锁** — P2
- **#7 G7 reaction 撤回** — P2
- **#8 G5 真实 LLM 接入** — P2 (需先破守门 #23 v2)
- **#9 G8 confetti 音效** — P2
- **#10 G2.3 confetti 跟 G7.1 reaction 互斥** — 已显式
- **#11 G11.2 W/T/M retention 默认值** — **DDD Review 必查**
- **#12 G9.2 admin 13 租户权限** — **DDD Review 必查**

## 4. 守门硬约束 (per 守门 #1 + 守门 #9 #3 + 守门 #13 + 守门 #14 + 守门 #15 + 守门 #23 v2)

- 文档结构严格 10 段 + 5 附录 (per DD-AGENT-RELATIONSHIP-001 模板), 不增不减
- 32 项每项 5 view + 详细实现 (Rust struct + TS interface + SQL DDL)
- 已知缺口 ≥ 12 个 (含 DDD Review 必查 #11 + #12)
- **15 张表 W/T/M 100% 覆盖 0 混在** (per 守门 #13, SQL DDL 完整)
- 22 API 端点 + 2 WebSocket 完整 spec
- 13 关键 class 跨域 (G1-G12 派生)
- 5 状态机 (sticky note 聚类 / dot voting / reaction / daily challenge / streak)
- 11 共享类型 (StickyNoteCluster / Vote / Reaction / Confetti / Level / Score / Streak / Powerup / Inventory / Badge / Quest)
- 4 关键时序图 (sticky note 聚类 / dot voting / confetti 触发 / daily challenge 完成)
- 74+ 测试用例 (per DD-AGENT-RELATIONSHIP-001 v0.1 §10 模板, GAMIFY ≥ 30 跨域)
- 守门 19 项 + 26 派生规 跨域 (含 #23 v2)
- 5 角色签字栏 per AGENTS.md §3
- **5 域 Lead ≠ Star 22 DDD bounded context** disclaimer 显式
- **5 域 Lead 真人未到位前 Mavis 临时代签, 真人到位后追溯签字**
- **AI mock 接口** (per 守门 #23 v2, G5 走 mock, 真实 LLM 留 P2, 跨域硬约束)
- **DB W/T/M 三類横展** (per 守门 #13, 15 张表 100% 覆盖)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)
- 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
- 0 文件改动除输出 DD
- 0 commit, 仅产出 markdown
- **不重写 4 SRS + 2 BD 6 commit** (per 守门 #1 禁回溯叙事, DD 是新方向)
- **G12 必须派生自 V0.1 game 4 份 PHASE 报告** (Roguelike + Manga + Theme + Settings + Game, 9/5 落地 125 tests pass, 仅画布集成引用, 不重写功能)
- **G5 sticky note 聚类 AI 必走 mock 接口** (per 守门 #23 v2, 跟 `ai_edit_mock.py` 一致)

## 5. 落地清单

| # | 文件 | 内容 | 行数预估 |
|---|---|---|---|
| 1 | `D:\Star\docs\design\DD-CANVAS-GAMIFY-001.md` | 10 段 + 5 附录, G1-G12 32 项详细 | ~1300-1900 行 |

预估 0 commit (root 统一 commit), 1 文件, ~70-100K 字。

## 6. 返报告知 (per 守门 #9 v27 collect_output)

子代理返回时, 报告必须含:
1. 实际写入文件路径 + 字节数
2. 10 段 + 5 附录是否齐全
3. 5 view 详细实现覆盖检查 (機能/データ/動作/モジュール/ネットワーク)
4. **15 张表 SQL DDL 完整 0 混在** (列出每张表的 W/T/M 归类)
5. 22 API 端点 + 2 WebSocket OpenAPI spec 完整 (列表)
6. **13 关键 class 跨域** (列出 class 名 + 字段数 + 方法数)
7. **5 状态机** (列出每个状态机的状态数 + 转移函数)
8. **11 共享类型** (列出每个类型定义)
9. **4 关键时序图** (sticky note 聚类 / dot voting / confetti 触发 / daily challenge 完成)
10. 已知缺口清单 (≥ 12 个, 含 DDD Review 必查 #11 + #12)
11. 跨专题引用 (引用了哪些 SRS / BD / DD / 设计 / 25 module / 守门 / 拍板, 具体 §)
12. **G12 派生自 V0.1 game 4 份 PHASE 报告** (Roguelike + Manga + Theme + Settings + Game, 9/5 落地 125 tests pass, 仅画布集成引用)
13. **G5 必走 mock 接口** (per 守门 #23 v2, 真实 LLM 留 P2, 跟 `ai_edit_mock.py` 一致)
14. 守门 19/19 跨域覆盖 (含 #1 v15 / #9 #3 / #9 v19 / #10 / #11 / #13 / #14 v2/v3/v4 / #15 / #23 v2)
15. 任何意外 / 偏离 / 简化 / 跳过 项, 显式标注

不要只回 "done" — 必须给可验证证据.

## 7. 元数据 (per AGENTS.md §3 守门 #10 + 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 + 9/10 12:45 JST v0.62 反转 + 17:08 JST 双核心 + 守门 #23 v2)

- 修订人: `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手`
- 审批: `架构师 (Mavis 接手 agent per DEC-008)` (5 角色签字栏 per AGENTS.md §3)
- 日期: 2026-09-10 JST
- 关联 commit: 留空 (root 统一 commit 时填)
- 关联文档: BD-CANVAS-GAMIFY-001.md v0.1 (本 DD 派生源) + 2 份平行 SRS + 2 份平行 BD + V0.1 game 4 份 PHASE + 现有 canvas design
- 修订履历必须含 v0.1 (2026-09-10 18:25 JST 拍板"完善详细设计文档" + 17:08 JST 双核心 + 守门 #23 v2), 显式标 3 阶段拍板

## 8. 起点

读完 13 份必读后, 用 Write 工具写 `D:\Star\docs\design\DD-CANVAS-GAMIFY-001.md` (10 段 + 5 附录, 全量覆盖).
