# Brief: srs-canvas-agent-001

**Agent**: worker (root 派发, 2 子代理并行之 1/2 per 守门 #9 v20 + v27 3 段 fallback)
**Phase**: P3-D.5 无限画布需求文档化 (Ulysses 17:08 JST 拍板: agent 管理 + 游戏化, 避免过度冗余; **17:21 JST 补充: 画布内体现 agent 之间关系的图论构造 (ARG)**)
**Created**: 2026-09-10 17:17 JST (重写自原 srs-canvas-collab-001, 方向重置)
**Updated**: 2026-09-10 17:22 JST (新增 A11 ARG 子能力 10 项, 28 项 → 38 项, 引用 `SRS-AGENT-RELATIONSHIP-001.md` v0.1)
**Token 预算**: ~0.4M (守门 #4 / #19 估算, 1 SRE·周 = 1.2M 留 3x 缓冲)
**Worktree**: 在 root 当前 main worktree 直实装 (per 守门 #9 #3 实证 5/5 RPC 不可靠, 不派二级子代理)

---

## 0. 触发

2026-09-10 17:08 JST Ulysses 拍板"审核时的观点是要明确, 我这个无限画布主要功能是管理 agent 和游戏化, 避免过度冗余" — 推翻前一轮 (17:00 JST) "Miro 差距 50 项全面对标" 方向, 聚焦双核心: (1) agent 管理 (节点 / 拓扑 / handoff / 状态 / worktree 关联), (2) 游戏化 (gamification 节点 / 奖励成就 / 积分 / 等级 / 排行榜 / 投票 / confetti)。

撤回范围: 12 大类 #1 协作 / #2 演示 / #6 导出分享 / #7 互动 / #8 版本 / #9 集成 / #11 移动 (Miro 通用功能) 全部砍掉, 留 P3+ 评估。

## 1. 范围 (in-scope)

### 1.1 本专题覆盖 — Agent 管理域 (双核心之 1)

**核心**: 画布上**管理 agent 节点** + 拓扑图 + 状态实时同步 + handoff + worktree 关联 + 监控操作。

| 子能力 | 范围 | 项数 |
|---|---|---|
| A1 agent 节点渲染 | element kind `agent_cursor` 扩展为 `agent_node` (完整卡) | 3 项 |
| A2 agent 拓扑图 | 多 agent 关系图 (1:N handoff, 跨 5 域) | 4 项 |
| A3 agent 状态实时同步 | 14 状态机 (queued/spawning/.../completed) 实时色码 | 3 项 |
| A4 agent 关联 worktree | 1 agent → N worktree, 跟 StatusPill 60+ 同步 | 2 项 |
| A5 agent 关联 work-item | 1 agent → N work-item, drag in/out | 2 项 |
| A6 agent 操作菜单 | 右键菜单 (启停/重启/查看 logs/查看 token 用量/查看 settings) | 4 项 |
| A7 agent 监控面板 | 实时 status / token / cost / runtime 仪表 | 3 项 |
| A8 agent 聚类 / 排序 / 过滤 | 按 role / kind / status 聚类, 按 token / 启动时间 排序 | 3 项 |
| A9 agent session 跨域引用 | 跟 SRS-AGENT-VIEW-001 协同, 双向跳 | 2 项 |
| A10 agent settings (V0.1 已实装) | agent-settings tab 集成, 引用 `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` | 2 项 |
| **A11 ARG 图论构造** (per 2026-09-10 17:21 JST Ulysses 补充) | **10 类关系边 + 4 维度协作影响 + 5 团队模板 + 同步桥 + 成就系统**, 引用 `SRS-AGENT-RELATIONSHIP-001.md` v0.1 (9/8 落档) | **10 项** |
| **合计** | | **38 项** |

子代理必须**逐项展开**为 SRS 需求条目, 不得合并 / 跳过 / 简写。每项含:
- ID (e.g. `F-AGENT-A1.1` agent_node 完整卡)
- 标题 + 1 句描述
- 优先级 (P0/P1/P2/P3, 默认 P0=核心管理, P1=监控, P2=辅助)
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
| Ulysses 17:08 JST 拍板 | **本 SRS 的核心方向锚点** (agent + 游戏化) | 本 brief §0 |
| 现有 agent cursor (V0.1) | element kind `agent_cursor` 基础 | `frontend/src/components/CanvasView.tsx` line 218-235 |
| 现有 agent 联动 (V0.1) | §4.6 PresenceCursor 升级 | `docs/frontend-canvas-design.md` §4.6 |
| 现有 agent 模块 (25 module) | agent-session / agent-context / agent-runtime | per `docs/requirements/SRS-STAR-AGENT-RUNTIME-001.md` |
| 现有 agent view (SRS 已落档) | 协同引用 | `docs/requirements/SRS-AGENT-VIEW-001.md` v1.0 |
| 现有 agent settings (V0.1) | 已实装, 不重写 | `frontend/src/components/agent-game/AgentSettingsTab.tsx` + `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1 |
| 现有 StatusPill 60+ 色码 | 状态色码同步基线 | per `frontend-canvas-design.md` §3.4 + ADR-FE-013 |
| 现有 StateMachineDiagram 5×4 grid | layout 算法复用 | per `frontend/src/lib/agent-view/layout.ts` |
| SRS 模板 (9 段 IPA SEC 结构) | 严格按 §0~§9 | `docs/requirements/SRS-AGENT-VIEW-001.md` (31KB) |
| **ARG (Agent Relationship Graph) SRS 主源** | **A11 子能力的源文档**, 必读 §1-§8 (10 类关系 + 4 维度 + 5 模板 + 4 表 + 8 UC) | `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1 (37KB, 9/8 落档) |
| **ARG 基本设计** | A11 实现参考, Memgraph schema + 4 表 + 同步桥 | `docs/design/BD-AGENT-RELATIONSHIP-001.md` (55KB) |
| **ARG 详细设计** | A11 详细实现, 8 UC 流程 | `docs/design/DD-AGENT-RELATIONSHIP-001.md` (94KB) |
| **ARG DDD Review** | A11 设计评审, 跨 DDD 边界确认 | `docs/design/DDD-REVIEW-AGENT-RELATIONSHIP-001.md` (30KB) |

### 1.3 文档结构 (per SRS-AGENT-VIEW-001 9 段模板, 不得改结构)

```
§0 文档信息 / 修订履历
§1 文档目的 / 适用范围 (含 1.3 包含范围 + 1.4 不包含范围 + 1.5 用户故事)
§2 用語定義 (用語集 / Ubiquitous Language)
§3 業務背景 / 前提条件
§4 功能需求 (F-x.y 列表, 含数据 schema 增项)
§5 非功能需求 (NFR-x.y, 性能/可访问性/安全/扩展性)
§6 接口需求 (BFF API / WebSocket / 第三方集成)
§7 约束 / 依赖 / 风险
§8 验收标准 (AC-x.y + 验收场景)
§9 修订履历 (v0.1 + 修订人 + 修订内容 + 触发)
```

### 1.4 输出文件

| 文件 | 内容 | 预估大小 |
|---|---|---|
| `docs/requirements/SRS-CANVAS-AGENT-001.md` | 本专题 SRS 完整 9 段, 28 项展开 | ~25-35K 字 |

**仅输出 1 份文件**, 不拆 commit, 不写 report, 不动 implementation。

## 2. 范围外 (out-of-scope, 由其他子代理 / root 处理)

| 类别 | 处理方 |
|---|---|
| 双核心之 2: 游戏化 (28 项) | 子代理 2 (SRS-CANVAS-GAMIFY-001) |
| 总册 SRS-CANVAS-001 (双核心索引 + 跨块接口 + 共享约束) | root (重写, 聚焦双核心) |
| Miro 通用协作 (多人编辑 / 实时 cursor / 评论) | ❌ 砍掉, 留 P3+ |
| Miro 通用演示 (Frame as slide / Guided Tour) | ❌ 砍掉, 留 P3+ |
| Miro 通用导出 (PDF / Word / Excel / CSV) | ❌ 砍掉, 留 P3+ |
| Miro 通用互动 (Timer / Reaction / Confetti workshop 用) | 部分砍掉, **Confetti 留下** (游戏化用) |
| Miro 通用集成 (Slack / Jira / Asana / Figma / GitHub) | ❌ 砍掉, 留 P3+ |
| Miro 通用版本 (Version history / Branching) | ❌ 砍掉, 留 P3+ |
| Miro 通用移动 (iOS / Android / Touch) | ❌ 砍掉, 留 P3+ |
| Miro 通用 12 种 diagram | ❌ 砍掉, 留 P3+ |
| Miro 模板库 (2500+) | ❌ 砍掉, 留 P3+ |
| Miro AI 能力 (聚类 / 文本生成 diagram) | ❌ 部分砍掉, **sticky note 聚类 留下** (游戏化用) |
| Miro Tables / Chart widget / Form | ❌ 砍掉, 留 P3+ |
| 完整 a11y (WCAG 2.1 AA) | ❌ 砍掉, 留 P3+ |
| 基础画布能力 (pan/zoom/select/element/connector/frame) | ✅ V0.1 已有, 不重写 |
| 已有 agent 联动 (V0.1 §4.6 PresenceCursor) | ✅ V0.1 已有, 不重写, 仅扩展 |

## 3. 已知缺口 (per 守门 #11 缺标比错标)

子代理须在 SRS §3 / §7 / §8 显式列已知缺口, 不得隐藏:
- 5 域 Lead 真人未到位, 跨域编排决策延后 → §7 约束
- A1 agent_node 完整卡 跟 V0.1 agent_cursor 关系 (是扩展还是替代) → §3 业务背景明确
- A2 跨 5 域 handoff 拓扑 涉及 5 域 Lead 责任边界, 真人到位后追溯签字 (per 守门 #14 v2 + 9/5 10:43 JST 拍板 D)
- A3 实时状态同步 WebSocket 选型未拍板 (跟原 COLLAB 专题同源) → §7 风险
- A4 agent ↔ worktree 1:N 关系当前 schema 可能缺 (per `SRS-AGENT-VIEW-001.md` §3 缺口 #4 schema 缺 `WorkItem.agent_session_id` 字段) → §7 风险
- A6 启停 / 重启操作权限边界 (谁能操作) → §7 约束 (per 守门 #3 5 域独立 Lead 不接受兼任 + 9/3 11:35 JST 拍板 B)
- A7 token / cost 监控涉及 SRE Lead 责任 (per 守门 #4 OLU 预算) → §7 风险
- A8 聚类 / 排序 / 过滤 跟 SRS-AGENT-VIEW-001 协同, 边界划清 → §6 接口必含引用
- A9 跨域引用 (跟 SRS-AGENT-VIEW-001 / SRS-AGENT-RELATIONSHIP-001) → §6 接口必含引用
- A10 agent settings V0.1 已有, 仅做画布集成引用, 不重写功能 → §1.4 不包含范围
- **A11.1 10 类关系边 颜色区分** (4 核心 + 6 扩展, 各边色码映射, 跟 StatusPill 60+ 协调) → §7 风险
- **A11.2 关系编辑 UI 拖拽 + type 选择 + weight 滑块** (per SRS-AGENT-RELATIONSHIP-001 §4.1.1+4.1.2) → §4 FR 必含
- **A11.3 4 维度协作影响 UI 指示器** (dispatch / context / trust / review, 4 类 effect indicator) → §4 FR 必含
- **A11.4 5 团队模板 1-click 部署** (Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council, per SRS-AGENT-RELATIONSHIP-001 §4.5) → §4 FR 必含
- **A11.5 关系 audit log** (per V0.1 audit + 守门 #13 Transaction append-only + SCD Type 2) → §6 接口必含
- **A11.6 关系权重可视化** (边粗细 0.0-1.0 + 颜色渐变灰→绿) → §4 FR 必含
- **A11.7 关系 archive / restore** (per SRS-AGENT-RELATIONSHIP-001 §4.1.3 `archived` 字段) → §4 FR 必含
- **A11.8 关系版本控制 (SCD Type 2)** (per 守门 #13, 改关系 version +1) → §6 接口必含
- **A11.9 同步桥 UI 状态显示** (Memgraph ↔ LangGraph StateGraph, sync status indicator, per SRS-AGENT-RELATIONSHIP-001 §4.4) → §4 FR 必含
- **A11.10 跟 SRS-AGENT-RELATIONSHIP-001 协同** (Agent View "Relationship" tab 切换, per SRS-AGENT-RELATIONSHIP-001 §1.3) → §6 接口必含引用
- **A11 cross-cutting 5 域 Lead 真人未到位** (关系定义 Mavis 临时代签, 真人到位后追溯签字 per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事) → §7 约束
- **A11 cross-cutting Memgraph 部署** (Docker 启动, port 7687 Bolt + 7444 HTTP, 数据卷持久化, per SRS-AGENT-RELATIONSHIP-001 §3.2 PR-4) → §7 依赖

## 4. 守门硬约束 (per 守门 #1 + 守门 #13 + 守门 #14 v2 + 守门 #15 docs 同步饱和)

- 文档结构严格 9 段, 不增不减
- **38 项每项全部展开** (FR/NFR/AC), 不合并, 不简写
- 用户故事 ≥ 23 个 (38 项 × 60% 覆盖), US-x 格式
- 验收标准 ≥ 38 个 (每项 1-2 个 AC)
- 已知缺口 ≥ 8 个 (含 A11 跨专题 5 缺口)
- **A11.5 关系 audit log 必含 W/T/M 三類横展** (per 守门 #13: agents = Master, audit = Transaction, template instances = Work, 100% 表覆盖)
- **A11 cross-cutting 5 域 Lead 真人未到位前 Mavis 临时代签, 真人到位后追溯签字** (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
- **不写 Miro 通用功能** (本专题仅 agent 管理, 跟游戏化交叉部分在总册 §4.4 跨块接口)
- commit author = `Ulysses <ulysses@mavis.local>` (per 守门 #10 + 8/27 19:39 JST 授权)
- 修订人 / 审批者 = `Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手` (per 守门 #14 v3 Mavis 永久代签 + 9/8 15:19 JST 第 6 次强化)
- 0 子代理调用 (per 守门 #9 #3 实证 5/5 RPC 不可靠)
- 0 文件改动除输出 SRS
- 0 commit, 仅产出 markdown (root 统一 commit per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)

## 5. 子能力 28 项 详细 (root 输入)

### A1 agent 节点渲染 (3 项)
- A1.1 `agent_cursor` (V0.1) 升级为 `agent_node` 完整卡 (含 agent 头像 / 名字 / role / kind / status / token 用量 / 启动时间) — 取代原"圆点 + 名字"
- A1.2 agent_node 状态色码 走 StatusPill 60+ (per V0.1 §3.4) — 14 状态机映射到 60+ 色码
- A1.3 agent_node 双击 → 跳 agent 详情 (V0.1 联动 2 已实装, 扩展)

### A2 agent 拓扑图 (4 项)
- A2.1 1:N handoff connector (从 V0.1 `agent_handoff` kind 扩展, 含 handoff 时间 / 状态 / 备注)
- A2.2 跨 5 域拓扑图 (5 域分组 Frame, 域内 agent 自动聚类)
- A2.3 父子 agent 关系 (1 parent → N child, e.g. supervisor → worker)
- A2.4 agent pipeline 视图 (顺序执行链, 跟 worktree 关联)

### A3 agent 状态实时同步 (3 项)
- A3.1 14 状态机实时色码同步 (per `SRS-STAR-AGENT-RUNTIME-001.md` §8)
- A3.2 状态变化触发 audit (action: agent.status.change, per V0.1 §4.1)
- A3.3 状态变化触发 notification (e.g. 失败 → @ 5 域 Lead)

### A4 agent 关联 worktree (2 项)
- A4.1 1 agent → N worktree, worktree_node 关联 (V0.1 §4.4 已实装, 扩展)
- A4.2 worktree status 变化 → agent_node 状态联动 (V0.1 §4.4 已实装, 扩展)

### A5 agent 关联 work-item (2 项)
- A5.1 1 agent → N work-item, work_item_card 关联 (V0.1 §4.2 已实装, 扩展)
- A5.2 work-item 状态变化 → agent_node 状态联动 (V0.1 §4.4 已实装, 扩展)

### A6 agent 操作菜单 (4 项)
- A6.1 启停 (start / stop), 权限: SRE Lead / 5 域 Lead
- A6.2 重启 (restart), 权限: SRE Lead / 5 域 Lead
- A6.3 查看 logs (跳 agent-runtime logs)
- A6.4 查看 settings (跳 /agent-settings, V0.1 已实装)

### A7 agent 监控面板 (3 项)
- A7.1 实时 status / token / cost / runtime 仪表
- A7.2 token 用量对比 budget (per 守门 #4 OLU 预算 + STAR-OLU-001 v0.1)
- A7.3 异常告警 (e.g. token 超预算, runtime 异常)

### A8 agent 聚类 / 排序 / 过滤 (3 项)
- A8.1 按 role (supervisor / worker / reviewer) 聚类
- A8.2 按 kind (claude / gpt-4 / custom) 排序
- A8.3 按 status / token 用量 / 启动时间 过滤

### A9 agent session 跨域引用 (2 项)
- A9.1 跟 SRS-AGENT-VIEW-001 协同 (双向跳 /agent-view?agent=)
- A9.2 跟 SRS-AGENT-RELATIONSHIP-001 协同 (5 种关系)

### A10 agent settings (V0.1 已实装) (2 项)
- A10.1 agent-settings tab 集成 (V0.1 `AgentSettingsTab.tsx`)
- A10.2 画布操作调用 settings (e.g. 改 agent 角色 → 画布重新聚类)

### A11 ARG 图论构造 (per 17:21 JST Ulysses 补充, 10 项, 主源: `SRS-AGENT-RELATIONSHIP-001.md` v0.1)
- A11.1 **10 类关系边渲染** (4 核心: delegates_to / consults / collaborates_with / reports_to; 6 扩展: mentors / peer_reviews / stand_in_for / shadows / challenges / trusts; 颜色区分, per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.1+4.1.2)
- A11.2 **关系编辑** (UI 拖拽建边 + type 选择器 + weight 0.0-1.0 滑块 + metadata JSON 字段, per §4.1.3 关系属性)
- A11.3 **4 维度协作影响 UI 指示器** (dispatch 路由 / 上下文共享 / 信任度加权 / 产出评估, per §4.3 核心需求)
- A11.4 **5 团队模板 1-click 部署** (Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council, per §4.5 + UC-05)
- A11.5 **关系 audit log** (per 守门 #13 Transaction append-only, SCD Type 2 version +1, 4 表 agents / agent_relationship_edges / agent_relationship_edges_audit / team_template_instances)
- A11.6 **关系权重可视化** (边粗细 0.0-1.0 + 颜色渐变灰→绿, 信任度动态: 成功 +0.01 / 失败 -0.05, per §4.3.3 + UC-07)
- A11.7 **关系 archive / restore** (`archived` 字段, archived 关系不影响协作, per §4.1.3)
- A11.8 **关系版本控制 (SCD Type 2)** (改关系 version +1, optimistic lock, per §4.4 冲突解决)
- A11.9 **同步桥 UI 状态显示** (Memgraph ↔ LangGraph StateGraph, sync status indicator + last_sync_at + sync_error, per §4.4 + UC-01)
- A11.10 **跟 SRS-AGENT-RELATIONSHIP-001 协同** (Agent View "Relationship" tab 切换, Agent Relationship Editor + Relationship View + Achievement Wall, per §1.3 + §6)

### 优先级建议
- P0: A1.1-1.3, A2.1-2.2, A3.1, A4.1-4.2, A5.1-5.2, A6.1-6.2, **A11.1-11.3, A11.5** (核心管理 + ARG 基础)
- P1: A2.3-2.4, A3.2-3.3, A6.3-6.4, A7.1-7.3, A8.1-8.3, **A11.4, A11.6, A11.9-11.10** (监控 + 操作 + ARG 高级)
- P2: A9.1-9.2, A10.1-10.2, **A11.7-11.8** (跨域集成 + ARG 维护)

## 6. 落地清单

| # | 文件 | 内容 | 行数预估 |
|---|---|---|---|
| 1 | `docs/requirements/SRS-CANVAS-AGENT-001.md` | 9 段 SRS, 38 项展开 (含 A11 ARG 10 项) | ~1100-1500 行 |

预估 0 commit (root 统一 commit), 1 文件, ~30-40K 字。

## 7. 返报告知 (per 守门 #9 v27 collect_output)

子代理返回时, 报告必须含:
1. 实际写入文件路径 + 字节数
2. 9 段是否齐全
3. **38 项**展开计数: FR 数量 + NFR 数量 + AC 数量 + US 数量
4. 已知缺口清单 (≥ 8 个, 含 A11 跨专题 5 缺口)
5. 跨专题引用清单 (引用了 SRS-CANVAS-GAMIFY-001 / SRS-CANVAS-001 / 现有 SRS-AGENT-VIEW-001 / **SRS-AGENT-RELATIONSHIP-001 §1-§8** / BD + DD + DDD-REVIEW-AGENT-RELATIONSHIP-001 / SRS-STAR-AGENT-RUNTIME-001 的具体 §)
6. **A11 必含 4 表 W/T/M 三類横展** (per 守门 #13: agents = Master / agent_relationship_edges = Master / agent_relationship_edges_audit = Transaction / team_template_instances = Work) — 列出每张表归到 W/T/M 哪類
7. **A11 必含 10 类关系 + 4 维度 + 5 团队模板** 完整列表 (per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.1+4.1.2+4.3+4.5)
8. **不写 Miro 通用功能** 清单 (本专题明确砍掉的, 防止 scope creep)
9. 任何意外 / 偏离 / 简化 / 跳过 项, 显式标注

不要只回 "done" — 必须给可验证证据.
