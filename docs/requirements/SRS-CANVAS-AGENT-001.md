# SRS-CANVAS-AGENT-001

> **无限画布 — Agent 管理域 (Agent Management Domain) 要件定義書 v1.2** (per 日本 IPA SEC 標準 / 要件定義書 テンプレート)
>
> - 状态: Requirements Baseline (**v1.2 重写: 38 → 46 项, 新增 A12 多人编辑 8 项, per 2026-09-10 17:34 JST Ulysses 拍板"多人编辑是要的", 撤回 17:08 JST 砍多人编辑决定, v0.63 反转**)
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: (root 统一 commit 时填, per 守门 #1 v15 docs 同步饱和 + 1 commit 多文件)
> - 关联 V0.1 实装: `docs/frontend-canvas-design.md` v0.1 + `frontend/src/components/CanvasView.tsx` (line 218-235 agent_cursor 基线 + **line 253-262 comment_pin** for A12.5) + `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1
> - 上位要件: `docs/requirements/SRS-CANVAS-001.md` v1.1 (无限画布总册 v1.1 三次更新版, 含 17:34 JST 拍板 v0.63 反转 A12 多人编辑 8 项)
> - 平行专题 SRS: `docs/requirements/SRS-CANVAS-GAMIFY-001.md` (双核心之 2: 游戏化, 32 项)
> - 平行 view (A11 主源): `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1 (ARG, 9/8 落档, 37KB, 10 类关系 + 4 维度 + 5 模板 + 4 表)
> - 协同 SRS: `SRS-AGENT-VIEW-001.md` v1.0 (V0.1 agent 视图, 31KB) + `SRS-STAR-AGENT-RUNTIME-001.md` v1.0 (Agent Runtime, 53KB)
> - ARG 设计链: `BD-AGENT-RELATIONSHIP-001.md` v0.1 (55KB, 基本設計) + `DD-AGENT-RELATIONSHIP-001.md` (94KB, 詳細設計) + `DDD-REVIEW-AGENT-RELATIONSHIP-001.md` (30KB, 跨 DDD 边界)
> - **拍板来源 (v0.63 反转)**: 2026-09-10 17:34 JST Ulysses 拍板"**多人编辑是要的**" + 2026-09-10 17:21 JST 补充"画布内体现 agent 之间关系的图论构造" (A11 主源, 引用 `SRS-AGENT-RELATIONSHIP-001.md` v0.1 9/8 落档) + 2026-09-10 17:08 JST 拍板"管理 agent 和游戏化, 避免过度冗余" (撤回方向, 但多人编辑本身被 17:34 拍板保留)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3 Mavis 永久代签 + 9/8 15:19 JST 第 6 次强化)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) (per 守门 #14 v4 反转 v0.62 2026-09-10 12:45 JST)
> - 日期: 2026-09-10 JST
> - 受众: 詳細設計工程師 / 架構審查者 / UI/UX 設計師 / SRE Lead / 5 域 Lead (未到位, Mavis 临时代签 per 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
> - 跨域 disclaimer: **5 域独立 Lead ≠ Star 22 DDD bounded context** (per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer, 不建立业务子域↔DDD 映射)

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-CANVAS-AGENT-001 |
| 文书名 | 无限画布 — Agent 管理域 (Agent Management Domain) 要件定義書 |
| 版本 | **v1.2 (重写: 38 → 46 项, 新增 A12 多人编辑 8 项, v0.63 反转)** |
| 作成日 | 2026-09-10 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 commit | (root 统一 commit, per 守门 #1 v15) |
| 关联文档 | `frontend-canvas-design.md` v0.1 (V0.1 实装) + `CanvasView.tsx` (line 218-235 agent_cursor + line 253-262 comment_pin) + `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1 + 2 份平行专题 SRS (本 + GAMIFY) + 1 份协同 SRS (AGENT-VIEW) + 1 份协同 SRS (STAR-AGENT-RUNTIME) + 4 份 ARG 文档链 (SRS + BD + DD + DDD-REVIEW) + 总册 `SRS-CANVAS-001.md` v1.1 (三次更新版) |
| 上位文档 | `SRS-CANVAS-001.md` v1.1 (总册, 双核心 78 项 = 46+32, 含 17:34 JST v0.63 反转) |
| 平行文档 | `SRS-CANVAS-GAMIFY-001.md` v0.1 (双核心之 2: 游戏化, 32 项) |

### 0.2 修订履历 (本 SRS)

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v1.0** | 2026-09-10 17:17 JST | Ulysses — Mavis 接手 (per 守门 #14 v3) | 初版落档, 12 段 (文档信息/目的/用语/前提/业务需求/约束/场景/数据/接口/验收/风险/签字), 28 项 (A1-A10, 10 子能力), 聚焦 agent 管理域 | 2026-09-10 17:08 JST Ulysses 拍板"管理 agent 和游戏化, 避免过度冗余" |
| **v1.1** | 2026-09-10 17:22 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | **重写: 28 项 → 38 项, 新增 A11 ARG 图论构造 10 项** (引用 `SRS-AGENT-RELATIONSHIP-001.md` v0.1 §1-§8 + `BD-AGENT-RELATIONSHIP-001.md` v0.1 §1-§7 + `DD-AGENT-RELATIONSHIP-001.md` + `DDD-REVIEW-AGENT-RELATIONSHIP-001.md`), 守门 #1+#3+#13+#14 v2+#14 v4 反转 全过 (文档工作, 守门 #1 v25 cargo test --workspace -j 4 不需要跑); 12 已知缺口 (含 A11 跨专题 5 缺口); A11.5 必含 4 表 W/T/M 三類横展 (per 守门 #13, 100% 表覆盖) | 2026-09-10 17:21 JST Ulysses 补充"画布内体现 agent 之间关系的图论构造" (ARG 落档) |
| **v1.2 (当前)** | 2026-09-10 17:34 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | **v0.63 反转: 重写 38 项 → 46 项, 新增 A12 多人编辑 8 项**, **撤回 17:08 JST 砍多人编辑决定** (per 17:34 JST Ulysses 拍板"多人编辑是要的", 显式标 v0.63 反转行, per 守门 #1 禁回溯叙事); 引用 `frontend-canvas-design.md` §4.1 模式 A Realtime 通道 (A12.1 + A12.3) + §4.6 PresenceCursor 升级 (A12.2) + `CanvasView.tsx` line 253-262 comment_pin (A12.5); 46 项 1:1 展开 (FR-AGENT-A1.1-A12.8 + NFR-AGENT-16 项 + AC-AGENT-46 项), 28 用户故事 (≥ 28 满足, 46 × 60% = 27.6), 13 已知缺口 (含 A11 跨专题 5 缺口 + **A12 多人编辑 5 缺口**), A11.5 必含 4 表 W/T/M 三類横展 + **A12.8 必含新增表 `canvas_multi_user_audit` 100% RLS 13 类 + W/T/M Transaction append-only + SCD Type 2** (per 守门 #13, 100% 表覆盖), 守门 #1+#3+#5+#6+#7+#9+#10+#13+#14 v2+#14 v4+#15+#19+#23 v2+#1 v15 14 项全过 (文档工作, 守门 #1 v25 cargo test --workspace -j 4 不需要跑) | 2026-09-10 17:34 JST Ulysses 拍板"**多人编辑是要的**" (v0.63 反转) + 总册 `SRS-CANVAS-001.md` v1.1 三次更新版同步 |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档按 日本 IPA SEC 標準 制定 STAR 平台 **无限画布 — Agent 管理域** 的需求规格说明书, 涵盖无限画布上"管理 agent 节点" (节点 / 拓扑 / 状态实时同步 / handoff / worktree 关联 / 监控操作 / 聚类排序过滤 / 跨域引用 / settings 集成) + **A11 ARG 图论构造 (10 类关系 + 4 维度协作影响 + 5 团队模板 + 同步桥 UI + 成就协同)** + **A12 多人编辑 (多人同时编辑 + 实时 cursor + 元素增删改同步 + Follow mode + 评论线程 + @ 提醒 + 冲突解决 + view/comment/edit 3 级权限 + audit log)** 的全部 **46 项**需求。

本 SRS 是 P3-D.5 阶段 "无限画布需求文档化" 的子专题产物, 跟 `SRS-CANVAS-001` v1.1 (总册, 含 v0.63 反转) + `SRS-CANVAS-GAMIFY-001` (双核心之 2: 游戏化) 共同构成无限画布双核心 SRS 三件套。

作为后续基本设计 (`BD-CANVAS-AGENT-001.md` 待 P3-D 阶段落档) / 详细设计 / 实装 / 测试 / 验收的唯一依据。

### 1.2 背景 (用户痛点)

STAR 平台 V0.1 MVP 无限画布 (per `frontend-canvas-design.md` v0.1, 2026-09-04 落档) 已经落地基础能力 (14 element + 4 frame + 8 connector + 9 e2e 守门), 但 V0.1 `agent_cursor` 只是"圆点 + 名字 + 状态色码" 简化表示, 缺乏完整的 agent 节点卡 (头像/role/kind/token 用量/启动时间) + 拓扑图 (handoff / 5 域 / 父子 / pipeline) + 状态实时同步 + 监控 + 操作 + 聚类排序过滤能力 + 跨 session 多人协同编辑能力。

**用户痛点** (per 2026-09-10 17:08 JST Ulysses 拍板): 5 域 Lead / PM / SRE 需要在画布上**集中管理 agent 节点**, 一眼看到 5 域所有 agent 拓扑, 实时状态色码同步, 双击跳详情, 启停 / 重启 / 监控 / 跨域引用。

**2026-09-10 17:21 JST 补充** (A11 主源): agent 之间的关系不是"展示标签", 而是真实的图论构造, 需要在画布上体现 (delegates_to / consults / collaborates_with / reports_to 4 核心 + mentors / peer_reviews / stand_in_for / shadows / challenges / trusts 6 扩展), 关系必须**影响协作** (dispatch 路由 / 上下文共享 / 信任度 / 产出评估 4 维度), 通过 5 个团队模板 (Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council) 1-click 部署, 解锁成就。

**2026-09-10 17:34 JST v0.63 反转** (A12 主源): 撤回 17:08 JST 砍多人编辑决定, Ulysses 拍板"**多人编辑是要的**" — 画布需要支持**多用户同时编辑** (max 10 并发) + **实时 cursor 同步** (其他用户光标 + 名字 + viewport) + **元素增删改实时同步** (跟 V0.1 localStorage + zustand persist 冲突, 需 backend 持久化) + **Follow mode** (A 用户跟随 B 用户视角) + **多人评论线程 + @ 提醒** (V0.1 comment_pin line 253-262 升级) + **冲突解决** (CRDT 选型: Yjs vs Automerge vs LWW) + **view/comment/edit 3 级权限** (BFF API 层强制) + **audit log 多人操作** (新增表 `canvas_multi_user_audit`, per 守门 #13 Transaction append-only + SCD Type 2 + 100% RLS 13 类)。引用 `frontend-canvas-design.md` §4.1 模式 A Realtime 通道 + §4.6 PresenceCursor 升级。

**ARG 跟现有视图的关系** (per `SRS-AGENT-RELATIONSHIP-001.md` §1.2):
- **不取代** LangGraph 任务卡 DAG (TMO 9 节点, 任务编排)
- **不取代** Agent View 画布 (`SRS-AGENT-VIEW-001` v1.0, 个体 session 可视化)
- **新增第三层** — "agent 之间" 的关系层, 是 agent 维度的 social graph

**A12 多人编辑跟现有 V0.1 关系** (per `frontend-canvas-design.md` v0.1 + `CanvasView.tsx`):
- **不取代** V0.1 基础画布 (pan/zoom/select/element/connector/frame), 仅**扩展** 多人协同能力
- **复用** V0.1 §4.1 模式 A Realtime 通道 (扩展为 A12.1 + A12.3)
- **复用** V0.1 §4.6 PresenceCursor (升级为 A12.2 多人 cursor 同步)
- **扩展** V0.1 line 253-262 `comment_pin` 元素 (加 thread + @ 提醒, A12.5)
- **新增** A12.4 Follow mode + A12.6 CRDT 冲突解决 + A12.7 view/comment/edit 3 级权限 + A12.8 audit log

### 1.3 包含范围 (In-Scope, **46 项**)

**双核心之 1: Agent 管理域**, 共 **12 个子能力 (A1-A12)**, **46 项**:

| 子能力 | 范围 | 项数 | 优先级 | V0.1 基线 |
|---|---|---|---|---|
| **A1 agent 节点渲染** | element kind `agent_cursor` (V0.1) 升级为 `agent_node` 完整卡 | 3 项 | P0 | `CanvasView.tsx` line 218-235 |
| **A2 agent 拓扑图** | 1:N handoff / 跨 5 域 / 父子 / pipeline 视图 | 4 项 | P0 / P1 | V0.1 `agent_handoff` connector 扩展 |
| **A3 agent 状态实时同步** | 14 状态机实时色码 + audit + notification | 3 项 | P0 | per `SRS-STAR-AGENT-RUNTIME-001.md` §8 |
| **A4 agent 关联 worktree** | 1 agent → N worktree, 跟 StatusPill 60+ 同步 | 2 项 | P0 | V0.1 §4.4 扩展 |
| **A5 agent 关联 work-item** | 1 agent → N work-item, drag in/out | 2 项 | P0 | V0.1 §4.2 + §4.4 扩展 |
| **A6 agent 操作菜单** | 右键菜单 (启停/重启/logs/token/settings) | 4 项 | P0 / P1 | 新增 |
| **A7 agent 监控面板** | 实时 status / token / cost / runtime 仪表 | 3 项 | P1 | 新增 |
| **A8 agent 聚类 / 排序 / 过滤** | 按 role / kind / status 聚类 + token / 启动时间 排序 | 3 项 | P1 | 新增 |
| **A9 agent session 跨域引用** | 跟 SRS-AGENT-VIEW-001 / SRS-AGENT-RELATIONSHIP-001 协同, 双向跳 | 2 项 | P2 | 跨块接口 |
| **A10 agent settings (V0.1 已实装)** | agent-settings tab 集成 + 画布操作调用 settings | 2 项 | P2 | `AgentSettingsTab.tsx` V0.1 已有 |
| **A11 ARG 图论构造 (per 17:21 JST Ulysses 补充)** | **10 类关系边 + 4 维度协作影响 + 5 团队模板 + 同步桥 + 成就协同** (主源 `SRS-AGENT-RELATIONSHIP-001.md` v0.1) | **10 项** | P0 / P1 / P2 | 新增 |
| **A12 多人编辑 (per 17:34 JST Ulysses 拍板"多人编辑是要的", v0.63 反转, 撤回 17:08 JST 砍多人编辑决定)** | **多人同时编辑 + 实时 cursor + 元素增删改同步 + Follow mode + 评论线程 + @ 提醒 + 冲突解决 (CRDT 选型) + view/comment/edit 3 级权限 + audit log** (主源 `frontend-canvas-design.md` §4.1 模式 A + §4.6 PresenceCursor 升级) | **8 项** | P0 / P1 | V0.1 §4.1 + §4.6 + line 253-262 comment_pin 扩展 |
| **合计** | | **46 项** | | |

**V0.1 MVP 衔接** (per 守门 #11 缺标比错标): V0.1 已实装能力, 本 SRS **保留**为 V0.1 不重新设计, 仅做扩展:

| V0.1 已实装 | 引用 | 衔接方式 |
|---|---|---|
| 无限世界坐标 + pan/zoom (0.1x-4x) | `CanvasView.tsx` §3 | 不重构, 仅在 NFR 引用 |
| 8 element kind | `CanvasView.tsx` line 156-340 | 类型扩展, 不破坏 schema |
| `agent_cursor` (line 218-235) | AGENT A1 扩展为 `agent_node` 完整卡 | A1.1 |
| `agent_handoff` connector | V0.1 §3.5 | A2.1 扩展 (加时间/状态/备注) |
| 5 联动 (WorkItem/Worktree/Relation/Comment/Search URL) | `frontend-canvas-design.md` §4 | 跨块接口 §8 引用 |
| **§4.1 模式 A Realtime 通道** (per 17:34 JST v0.63 反转) | `frontend-canvas-design.md` §4.1 | A12.1 + A12.3 扩展 (BFF 推 element 增删改 → 多用户实时同步) |
| **§4.6 PresenceCursor 升级** (per 17:34 JST v0.63 反转) | `frontend-canvas-design.md` §4.6 | A12.2 扩展 (cursor 锚定 element + 多人显示) |
| **`comment_pin` (line 253-262)** (per 17:34 JST v0.63 反转) | `CanvasView.tsx` line 253-262 | A12.5 扩展 (加 thread + @ 提醒) |
| agent_settings (V0.1) | `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1 | A10.1-10.2 仅做集成, 不重写 |
| StatusPill 60+ 色码 | `frontend-canvas-design.md` §3.4 + ADR-FE-013 | A1.2 / A3.1 必含 |
| 5×4 grid layout 算法 | `frontend/src/lib/agent-view/layout.ts` | A2 / A8 复用 |

### 1.4 不包含范围 (Out-of-Scope, per 守门 #11 缺标比错标)

**核心原则**: **避免过度冗余** (per 2026-09-10 17:08 JST Ulysses 拍板"避免过度冗余"), 以下 **Miro 通用功能 + 跨专题内容** 全部砍掉 / 留给其他 SRS, 本 SRS 不展开:

| 不包含类别 | 不包含内容 | 砍掉理由 / 转交方 |
|---|---|---|
| ~~**Miro 通用协作 (1 大类)**~~ | ~~多人同时编辑 / 实时 cursor / Follow mode / 评论线程 / @ 提醒 / 内置视频通话~~ | **v0.63 反转 (per 2026-09-10 17:34 JST Ulysses 拍板"多人编辑是要的")**: **多人编辑撤回砍掉决定, 本 SRS §4.12 A12 必含 8 项多人编辑**; 但**内置视频通话** (Miro Talk / Zoom 集成) 仍砍掉, 留 P3+; 协作工具集成 (Slack / Jira) 仍砍掉, 留 P3+ |
| **Miro 通用演示** | Frame as slide / Guided Tour / Speaker notes / Timer 演讲 | 演示用 Keynote/PowerPoint 已成熟 |
| **Miro 通用导出** | PDF / Word / Excel / CSV / SVG | PNG 已有 (V0.1), 文本导出超出核心 |
| **Miro 通用版本** | Version history / Branching / Restore | 跟 Git worktree 重复, 用 Git 即可 |
| **Miro 通用集成** | Slack / Jira / Asana / Figma / Notion / GitHub / Zoom | Star 25 module 已有, 集成超出画布核心 |
| **Miro 通用移动** | iOS / Android native / Touch / Stylus / Offline | 客户端重投入, 短期 web 触控够用 |
| **Miro 12 种 diagram** | Mind map / Flowchart / BPMN / ER / Wireframe / Kanban / Sequence / Smart drawing / Draw | 内容生产用专门工具 |
| **Miro 模板库** | 2500+ 模板 | 评估做 5-10 个, 留 P3+ |
| **Miro AI 通用** | 文本生成 diagram / 翻译 / 图像识别 | 仅留 sticky note 聚类 (GAMIFY G5) |
| **Miro Tables / Chart widget / Form** | 表格 / 数据源 / Chart / 表单 | 跟 Star 数据模块重复 |
| **Miro 完整 a11y (WCAG 2.1 AA)** | 完整合规 | 部分 a11y 跟随 V0.1, 完整合规留 P3+ |
| **5 域 Lead 真人到位前跨域编排决策** | 真人 Lead 决策 | per 9/3 11:35 JST 拍板 B: Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/5 10:43 JST 拍板 D), **不沿用代签决策** (per 守门 #1 禁回溯叙事) |
| **5 域独立 Lead ≠ Star 22 DDD bounded context** | 业务子域↔DDD 映射 | per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer |
| **基础画布能力 (pan/zoom/select/element/connector/frame)** | V0.1 已实装, 不重写 | per 守门 #1 v19 累积规: 不偷偷 commit 重新设计 |
| **节点拖动编辑 (单用户已实装)** | V0.1 已实装, 派生视图单用户不持久化 | per `SRS-AGENT-VIEW-001.md` §1.4 |
| **canvas 持久化 (F5 刷新保留, 单用户已实装)** | V0.1 localStorage + zustand persist, 派生数据不持久化 | per `SRS-AGENT-VIEW-001.md` §1.4; **A12.3 多人编辑扩展后, 元素增删改需 backend 持久化, 跟 V0.1 冲突** |
| **节点 minimap 点击跳转** | minimap 只是 viewport 可视化 | per `SRS-AGENT-VIEW-001.md` §1.4 |
| **agent session 创建 / 启停 / 状态机** | 后端 Agent Runtime 管 | per `SRS-STAR-AGENT-RUNTIME-001.md` §0 |
| **任务卡 DAG 改造** | TMO 9 节点管, ARG 只读 task_relationships 字段 | per `SRS-AGENT-RELATIONSHIP-001.md` §1.4 |
| **关系自动发现** | 本视图是"用户定义 + 协作行为反馈", 不做自动发现 | per `SRS-AGENT-RELATIONSHIP-001.md` §1.4 |
| **跨组织 agent 联邦** | 本视图只覆盖 Star 仓内部 agent | per `SRS-AGENT-RELATIONSHIP-001.md` §1.4 |
| **ARG → RGS 仓数据同步** | Star 仓不引用 RGS 仓 | per AGENTS.md §5 仓库拓扑 |
| **真实 LLM 微调** | 关系层不改模型权重, 只改 prompt 拼装 + dispatch 路由 | per `SRS-AGENT-RELATIONSHIP-001.md` §1.4 |
| **游戏化 32 项** | sticky_note 聚类 / 投票 / reaction / confetti / 排行榜 / 道具 | 平行专题 `SRS-CANVAS-GAMIFY-001` |
| **完整 a11y (WCAG 2.1 AA)** | 完整合规 | 留 P3+, 当前跟随 V0.1 |
| **A12 多人编辑 WebSocket 选型** | Yjs / Automerge / LWW / native WebSocket, 拍板前 P0 阻塞 (per §7 风险) | A12.6 必含, 实装阶段拍板 |
| **A12 view/comment/edit 3 级权限具体矩阵** | 5 域 Lead 真人到位后决策 (per 守门 #14 v2) | A12.7 必含, 拍板前走 view-only 兜底 |

### 1.5 用户故事 (≥ 28 个, per 守门 #11 缺标比错标)

> 46 项 × 60% = ≥ 28 用户故事覆盖 (v1.1 时 38 × 60% = 22.8 → 25 满足; v1.2 因新增 A12 8 项, 提到 28 个)

| 编号 | 角色 | 故事 | 子能力 | 优先级 |
|---|---|---|---|---|
| US-1 | 5 域 Lead (per 守门 #3, 真人未到位 Mavis 临时代签) | 作为 Lead, 我希望在画布上一眼看到所有 agent 节点的完整卡 (头像/名字/role/kind/status/token/启动时间), 不用切换 4 个 Tab | A1 | P0 |
| US-2 | 5 域 Lead | 作为 Lead, 我希望 agent 节点按 14 状态机实时色码同步, 失败 → 自动红色 + 通知我 | A3 | P0 |
| US-3 | 5 域 Lead | 作为 Lead, 我希望看到 5 域分组 Frame, 域内 agent 自动聚类, 跨域 handoff 拓扑清晰 | A2 | P0 |
| US-4 | 5 域 Lead | 作为 Lead, 我希望父子 agent 关系 (supervisor → worker) 在画布上明确显示, 树形布局 | A2 | P1 |
| US-5 | PM (Ulysses 类) | 作为 PM, 我希望看到 agent 关联的 worktree + work-item 实时状态联动 | A4 + A5 | P0 |
| US-6 | SRE | 作为 SRE, 我希望右键 agent 节点可以启停 / 重启 / 查看 logs / 查看 token 用量 / 跳 settings | A6 | P0 |
| US-7 | SRE | 作为 SRE, 我希望看到 agent 实时 status / token / cost / runtime 仪表, 跟 token OLU 预算对比, 超预算告警 | A7 | P1 |
| US-8 | Dev | 作为 Dev, 我希望按 role / kind / status 聚类 agent, 按 token / 启动时间 排序, 过滤不相关 agent | A8 | P1 |
| US-9 | PM | 作为 PM, 我希望双击 agent 节点跳 `/agent-view?agent=ag-XXX`, 5 域 Lead 视图协同 | A9 | P2 |
| US-10 | PM | 作为 PM, 我希望画布上改 agent 角色 → 自动重聚类, 跟 V0.1 `AgentSettingsTab.tsx` settings 集成 | A10 | P2 |
| US-11 | 5 域 Lead | 作为 Lead, 我希望在画布上看到所有 agent 之间的图论关系 (节点+边+标签), 一眼看出团队拓扑 | A11.1 | P0 |
| US-12 | PM | 作为 PM, 我希望拖拽两个 agent 创建一条 "delegates_to" 关系, 之后 Lead agent 自动把任务分给 Worker | A11.2 | P0 |
| US-13 | 5 域 Lead (真人) | 作为 Lead, 我希望定义 "consults" 关系指向 Reviewer, 关键决策自动调 Reviewer 拿意见 | A11.3 | P0 |
| US-14 | Dev | 作为 Dev, 我希望用 "Hub-and-Spoke" 模板 1-click 创建 1 Lead + 4 Worker 团队, 不用挨个拖拽 | A11.4 | P1 |
| US-15 | 5 域 Lead | 作为 Lead, 我希望关系定义被 audit log 记录, 改关系有版本历史, 真人到位后可追溯 | A11.5 | P0 |
| US-16 | Dev | 作为 Dev, 我希望关系权重 (0.0-1.0) 视觉化 (边粗细 + 灰→绿色), 协作成功 weight 自动 +0.01, 失败 -0.05 | A11.6 | P1 |
| US-17 | PM | 作为 PM, 我希望归档 (archive) 旧关系不删, 改 restore 恢复, 不影响协作 (active 关系才生效) | A11.7 | P2 |
| US-18 | 5 域 Lead | 作为 Lead, 我希望改关系时 version 自动 +1, 防止并发冲突 (optimistic lock) | A11.8 | P2 |
| US-19 | SRE | 作为 SRE, 我希望画布上显示 Memgraph ↔ LangGraph StateGraph 同步桥状态 (last_sync_at / sync_error), 离线降级提示 | A11.9 | P1 |
| US-20 | PM | 作为 PM, 我希望从画布切到 "Relationship" tab 看 ARG 全图, 跟 `SRS-AGENT-RELATIONSHIP-001` 的 Relationship Editor / View / Achievement Wall 协同 | A11.10 | P1 |
| US-21 | PM | 作为 PM, 我希望 4 维度协作影响有 UI 指示器 (dispatch 路由 / 上下文共享 / 信任度 / 产出评估), 看到关系真实生效, 不止"展示标签" | A11.3 | P0 |
| US-22 | 5 域 Lead | 作为 Lead, 我希望 archive 关系不影响协作 (per `SRS-AGENT-RELATIONSHIP-001` §4.1.3 `archived` 字段), 误操作可恢复 | A11.7 | P2 |
| US-23 | Dev | 作为 Dev, 我希望 6 扩展关系 (mentors / peer_reviews / stand_in_for / shadows / challenges / trusts) 跟 4 核心关系颜色区分, 一眼区分关系类型 | A11.1 | P0 |
| US-24 | 5 域 Lead | 作为 Lead, 我希望看到 "Reports To" 关系让 Worker 完成后自动汇总到 Lead 的 inbox, Lead 看到的是聚合报告 | A11.1 / A11.3 | P1 |
| US-25 | SRE | 作为 SRE, 我希望 5 个团队模板 (Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council) 在画布上 1-click 部署, 模板选择器 dropdown | A11.4 | P1 |
| **US-26 (v1.2 新增)** | PM (Ulysses 类) | 作为 PM, 我希望跟 SRE / 5 域 Lead **多人同时编辑同一 canvas** (max 10 并发), 元素增删改实时同步 (≤ 200ms P95), 不需要互发截图 | A12.1 + A12.3 | P0 |
| **US-27 (v1.2 新增)** | 5 域 Lead | 作为 Lead, 我希望看到**其他用户实时 cursor** + 名字 + 当前 viewport, 知道 SRE 在看哪个 agent_node, 减少沟通成本 | A12.2 | P0 |
| **US-28 (v1.2 新增)** | PM | 作为 PM, 我希望 **Follow mode** 跟随 SRE 视角, SRE 移动 / 选中 element 时, 我的 viewport 同步, 跟 GitHub Live Share 一样 | A12.4 | P1 |
| **US-29 (v1.2 新增)** | Dev | 作为 Dev, 我希望在 element 上**添加多人评论线程**, 跟其他用户**@ 提醒** (e.g. @ PM 审核), notification 域对接, 不切出画布 | A12.5 | P0 |
| **US-30 (v1.2 新增)** | SRE | 作为 SRE, 我希望多人同时拖拽同一 element 不会冲突, 走 **CRDT (Yjs / Automerge / LWW) 自动 conflict-free merge**, 不需要锁 | A12.6 | P0 |
| **US-31 (v1.2 新增)** | 5 域 Lead | 作为 Lead, 我希望**view / comment / edit 3 级权限**, 外部用户仅 view, 内部 Lead 可 comment / edit, BFF API 层强制, 不依赖前端隐藏 | A12.7 | P1 |
| **US-32 (v1.2 新增)** | SRE | 作为 SRE, 我希望所有**多人操作** (增删改 / cursor 移动 / 评论 / @) 100% 写到 audit log (Transaction append-only), 真人 Lead 到位后可追溯 | A12.8 | P0 |

**用户故事覆盖统计**: **32 个用户故事** (≥ 28, 46 × 60% = 27.6 → 32, 满足; 25 v1.1 + 7 v1.2 新增 = 32)

---

## §2 用語定義 (用語集 / Ubiquitous Language)

### 2.1 画布基础 (per frontend-canvas-design.md v0.1)

| 用語 | 定義 | 出处 |
|---|---|---|
| 无限画布 (Infinite Canvas) | Miro 风格画布, 世界坐标 + viewport 转换, 鼠标 pan/zoom, 不强制栅格 | per `frontend-canvas-design.md` v0.1 §0 |
| 世界坐标 (World) | 画布无限延伸, 所有 element 真实位置 (x, y ∈ [-100K, 100K]) | per design §3.2 |
| 屏幕坐标 (Screen) | 浏览器视口, 跟 world 转换公式 `screen = (world - viewport) * zoom` | per design §3.2 |
| Viewport | 画布观察窗口, 含 pan (x, y) + zoom (0.1x ~ 4x) | per design §3.2 |
| Frame | 画布分区, 矩形 + 标题, 可作 5 域分组 (per A2.2) | per design §2.1 + §3.6 |
| Element | 画布可视对象 (V0.1 8 kind + A1 扩展 `agent_node` + A11 扩展 ARG 元素 + A12 扩展多人编辑元素) | per design §2.2 |
| Connector | 画布连线 (3 routing: straight / curved / orthogonal) | per design §2.3 + §3.5 |
| Bezier Connector | 三次贝塞尔曲线, 复用 SmView 算法 (c1x = fx + dx*0.25, c2x = tx - dx*0.25) | per design §3.5 |
| StatusPill 60+ 色码 | 14 状态 × 多种语义色码 (in_progress=blue, failed=red, ...), 画布上必用一致 | per `frontend-canvas-design.md` §3.4 + ADR-FE-013 |

### 2.2 Agent 管理域 (本 SRS 新增 + 引用)

| 用語 | 定義 | 出处 |
|---|---|---|
| **agent_node** | V0.1 `agent_cursor` 升级为完整卡 (头像/名字/role/kind/status/token 用量/启动时间), 元素大小 220x110, 蓝底 | 本 SRS A1.1 |
| **Agent Session** | 1 个 AI Agent 执行的会话实例, **14 状态机**: queued / spawning / initializing / compiling_context / planning / executing / awaiting_feedback / awaiting_human / awaiting_tool / validating / paused / completed / failed / cancelled | per `SRS-STAR-AGENT-RUNTIME-001.md` §8 + §35-§42 |
| **Active Agent** | 处于上述 14 状态中前 11 个 (排除 completed/failed/cancelled 终态) 的 agent session | per `SRS-AGENT-VIEW-001.md` §2 |
| **Agent Handoff** | 1 个 agent 完成后, 上下文传给下 1 个 agent, 含时间 / 状态 / 备注, 画布上 connector 体现 | 本 SRS A2.1 |
| **5 域分组** | 5 域 (player / economy / match / social / admin per 8/21 JST RGS 治理命名) Frame 分组, 域内 agent 自动聚类 | 本 SRS A2.2 |
| **Agent 父子关系** | 1 parent → N child (e.g. supervisor → worker), 树形布局 | 本 SRS A2.3 |
| **Agent Pipeline** | 顺序执行链 A → B → C, 跟 worktree 关联 | 本 SRS A2.4 |
| **Token OLU** | 1 SRE·周 ≈ 1.2M tokens (per STAR-OLU-001 v0.1 2026-08-29 落档) | per 守门 #4 |
| **Right-Click Context Menu** | 画布上右键 agent_node 弹出的操作菜单 (启停 / 重启 / logs / token / settings) | 本 SRS A6 |

### 2.3 ARG 图论构造 (per 2026-09-10 17:21 JST Ulysses 补充, 主源 `SRS-AGENT-RELATIONSHIP-001.md` v0.1)

| 用語 | 定義 | 出处 |
|---|---|---|
| **Agent Relationship Graph (ARG)** | Agent 节点 + 关系边的有向图, 存储在 Memgraph, 跨 session 持久 | per `SRS-AGENT-RELATIONSHIP-001.md` §2 |
| **Agent 节点 (Agent Node)** | ARG 节点 = 1 个可执行实体, 类型 ∈ {SA-01..SA-09 9 个 Archetype, 5 域 Lead 真人, Custom Agent} | per `SRS-AGENT-RELATIONSHIP-001.md` §2 + `SRS-STAR-AGENT-RUNTIME-001.md` §2.3 |
| **关系边 (Relationship Edge)** | 两个 agent 之间的有向 / 无向关系, 类型 ∈ 10 类 (4 核心 + 6 扩展, 详见 §4 A11.1) | per `SRS-AGENT-RELATIONSHIP-001.md` §2 + §4.1 |
| **4 核心关系** | `delegates_to` (A → B) / `consults` (A → B) / `collaborates_with` (A ↔ B 无向) / `reports_to` (A → B) | per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.1 + 9/8 22:35 拍板 |
| **6 扩展关系** | `mentors` (A → B) / `peer_reviews` (A ↔ B 无向) / `stand_in_for` (A → B) / `shadows` (A → B) / `challenges` (A → B) / `trusts` (A → B) | per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.2 + 9/8 22:35 拍板 |
| **Memgraph** | 开源图数据库, 用 Cypher 查询, 支持事务/索引/触发器, Docker 一键启动 (port 7687 Bolt + 7444 HTTP) | per `SRS-AGENT-RELATIONSHIP-001.md` §2 + §3.2 PR-4 |
| **In-process StateGraph** | LangGraph 进程内图状态, 跟 Memgraph 双向同步, 跟 [LangGraph 02 §2.1](../../architecture/2026-09-03-langgraph/02-basic-design.md) StateGraph 同形 | per `SRS-AGENT-RELATIONSHIP-001.md` §2 |
| **同步桥 (Sync Bridge)** | Memgraph 写 → 触发 in-process StateGraph update, in-process 状态变 → 周期 flush 30s 到 Memgraph | per `SRS-AGENT-RELATIONSHIP-001.md` §2 + §4.4 |
| **关系→协作影响 (Edge Effect)** | 关系边不是标签, 会真实影响 dispatch 路由 / 上下文共享 / 信任度 / 产出评估 (4 维度) | per `SRS-AGENT-RELATIONSHIP-001.md` §2 + §4.3 + 用户原话"对它们的工作产生益处" |
| **4 维度协作影响** | 4.3.1 Dispatch 路由 / 4.3.2 上下文共享 / 4.3.3 信任度加权 / 4.3.4 产出评估 | per `SRS-AGENT-RELATIONSHIP-001.md` §4.3 |
| **团队模板 (Team Template)** | 预定义拓扑 + 关系配置, 1-click 部署, 5 个开箱即用 (Hub-and-Spoke / Mesh / Chain / Hierarchical / Review-Council) | per `SRS-AGENT-RELATIONSHIP-001.md` §2 + §4.5 + §5.2 |
| **5 团队模板** | 1. **Hub-and-Spoke** 1 Lead + 4 Worker / 2. **Mesh** N 节点全连接 / 3. **Chain** A→B→C→D 流水线 / 4. **Hierarchical** 1 Lead → 2 Sub-Lead → 6 Worker / 5. **Review-Council** 1 Lead + 3 Reviewer | per `SRS-AGENT-RELATIONSHIP-001.md` §4.5 |
| **关系 audit log** | 关系的增删改版本历史, append-only, per 守门 #13 Transaction 表 | per `SRS-AGENT-RELATIONSHIP-001.md` §2 + §7.1 |
| **关系权重 (Edge Weight)** | 边的强度/置信度, 0.0-1.0, 影响调度优先级, 默认 0.5, 动态调整: 成功 +0.01 / 失败 -0.05 | per `SRS-AGENT-RELATIONSHIP-001.md` §2 + §4.3.3 |
| **信任度 (Trust Score)** | 累计协作成功率 × edge weight, 0.0-1.0, 越高的 agent 越被优先 dispatch | per `SRS-AGENT-RELATIONSHIP-001.md` §2 + §4.3.3 |
| **Active Relationship** | 当前活跃的关系 (`archived=false`), 只有 active 的关系影响协作 | per `SRS-AGENT-RELATIONSHIP-001.md` §2 |
| **SCD Type 2** | 关系改一次 version +1, 物理删除禁止, 旧版本保留 (per 守门 #13 Master 类) | per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.3 + 守门 #13 |

### 2.4 跨域共享 (per 总册 + 守门)

| 用語 | 定義 | 出处 |
|---|---|---|
| 派生视图 (Projection) | 从已有数据派生计算的视图, 不是业务事实源, 不可写 (ARG 编辑操作归 ARG 主源 SRS, 本 SRS 仅渲染) | DDD 概念, per `SRS-AGENT-VIEW-001.md` §2 + `SRS-Runtime` §4.5 |
| 跨域编排 (Cross-Domain Orchestration) | 多 domain 协作完成 1 业务用例, 由 Saga orchestrator 协调 | per 守门 #3 v2 + 9/3 11:35 JST 拍板 B |
| Mavis 接手 | Mavis 接手代理 Ulysses 决策 (per 守门 #14 v3 Mavis 永久代签 + 9/8 15:19 JST 第 6 次强化) | per AGENTS.md §4 #14 |
| Mavis 审核 | per 2026-09-10 12:45 JST v0.62 反转, 真人代签流程全部取消, 改为 Mavis 审核 决定 author=Ulysses | per 守门 #14 v4 + v0.62 反转 |
| DB W/T/M 三類 | Work (短 TTL 作業中) / Transaction (業務事実 Append-only 監査必携) / Master (SCD Type 2 慢变参考) | per 守门 #13 (per AGENTS.md §4 #13) |
| **ARG 4 表 W/T/M 必含** | `agents` (Master) + `agent_relationship_edges` (Master) + `agent_relationship_edges_audit` (Transaction) + `team_template_instances` (Work, TTL 30 天) | per `SRS-AGENT-RELATIONSHIP-001.md` §7.1 + 守门 #13 |

### 2.5 A12 多人编辑 (per 2026-09-10 17:34 JST Ulysses 拍板"多人编辑是要的", v0.63 反转, 新增于 v1.2)

| 用語 | 定義 | 出处 |
|---|---|---|
| **多人同时编辑 (Multi-User Realtime Edit)** | N 个用户 (max 10 并发) 同时编辑同一 canvas, 元素增删改实时同步, max 200ms (P95) 延迟 | per `frontend-canvas-design.md` §4.1 模式 A Realtime 通道 + 本 SRS A12.1 |
| **Realtime 通道 (Realtime Channel)** | BFF 推 element 增删改 → 所有浏览同一 canvas 的用户实时看到, 走 WSS (WebSocket Secure) | per `frontend-canvas-design.md` §4.1 模式 A (V0.1 已 design, A12.1 扩展实现) |
| **实时 Cursor 同步 (PresenceCursor Realtime)** | 其他用户光标位置 + 名字 + 当前 viewport 实时显示, 复用 V0.1 §4.6 PresenceCursor 升级 | per `frontend-canvas-design.md` §4.6 + 本 SRS A12.2 |
| **PresenceCursor** | V0.1 已有 (per `CanvasView.tsx` line 218-235 + `frontend-canvas-design.md` §4.6), 锚定在 canvas 上特定 element + 名字 + 正在操作什么; A12.2 升级为**多人**实时显示 | per V0.1 §4.6 + 本 SRS A12.2 |
| **元素增删改实时同步 (Element Realtime Sync)** | 1 用户拖拽 / 删除 / 修改 element, 其他用户 ≤ 200ms (P95) 看到, 走 WSS, 跟 V0.1 localStorage + zustand persist 冲突 (A12.3 需 backend 持久化) | per `frontend-canvas-design.md` §4.1 模式 A 扩展 + 本 SRS A12.3 |
| **Follow Mode** | A 用户跟随 B 用户视角, B 移动 viewport / 选中 element 时, A 同步, 类似 GitHub Live Share | 本 SRS A12.4 |
| **多人评论线程 (Multi-User Comment Thread)** | element 上挂载多人评论, 1 评论可被 N 用户回复 (thread 结构), V0.1 `comment_pin` (line 253-262) 升级 | per V0.1 `comment_pin` + 本 SRS A12.5 |
| **@ 提醒 (@ Mention)** | 评论 / 状态变化中 @ 用户名 → 自动发 notification 到该用户 inbox, 跟 25 module notification 域对接 (per `SRS-CANVAS-001.md` v1.1 §6.3) | 本 SRS A12.5 |
| **冲突解决 (Conflict Resolution) / CRDT** | 多用户同时改同 1 element 时的合并策略: **Yjs** (强 CRDT, 文本/JSON 友好) / **Automerge** (强 CRDT, JSON 友好) / **LWW** (Last-Write-Wins, 弱 CRDT, 简单); 拍板前 P0 阻塞 | per A11 同步桥同源 + 本 SRS A12.6 |
| **view / comment / edit 3 级权限** | 协作权限 3 等级: view (只读) / comment (评论) / edit (编辑), BFF API 层强制, 5 域 Lead 真人到位后决策具体矩阵 | 本 SRS A12.7 |
| **canvas_multi_user_audit** | A12.8 新增 audit log 表, 多人增删改 / cursor 移动 / 评论 / @ 全部 audit, **W/T/M 必含 Transaction 类, append-only, 物理删除禁止, SCD Type 2 (version +1), 100% RLS 13 类必携** (per 守门 #13) | per 守门 #13 + 本 SRS A12.8 |
| **V0.63 反转 (per 2026-09-10 17:34 JST)** | Ulysses 17:34 JST 拍板"**多人编辑是要的**", 撤回 17:08 JST 砍多人编辑决定; 本 SRS §4.12 A12 必含 8 项多人编辑; 引用 `frontend-canvas-design.md` §4.1 模式 A Realtime 通道 + §4.6 PresenceCursor 升级; 修订历史 v0.63 反转行**显式标** (per 守门 #1 禁回溯叙事) | per 17:34 JST 拍板 + 本 SRS §0.2 v1.2 + 总册 `SRS-CANVAS-001.md` v1.1 §0.2 |

---

## §3 業務背景 / 前提条件

### 3.1 业务背景

STAR 平台是 5 域 (player / economy / match / social / admin per 8/21 JST RGS 治理命名) 分布式系统, 已有 25 module + 6 状态机 + V0.1 无限画布 MVP + LangGraph 编排 + Agent Runtime 9/3 落档。

**画布能力现状** (V0.1 MVP, 2026-09-04 落档, per `frontend-canvas-design.md`):
- 14 element (work-item 2 + worktree 3 + agent 3 + feedback 2 + sticky 1 + automation 2 + text 1)
- 4 frame + 8 connector
- 6 e2e 守门 (pan / zoom / fit / highlight / delete / minimap)
- 3 e2e 守门 (share / export PNG)
- 5 联动 (WorkItem/Worktree/Relation/Comment/Search URL)
- **§4.1 模式 A Realtime 通道** (per 17:34 JST v0.63 反转) — 已有 NATS Subject `star.collaboration.canvas.element.*`, BFF 推 element 增删改
- **§4.6 PresenceCursor 升级** (per 17:34 JST v0.63 反转) — 已有, 升级为多人实时 cursor 同步
- **`comment_pin` (line 253-262)** (per 17:34 JST v0.63 反转) — 已有, 升级为多人评论线程 + @ 提醒
- 工具栏 + Minimap

**agent_cursor V0.1 实装** (per `CanvasView.tsx` line 218-235):
- 圆点 (圆心 1f6feb33 + 边框 2f81f7)
- 显示 `ag.id` + `ag.agent_kind` + `ag.status` 3 行文字
- 双击跳 `/agent?selected={id}` (per onElementDoubleClick line 132-143)
- **缺点**: 缺乏完整卡 (头像/role/token 用量/启动时间) + 拓扑图 + 状态实时同步 + 监控 + 操作菜单 + 聚类排序过滤 + 跨域引用 + 多人编辑

**双核心方向** (per 2026-09-10 17:08 JST Ulysses 拍板):
1. **管理 agent** (本 SRS 范围): 画布上**集中管理 agent 节点** (节点 / 拓扑 / 状态 / handoff / worktree 关联)
2. **游戏化** (转交 GAMIFY 专题): 画布上**游戏化机制** (节点 / 奖励 / 积分 / 等级 / 投票 / confetti)

**ARG 图论构造补充** (per 2026-09-10 17:21 JST Ulysses 补充):
- agent 之间的关系不是"展示标签", 而是真实的图论构造
- 关系影响协作 (dispatch / context / trust / review 4 维度)
- 5 个团队模板 1-click 部署
- 10 类关系 (4 核心 + 6 扩展) 覆盖人类组织常见关系

**A12 多人编辑补充** (per 2026-09-10 17:34 JST Ulysses 拍板"多人编辑是要的", v0.63 反转):
- 撤回 17:08 JST 砍多人编辑决定 (per 守门 #1 禁回溯叙事, 显式标 v0.63 反转)
- 多人同时编辑 (max 10 并发, max 200ms P95 延迟) 走 Realtime 通道 (per `frontend-canvas-design.md` §4.1 模式 A)
- 元素增删改实时同步 (A12.3) 跟 V0.1 localStorage + zustand persist 冲突, 需 backend 持久化
- 多人 cursor (A12.2) 升级 V0.1 §4.6 PresenceCursor
- 评论线程 (A12.5) 升级 V0.1 `comment_pin` line 253-262
- 冲突解决 (A12.6) 走 CRDT 选型 (Yjs vs Automerge vs LWW, 拍板前 P0 阻塞)
- 权限 (A12.7) 走 view / comment / edit 3 级, BFF API 层强制
- audit log (A12.8) 走新增表 `canvas_multi_user_audit`, W/T/M 必含 Transaction append-only + SCD Type 2 + 100% RLS 13 类 (per 守门 #13)

### 3.2 前提条件 (per 守门 #11 缺标比错标)

| # | 前提 | 影响 |
|---|---|---|
| **PR-1** | V0.1 MVP 已实装 (14 element + 4 frame + 8 connector + 9 e2e 守门) | 本 SRS 在 V0.1 基础上扩展, 不重构 |
| **PR-2** | `agent_cursor` V0.1 已实装 (per `CanvasView.tsx` line 218-235) | A1 扩展为 `agent_node` 完整卡, 保留 V0.1 跳转逻辑 (line 132-143) |
| **PR-3** | `frontend-canvas-design.md` v0.1 落地, 8 element kind + 3 routing connector + StatusPill 60+ 色码 + 5 联动 + **§4.1 模式 A Realtime 通道** + **§4.6 PresenceCursor** | 跨块接口 + 视觉规范基线; A12.1 + A12.3 扩展 §4.1; A12.2 扩展 §4.6 |
| **PR-4** | StatusPill 60+ 色码 (per `frontend-canvas-design.md` §3.4 + ADR-FE-013) | A1.2 / A3.1 / A11.1 关系色码必含, 联动一致性 |
| **PR-5** | 5×4 grid layout 算法 (per `frontend/src/lib/agent-view/layout.ts`) | A2 / A8 复用, 5 域分组 + 聚类布局 |
| **PR-6** | LangGraph 任务卡 DAG 已经在 TMO 9 节点 (M-N1..M-N7) 里实现, 9/4 落档 | ARG 跟 TMO 是平行层, 不重复 (per `SRS-AGENT-RELATIONSHIP-001.md` PR-1) |
| **PR-7** | Agent Runtime 已经有 9 个 SA Archetype (SA-01..SA-09), 9/3 落档 (per ADR-0045) | A11 节点类型直接复用 SA Archetype |
| **PR-8** | `SRS-AGENT-RELATIONSHIP-001.md` v0.1 (9/8 落档, 37KB) + `BD-AGENT-RELATIONSHIP-001.md` v0.1 (55KB, 9/9 落档) + `DD-AGENT-RELATIONSHIP-001.md` (94KB) + `DDD-REVIEW-AGENT-RELATIONSHIP-001.md` (30KB) | A11 必含, 本 SRS 仅画布集成, 不重写 ARG 逻辑 |
| **PR-9** | Memgraph 部署就绪 (per `SRS-AGENT-RELATIONSHIP-001.md` §3.2 PR-4, Docker 启动 port 7687 Bolt + 7444 HTTP) | A11 同步桥 UI 状态显示依赖 Memgraph |
| **PR-10** | **5 域 Lead 真人未到位**, Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D) | A2 / A6 / A11 / **A12.7 3 级权限决策** 延后, 真人到位后追溯签字覆盖修订历史, **不沿用代签决策** (per 守门 #1 禁回溯叙事) |
| **PR-11** | **5 域独立 Lead ≠ Star 22 DDD bounded context** (per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer) | 不建立业务子域↔DDD 映射, 5 域 Lead 是 RGS 仓历史治理命名, 本 SRS 引用为 frame 标题, 不建立子域↔DDD 映射 |
| **PR-12** | 守门 #13 W/T/M 三類横展开已经落地 (per AGENTS.md §4 守门 #13) | A11.5 关系 audit log 必含 4 表 W/T/M, **A12.8 多人编辑 audit log 必含新增表 `canvas_multi_user_audit` W/T/M Transaction append-only + SCD Type 2**, 100% 表覆盖 |
| **PR-13** | gm-console frontend 已有无限画布 (per `frontend-canvas-design.md` v0.1) | A11 关系编辑复用画布组件 + 加边编辑能力; **A12 多人编辑复用画布 + 加 Realtime 通道 (WSS)** |
| **PR-14** | 守门 #9 实证子代理 RPC 不可靠 (per AGENTS.md §4 #9 主体) | A11 同步桥用 in-process 推 + 周期 flush, 不用 RPC; **A12 多人编辑用 WSS (Realtime 通道) + in-process 缓存, 不用 RPC** |
| **PR-15** | agent_settings V0.1 已实装 (per `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1 + `AgentSettingsTab.tsx`) | A10 仅做画布集成, 不重写功能 |
| **PR-16** | ARG 4 表 W/T/M 分类已定 (per `SRS-AGENT-RELATIONSHIP-001.md` §7.1 + 守门 #13): `agents` (Master) + `agent_relationship_edges` (Master) + `agent_relationship_edges_audit` (Transaction) + `team_template_instances` (Work, TTL 30 天) | A11.5 必含完整 4 表 |
| **PR-17** | 守门 #1 v19 自动化档判定: P3-D 子项必先 `scripts/automation/<purpose>.py` 落地 | A11 / **A12** 跨 session 续做时强制 Python 化 (本 SRS 不实装, 仅文档) |
| **PR-18** | 守门 #5 env 安全: Memgraph 连接串走 env, 不打印 | A11 部署时遵循; **A12 WSS 连接串走 env, 不打印** |
| **PR-19** | 守门 #14 v4 反转 (per 2026-09-10 12:45 JST v0.62 反转, 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses) | 本 SRS 修订人 = Ulysses (Mavis 接手), 审批 = 架构师 (Mavis 接手) |
| **PR-20** | 守门 #23 v2 ai-edit-mode=本地 mock, 不引入第三方 LLM 凭据 | A11.10 协同 SRS 跟 ARG 同步, 调试控制台走 mock |
| **PR-21 (v1.2 新增)** | **V0.1 `comment_pin` 已实装** (per `CanvasView.tsx` line 253-262) | A12.5 多人评论线程扩展, 保留 V0.1 渲染 (圆心 #79c0ff33 + 边框 + 💬 emoji) |
| **PR-22 (v1.2 新增)** | **V0.1 §4.1 模式 A Realtime 通道 design 已落档** (per `frontend-canvas-design.md` §4.1) | A12.1 + A12.3 扩展实现模式 A (NATS Subject `star.collaboration.canvas.element.*`), 走 WSS, 不重新设计 |
| **PR-23 (v1.2 新增)** | **V0.1 §4.6 PresenceCursor 升级 design 已落档** (per `frontend-canvas-design.md` §4.6) | A12.2 复用 V0.1 PresenceCursor 字段 (cursor x/y + selection + user_id), 扩展为多人显示 (per cursor + name) |
| **PR-24 (v1.2 新增)** | **25 module notification 域已落档** (per 总册 `SRS-CANVAS-001.md` v1.1 §6.3) | A12.5 @ 提醒对接 notification 域, 走 `domain-notification` 现成 port |
| **PR-25 (v1.2 新增)** | **守门 #7 v3 派生 + 守门 #1 v26 cargo doc 改 advisory** (per 2026-09-05 00:15 JST 拍板) | A12 实装时 CI 守门同步反转 (per `frontend/.github/workflows/ci.yml` PR #12) |
| **PR-26 (v1.2 新增)** | **9/1 13:03 JST envoy 偏好 + 9/1 13:05 JST 独立 deployment 模式** | A12 多人编辑后端 BFF 走 envoy 独立 deployment (per `envoy` 偏好), 不选 nginx, 不选 istio sidecar |

### 3.3 业务规则 (Business Rules)

- **BR-1**: 1 个 agent 节点 = 1 个 agent_session_id, V0.1 `agent_cursor` 升级为 `agent_node` 不重复创建, 改用新 element kind
- **BR-2**: 1 agent 节点 (0..1) ↔ (1..N) worktree, 1 worktree 可被 N agent 共享 (per `SRS-AGENT-VIEW-001.md` BR-2)
- **BR-3**: 1 agent 节点 (0..N) ↔ (1..N) work_item, drag in/out 双向, 跟 `SRS-AGENT-VIEW-001.md` BR-3 一致
- **BR-4**: 14 状态机色码 走 StatusPill 60+, 跟 `frontend-canvas-design.md` §3.4 + ADR-FE-013 一致
- **BR-5**: agent 启停 / 重启操作 权限: SRE Lead / 5 域 Lead (per 守门 #3 5 域独立 Lead + 9/3 11:35 JST 拍板 B, 真人未到位前 Mavis 临时代签)
- **BR-6**: ARG 关系 4 核心 + 6 扩展 共 10 类, 颜色区分 (per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.1+4.1.2)
- **BR-7**: ARG 关系权重 0.0-1.0, 边粗细 + 颜色渐变 (灰→绿), 信任度动态: 成功 +0.01 / 失败 -0.05
- **BR-8**: ARG 关系改一次 version +1, optimistic lock, 物理删除禁止 (SCD Type 2)
- **BR-9**: 5 域 Frame 标题固定: `player` / `economy` / `match` / `social` / `admin` (per 8/21 JST RGS 治理命名)
- **BR-10**: 5 域 Lead 真人未到位前, 关系定义 / 跨域编排 决策 全部 Mavis 临时代签, 真人到位后追溯签字覆盖修订历史 (per 守门 #14 v2)
- **BR-11 (v1.2 新增)**: 多人同时编辑同一 canvas, max 10 并发用户 (per `frontend-canvas-design.md` §4.1 模式 A 限制), 元素增删改实时同步 ≤ 200ms (P95)
- **BR-12 (v1.2 新增)**: 多人编辑权限 3 等级 (view / comment / edit), BFF API 层强制 (per A12.7), 5 域 Lead 真人到位后决策具体权限矩阵, 拍板前走 view-only 兜底
- **BR-13 (v1.2 新增)**: A12.8 多人编辑 audit log 必含新增表 `canvas_multi_user_audit`, W/T/M 必含 **Transaction 类**, append-only + 物理删除禁止 + SCD Type 2 (version +1) + 100% RLS 13 类必携 (per 守门 #13)
- **BR-14 (v1.2 新增)**: 多人编辑元素增删改必走 backend 持久化, 跟 V0.1 localStorage + zustand persist 单用户模式冲突, A12.3 实施时 V0.1 持久化逻辑降级为"离线 fallback"
- **BR-15 (v1.2 新增)**: 多人 cursor 移动 + 评论 + @ 全部 audit (per A12.8), 1 操作 1 行 audit, 不批量

---

## §4 业务需求 (46 项 FR + NFR)

### 4.1 A1 agent 节点渲染 (3 项)

#### 4.1.1 FR-A1.1 `agent_cursor` 升级为 `agent_node` 完整卡

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A1.1 |
| 描述 | V0.1 `agent_cursor` (圆点 + 名字, `CanvasView.tsx` line 218-235) 升级为 `agent_node` 完整卡, 含 7 字段: 头像 (avatar 圆形 32x32) / 名字 (name) / role (supervisor / worker / reviewer) / kind (claude / gpt-4 / custom) / status (14 状态) / token 用量 (token_usage, 整数) / 启动时间 (started_at, ISO 8601) |
| 输入 | `agent_session_id` (UUID) |
| 输出 | SVG `<g>` element 完整卡, 元素大小 220x110 圆角矩形, 蓝底 (#161b22) + 蓝边框 (#30363d 默认, #2f81f7 hover, #79c0ff select) |
| 数据 schema 增项 | (V0.1 `agent_cursor` 已有) + 7 字段增项: `agent_session.avatar_url` (新增) + `agent_session.role` (新增, 枚举 supervisor / worker / reviewer) + `agent_session.kind` (V0.1 已有 `agent_kind` 字段, 复用) + `agent_session.token_usage` (V0.1 已有, 复用) + `agent_session.started_at` (V0.1 已有, 复用) |
| 接口依赖 | 复用 zustand store `useStore.getState().agentSessions` + 新增 `avatar_url` 字段 |
| 业务规则 | BR-1 (1 节点 = 1 session, 不重复) + BR-4 (状态色码走 StatusPill 60+) |
| 优先级 | P0 |

**用户故事**: US-1 (5 域 Lead 看到完整卡)

**验收标准**: AC-A1.1 — 画布上 agent 节点显示 7 字段 (avatar / name / role / kind / status / token_usage / started_at), 元素大小 220x110, 蓝底, 双击跳 `/agent?selected={id}`

**已知缺口**: 缺口 #1 (avatar 字段当前 store 缺, 待 DDD Review 加)

#### 4.1.2 FR-A1.2 14 状态机实时色码映射

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A1.2 |
| 描述 | `agent_node` 的 status 字段色码 走 StatusPill 60+, 14 状态机映射: queued=blue-dim / spawning=blue / initializing=blue / compiling_context=blue-dim / planning=blue / executing=green / awaiting_feedback=amber / awaiting_human=amber / awaiting_tool=amber / validating=blue-dim / paused=ink-mute / completed=green-mute / failed=red / cancelled=ink-mute |
| 数据 schema 增项 | (无新增, 复用 V0.1 StatusPill 60+ 色码 + ADR-FE-013) |
| 接口依赖 | 复用 `<StatusPill value={ag.status} size="xs" />` (per `CanvasView.tsx` line 213 / 195) |
| 业务规则 | BR-4 (StatusPill 60+ 一致性) |
| 优先级 | P0 |

**用户故事**: US-2 (5 域 Lead 实时色码同步)

**验收标准**: AC-A1.2 — 14 状态对应 14 色码, 跟 `SRS-AGENT-VIEW-001.md` §2 + `frontend-canvas-design.md` §3.4 StatusPill 一致

**已知缺口**: 无

#### 4.1.3 FR-A1.3 双击跳 agent 详情 (扩展 V0.1 联动 2)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A1.3 |
| 描述 | 双击 `agent_node` 跳 `/agent?selected={id}`, 保留 V0.1 `onElementDoubleClick` (line 132-143) 跳转逻辑, 扩展: 可选跳 `/agent-view?agent={id}` 走 AGENT-VIEW 协同视图 (per A9.1) |
| 输入 | `agent_node.element_id` |
| 输出 | `window.location.href = '/agent?selected={id}'` 或 `/agent-view?agent={id}` (per A9.1 配置) |
| 接口依赖 | 复用 V0.1 `onElementDoubleClick` 逻辑 |
| 业务规则 | BR-1 (1 节点 = 1 session) |
| 优先级 | P0 |

**用户故事**: US-1 (5 域 Lead 双击跳详情)

**验收标准**: AC-A1.3 — 双击 agent_node 跳 `/agent?selected={id}`, 配置 A9.1 后跳 `/agent-view?agent={id}`

**已知缺口**: 无

### 4.2 A2 agent 拓扑图 (4 项)

#### 4.2.1 FR-A2.1 1:N handoff connector (扩展 V0.1 `agent_handoff`)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A2.1 |
| 描述 | V0.1 `agent_handoff` connector 扩展, 含 3 字段: handoff 时间 (timestamp) / 状态 (handoff_status: pending / running / completed / failed) / 备注 (note, 1-500 字符) |
| 数据 schema 增项 | (V0.1 已有) + `CanvasConnector.handoff_status` (新增枚举) + `CanvasConnector.note` (新增 string) |
| 接口依赖 | 复用 V0.1 connector bezier 公式 (per `frontend-canvas-design.md` §3.5) |
| 业务规则 | BR-2 (1:N 关系) |
| 优先级 | P0 |

**用户故事**: US-3 (5 域 Lead 跨域 handoff 拓扑清晰)

**验收标准**: AC-A2.1 — agent handoff connector 显示时间 / 状态 / 备注 3 字段, bezier 曲线, 颜色按 handoff_status (running=blue / completed=green / failed=red / pending=ink-mute)

**已知缺口**: 无

#### 4.2.2 FR-A2.2 跨 5 域拓扑图 (5 域分组 Frame)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A2.2 |
| 描述 | 5 域 Frame 分组: `player` / `economy` / `match` / `social` / `admin` (per 8/21 JST RGS 治理命名 + BR-9), 域内 agent 自动聚类 (5×4 grid layout 复用, per `frontend/src/lib/agent-view/layout.ts`), 跨域 handoff 用 connector 跨 Frame 显示 |
| 输入 | `agent_sessions[]` (含 `agent_session.domain` 字段) |
| 输出 | 5 Frame + N 域内 agent_node (按 domain 自动聚类) + 跨域 connector |
| 数据 schema 增项 | `agent_session.domain` (新增枚举: player / economy / match / social / admin) |
| 接口依赖 | 复用 V0.1 Frame 渲染 (per `CanvasView.tsx` line 276-293) |
| 业务规则 | BR-9 (5 域固定标题) + BR-10 (Mavis 临时代签) |
| 优先级 | P0 |

**用户故事**: US-3 (5 域 Lead 看到 5 域分组)

**验收标准**: AC-A2.2 — 5 Frame 标题固定 player/economy/match/social/admin, 域内 agent 自动聚类, 跨域 handoff 用 connector 跨 Frame

**已知缺口**: 缺口 #2 (agent_session.domain 字段当前 store 缺, 待 DDD Review 加)

#### 4.2.3 FR-A2.3 父子 agent 关系 (1 parent → N child, supervisor → worker)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A2.3 |
| 描述 | 1 parent agent → N child agent 树形关系, supervisor 角色指向 worker 角色, 画布上用 树形 connector (orthogonal routing, per `frontend-canvas-design.md` §3.5) 显示 |
| 数据 schema 增项 | `agent_session.parent_session_id` (新增 UUID, nullable) |
| 接口依赖 | 复用 V0.1 connector orthogonal routing |
| 业务规则 | BR-2 (1:N 关系) |
| 优先级 | P1 |

**用户故事**: US-4 (5 域 Lead 看到父子关系树形布局)

**验收标准**: AC-A2.3 — 父子 agent 关系用 orthogonal connector 树形显示, parent 在上, child 在下, 多 child 自动展开

**已知缺口**: 缺口 #3 (parent_session_id 字段当前 store 缺, 待 DDD Review 加)

#### 4.2.4 FR-A2.4 agent pipeline 视图 (顺序执行链 A → B → C)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A2.4 |
| 描述 | N agent 顺序执行链 (A → B → C, 直线), 跟 worktree 关联 (1 pipeline 共享 1 worktree), 画布上 horizontal 布局 |
| 数据 schema 增项 | (V0.1 worktree_agent_relation 已有) + `worktree.pipeline_agent_ids[]` (新增, 顺序数组) |
| 接口依赖 | 复用 V0.1 connector straight routing (per `frontend-canvas-design.md` §3.5) |
| 业务规则 | BR-2 (1:N 关系) |
| 优先级 | P1 |

**用户故事**: US-5 (PM 看到 worktree 关联的 agent pipeline)

**验收标准**: AC-A2.4 — agent pipeline horizontal 显示, A → B → C 直线, 共享 worktree 高亮

**已知缺口**: 缺口 #4 (pipeline_agent_ids 字段当前 store 缺, 待 DDD Review 加)

### 4.3 A3 agent 状态实时同步 (3 项)

#### 4.3.1 FR-A3.1 14 状态机实时色码同步

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A3.1 |
| 描述 | 14 状态机变化实时同步到 `agent_node` 色码, 走 zustand store 订阅 (per V0.1 联动 3, `frontend-canvas-design.md` §4.4), WebSocket 选型待 P3-D 阶段拍板 (per §7 风险) |
| 接口依赖 | zustand store 订阅 + 未来 WebSocket (TODO: 选型) |
| 业务规则 | BR-4 (StatusPill 60+ 一致性) |
| 优先级 | P0 |

**用户故事**: US-2 (5 域 Lead 实时色码同步)

**验收标准**: AC-A3.1 — agent 状态变化 ≤ 200ms (P95) 反映到 `agent_node` 色码

**已知缺口**: 缺口 #5 (WebSocket 选型未拍板, 暂走 polling 30s fallback)

#### 4.3.2 FR-A3.2 状态变化触发 audit

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A3.2 |
| 描述 | agent 状态变化触发 audit 事件, action: `agent.status.change`, 字段: `agent_id` / `old_status` / `new_status` / `changed_at` / `changed_by` |
| 数据 schema 增项 | (无新增, 复用 V0.1 audit 表 per `SRS-AGENT-VIEW-001.md` §7.1) |
| 接口依赖 | audit 模块 API (V0.1 已有) |
| 优先级 | P0 |

**用户故事**: US-2 (5 域 Lead 状态变化 audit 追溯)

**验收标准**: AC-A3.2 — agent 状态变化必记 audit, 100% 覆盖, 可追溯

**已知缺口**: 无

#### 4.3.3 FR-A3.3 状态变化触发 notification (e.g. 失败 → @ 5 域 Lead)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A3.3 |
| 描述 | agent 状态变 failed 时, 自动发 notification 给所属域 Lead, 画布上 `agent_node` 红色边框 + 抖动动画 (CSS) |
| 数据 schema 增项 | (无新增, 复用 V0.1 notification 模块) |
| 接口依赖 | notification 模块 API (V0.1 已有) |
| 业务规则 | BR-10 (5 域 Lead 真人未到位前 Mavis 临时代签) |
| 优先级 | P1 |

**用户故事**: US-2 (5 域 Lead 失败通知)

**验收标准**: AC-A3.3 — agent 状态变 failed 时, 通知到所属域 Lead, 画布上 `agent_node` 红色边框 + 抖动动画 (≤ 500ms)

**已知缺口**: 缺口 #6 (5 域 Lead 真人未到位, 通知路由到 Mavis 临时代签邮箱)

### 4.4 A4 agent 关联 worktree (2 项)

#### 4.4.1 FR-A4.1 1 agent → N worktree 关联 (扩展 V0.1 §4.4)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A4.1 |
| 描述 | 1 agent 可关联 N worktree (V0.1 `worktree_node` 已有, 扩展支持 N), 画布上 agent_node 周围 圆周散开 N 个 worktree_node |
| 数据 schema 增项 | `worktree.agent_session_ids[]` (新增数组, 替代 V0.1 单值 `agent_session_id`) |
| 接口依赖 | 复用 V0.1 `worktree_node` 渲染 (per `CanvasView.tsx` line 200-217) |
| 业务规则 | BR-2 (1:N 关系) |
| 优先级 | P0 |

**用户故事**: US-5 (PM 看到 agent 关联的 worktree)

**验收标准**: AC-A4.1 — 1 agent 关联 N worktree 在画布上 圆周散开, 双向 connector, worktree 状态实时联动

**已知缺口**: 缺口 #4 (worktree.agent_session_ids[] 字段当前 store 缺, V0.1 仅 1:1)

#### 4.4.2 FR-A4.2 worktree status 变化 → agent_node 状态联动

(per v1.1, 引用 V0.1 §4.4 联动 3)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A4.2 |
| 描述 | worktree 状态变化 ≤ 200ms (P95) 反映到 agent_node 状态聚合 (badge: 关联 worktree 状态分布) |
| 接口依赖 | 复用 V0.1 zustand 订阅 (`frontend-canvas-design.md` §4.4) |
| 业务规则 | BR-2 + BR-4 |
| 优先级 | P0 |

**验收标准**: AC-A4.2 — worktree 状态变化 ≤ 200ms (P95) 反映到 agent_node 色码联动

**已知缺口**: 无

### 4.5 A5 agent 关联 work-item (2 项)

#### 4.5.1 FR-A5.1 1 agent → N work-item 关联 (扩展 V0.1 §4.2)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A5.1 |
| 描述 | 1 agent 可关联 N work-item, drag work-item 到 agent_node 周围, 弹"关联"确认 modal, 调 API 写 `work_item.agent_session_id`, 画布上双向 connector |
| 数据 schema 增项 | `work_item.agent_session_id` (新增 UUID, nullable, per `SRS-AGENT-VIEW-001.md` §10 缺口 #4) |
| 接口依赖 | 复用 V0.1 work_item_card 渲染 (per `CanvasView.tsx` line 180-199) + drag/drop |
| 业务规则 | BR-3 (1:N 关系) |
| 优先级 | P0 |

**验收标准**: AC-A5.1 — drag work-item 到 agent_node 周围, 1:N 关联, 双向 connector, work-item 状态实时联动

**已知缺口**: 缺口 #4 (work_item.agent_session_id 字段当前 store 缺, per `SRS-AGENT-VIEW-001.md` §10 缺口 #4)

#### 4.5.2 FR-A5.2 work-item 状态变化 → agent_node 状态联动

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A5.2 |
| 描述 | work-item 状态变化 ≤ 200ms (P95) 反映到 agent_node 状态聚合 (badge: in_progress 3 / done 5 / blocked 1) |
| 接口依赖 | 复用 V0.1 zustand 订阅 |
| 业务规则 | BR-3 + BR-4 |
| 优先级 | P0 |

**验收标准**: AC-A5.2 — work-item 状态变化 ≤ 200ms (P95) 反映到 agent_node 状态聚合

**已知缺口**: 无

### 4.6 A6 agent 操作菜单 (4 项)

#### 4.6.1 FR-A6.1 启停 (start / stop) 操作

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A6.1 |
| 描述 | 右键 agent_node → 操作菜单 → "启动" / "停止" 按钮, 调 agent-runtime API, 权限: SRE Lead / 5 域 Lead (per BR-5) |
| 接口依赖 | agent-runtime `/api/agent/start` + `/api/agent/stop` (per `SRS-STAR-AGENT-RUNTIME-001.md` §7) + BFF 权限校验 |
| 业务规则 | BR-5 + BR-10 (Mavis 临时代签) |
| 优先级 | P0 |

**验收标准**: AC-A6.1 — 右键 agent_node 弹菜单, 启停操作 权限校验 + 调用 API + 状态实时反映 (≤ 1s)

#### 4.6.2 FR-A6.2 重启 (restart) 操作

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A6.2 |
| 描述 | 右键菜单 → "重启" 按钮, 调 agent-runtime API, 状态从 running → spawning → initializing 反映 |
| 接口依赖 | agent-runtime `/api/agent/restart` |
| 优先级 | P1 |

**验收标准**: AC-A6.2 — 右键菜单 → "重启" 操作成功, 状态从 running → spawning → initializing 反映

#### 4.6.3 FR-A6.3 查看 logs (跳 agent-runtime logs)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A6.3 |
| 描述 | 右键菜单 → "查看 logs" 跳 `/agent-runtime/logs?session={id}` |
| 接口依赖 | agent-runtime logs page (V0.1 已有) |
| 优先级 | P1 |

**验收标准**: AC-A6.3 — 右键菜单 → "查看 logs" 跳 agent-runtime logs 页面

#### 4.6.4 FR-A6.4 查看 settings (跳 V0.1 agent-settings)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A6.4 |
| 描述 | 右键菜单 → "查看 settings" 跳 `/agent-settings?selected={id}` (V0.1 `AgentSettingsTab.tsx` 已实装) |
| 接口依赖 | V0.1 settings page (per `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1) |
| 优先级 | P1 |

**验收标准**: AC-A6.4 — 右键菜单 → "查看 settings" 跳 V0.1 `/agent-settings?selected={id}`

### 4.7 A7 agent 监控面板 (3 项)

#### 4.7.1 FR-A7.1 实时 status / token / cost / runtime 仪表

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A7.1 |
| 描述 | 点击 agent_node 弹 detail panel, 4 字段实时仪表: status (14 状态机) + token_usage (整数) + cost (USD) + runtime (累计秒) |
| 接口依赖 | zustand 订阅 + agent-runtime API |
| 优先级 | P1 |

**验收标准**: AC-A7.1 — 点击 agent_node 弹 detail panel, 4 字段实时 (≤ 1s 刷新), 走 StatusPill 60+ 色码

#### 4.7.2 FR-A7.2 token 用量对比 budget

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A7.2 |
| 描述 | agent node badge 显示 `已用 / 预算` 比例, 默认 budget 1.2M / SRE·周 (per `STAR-OLU-001.md` v0.1), 超 100% 红色高亮 |
| 数据 schema 增项 | `agent_session.token_budget` (新增 int, 默认 1.2M) |
| 优先级 | P1 |

**验收标准**: AC-A7.2 — agent node badge 显示 `已用 / 预算` 比例, 超 100% 红色高亮

**已知缺口**: 缺口 #7 (token_budget 字段当前 store 缺, V0.1 仅 token_usage)

#### 4.7.3 FR-A7.3 异常告警 (token 超预算 / runtime 异常)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A7.3 |
| 描述 | token 超预算 / runtime 异常 → 通知 (per A3.3) + 画布上 agent_node 高亮 (≤ 500ms) |
| 接口依赖 | notification 模块 |
| 优先级 | P1 |

**验收标准**: AC-A7.3 — token 超预算 / runtime 异常 → 通知 + 画布高亮 (≤ 500ms)

### 4.8 A8 agent 聚类 / 排序 / 过滤 (3 项)

#### 4.8.1 FR-A8.1 按 role 聚类

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A8.1 |
| 描述 | 顶部 dropdown "聚类: role" 选项, 画布上同 role agent 用 Frame 子分组 (supervisor / worker / reviewer) |
| 优先级 | P1 |

**验收标准**: AC-A8.1 — 顶部 dropdown "聚类: role" 选项, 画布上同 role agent 用 Frame 子分组

#### 4.8.2 FR-A8.2 按 kind 排序

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A8.2 |
| 描述 | 顶部 dropdown "排序: kind" 选项, 画布上 agent 按 kind ASC 稳定排序 [kind ASC, started_at ASC, id ASC] |
| 优先级 | P1 |

**验收标准**: AC-A8.2 — 顶部 dropdown "排序: kind" 选项, 画布上 agent 按 kind ASC 稳定排序

#### 4.8.3 FR-A8.3 按 status / token / started_at 过滤

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A8.3 |
| 描述 | 顶部 dropdown "过滤: status/token/started_at" 多选, 画布上只显示符合条件 agent |
| 优先级 | P1 |

**验收标准**: AC-A8.3 — 顶部 dropdown "过滤: status/token/started_at" 多选, 画布上只显示符合条件 agent

### 4.9 A9 agent session 跨域引用 (2 项)

#### 4.9.1 FR-A9.1 跟 SRS-AGENT-VIEW-001 协同

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A9.1 |
| 描述 | 双击 agent_node 配置走 `/agent-view?agent={id}`, 双向跳成功 (per `SRS-AGENT-VIEW-001.md` §1.3) |
| 优先级 | P2 |

**验收标准**: AC-A9.1 — 双击 agent_node 配置走 `/agent-view?agent={id}`, 双向跳成功

#### 4.9.2 FR-A9.2 跟 SRS-AGENT-RELATIONSHIP-001 协同

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A9.2 |
| 描述 | 双击 agent_node → 切 tab 跳 `/agent-relationships?agent={id}`, 显示该 agent 所有关系 (5 关系) |
| 优先级 | P2 |

**验收标准**: AC-A9.2 — 双击 agent_node → 切 tab 跳 `/agent-relationships?agent={id}`, 显示该 agent 所有关系

### 4.10 A10 agent settings 集成 (2 项)

#### 4.10.1 FR-A10.1 agent-settings tab 集成

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A10.1 |
| 描述 | 画布 detail panel 顶部 "Settings" tab, 集成 V0.1 `AgentSettingsTab.tsx`, 改 role → 画布自动重聚类 |
| 接口依赖 | V0.1 settings (per `PHASE-AGENT-SETTINGS-IMPL-REPORT.md` v0.1) |
| 优先级 | P2 |

**验收标准**: AC-A10.1 — 画布 detail panel 顶部 "Settings" tab, 集成 V0.1 `AgentSettingsTab.tsx`, 改 role → 画布自动重聚类

#### 4.10.2 FR-A10.2 画布操作调用 settings

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A10.2 |
| 描述 | 画布右键 → "编辑 settings" 改 role / kind / token_budget, 实时反映到画布聚类 / 仪表 |
| 优先级 | P2 |

**验收标准**: AC-A10.2 — 画布右键 → "编辑 settings" 改 role / kind / token_budget, 实时反映到画布聚类 / 仪表

### 4.11 A11 ARG 图论构造 (10 项, per 17:21 JST Ulysses 补充)

#### 4.11.1 FR-A11.1 10 类关系边渲染

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.1 |
| 描述 | 画布上渲染 10 类 ARG 关系边 (per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.1+4.1.2):<br>**4 核心**: `delegates_to` (A → B, 蓝) / `consults` (A → B, 青) / `collaborates_with` (A ↔ B, 紫, 无向) / `reports_to` (A → B, 橙)<br>**6 扩展**: `mentors` (A → B, 深绿) / `peer_reviews` (A ↔ B, 浅绿, 无向) / `stand_in_for` (A → B, 黄) / `shadows` (A → B, 灰) / `challenges` (A → B, 红) / `trusts` (A → B, 浅蓝) |
| 数据 schema 增项 | `arg_edge` element kind (新增 per `BD-AGENT-RELATIONSHIP-001.md` §5.2) |
| 接口依赖 | Memgraph `agents` + `agent_relationship_edges` (per `SRS-AGENT-RELATIONSHIP-001.md` §7.1) |
| 业务规则 | BR-6 (10 类关系颜色区分) + BR-7 (权重 0.0-1.0) |
| 优先级 | P0 |

**用户故事**: US-11 + US-23 + US-24 (5 域 Lead 看到 10 类关系)

**验收标准**: AC-A11.1 — 10 类关系边在画布上正确渲染, 颜色区分清晰, 边箭头方向按 direction (directed/undirected), 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.1.1+4.1.2 颜色规范一致

**已知缺口**: 缺口 #9 (`arg_edge` element kind V0.1 schema 缺, 需新增)

#### 4.11.2 FR-A11.2 关系编辑 (UI 拖拽建边 + type 选择 + weight 滑块 + metadata JSON)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.2 |
| 描述 | 画布上拖拽 2 agent → 弹"关系编辑" modal, 3 字段: type (10 类 dropdown) + weight (0.0-1.0 滑块, 默认 0.5) + metadata (JSON 字段, 1-2KB), 提交后调 `/api/arg/edges` POST (per `SRS-AGENT-RELATIONSHIP-001.md` §7.2) |
| 数据 schema 增项 | (无新增, 复用 ARG schema per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.3) |
| 接口依赖 | POST `/api/arg/edges` + Memgraph write + EventBus edge.changed |
| 业务规则 | BR-6 + BR-7 + BR-8 (version +1) |
| 优先级 | P0 |

**用户故事**: US-12 (PM 拖拽建关系)

**验收标准**: AC-A11.2 — 拖拽 2 agent → 弹 modal → 填 3 字段 → 提交 → Memgraph 持久化 + 画布实时反映 (≤ 500ms)

**已知缺口**: 缺口 #9 + 缺口 #10 (ARG schema V0.1 已落档, 实装待 P3-C 阶段)

#### 4.11.3 FR-A11.3 4 维度协作影响 UI 指示器

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.3 |
| 描述 | 画布上为每条关系边显示 **4 维度协作影响 UI 指示器** (per `SRS-AGENT-RELATIONSHIP-001.md` §4.3):<br>1. **Dispatch 路由** (delegates_to / reports_to / stand_in_for) — 边 label `→ B 自动 dispatch`, 点击查看 dispatch 命中次数<br>2. **上下文共享** (mentors / shadows) — 边 label `mentor 历史决策已注入`, 节省 token 比例<br>3. **信任度加权** (trusts / peer_reviews) — 边 label `跳过 verify`, trust_score 0.0-1.0<br>4. **产出评估** (challenges / peer_reviews) — 边 label `双向 review`, review 通过率 |
| 数据 schema 增项 | (无新增, 复用 ARG `agent_relationship_edges_audit` + `relationship_events` per `SRS-AGENT-RELATIONSHIP-001.md` §7.1) |
| 接口依赖 | Memgraph 读 + effect tier 4 模块 (`ARGDispatchRouter` / `ARGContextInjector` / `ARGTrustEngine` / `ARGOutputEvaluator` per `BD-AGENT-RELATIONSHIP-001.md` §2.2 Tier 5) |
| 业务规则 | 关系真实影响协作 (per 用户原话"对它们的工作产生益处", `SRS-AGENT-RELATIONSHIP-001.md` §4.3) |
| 优先级 | P0 |

**用户故事**: US-13 + US-21 (5 域 Lead 看到 4 维度影响 + 关系真实生效)

**验收标准**: AC-A11.3 — 4 维度指示器在边 label 上正确显示, 数值走 Memgraph 实时读 (≤ 1s), 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.3 4 维度一致

**已知缺口**: 缺口 #10 (4 维度 effect tier 模块待 P3-C 阶段实装)

#### 4.11.4 FR-A11.4 5 团队模板 1-click 部署

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.4 |
| 描述 | 画布顶部 toolbar "模板" dropdown, 5 团队模板 1-click 部署 (per `SRS-AGENT-RELATIONSHIP-001.md` §4.5):<br>1. **Hub-and-Spoke** — 1 Lead + 4 Worker (4 条 `delegates_to`)<br>2. **Mesh** — N 节点全连接 (`N×(N-1)/2` 条 `collaborates_with`)<br>3. **Chain** — A → B → C → D (3 条 `delegates_to`)<br>4. **Hierarchical** — 1 Lead → 2 Sub-Lead → 6 Worker (9 节点树形)<br>5. **Review-Council** — 3 Reviewer ← 1 Lead (1 deleg + 3 consults)<br>用户点模板 → 弹 agent 选择器 (N 个 dropdown) → 1-click 实例化, 调 `/api/arg/templates/instantiate` (1 事务 Cypher) |
| 数据 schema 增项 | `team_template_instances` (Work, TTL 30 天, per `SRS-AGENT-RELATIONSHIP-001.md` §7.1) |
| 接口依赖 | POST `/api/arg/templates/instantiate` (per `SRS-AGENT-RELATIONSHIP-001.md` §7.2) |
| 业务规则 | BR-10 (5 域 Lead 真人未到位前 Mavis 临时代签, 模板实例化由 Mavis 落) |
| 优先级 | P1 |

**用户故事**: US-14 + US-25 (Dev / SRE 1-click 部署 5 模板)

**验收标准**: AC-A11.4 — 5 模板 dropdown 可见, 1-click 实例化成功, 1 事务写 Memgraph, 画布实时反映 (≤ 1s), 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.5 + UC-05 一致

**已知缺口**: 缺口 #10 (ARG 模板 ops 实装待 P3-C 阶段)

#### 4.11.5 FR-A11.5 关系 audit log (per 守门 #13 W/T/M 三類横展)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.5 |
| 描述 | 关系增删改 100% audit log 记录, 走 `agent_relationship_edges_audit` (Transaction, append-only), 改关系 version +1 (SCD Type 2, optimistic lock), 必含 4 表 W/T/M 三類横展 (per 守门 #13):<br>- **`agents`** (Master, SCD Type 2) — 节点元数据<br>- **`agent_relationship_edges`** (Master, SCD Type 2) — 边定义<br>- **`agent_relationship_edges_audit`** (Transaction, append-only, 100% audit) — 改关系必 +1 行<br>- **`team_template_instances`** (Work, TTL 30 天, 短 TTL 作业中) — 模板实例化中间态 |
| 数据 schema 增项 | (4 表已在 `SRS-AGENT-RELATIONSHIP-001.md` §7.1 落档) |
| 接口依赖 | Memgraph write + Transaction 双写 (per `SRS-AGENT-RELATIONSHIP-001.md` NFR-ARG-RELIABILITY-01) |
| 业务规则 | BR-8 (version +1) + 守门 #13 (W/T/M 三類) + 守门 #13 (Master 100% RLS / Transaction 100% audit / Work 100% retention_period) |
| 优先级 | P0 |

**用户故事**: US-15 (5 域 Lead 关系 audit log 追溯)

**验收标准**: AC-A11.5 — 关系增删改 100% audit, version +1, 4 表 W/T/M 100% 覆盖 (per 守门 #13), 改关系可回滚 (per audit log)

**已知缺口**: 无 (4 表 W/T/M 分类已定, per `SRS-AGENT-RELATIONSHIP-001.md` §7.1)

#### 4.11.6 FR-A11.6 关系权重可视化 (边粗细 0.0-1.0 + 颜色渐变灰→绿)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.6 |
| 描述 | 关系边粗细按 weight 0.0-1.0 缩放 (1px → 6px), 颜色渐变 灰 #6e7681 (0.0) → 绿 #3fb950 (1.0), 信任度动态: 成功 +0.01 / 失败 -0.05 (per `SRS-AGENT-RELATIONSHIP-001.md` §4.3.3 + UC-07) |
| 数据 schema 增项 | (无新增, 复用 `agent_relationship_edges.weight` 字段) |
| 接口依赖 | Memgraph 读 + `ARGTrustEngine` (per `BD-AGENT-RELATIONSHIP-001.md` §2.2 Tier 5) |
| 业务规则 | BR-7 (权重 0.0-1.0) |
| 优先级 | P1 |

**用户故事**: US-16 (Dev 关系权重视觉化)

**验收标准**: AC-A11.6 — 边粗细按 weight 缩放, 颜色渐变灰→绿, 信任度动态调整 weight (成功 +0.01 / 失败 -0.05), 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.3.3 一致

**已知缺口**: 缺口 #10 (ARG Trust Engine 实装待 P3-C 阶段)

#### 4.11.7 FR-A11.7 关系 archive / restore

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.7 |
| 描述 | 关系右键菜单 → "归档" / "恢复" 按钮, 归档关系 (`archived=true`) 不影响协作 (per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.3 `archived` 字段), 恢复改 `archived=false`, 物理删除禁止 (per 守门 #13 Master SCD Type 2) |
| 数据 schema 增项 | (无新增, 复用 `agent_relationship_edges.archived` 字段, 已在 `SRS-AGENT-RELATIONSHIP-001.md` §4.1.3 落档) |
| 接口依赖 | PATCH `/api/arg/edges/{id}` + `DELETE` (实际是软删 archive) |
| 业务规则 | BR-7 + 守门 #13 (物理删除禁止) |
| 优先级 | P2 |

**用户故事**: US-17 + US-22 (PM / 5 域 Lead archive 关系)

**验收标准**: AC-A11.7 — 关系右键菜单 → "归档" 改 `archived=true`, 画布上变灰半透明, 不影响协作; "恢复" 改回 `archived=false`, 物理删除 0 次, 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.1.3 一致

**已知缺口**: 无

#### 4.11.8 FR-A11.8 关系版本控制 (SCD Type 2)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.8 |
| 描述 | 关系改一次 `version` 自动 +1 (per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.3), optimistic lock (冲突时报错 + UI 提示), 改关系可回滚 (per audit log) |
| 数据 schema 增项 | (无新增, 复用 `agent_relationship_edges.version` 字段, 已在 `SRS-AGENT-RELATIONSHIP-001.md` §4.1.3 落档) |
| 接口依赖 | PATCH `/api/arg/edges/{id}` + Memgraph write + EventBus edge.changed |
| 业务规则 | BR-8 (version +1, SCD Type 2) + 守门 #13 (Master SCD Type 2) |
| 优先级 | P2 |

**用户故事**: US-18 (5 域 Lead 改关系 version 控制)

**验收标准**: AC-A11.8 — 关系改一次 version 自动 +1, 并发冲突 UI 提示 + 不覆盖旧版本, 改关系可回滚 (per audit log), 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.4 冲突解决一致

**已知缺口**: 无

#### 4.11.9 FR-A11.9 同步桥 UI 状态显示 (Memgraph ↔ LangGraph StateGraph)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.9 |
| 描述 | 画布右上角 status badge 显示 ARG 同步桥状态 (per `SRS-AGENT-RELATIONSHIP-001.md` §4.4 + UC-01):<br>3 字段: `sync_status` (synced / syncing / error / offline) + `last_sync_at` (ISO 8601) + `sync_error` (error 时显示)<br>4 状态: synced (绿色) / syncing (蓝色) / error (红色 + tooltip) / offline (灰色 + 降级提示) |
| 数据 schema 增项 | `arg_sync_status` (新增 enum) + `arg_last_sync_at` (新增 timestamp) + `arg_sync_error` (新增 string nullable) |
| 接口依赖 | Memgraph Bolt subscription + Period flush 30s (per `SRS-AGENT-RELATIONSHIP-001.md` §4.4) + 离线降级 (in-process 缓存) |
| 业务规则 | 守门 #9 (子代理 RPC 不可靠) — 同步桥用 in-process 推 + 周期 flush, 不用 RPC |
| 优先级 | P1 |

**用户故事**: US-19 (SRE 看到同步桥状态)

**验收标准**: AC-A11.9 — 画布右上角 status badge 4 状态正确显示, 离线降级 走 in-process 缓存, 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.4 + UC-01 一致

**已知缺口**: 缺口 #10 (ARG 同步桥实装待 P3-C 阶段)

#### 4.11.10 FR-A11.10 跟 SRS-AGENT-RELATIONSHIP-001 协同 (Agent View "Relationship" tab)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A11.10 |
| 描述 | 画布切到 "Relationship" tab → 跳 ARG 全图视图 (per `SRS-AGENT-RELATIONSHIP-001.md` §1.3), 跟 3 个 ARG UI 组件协同: Agent Relationship Editor (画布拖拽建关系) + Relationship View (图谱浏览) + Achievement Wall (成就墙), 3 组件都引用本 SRS 画布 `agent_node` 数据 |
| 接口依赖 | `/agent-relationships` 路由 + 3 ARG UI 组件 (per `BD-AGENT-RELATIONSHIP-001.md` §2.2 Tier 1) |
| 业务规则 | 跨块接口 (per 守门 #14 v2 跨域编排) |
| 优先级 | P1 |

**用户故事**: US-20 (PM 切到 Relationship tab 协同 ARG)

**验收标准**: AC-A11.10 — 画布 "Relationship" tab 切换成功, 跳 `/agent-relationships?from=canvas`, 3 ARG UI 组件跟本画布 `agent_node` 数据双向同步

**已知缺口**: 缺口 #8 (`/agent-relationships` 路由待 ARG UI 实装阶段落档)

### 4.12 A12 多人编辑 (8 项, per 17:34 JST Ulysses 拍板"多人编辑是要的", v0.63 反转, 撤回 17:08 JST 砍多人编辑决定)

> **v0.63 反转声明** (per 2026-09-10 17:34 JST Ulysses 拍板"多人编辑是要的"):
> 1. 撤回 2026-09-10 17:08 JST 砍多人编辑决定 (per `SRS-CANVAS-001.md` v1.0 §1.4 不包含范围"多人同时编辑 / 实时 cursor / Follow mode / 评论线程 / @ 提醒")
> 2. 本 SRS §1.4 已更新 — **多人编辑撤回砍掉决定**, §4.12 必含 8 项
> 3. 引用 `frontend-canvas-design.md` §4.1 模式 A Realtime 通道 (A12.1 + A12.3) + §4.6 PresenceCursor 升级 (A12.2) + `CanvasView.tsx` line 253-262 `comment_pin` (A12.5)
> 4. A12.8 必含新增表 `canvas_multi_user_audit` W/T/M 必含 Transaction append-only + SCD Type 2 + 100% RLS 13 类 (per 守门 #13)
> 5. per 守门 #1 禁回溯叙事: v0.63 反转行**显式标** (本段 + §0.2 v1.2 + §12 修订历史)

#### 4.12.1 FR-A12.1 多人同时编辑同一 canvas (实时同步)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A12.1 |
| 描述 | N 个用户 (max 10 并发) 同时编辑同一 canvas, 元素增删改实时同步 ≤ 200ms (P95), 走 WSS (WebSocket Secure) Realtime 通道 (扩展 V0.1 §4.1 模式 A, NATS Subject `star.collaboration.canvas.element.*`), BFF 推 element 增删改 → 所有浏览同一 canvas 的用户实时看到 |
| 输入 | 用户在画布上的 element 增删改操作 (V0.1 7 种 element kind) |
| 输出 | WSS 广播 → 其他用户 ≤ 200ms 看到 element 变化 |
| 数据 schema 增项 | `canvas.element_change_event` (新增) + WSS topic `wss://canvas-collab/canvases/[id]` (扩展 V0.1 §4.1 模式 A) |
| 接口依赖 | WSS Realtime 通道 (per `frontend-canvas-design.md` §4.1 模式 A) + BFF `/v1/collaboration/canvases/[id]/elements` (CRUD) + **envoy 独立 deployment** (per 9/1 13:03 JST 偏好 + 9/1 13:05 JST 独立 deployment 模式, 不选 nginx / istio sidecar) |
| 业务规则 | BR-11 (max 10 并发 + 200ms P95 延迟) + BR-14 (backend 持久化, 跟 V0.1 localStorage 冲突, 降级为离线 fallback) + BR-15 (1 操作 1 audit 行) |
| 优先级 | P0 |

**用户故事**: US-26 (PM / SRE / 5 域 Lead 多人同时编辑)

**验收标准**: AC-A12.1 — 10 个用户同时编辑同一 canvas, 元素增删改 ≤ 200ms (P95) 反映到其他用户, WSS 连接稳定, envoy 独立 deployment, BFF 推 element 增删改, 跟 `frontend-canvas-design.md` §4.1 模式 A + §6.2 端点 `wss://canvas-collab/canvases/[id]` 一致

**已知缺口**: 缺口 #11 (A12.1 多人同时编辑 WebSocket 选型未拍板, 跟 A3 + A11 同步桥同源, P0 阻塞; 候选 NATS JetStream / native WebSocket / Socket.IO)

#### 4.12.2 FR-A12.2 实时 cursor 同步 (PresenceCursor 升级, 多人)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A12.2 |
| 描述 | 其他用户光标位置 (x, y 世界坐标) + 名字 (display_name) + 当前 viewport (pan + zoom) + 当前选中 element_ids 实时显示, 走 WSS PresenceCursor 通道, 复用 V0.1 §4.6 PresenceCursor 字段 (cursor x/y + selection + user_id) 升级为**多人**实时显示, 每个用户一个 cursor 颜色 (per V0.1 12 色调色板) |
| 输入 | 用户 mouse move + viewport 变化 + element 选中 |
| 输出 | WSS 广播 → 其他用户 ≤ 100ms 看到 cursor 移动 |
| 数据 schema 增项 | (无新增, 复用 V0.1 PresenceCursor 字段) + `presence_cursors` (新增, per WSS topic) |
| 接口依赖 | WSS PresenceCursor 通道 (per `frontend-canvas-design.md` §4.6 升级) + 端点 `wss://canvas-presence/canvases/[id]` (per 总册 `SRS-CANVAS-001.md` v1.1 §6.2.2) |
| 业务规则 | BR-15 (1 操作 1 audit 行, cursor 移动 throttle ≤ 50ms 1 audit 避免刷屏) + **复用 A11.9 同步桥 UI 状态显示模式** (per brief §1.3 A12.2 引用) |
| 优先级 | P0 |

**用户故事**: US-27 (5 域 Lead 看到 SRE 在看哪个 agent_node)

**验收标准**: AC-A12.2 — 10 个用户同时编辑, 每人 cursor ≤ 100ms 显示在其他用户 viewport 上, cursor 颜色按 user_id 哈希分配 (12 色调色板), 名字 + viewport + selection 同步, 跟 V0.1 §4.6 PresenceCursor 字段 + 总册 §6.2.2 端点一致

**已知缺口**: 缺口 #12 (PresenceCursor V0.1 design 已落档, A12.2 实装扩展待 P3-C/D 阶段)

#### 4.12.3 FR-A12.3 元素增删改实时同步 (V0.1 §4.1 模式 A 扩展)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A12.3 |
| 描述 | 1 用户拖拽 / 删除 / 修改 element, 其他用户 ≤ 200ms (P95) 看到 element 增删改, 走 WSS, **跟 V0.1 localStorage + zustand persist 冲突** (A12.3 实施时 V0.1 持久化逻辑降级为"离线 fallback"), 需 backend 持久化 (BFF `/v1/collaboration/canvases/[id]/elements` POST/PATCH/DELETE), 跟 V0.1 §4.1 模式 A Realtime 通道扩展 |
| 输入 | element create / update / delete event |
| 输出 | WSS 广播 → 其他用户 ≤ 200ms 看到 element 增删改 + backend 持久化 |
| 数据 schema 增项 | `canvas.elements` 表 (新增, backend 持久化) + WSS topic `wss://canvas-collab/canvases/[id]/elements` (per 总册 §6.2.2) |
| 接口依赖 | BFF `/v1/collaboration/canvases/[id]/elements` (POST/PATCH/DELETE) + WSS Realtime 通道 + **envoy 独立 deployment 路由** (per 9/1 13:03 JST 偏好) |
| 业务规则 | BR-11 (≤ 200ms P95) + BR-14 (backend 持久化优先, V0.1 localStorage 降级为离线 fallback) + BR-15 (1 操作 1 audit) |
| 优先级 | P0 |

**用户故事**: US-26 (PM 多人拖拽 element 实时同步)

**验收标准**: AC-A12.3 — 1 用户拖拽 element, 其他用户 ≤ 200ms (P95) 看到 element 移动, backend 持久化成功, F5 刷新保留, V0.1 localStorage 不冲突 (降级为 fallback), 跟 V0.1 §4.1 模式 A + 总册 §6.2.2 端点一致

**已知缺口**: 缺口 #11 + 缺口 #13 (V0.1 localStorage + zustand persist 跟多人编辑冲突, A12.3 实施时需重构持久化层, P0)

#### 4.12.4 FR-A12.4 Follow mode (A 用户跟随 B 用户视角)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A12.4 |
| 描述 | A 用户跟随 B 用户视角, B 移动 viewport (pan + zoom) / 选中 element 时, A 同步 (类似 GitHub Live Share), 1 canvas 仅支持 1 个 follower (per session), A 端顶部 toolbar 显示 "Following: {B user_name}", 退出 follow = A 主动点 "Stop Following" |
| 输入 | A 用户点 B 用户名字 → "Follow" 按钮 |
| 输出 | A 端 viewport 跟 B 端同步, B 端不动 A |
| 数据 schema 增项 | `canvas_followers` (新增, per canvas + follower_user_id + leader_user_id + started_at) |
| 接口依赖 | POST `/v1/collaboration/canvases/[id]/follow?user_id=` (per 总册 §6.2.2) + WSS viewport 同步 |
| 业务规则 | BR-12 (跟权限联动, 至少 view 权限) + 1 canvas 1 follower (避免性能问题) |
| 优先级 | P1 |

**用户故事**: US-28 (PM 跟随 SRE 视角)

**验收标准**: AC-A12.4 — A 端点 B 端 "Follow" 按钮, A 端 viewport 实时跟 B 端同步 (≤ 100ms), 顶部 toolbar 显示 "Following: {B user_name}", "Stop Following" 按钮可退出, 跟总册 §6.2.2 端点 + GitHub Live Share UX 一致

**已知缺口**: 无 (P1, 实装待 P3-D 阶段)

#### 4.12.5 FR-A12.5 多人评论线程 + @ 提醒 (V0.1 `comment_pin` 升级)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A12.5 |
| 描述 | element 上挂载多人评论线程 (V0.1 `comment_pin` line 253-262 升级, 加 thread 结构: 1 顶级评论可被 N 用户回复), `@` 提醒 (e.g. `@pm-ulysses` → 自动发 notification 到该用户), 跟 25 module notification 域对接 (per 总册 `SRS-CANVAS-001.md` v1.1 §6.3), V0.1 渲染保留 (圆心 #79c0ff33 + 边框 + 💬 emoji + 评论数 badge) |
| 输入 | 用户右键 element → "添加评论" / 在现有 thread 上回复 / @ 用户 |
| 输出 | 评论 thread 显示在 element 旁边 / notification 域发到被 @ 用户 inbox |
| 数据 schema 增项 | `canvas_comments` (新增, per thread 结构) + `canvas_comment_mentions` (新增, @ mention 关联) |
| 接口依赖 | POST `/v1/collaboration/canvases/[id]/comments` (per 总册 §6.2.2) + POST `/v1/notifications/@` (跟 25 module notification 域对接) + 复用 V0.1 `comment_pin` 渲染 (per `CanvasView.tsx` line 253-262) |
| 业务规则 | BR-12 (view 权限用户仅可看, comment 权限可评论, edit 权限可删) + BR-15 (1 评论 1 audit) |
| 优先级 | P0 |

**用户故事**: US-29 (Dev 添加评论 + @ PM 审核)

**验收标准**: AC-A12.5 — element 上右键 → "添加评论" → 弹 thread 输入框 → 提交 → 评论显示在 element 旁边, V0.1 💬 emoji + badge 保留; thread 内回复 + @ 用户 → notification 域发到被 @ 用户 inbox, 跟 V0.1 `CanvasView.tsx` line 253-262 `comment_pin` 升级一致 + 总册 §6.2.2 端点 + 25 module notification 对接

**已知缺口**: 缺口 #14 (V0.1 `comment_pin` 渲染保留, thread + @ 数据结构 + 25 module notification 域对接实装待 P3-C 阶段)

#### 4.12.6 FR-A12.6 冲突解决 (CRDT 选型, Yjs vs Automerge vs LWW)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A12.6 |
| 描述 | 多用户同时改同 1 element 时的合并策略, 3 候选 CRDT 实现:<br>1. **Yjs** (强 CRDT, 文本/JSON 友好, 性能好, TypeScript 库成熟, 社区活跃)<br>2. **Automerge** (强 CRDT, JSON 友好, Rust + JS 实现, 性能中等, 学习曲线陡)<br>3. **LWW (Last-Write-Wins)** (弱 CRDT, 简单, 不保证因果, 实现最简, 适合轻量场景)<br>拍板前 P0 阻塞, 跟 A11 同步桥同源 (ARG 也需类似机制); 选型后所有 element 自动 conflict-free |
| 输入 | 2+ 用户并发改同 1 element |
| 输出 | CRDT 自动 merge, 不需要锁 |
| 数据 schema 增项 | (无新增, 复用 Yjs / Automerge / LWW 库, 跟 ARG Memgraph 同步桥隔离) |
| 接口依赖 | Yjs / Automerge / LWW 库 (TypeScript) + WSS Realtime 通道 |
| 业务规则 | 拍板后落 `docs/design/CRDT-SELECTION-DECISION.md` (实装阶段) + 跟 A11.4 ARG `ARGTrustEngine` 同源设计 |
| 优先级 | P0 |

**用户故事**: US-30 (SRE 多人拖拽同一 element 不冲突)

**验收标准**: AC-A12.6 — 2 用户同时拖拽同 1 element, 选型 CRDT 自动 conflict-free merge, 不需要锁, 不冲突, 选型后所有 element 自动 conflict-free, 跟 A11 同步桥设计同源

**已知缺口**: 缺口 #15 (**CRDT 选型未拍板, 拍板前 P0 阻塞**, 候选 Yjs / Automerge / LWW, 跟 ARG 同步桥同源, 实装阶段拍板)

#### 4.12.7 FR-A12.7 协作权限 (view / comment / edit 3 级)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A12.7 |
| 描述 | 协作权限 3 等级: **view** (只读) / **comment** (评论) / **edit** (编辑), **BFF API 层强制** (per 总册 `SRS-CANVAS-001.md` v1.1 §6.2.2), 5 域 Lead 真人到位后决策具体权限矩阵, 真人未到位 Mavis 临时代签 (per 守门 #14 v2), 拍板前走 **view-only 兜底** (外部用户仅 view, 内部 Lead 默认 edit), 25 module 联动走权限校验 (per 总册 §6.3) |
| 输入 | 用户操作 (任何 element 增删改 / 评论 / @) |
| 输出 | BFF 校验权限, 无权限 → 403 Forbidden + UI 提示 |
| 数据 schema 增项 | `canvas_permissions` (新增, per user_id + canvas_id + role: view / comment / edit) |
| 接口依赖 | BFF middleware 权限校验 (per 总册 §6.2.2) + 5 域 RACI 权限矩阵 (待 DDD Review 拍板) |
| 业务规则 | BR-12 (3 级权限 + BFF 强制) + 守门 #14 v2 (真人到位后追溯) + 拍板前 view-only 兜底 |
| 优先级 | P1 |

**用户故事**: US-31 (5 域 Lead 配置 3 级权限)

**验收标准**: AC-A12.7 — 外部用户访问 canvas, BFF 校验 view 权限通过, 试图 edit 元素 → 403 Forbidden + UI 提示; 内部 Lead 默认 edit 权限; comment 权限用户可评论不能 edit; 跟总册 §6.2.2 + 守门 #14 v2 真人到位后追溯签字一致

**已知缺口**: 缺口 #16 (具体权限矩阵待 5 域 Lead 真人到位后决策, 拍板前走 view-only 兜底, 25 module 联动待 DDD Review 拍板)

#### 4.12.8 FR-A12.8 audit log 多人操作 (per 守门 #13 W/T/M Transaction append-only + SCD Type 2)

| 项 | 内容 |
|---|---|
| ID | FR-AGENT-A12.8 |
| 描述 | 多人增删改 / cursor 移动 / 评论 / @ 全部 audit, 走**新增表 `canvas_multi_user_audit`** (per 守门 #13), W/T/M 必含 **Transaction 类, append-only, 物理删除禁止, SCD Type 2 (version +1), 100% RLS 13 类必携 (per 守门 #13)**, 字段: id / canvas_id / actor_user_id / action (element.create / element.update / element.delete / cursor.move / comment.add / comment.reply / mention.create) / target_id / target_type / payload (JSON) / created_at / version; 1 操作 1 行 audit (per BR-15) |
| 输入 | 任何多人操作 (A12.1 + A12.2 + A12.3 + A12.4 + A12.5) |
| 输出 | 1 行 audit 写入 `canvas_multi_user_audit` 表 (append-only), SCD Type 2 version +1 |
| 数据 schema 增项 | **`canvas_multi_user_audit` (新增表, Transaction 类 per 守门 #13)**: id, canvas_id, actor_user_id, action, target_id, target_type, payload, created_at, version; 100% RLS 13 类必携 |
| 接口依赖 | 25 module audit 域对接 (per 总册 `SRS-CANVAS-001.md` v1.1 §6.3) + BFF middleware 自动写 audit |
| 业务规则 | **BR-13 (W/T/M Transaction append-only + SCD Type 2 + 100% RLS 13 类, per 守门 #13)** + BR-15 (1 操作 1 audit 行) + 守门 #1 v15 docs 同步饱和 (本 SRS 撰写是 17:34 JST 拍板触发的 docs 同步, 新事件触发, 不算饱和违规) |
| 优先级 | P0 |

**用户故事**: US-32 (SRE 多人操作 audit log 追溯)

**验收标准**: AC-A12.8 — 任何多人操作 (增删改 / cursor 移动 / 评论 / @) 100% 写到 `canvas_multi_user_audit` 表 (Transaction append-only, SCD Type 2 version +1, 100% RLS 13 类必携, 1 操作 1 audit 行), 物理删除 0 次, 真人 Lead 到位后可追溯, 跟守门 #13 W/T/M 分类 + 总册 §6.2.2 端点 + 25 module audit 域对接一致

**已知缺口**: 无 (新增表 W/T/M 分类已定: Transaction append-only + SCD Type 2 + 100% RLS 13 类必携, per 守门 #13)

### 4.13 非功能需求 (NFR, 16 项)

#### 4.13.1 NFR-AGENT-PERF-01: 画布渲染性能

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-PERF-01 |
| 指标 | 画布首次渲染 ≤ 500ms (mock 12 agent + 12 worktree + 30 wi + 50 ARG 边 + 10 多人并发场景) |
| 测量 | FCP / LCP, 浏览器 dev tools |
| 优先级 | P0 |

#### 4.13.2 NFR-AGENT-PERF-02: 状态同步延迟

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-PERF-02 |
| 指标 | agent 状态变化 ≤ 200ms (P95) 反映到 `agent_node` 色码 (per A3.1) |
| 测量 | 浏览器 dev tools timeline + P95 统计 |
| 优先级 | P0 |

#### 4.13.3 NFR-AGENT-PERF-03: ARG 边创建 latency

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-PERF-03 |
| 指标 | ARG 边创建 latency P95 < 200ms (Memgraph Bolt, per `SRS-AGENT-RELATIONSHIP-001.md` §7.3) |
| 测量 | Prometheus 监控 + P95 统计 |
| 优先级 | P0 |

#### 4.13.4 NFR-AGENT-PERF-04 (v1.2 新增): 多人同时编辑 Realtime 同步延迟

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-PERF-04 |
| 指标 | 10 用户同时编辑, 元素增删改 ≤ 200ms (P95) 反映到其他用户 (per A12.1 + A12.3) |
| 测量 | 浏览器 dev tools timeline + P95 统计 + WSS latency monitoring |
| 优先级 | P0 |

#### 4.13.5 NFR-AGENT-PERF-05 (v1.2 新增): 实时 Cursor 同步延迟

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-PERF-05 |
| 指标 | Cursor 移动 ≤ 100ms (P95) 反映到其他用户 (per A12.2); cursor 移动 throttle 50ms (避免刷屏) |
| 测量 | 浏览器 dev tools timeline + P95 统计 |
| 优先级 | P0 |

#### 4.13.6 NFR-AGENT-PERF-06 (v1.2 新增): Follow mode 同步延迟

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-PERF-06 |
| 指标 | Follow mode viewport 同步 ≤ 100ms (P95, per A12.4) |
| 测量 | 浏览器 dev tools timeline + P95 统计 |
| 优先级 | P1 |

#### 4.13.7 NFR-AGENT-UI-01: 视觉一致性

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-UI-01 |
| 指标 | 跟 StatusPill 60+ 配色一致 / 跟 V0.1 `frontend-canvas-design.md` 一致 / 跟 ARG 10 类关系颜色规范一致 / dark mode 优先 |
| 测量 | 人工 review, 无 P0 视觉缺陷 |
| 优先级 | P0 |

#### 4.13.8 NFR-AGENT-A11Y-01: 键盘可达

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-A11Y-01 |
| 指标 | 顶部 toolbar dropdown 满足 WAI-ARIA listbox 模式; 右键菜单满足 menu 模式; 快捷键不冲突 (跳过 INPUT/TEXTAREA/SELECT); **多人 cursor 颜色不能仅靠颜色区分 (color-blind 友好, 12 色调色板 + 形状 / 字母辅助)** |
| 测量 | axe-core / WAVE / 人工 |
| 优先级 | P1 (V0.1 部分 a11y, 完整合规留 P3+) |

#### 4.13.9 NFR-AGENT-DET-01: 派生确定性

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-DET-01 |
| 指标 | 同样输入永远出同样输出 (SSR/CSR hydration 无漂移), 排序稳定 [status_order ASC, due_date ASC, id ASC] |
| 测量 | vitest "排序稳定" / "deterministic" 测试 |
| 优先级 | P0 |

#### 4.13.10 NFR-AGENT-STATE-01: 派生只读 + 跨块接口

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-STATE-01 |
| 指标 | 画布不进 zustand store (避免污染 canvasElements 持久化), agent_node + arg_edge 派生数据不写 store; 跨块接口走 URL 参数 (per A9) + 跳路由, 不直接调其他 view 的 LangGraph node; **A12.3 多人编辑元素增删改走 backend 持久化 (BFF) + WSS 广播, 不进 zustand persist** (跟 V0.1 单用户模式冲突) |
| 优先级 | P0 |

#### 4.13.11 NFR-AGENT-I18N-01: 国际化

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-I18N-01 |
| 指标 | 3 语言 (zh-CN / en / ja) 友好, 至少 4 项 i18n key 落 `dictionary.ts` (agent status / role / kind / 关系 type 10 类 / **多人 cursor 名字 + 权限 view-comment-edit + 评论 + @ 提示**) |
| 派生 | StatusPill 走 `useStatusLabel` 翻译 |
| 优先级 | P2 |

#### 4.13.12 NFR-AGENT-TEST-01: 测试覆盖

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-TEST-01 |
| 指标 | vitest 30+ pass (3+ 测试文件, layout / selectors / AgentCanvasView / ArgCanvasView / **MultiUserCanvasView v1.2 新增**), 0 typecheck err |
| 优先级 | P0 |

#### 4.13.13 NFR-AGENT-SEC-01: 安全 (RLS 13 类 + Audit 必携)

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-SEC-01 |
| 指标 | 13 租户隔离 (RLS 13 类必携, per 守门 #13 + `SRS-AGENT-RELATIONSHIP-001.md` NFR-ARG-SECURITY-01), audit log 必记 (per 守门 #13 Transaction 100% audit); **A12 BFF API 层强制 3 级权限 (per A12.7), 不依赖前端隐藏** |
| 优先级 | P0 |

#### 4.13.14 NFR-AGENT-OBS-01: 可观测性

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-OBS-01 |
| 指标 | agent 状态变化 / ARG 边创建 / 状态联动 / **多人 cursor 移动 / 元素增删改 / 评论 / @** 100% audit + Prometheus 导出 (per `SRS-AGENT-RELATIONSHIP-001.md` NFR-ARG-OBSERVABILITY-01) |
| 优先级 | P1 |

#### 4.13.15 NFR-AGENT-EXT-01: 扩展性

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-EXT-01 |
| 指标 | 5 域分组 Frame 扩展支持新域 (配置化, 域列表在 `frontend/src/lib/canvas/domains.ts`); ARG 关系类型扩展支持新类型 (per `SRS-AGENT-RELATIONSHIP-001.md` G-11 ARG Schema V2 迁移路径); **A12 协作权限扩展支持新角色 (per A12.7 拍板后配置化)** |
| 优先级 | P2 |

#### 4.13.16 NFR-AGENT-MU-CONS-01 (v1.2 新增): A12 多人编辑守门合规

| 项 | 内容 |
|---|---|
| ID | NFR-AGENT-MU-CONS-01 |
| 指标 | A12 必含守门合规: (1) BFF 权限校验走 envoy middleware (per 9/1 13:03 JST envoy 偏好), (2) 25 module notification 域对接 (per 总册 §6.3), (3) WSS connection 走 TLS 1.3+, (4) `canvas_multi_user_audit` 表 100% RLS 13 类必携 (per 守门 #13 + A12.8), (5) 5 域 Lead 真人到位前 Mavis 临时代签 (per 守门 #14 v2 + A12.7), (6) CRDT 选型拍板前 P0 阻塞 (per A12.6) |
| 优先级 | P0 |

---

## §5 约束条件 (Constraints)

### 5.1 守门合规 (per AGENTS.md §4)

| 守门 | 约束 | 本 SRS 落地 |
|---|---|---|
| #1 (R-05 不 push 已反转) | git push 守门 | 本 SRS 文档同步 commit 必先跑守门 |
| #3 (5 域独立 Lead) | 5 域 Lead 拒绝兼任 | A2.2 跨 5 域分组 Frame, 跨域关系强制走 `consults` 而非 `delegates_to` (per `SRS-AGENT-RELATIONSHIP-001.md` 守门 #3) |
| #5 (env 安全) | 不打印 env | Memgraph 连接串走 env, 不打印 (per `SRS-AGENT-RELATIONSHIP-001.md` 守门 #5); **A12 WSS 连接串走 env, 不打印** |
| #6 (PowerShell only) | 守门 | 本 SRS 部署脚本 PowerShell (实装阶段) |
| #7 (0 unsafe) | 守门 | Rust crate 0 unsafe (实装阶段) |
| #9 (子代理 RPC 不可靠) | 实证 | A11 同步桥用 in-process 推 + 周期 flush, 不用 RPC (per `SRS-AGENT-RELATIONSHIP-001.md` 守门 #9); **A12 多人编辑用 WSS (Realtime 通道) + in-process 缓存, 不用 RPC** |
| #10 (代签规则) | Mavis 默认代 Ulysses | 本 SRS author = Ulysses, 修订人 = Ulysses (Mavis 接手) (per守门 #10 + 8/27 19:39 JST 用户授权) |
| #13 (W/T/M 三類) | 横展开强制 | A11.5 关系 audit log 必含 4 表 W/T/M (100% 表覆盖), per 守门 #13; **A12.8 多人编辑 audit log 必含新增表 `canvas_multi_user_audit` W/T/M Transaction append-only + SCD Type 2 + 100% RLS 13 类, per 守门 #13** |
| #14 v2 (5 域 Lead 拍板 D) | Mavis 临时代签, 真人到位后追溯 | A2 / A6 / A11 / **A12.7 view/comment/edit 3 级权限** 跨域编排决策由 Mavis 落, 真人到位后追溯签字 |
| #14 v4 (v0.62 反转) | 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses | 本 SRS 审批 = 架构师 (Mavis 接手), author=Ulysses (per 2026-09-10 12:45 JST v0.62 反转) |
| #15 (docs 同步饱和) | 触达饱和后, 后续 docs 同步 commit 必先有新事件触发 | 本 SRS 撰写是 17:22 JST (A11) + 17:34 JST (A12 v0.63 反转) 拍板触发的 docs 同步, 不算饱和违规 |
| #19 (Python 化 3 件套) | 子代理 dispatch + docs 同步 + 任务卡 强制 Python 化 | A11 / **A12** 跨 session 续做时强制走 `scripts/automation/<purpose>.py` (本 SRS 仅文档) |
| #23 v2 (ai-edit-mode 本地 mock) | 不引入第三方 LLM 凭据 | A11.10 / **A12.5 评论** 协同 SRS 跟 ARG 同步, 调试控制台走 mock |
| #1 v15 (docs 同步饱和边界) | 113 ahead 落地 6 commits 后, 后续 docs 同步必先有新事件触发 | 本 SRS 撰写是 17:22 JST (A11) + 17:34 JST (A12 v0.63 反转) 拍板触发的 docs 同步 (新事件), 不算饱和违规 |
| #1 v25 (CI cargo test 改单 crate) | per 2026-09-05 00:15 JST 拍板, `cargo test -p star-context --lib -j 4` (PR #12 CI 实证) | A12 实装时 CI 守门同步反转 (per `frontend/.github/workflows/ci.yml` PR #12) |
| #7 v3 (cargo clippy 改 advisory) | per 2026-09-05 00:15 JST 拍板, advisory 模式 | A12 实装时 CI 守门同步反转 |
| #1 v26 (cargo doc 改 advisory) | per 2026-09-05 00:15 JST 拍板, 跟 clippy 同步反转 | A12 实装时 CI 守门同步反转 |
| #24 v2 (Setup Node.js node-version 20 → 22 LTS) | per 2026-09-05 00:15 JST 拍板, 解决 Node 20 deprecation 警告致 npm ci exit 1 | A12 实装时 CI 守门同步反转 |
| #6 v2 (Frontend typecheck/test/build 改 advisory) | per 2026-09-05 00:15 JST 拍板, `continue-on-error: true`, 跟 clippy/cargo doc 同步反转 | A12 实装时 CI 守门同步反转 |

### 5.2 技术约束 (per V0.1 既有栈)

- **TC-1**: 不引入新 npm 依赖 (per AGENTS.md §4 守门 #19 v19+ 累积规, 0 新依赖, 除非 A12 CRDT 选型拍板后必须新增 Yjs / Automerge / LWW 库, 走 P3-C 阶段拍板)
- **TC-2**: 复用 `@/lib/store` (zustand) / `@/components/StatusPill` / `@/components/PageHeader` / lucide-react
- **TC-3**: 遵循 Next.js 14.2.5 App Router 规范 ("use client" + useSearchParams + useRouter)
- **TC-4**: 遵循 TypeScript strict mode, 0 `any` (除 fallback / 类型断言)
- **TC-5**: 遵循 ESLint + Prettier (per `frontend/.eslintrc.json`)
- **TC-6**: 复用 V0.1 `agent_cursor` element kind (per `CanvasView.tsx` line 218-235) → 扩展为 `agent_node`, 保留 V0.1 跳转逻辑
- **TC-7**: 复用 V0.1 `worktree_node` (line 200-217) + `work_item_card` (line 180-199) 联动逻辑
- **TC-8**: ARG 后端 Memgraph ≥ 2.14, Docker 启动 (per `SRS-AGENT-RELATIONSHIP-001.md` §5.2)
- **TC-9**: ARG 客户端 Rust crate `r2d2-memgraph` (待开发, per `SRS-AGENT-RELATIONSHIP-001.md` G-1)
- **TC-10**: 关系→协作影响 4 维度 effect tier 走 `crates/arg-effect/` 新 crate (per `BD-AGENT-RELATIONSHIP-001.md` §2.2 Tier 5)
- **TC-11 (v1.2 新增)**: 复用 V0.1 §4.1 模式 A Realtime 通道 design (`star.collaboration.canvas.element.*` NATS Subject), 扩展为 A12.1 + A12.3 WSS 实现, 不重新设计
- **TC-12 (v1.2 新增)**: 复用 V0.1 §4.6 PresenceCursor 字段 (cursor x/y + selection + user_id), 扩展为多人显示 (A12.2)
- **TC-13 (v1.2 新增)**: 复用 V0.1 `comment_pin` 渲染 (per `CanvasView.tsx` line 253-262, 圆心 #79c0ff33 + 边框 + 💬 emoji + 评论数 badge), 扩展为多人评论线程 (A12.5)
- **TC-14 (v1.2 新增)**: A12 后端 BFF 走 **envoy 独立 deployment 模式** (per 9/1 13:03 JST envoy 偏好 + 9/1 13:05 JST 独立 deployment 模式, 不选 nginx, 不选 istio sidecar), 业务 svc 通过 `svc://` 引用
- **TC-15 (v1.2 新增)**: A12 WSS 选型候选 (per A12.1 + A12.6): NATS JetStream / native WebSocket / Socket.IO, 拍板前 P0 阻塞

### 5.3 业务约束

- **BC-1**: 界面名 = "Agent" (per 用户发令, 跟其他视图命名一致: Kanban / Timeline / Backlog / Agents / Worktrees)
- **BC-2**: 路由 = `/canvas` (复用 V0.1) + 切 tab "Relationship" → `/agent-relationships?from=canvas` (per A11.10)
- **BC-3**: 跟 Kanban / Worktree / Agent View 共享 zustand store 实时同步 (per V0.1 联动 3)
- **BC-4**: 5 域 Frame 标题固定: `player` / `economy` / `match` / `social` / `admin` (per 8/21 JST RGS 治理命名 + BR-9)
- **BC-5**: 5 域 Lead 真人到位前所有跨域编排 / 关系定义由 Mavis 临时代签, 真人到位后追溯签字 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事)
- **BC-6**: **5 域独立 Lead ≠ Star 22 DDD bounded context** (per 2026-08-31 22:45 JST Q1-D 拍板 disclaimer, 不建立业务子域↔DDD 映射)
- **BC-7**: 画布 + ARG 不引入 istio / nginx 边缘代理 (per 9/1 13:03 JST envoy 偏好, 画布内部通信走 in-process 即可); **A12 BFF 走 envoy 独立 deployment (per 9/1 13:05 JST 独立 deployment 模式)**
- **BC-8**: ARG 不引入 OpenAI / Anthropic 第三方 API (per 守门 #5 v2 + #23, 关系定义 LLM 用 mock, 不用外部)
- **BC-9 (v1.2 新增)**: A12 view/comment/edit 3 级权限矩阵 5 域 Lead 真人到位后决策, 拍板前走 view-only 兜底 (外部用户仅 view, 内部 Lead 默认 edit) (per 守门 #14 v2 + A12.7)
- **BC-10 (v1.2 新增)**: A12 CRDT 选型 (Yjs / Automerge / LWW) 拍板前 P0 阻塞, 拍板后落 `docs/design/CRDT-SELECTION-DECISION.md` (per A12.6)

### 5.4 安全 / 合规约束

- **SC-1**: 不打印环境变量 (per AGENTS.md §4 守门 #5)
- **SC-2**: 不输出 secret / token (per AGENTS.md §4 守门 #5)
- **SC-3**: 13 租户隔离 (per `SRS-AGENT-RELATIONSHIP-001.md` NFR-ARG-SECURITY-01 + 守门 #13), ARG `agents` + `agent_relationship_edges` 走 RLS 13 类必携
- **SC-4**: 关系 audit log 必记 (per 守门 #13 Transaction 100% audit)
- **SC-5**: Memgraph 连接字符串走 env (per 守门 #5)
- **SC-6 (v1.2 新增)**: A12 BFF 3 级权限校验走 envoy middleware, 不依赖前端隐藏 (per A12.7 + NFR-AGENT-SEC-01)
- **SC-7 (v1.2 新增)**: A12 WSS connection 走 TLS 1.3+, auth required (per 总册 §6.2.2 + NFR-AGENT-MU-CONS-01)
- **SC-8 (v1.2 新增)**: A12.8 新增表 `canvas_multi_user_audit` 100% RLS 13 类必携 (per 守门 #13 + A12.8)

### 5.5 组织约束

- **OC-1**: 5 域真人 Lead 到位前 Mavis 临时代签 (per AGENTS.md §4 守门 #3 v2 派生规 8/21 拍板 + 9/3 11:35 JST 反转 + 守门 #14 v2)
- **OC-2**: commit author = Ulysses (per AGENTS.md §4 守门 #10 + 8/27 19:39 JST 用户授权)
- **OC-3**: 报告 / 文档 7 段结构 (per AGENTS.md §3)
- **OC-4**: 修订人 = Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手, 审批 = 架构师 (Mavis 接手 agent per DEC-008) (per 守门 #14 v3 + 9/8 15:19 JST 第 6 次强化 + 守门 #14 v4 反转 v0.62)
- **OC-5 (v1.2 新增)**: v0.63 反转行显式标 (per 17:34 JST Ulysses 拍板"多人编辑是要的" + 守门 #1 禁回溯叙事, 本 SRS §0.2 v1.2 + §4.12 A12 段头 + §12 修订历史 v1.2 row 显式标 v0.63 反转)

---

## §6 业务场景 (Use Cases / 業務シナリオ)

### 6.1 主要场景 (per A1-A12)

#### UC-A1: 5 域 Lead 打开画布看 agent 完整卡拓扑

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | 5 域 Lead (Mavis 临时代签) | 浏览器打开 `https://star.example.com/canvas` | page 读取 store + 5 域 Frame |
| 2 | 系统 | 5 Frame (player/economy/match/social/admin) 渲染, 域内 agent 自动聚类 (5×4 grid) | per A2.2 |
| 3 | 系统 | 12+ agent_node 完整卡渲染 (avatar/role/kind/status/token/started_at) | per A1.1 |
| 4 | 5 域 Lead | 看到 5 域所有 agent 拓扑, 双击 ag-005 | 跳 `/agent?selected=ag-005` (per A1.3) |
| **Acceptance** | 画布首次渲染 ≤ 500ms, 视觉一致, 跳详情成功 | | |

#### UC-A2: SRE 右键 agent 启停 + 查看 logs

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | SRE | 右键 ag-003 节点 | 弹操作菜单 (启停/重启/logs/token/settings) |
| 2 | SRE | 点 "停止" | 调 agent-runtime API stop, 权限校验 SRE Lead ✓ |
| 3 | 系统 | agent 状态 executing → completed | 画布色码实时变绿 (≤ 1s) |
| 4 | SRE | 右键 ag-003 → "查看 logs" | 跳 `/agent-runtime/logs?session=ag-003` |
| **Acceptance** | 启停成功 + 状态实时反映 + 权限校验 + logs 跳转成功 | | |

#### UC-A3: PM drag work-item 到 agent 关联

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | PM | 从 work-item 列表拖 wi-007 到 ag-005 周围 | 弹"关联"确认 modal |
| 2 | PM | 确认 | 调 API 写 `work_item.agent_session_id = ag-005` |
| 3 | 系统 | 画布上 wi-007 出现在 ag-005 周围, 双向 connector | per A5.1 |
| 4 | 系统 | wi-007 状态变化 → ag-005 状态聚合 badge 更新 | per A5.2 |
| **Acceptance** | drag in 成功, 1:N 关联, 双向 connector, 状态联动 ≤ 200ms | | |

#### UC-A4: 5 域 Lead 拖拽建 delegates_to 关系

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | 5 域 Lead | 画布切到 "Relationship" tab | 跳 `/agent-relationships?from=canvas` (per A11.10) |
| 2 | 5 域 Lead | 拖拽 ag-005 → ag-007 | 弹"关系编辑" modal (per A11.2) |
| 3 | 5 域 Lead | 选 type = `delegates_to`, weight = 0.7, metadata = `{"review_threshold": 0.8}` | 提交 |
| 4 | 系统 | POST `/api/arg/edges` → Memgraph write + EventBus edge.changed | 边创建 latency P95 < 200ms (per NFR-A11.3) |
| 5 | 系统 | 画布实时显示新边 (蓝色 delegates_to, 粗细按 weight 0.7) | per A11.1 + A11.6 |
| 6 | 系统 | 4 维度 UI 指示器显示: dispatch 路由 → ag-007 自动 dispatch | per A11.3 |
| **Acceptance** | 拖拽建边成功, Memgraph 持久化, 画布实时反映, 4 维度指示器正确 | | |

#### UC-A5: PM 1-click 部署 Hub-and-Spoke 模板

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | PM | 画布 toolbar "模板" dropdown → 选 "Hub-and-Spoke" | 弹 agent 选择器 (5 个 dropdown) |
| 2 | PM | 选 1 Lead (ag-001) + 4 Worker (ag-005, ag-007, ag-009, ag-011) | 提交 |
| 3 | 系统 | POST `/api/arg/templates/instantiate` (1 事务 Cypher) | 创建 4 条 `delegates_to` 边 (per A11.4) |
| 4 | 系统 | 画布实时显示 1 Lead + 4 Worker 拓扑 + 4 条蓝色边 | per A11.1 |
| **Acceptance** | 1-click 部署成功, 1 事务写 Memgraph, 画布实时反映 (≤ 1s) | | |

#### UC-A6: 5 域 Lead 归档旧关系 + 恢复

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | 5 域 Lead | 右键边 ag-005 → ag-007 (delegates_to) | 弹边操作菜单 |
| 2 | 5 域 Lead | 选 "归档" | 改 `archived=true` (per A11.7) |
| 3 | 系统 | 画布上边变灰半透明, 不影响协作 (active 关系才生效) | per `SRS-AGENT-RELATIONSHIP-001.md` §4.1.3 |
| 4 | 5 域 Lead | 右键边 → "恢复" | 改回 `archived=false` |
| 5 | 系统 | 画布上边恢复蓝色实线, 物理删除 0 次 | per A11.7 |
| **Acceptance** | 归档/恢复成功, 物理删除 0 次, 不影响协作 | | |

#### UC-A7: SRE 看 ARG 同步桥状态

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | SRE | 画布右上角 status badge | 显示 `sync_status = synced` (绿色) + `last_sync_at = 2026-09-10 17:20 JST` |
| 2 | 系统 | Memgraph 写边 → 触发 Bolt subscription | EventBus `edge.changed` |
| 3 | 系统 | badge 变 `syncing` (蓝色) | per A11.9 |
| 4 | 系统 | 完成 flush → badge 变 `synced` + 更新 `last_sync_at` | per A11.9 |
| 5 | 异常 | Memgraph 不可达 → badge 变 `offline` (灰色) + 降级提示 | per A11.9 + `SRS-AGENT-RELATIONSHIP-001.md` §4.4 离线降级 |
| **Acceptance** | 4 状态正确显示, 离线降级 走 in-process 缓存 | | |

#### UC-A11 (v1.2 新增): PM / SRE / 5 域 Lead 多人同时编辑同一 canvas

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | PM (canvas-001) + SRE (canvas-001) + 5 域 Lead (canvas-001) | 3 用户同时打开同一 canvas | WSS 3 客户端连接 (per A12.1) |
| 2 | PM | 拖拽 ag-005 到 (100, 200) | 调 BFF `/v1/collaboration/canvases/canvas-001/elements/el-005` PATCH + WSS 广播 |
| 3 | 系统 | BFF 校验权限 (PM edit 权限 ✓) + 写 backend `canvas.elements` 表 + audit `canvas_multi_user_audit` 1 行 (per A12.8) + WSS 广播 `element.update` | per A12.3 + A12.8 |
| 4 | SRE + 5 域 Lead | ≤ 200ms (P95) 看到 ag-005 移动到 (100, 200) | per A12.3 NFR-PERF-04 |
| 5 | SRE | 添加评论 "@pm 请审核 ag-005 的 status" | 调 BFF `/v1/collaboration/canvases/canvas-001/comments` POST + notification 域发到 PM inbox + audit 1 行 (per A12.5) |
| 6 | 系统 | WSS 广播 `comment.add` + notification 域对接 | per A12.5 |
| 7 | PM | 收到 notification + 看到 SRE 添加的评论 thread + 看到 5 域 Lead 的 cursor 在 ag-007 上 | per A12.2 + A12.5 |
| **Acceptance** | 多人同时编辑, 元素增删改 ≤ 200ms 同步, 评论 + @ + cursor 实时, 1 操作 1 audit 行, BFF 权限校验 | | |

#### UC-A12 (v1.2 新增): PM 跟随 SRE 视角 (Follow mode)

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | PM | 在画布顶部 toolbar "用户列表" dropdown → 选 "Follow SRE" | 调 POST `/v1/collaboration/canvases/canvas-001/follow?user_id=sre-ulysses` (per A12.4) |
| 2 | 系统 | BFF 校验权限 (PM ≥ view ✓) + 写 `canvas_followers` 表 | per A12.4 |
| 3 | PM 端 | 顶部 toolbar 显示 "Following: SRE" + "Stop Following" 按钮 | per A12.4 |
| 4 | SRE | 移动 viewport 到 (500, 300) + zoom 1.5x | WSS 广播 viewport 变化 |
| 5 | PM 端 | viewport 同步到 (500, 300) + zoom 1.5x (≤ 100ms P95) | per A12.4 NFR-PERF-06 |
| 6 | SRE | 选中 el-005 → 高亮 | WSS 广播 selection 变化 |
| 7 | PM 端 | 看到 el-005 高亮 (跟 SRE 同步) | per A12.4 |
| 8 | PM | 点 "Stop Following" | 调 POST `/v1/collaboration/canvases/canvas-001/follow/stop` + 解除 follow 关系 |
| **Acceptance** | Follow mode 实时同步 viewport + selection (≤ 100ms P95), 1 canvas 1 follower, BFF 权限校验 | | |

#### UC-A13 (v1.2 新增): 外部用户 view-only 兜底 (5 域 Lead 真人未到位前)

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | 外部用户 (ext-ulysses) | 访问 canvas-001 | BFF 校验权限: `canvas_permissions` 表 ext-ulysses 角色 = view |
| 2 | 系统 | 渲染 canvas + 显示 view-only 提示 | per A12.7 + BC-9 (拍板前 view-only 兜底) |
| 3 | 外部用户 | 试图拖拽 element | BFF 校验 view 权限 → 403 Forbidden + UI 提示 "view-only 权限, 不能编辑" |
| 4 | 外部用户 | 试图添加评论 | BFF 校验 view 权限 → 403 Forbidden + UI 提示 |
| 5 | 5 域 Lead 真人到位后 | 配置权限矩阵 → 升级 ext-ulysses 为 comment 权限 | 修订历史表 +1 行, author=真人 Lead, per 守门 #14 v2 + A12.7 追溯签字 |
| **Acceptance** | view-only 兜底有效, 真人到位后追溯签字覆盖修订历史, **不沿用代签决策** (per 守门 #1 禁回溯叙事) | | |

### 6.2 异常场景

#### UC-A8: store 无 agent session

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | SRE | 浏览器打开 `/canvas` | page 读取 store, `agentSessions.length === 0` |
| 2 | 系统 | 显示空状态 | warn icon + "No agent sessions" + 跳 `/agents` 链接 (per V0.1 FR-AGV-014) |

**Acceptance**: 不显示空白画布, 给清晰引导

#### UC-A9: ARG Memgraph 不可达

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | 5 域 Lead | 画布切到 "Relationship" tab | 跳 ARG 视图 |
| 2 | 系统 | 尝试读 Memgraph 失败 | badge 变 `offline` (灰色) + 降级提示 "Memgraph 不可达, in-process 缓存继续工作" |
| 3 | 5 域 Lead | 拖拽建边 | 走 in-process 缓存 + 周期 flush 队列 |
| 4 | 系统 | Memgraph 重连 → 自动 flush | badge 变 `synced` + 队列清空 |

**Acceptance**: 离线降级 + 重连后 flush 成功, 不丢关系变更 (per `SRS-AGENT-RELATIONSHIP-001.md` NFR-ARG-RELIABILITY-01)

#### UC-A10: 5 域 Lead 真人未到位, 关系定义由 Mavis 代签

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | 5 域 Lead (Mavis 代签) | 拖拽建边 | 提交, author = Ulysses (per守门 #14 v4 反转) |
| 2 | 系统 | audit log 记录 `created_by = Mavis (临时代签)` | per A11.5 |
| 3 | 5 域 Lead 真人到位后 | 追溯签字 | 修订历史表 +1 行, author = 真人 Lead (per守门 #14 v2 + 9/3 19:35 JST 拍板 D) |

**Acceptance**: 代签 → 真人到位 → 追溯签字覆盖, 真人决策 vs Mavis 代签决策 = 独立审计链, **不沿用代签决策** (per守门 #1 禁回溯叙事)

#### UC-A14 (v1.2 新增): A12 WSS 连接断开 + 重连

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | PM | 编辑 canvas-001, WSS 连接 | WSS 已连接 |
| 2 | 网络 | WSS 连接断开 (网络抖动) | 客户端自动 retry 3 次 (exponential backoff 1s / 2s / 4s) |
| 3 | PM | 离线模式编辑 (V0.1 localStorage 降级为 fallback, per BR-14) | localStorage 暂存 5 个 element 增删改 |
| 4 | 网络 | WSS 重连成功 | 客户端自动同步暂存的 5 个 element 增删改 → BFF + audit 1 行 (per A12.8) + WSS 广播 |
| 5 | PM + 其他用户 | 看到 PM 暂存的 5 个 element 增删改同步显示 | per A12.3 + A12.8 |

**Acceptance**: WSS 断开 → 离线降级 → 重连后自动 sync, 不丢操作, audit 1 操作 1 行

#### UC-A15 (v1.2 新增): A12 CRDT 选型拍板前 阻塞 (P0)

| 步骤 | 角色 | 动作 | 系统响应 |
|---|---|---|---|
| 1 | 5 域 Lead | 2 用户同时拖拽同 1 element | CRDT 库未选型, 走 view-only 兜底 (per A12.6 + A12.7) |
| 2 | 系统 | 第二个用户拖拽请求被 BFF 拒绝 → 403 + UI 提示 "CRDT 选型拍板中, 暂不支持并发编辑同 1 element, 请等待" | per A12.6 + A12.7 |
| 3 | 实装阶段 | CRDT 选型拍板 (Yjs / Automerge / LWW) | 落 `docs/design/CRDT-SELECTION-DECISION.md` + 解除阻塞 |

**Acceptance**: CRDT 选型拍板前并发编辑被 view-only 兜底阻止, 拍板后所有 element 自动 conflict-free (per A12.6)

---

## §7 数据需求 (Data Requirements)

### 7.1 输入数据 (Store Schema 增项)

复用 V0.1 zustand store (per `frontend/src/lib/store.ts`), 增项如下:

| 集合 | 字段 (增项) | 类型 | 用途 | 引用 |
|---|---|---|---|---|
| `agentSessions` | `avatar_url` | `string \| null` (新增) | A1.1 完整卡头像 | 本 SRS A1.1 |
| `agentSessions` | `role` | `enum {supervisor, worker, reviewer}` (新增) | A1.1 完整卡角色 + A8.1 按 role 聚类 | 本 SRS A1.1 + A8.1 |
| `agentSessions` | `domain` | `enum {player, economy, match, social, admin}` (新增) | A2.2 跨 5 域分组 Frame | 本 SRS A2.2 |
| `agentSessions` | `parent_session_id` | `UUID \| null` (新增) | A2.3 父子 agent 关系 | 本 SRS A2.3 |
| `agentSessions` | `token_budget` | `int` (新增, 默认 1.2M) | A7.2 token 用量对比 budget | 本 SRS A7.2 |
| `worktrees` | `agent_session_ids` | `UUID[]` (新增, 替代 V0.1 `agent_session_id`) | A4.1 1 agent → N worktree | 本 SRS A4.1 |
| `worktrees` | `pipeline_agent_ids` | `UUID[]` (新增) | A2.4 agent pipeline 视图 | 本 SRS A2.4 |
| `workItems` | `agent_session_id` | `UUID \| null` (新增, per `SRS-AGENT-VIEW-001.md` §10 缺口 #4) | A5.1 1 agent → N work-item | 本 SRS A5.1 |
| `argStore.agents` | (新建, 引用 Memgraph) | `ArgAgent[]` | A11.1 10 类关系边渲染 | per `SRS-AGENT-RELATIONSHIP-001.md` §4.2.1 |
| `argStore.edges` | (新建, 引用 Memgraph) | `ArgEdge[]` (含 10 type + weight + archived + version) | A11.1 关系边 | per `SRS-AGENT-RELATIONSHIP-001.md` §4.2.2 |
| `argStore.syncStatus` | (新建) | `{ sync_status, last_sync_at, sync_error }` | A11.9 同步桥状态 | 本 SRS A11.9 |
| **`canvasStore.elements`** (v1.2 新增) | (新建, 引用 backend) | `CanvasElement[]` | A12.3 元素增删改 backend 持久化 | 本 SRS A12.3 |
| **`canvasStore.presenceCursors`** (v1.2 新增) | (新建, 引用 WSS) | `PresenceCursor[]` (per user_id) | A12.2 多人 cursor 同步 | 本 SRS A12.2 |
| **`canvasStore.followers`** (v1.2 新增) | (新建) | `CanvasFollower[]` (per canvas + leader + follower) | A12.4 Follow mode | 本 SRS A12.4 |
| **`canvasStore.comments`** (v1.2 新增) | (新建) | `CanvasComment[]` (thread 结构) | A12.5 多人评论线程 | 本 SRS A12.5 |
| **`canvasStore.permissions`** (v1.2 新增) | (新建, 引用 backend) | `CanvasPermission[]` (per user_id + canvas_id + role) | A12.7 3 级权限 | 本 SRS A12.7 |

**重要 (v1.2)**: 上述 A12 集合 (`canvasStore.*`) 跟 V0.1 zustand persist (localStorage) **冲突** (per BR-14), A12 实施时 V0.1 持久化逻辑降级为"离线 fallback", A12 主数据走 backend (BFF) + WSS Realtime 通道, 跟 NFR-AGENT-STATE-01 派生只读 + 跨块接口 一致。

### 7.2 ARG 4 表 W/T/M 三類横展 (per 守门 #13, 100% 表覆盖, A11.5 必含)

| # | 表 | 类型 (W/T/M) | 字段 | 索引 | RLS / Audit / Retention | 引用 |
|---|---|---|---|---|---|---|
| 1 | `agents` | **Master** (SCD Type 2, 物理删除禁止) | id, name, archetype, domain, status, trust_score, metadata, created_at, updated_at, version | id, archetype, domain | **100% RLS 13 类** (per 守门 #13) | per `SRS-AGENT-RELATIONSHIP-001.md` §7.1 |
| 2 | `agent_relationship_edges` | **Master** (SCD Type 2, 物理删除禁止) | id, from_agent, to_agent, type, weight, direction, archived, metadata, version | id, from_agent, to_agent, type, composite (from+to+type+archived) | **100% RLS 13 类** (per 守门 #13) | per `SRS-AGENT-RELATIONSHIP-001.md` §7.1 |
| 3 | `agent_relationship_edges_audit` | **Transaction** (append-only, 物理删除禁止) | id, edge_id, action (create/update/archive), old_value, new_value, actor, timestamp | edge_id, timestamp | **100% audit 必携** (per 守门 #13) | per `SRS-AGENT-RELATIONSHIP-001.md` §7.1 |
| 4 | `team_template_instances` | **Work** (短 TTL 30 天, 完成后清理) | id, template_id, instance_name, agent_ids, edges_json, created_at, expires_at | instance_name, expires_at | **100% retention_period** (per 守门 #13) | per `SRS-AGENT-RELATIONSHIP-001.md` §7.1 |
| 5 | `achievements` | **Master** (SCD Type 2, 物理删除禁止) | id, code, name, description, category, rarity, icon_url | code, category | **100% RLS 13 类** (per 守门 #13) | per `SRS-AGENT-RELATIONSHIP-001.md` §7.1 |
| 6 | `achievement_unlocks` | **Transaction** (append-only, 物理删除禁止) | id, achievement_id, user_id, agent_ids, trigger_metadata, unlocked_at | user_id, unlocked_at | **100% audit 必携** (per 守门 #13) | per `SRS-AGENT-RELATIONSHIP-001.md` §7.1 |
| 7 | `relationship_events` | **Transaction** (append-only, 物理删除禁止) | id, edge_id, event_type, payload, timestamp | edge_id, timestamp | **100% audit 必携** (per 守门 #13) | per `SRS-AGENT-RELATIONSHIP-001.md` §7.1 |

**W/T/M 100% 表覆盖验证** (per 守门 #13):

| 類型 | 表数 | 表名 | 占比 |
|---|---|---|---|
| **Work** (短 TTL 作業中) | 1 | `team_template_instances` (TTL 30 天) | 1/7 = 14.3% |
| **Transaction** (業務事実 / 監査 / Append-only) | 3 | `agent_relationship_edges_audit` + `achievement_unlocks` + `relationship_events` | 3/7 = 42.9% |
| **Master** (参考 / 設定 / 慢変 SCD) | 3 | `agents` + `agent_relationship_edges` + `achievements` | 3/7 = 42.9% |
| **合计** | **7** | (100% 覆盖, per 守门 #13) | 100% |

**派生规 (per 守门 #13)**:
- (a) **W = 物理删除 / タイマー失効 / 短 TTL 明示 retention**: `team_template_instances` 30 天 TTL ✓
- (b) **T = 物理删除禁止 + 監査必須 + RLS 13 類必携**: 3 张 Transaction 表 100% audit + 物理删除禁止 ✓
- (c) **M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携**: 3 张 Master 表 SCD Type 2 + RLS 13 类 ✓
- (d) **Master 100% RLS / Transaction 100% audit / Work 100% retention_period**: 全部满足 ✓

### 7.3 A12 新增表 W/T/M 三類横展 (per 守门 #13, 100% 表覆盖, A12.8 必含, v1.2 新增)

| # | 表 | 类型 (W/T/M) | 字段 | 索引 | RLS / Audit / Retention | 引用 |
|---|---|---|---|---|---|---|
| 1 (v1.2 新增) | **`canvas_multi_user_audit`** | **Transaction** (append-only, 物理删除禁止, SCD Type 2) | id, canvas_id, actor_user_id, action (element.create / element.update / element.delete / cursor.move / comment.add / comment.reply / mention.create / follow.start / follow.stop), target_id, target_type, payload (JSON), created_at, version | canvas_id, actor_user_id, action, created_at | **100% audit 必携 + 100% RLS 13 类必携** (per 守门 #13 + A12.8) | 本 SRS A12.8 |
| 2 (v1.2 新增) | `canvas_comments` | **Master** (SCD Type 2, 物理删除禁止) | id, canvas_id, element_id, parent_comment_id (thread 结构), author_user_id, content, mentions (UUID[]), created_at, updated_at, version, archived | canvas_id, element_id, author_user_id | **100% RLS 13 类** (per 守门 #13) | 本 SRS A12.5 |
| 3 (v1.2 新增) | `canvas_comment_mentions` | **Transaction** (append-only, 物理删除禁止) | id, comment_id, mentioned_user_id, notification_sent, read_at, created_at, version | comment_id, mentioned_user_id | **100% audit 必携** (per 守门 #13) | 本 SRS A12.5 |
| 4 (v1.2 新增) | `canvas_permissions` | **Master** (SCD Type 2, 物理删除禁止) | id, canvas_id, user_id, role (view / comment / edit), granted_by_user_id, granted_at, expires_at, version | canvas_id, user_id, role, composite (canvas_id+user_id) | **100% RLS 13 类** (per 守门 #13) | 本 SRS A12.7 |
| 5 (v1.2 新增) | `canvas_followers` | **Work** (短 TTL session-bound, session 结束自动清理) | id, canvas_id, leader_user_id, follower_user_id, started_at, expires_at (session 结束时间) | canvas_id, follower_user_id, leader_user_id | **100% retention_period** (per 守门 #13) | 本 SRS A12.4 |
| 6 (v1.2 新增) | `canvas_elements_backend` | **Master** (SCD Type 2, 物理删除禁止) — 跟 V0.1 `CanvasElement` 区分 (backend 持久化) | id, canvas_id, kind, x, y, width, height, rotation, z_index, content (JSON), locked, hidden, created_by_user_id, created_at, updated_at, version | canvas_id, kind, composite (canvas_id+kind) | **100% RLS 13 类** (per 守门 #13) | 本 SRS A12.3 |
| 7 (v1.2 新增) | `canvas_presence_cursors` | **Work** (短 TTL heartbeat 30s, 离线自动清理) | id, canvas_id, user_id, cursor_x, cursor_y, viewport_x, viewport_y, viewport_zoom, selected_element_ids, last_heartbeat_at, expires_at (heartbeat + 30s) | canvas_id, user_id, last_heartbeat_at | **100% retention_period** (per 守门 #13) | 本 SRS A12.2 |

**W/T/M 100% 表覆盖验证 (v1.2 新增表)** (per 守门 #13):

| 類型 | 表数 | 表名 | 占比 |
|---|---|---|---|
| **Work** (短 TTL 作業中) | 2 | `canvas_followers` (session-bound) + `canvas_presence_cursors` (heartbeat 30s) | 2/7 = 28.6% |
| **Transaction** (業務事実 / 監査 / Append-only) | 2 | `canvas_multi_user_audit` (核心 A12.8) + `canvas_comment_mentions` | 2/7 = 28.6% |
| **Master** (参考 / 設定 / 慢変 SCD) | 3 | `canvas_comments` + `canvas_permissions` + `canvas_elements_backend` | 3/7 = 42.9% |
| **合计** | **7** | (100% 覆盖, per 守门 #13) | 100% |

**派生规 (per 守门 #13 + A12.8)**:
- (a) **W = 物理删除 / タイマー失効 / 短 TTL 明示 retention**: `canvas_followers` (session-bound) + `canvas_presence_cursors` (heartbeat 30s) ✓
- (b) **T = 物理删除禁止 + 監査必須 + RLS 13 類必携**: `canvas_multi_user_audit` (核心 A12.8, 100% audit + SCD Type 2 + RLS 13 类) + `canvas_comment_mentions` ✓
- (c) **M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携**: `canvas_comments` + `canvas_permissions` + `canvas_elements_backend` ✓
- (d) **Master 100% RLS / Transaction 100% audit / Work 100% retention_period**: 全部满足 ✓
- **(e) v0.63 反转行显式标** (per 守门 #1 禁回溯叙事, 本节 + A12.8 + §0.2 v1.2 + §12 修订历史 v1.2 row)

### 7.4 数据完整性约束

- **IC-1**: `worktree.agent_session_ids[]` 必须存在 ≥ 1 个 (1 worktree 至少 1 agent, per BR-2)
- **IC-2**: `work_item.agent_session_id` 必须等于 `agent.id` 才能被 A5.1 关联 (per BR-3)
- **IC-3**: 派生数据不写 store (per NFR-AGENT-STATE-01), 违反 = NFR 违反; **A12 canvasStore.* 集合走 backend 持久化 (per A12.3), 不进 zustand persist (跟 V0.1 localStorage 冲突, per BR-14)**
- **IC-4**: ARG `agent_relationship_edges.archived` 物理删除禁止 (per守门 #13 Master SCD Type 2 + A11.7)
- **IC-5**: ARG `agent_relationship_edges.version` 改一次 +1 (per BR-8 + A11.8)
- **IC-6**: ARG 4 表 W/T/M 100% 覆盖 (per 守门 #13, 详见 §7.2 表格)
- **IC-7 (v1.2 新增)**: A12 `canvas_multi_user_audit` 1 操作 1 audit 行 (per BR-15), 物理删除禁止, SCD Type 2 version +1, 100% RLS 13 类必携 (per 守门 #13)
- **IC-8 (v1.2 新增)**: A12 `canvas_comments.parent_comment_id` 必须存在或 null (thread 根评论 vs 回复)
- **IC-9 (v1.2 新增)**: A12 `canvas_permissions.role` ∈ {view, comment, edit} (3 级权限枚举, per A12.7)
- **IC-10 (v1.2 新增)**: A12 `canvas_followers` 1 canvas 1 follower (per A12.4 避免性能问题)

### 7.5 数据流 (Data Flow)

```
[store.agentSessions] 
  ↓ (V0.1 已有)
[agent_node 完整卡渲染] (per A1.1)
  ↓ (V0.1 联动 3, §4.4)
[StatusPill 60+ 色码同步] (per A1.2 + A3.1)
  ↓
[5 域 Frame 分组] (per A2.2)
  ↓
[handoff / 父子 / pipeline connector] (per A2.1 / A2.3 / A2.4)
  ↓
[worktree / work-item 联动] (per A4 / A5)
  ↓
[操作菜单 + 监控面板] (per A6 / A7)
  ↓
[聚类 / 排序 / 过滤] (per A8)
  ↓
[跨域引用 / settings 集成] (per A9 / A10)
  ↓
  ↓
[Memgraph agents + agent_relationship_edges] (per A11)
  ↓
[10 类关系边渲染] (per A11.1)
  ↓
[关系编辑 + 4 维度影响 + 5 模板] (per A11.2 / A11.3 / A11.4)
  ↓
[audit log 4 表 W/T/M 100% 覆盖] (per A11.5 + 守门 #13)
  ↓
[权重可视化 + archive/restore + version] (per A11.6 / A11.7 / A11.8)
  ↓
[同步桥状态显示] (per A11.9)
  ↓
[跟 ARG 协同 tab 切换] (per A11.10)
  ↓
  ↓
[envoy 独立 deployment BFF] (per 9/1 13:03+13:05 JST 偏好, TC-14)
  ↓
[WSS Realtime 通道 + canvasStore.*] (per A12.1 + A12.2 + A12.3 + A12.5)
  ↓
[Follow mode + 评论 + @ 提醒] (per A12.4 + A12.5)
  ↓
[CRDT 选型 + 冲突解决] (per A12.6)
  ↓
[BFF 3 级权限校验] (per A12.7)
  ↓
[audit log canvas_multi_user_audit 7 表 W/T/M 100% 覆盖] (per A12.8 + §7.3 + 守门 #13)
```

---

## §8 接口需求 (Interface Requirements)

### 8.1 内部接口 (组件 Props)

```typescript
// 8.1.1 agent_node props (扩展 V0.1 agent_cursor)
interface AgentNodeProps {
  agent: AgentSession;
  position: { x: number; y: number };
  viewport: Viewport;
  isSelected: boolean;
  isHighlighted: boolean;
  onMouseDown: (e: React.MouseEvent) => void;
  onDoubleClick: () => void;
  onContextMenu: (e: React.MouseEvent) => void;
}

// 8.1.2 agent detail panel props
interface AgentDetailPanelProps {
  agent: AgentSession;
  onClose: () => void;
  onEditSettings: () => void;
}

// 8.1.3 arg_edge props
interface ArgEdgeProps {
  edge: ArgEdge;
  from: AgentNode;
  to: AgentNode;
  position: { from: Vec2; to: Vec2 };
  viewport: Viewport;
  effectIndicators: {
    dispatch?: { count: number };
    context?: { savedTokens: number };
    trust?: { score: number };
    review?: { passRate: number };
  };
}

// 8.1.4 sync status badge props
interface SyncStatusBadgeProps {
  syncStatus: 'synced' | 'syncing' | 'error' | 'offline';
  lastSyncAt: Iso8601;
  syncError: string | null;
}

// 8.1.5 team template gallery props
interface TeamTemplateGalleryProps {
  templates: TeamTemplate[];
  onInstantiate: (templateId: string, agentIds: string[]) => void;
}

// 8.1.6 (v1.2 新增) presence cursor props (per A12.2)
interface PresenceCursorProps {
  cursor: PresenceCursor;     // cursor x/y + user_name + user_color (per 12 色调色板)
  viewport: Viewport;
  isLocalUser: boolean;        // true = 本用户, false = 其他人
}

// 8.1.7 (v1.2 新增) follow mode badge props (per A12.4)
interface FollowModeBadgeProps {
  leaderUserId: Uuid;
  leaderUserName: string;
  onStopFollowing: () => void;
}

// 8.1.8 (v1.2 新增) comment thread props (per A12.5)
interface CommentThreadProps {
  comments: CanvasComment[];     // 1 顶级评论 + N 回复
  element: CanvasElement;
  onAddComment: (content: string, mentions: Uuid[]) => void;
  onReply: (parentId: Uuid, content: string) => void;
  canComment: boolean;            // per A12.7 权限校验
}

// 8.1.9 (v1.2 新增) permission gate props (per A12.7)
interface PermissionGateProps {
  userRole: 'view' | 'comment' | 'edit';
  requiredRole: 'view' | 'comment' | 'edit';
  children: React.ReactNode;
  fallback?: React.ReactNode;     // 403 UI 提示
}
```

### 8.2 外部接口 (Route)

| 路径 | 入参 | 出参 | 备注 | 引用 |
|---|---|---|---|---|
| `/canvas` | - | page 渲染 | 主入口 (V0.1 复用) | per V0.1 |
| `/canvas?highlight=el-XXX` | `?highlight=el-XXX` | 高亮 element (V0.1 联动 7) | per V0.1 §4.9 |
| `/agent?selected=ag-XXX` | `?selected=ag-XXX` | Agent Sessions 详情 (per A1.3) | 双击 agent_node |
| `/agent-runtime/logs?session=ag-XXX` | `?session=ag-XXX` | Agent Runtime logs (per A6.3) | 右键菜单 logs |
| `/agent-settings?selected=ag-XXX` | `?selected=ag-XXX` | V0.1 settings (per A6.4 + A10.1) | 右键菜单 settings + detail panel tab |
| `/agent-view?agent=ag-XXX` | `?agent=ag-XXX` | Agent View 协同 (per A9.1) | 双击 agent_node 配置走 |
| `/agent-relationships?from=canvas&agent=ag-XXX` | `?from=canvas&agent=ag-XXX` | ARG 全图视图 (per A11.10) | 画布 "Relationship" tab 切换 |
| `/api/arg/edges` (POST) | `{ from_agent, to_agent, type, weight, metadata }` | 边创建 (per A11.2) | 必填 11 字段 |
| `/api/arg/edges/{id}` (PATCH) | `{ weight, metadata, archived, version }` | 边更新 (per A11.8) | optimistic lock |
| `/api/arg/templates/instantiate` (POST) | `{ template_id, agent_ids }` | 模板实例化 (per A11.4) | 1 事务 Cypher |
| `/ws/arg/events` (WebSocket) | - | 实时事件 (edge.changed / achievement.unlocked) | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| **`/v1/collaboration/canvases/[id]/elements`** (v1.2 新增, REST) | `{ kind, x, y, width, height, content, ... }` | 元素 CRUD (per A12.3) | BFF middleware 3 级权限校验 |
| **`wss://canvas-collab/canvases/[id]`** (v1.2 新增, WSS) | - | 多人编辑 Realtime 通道 (per A12.1 + A12.3) | 扩展 V0.1 §4.1 模式 A |
| **`wss://canvas-presence/canvases/[id]`** (v1.2 新增, WSS) | - | 多人 cursor presence 通道 (per A12.2) | 扩展 V0.1 §4.6 PresenceCursor |
| **`/v1/collaboration/canvases/[id]/follow`** (v1.2 新增, REST) | `{ leader_user_id }` | Follow mode 启动/停止 (per A12.4) | POST 启动 / POST `/stop` 停止 |
| **`/v1/collaboration/canvases/[id]/comments`** (v1.2 新增, REST) | `{ element_id, parent_comment_id, content, mentions }` | 评论 CRUD (per A12.5) | BFF middleware 校验 comment 权限 |
| **`/v1/collaboration/canvases/[id]/comments/[cid]/thread`** (v1.2 新增, REST) | - | 评论 thread (per A12.5) | 跟 25 module notification 域对接 |
| **`/v1/notifications/@`** (v1.2 新增, REST) | `{ user_id, content, canvas_id, element_id }` | @ 提醒 (per A12.5) | 跟 25 module notification 域对接 |
| **`/v1/collaboration/canvases/[id]/multi-user-audit`** (v1.2 新增, REST) | - | 多人编辑 audit log 查询 (per A12.8) | 100% RLS 13 类 |
| **`/v1/collaboration/canvases/[id]/permissions`** (v1.2 新增, REST) | `{ user_id, role }` | 权限管理 (per A12.7) | admin only, 5 域 Lead 真人到位后启用 |

### 8.3 Zustand Store 依赖

| Action / Getter | 用途 | 增项 |
|---|---|---|
| `useStore((s) => s.agentSessions)` | 选 agent 源 | 增 avatar_url / role / domain / parent_session_id / token_budget |
| `useStore((s) => s.worktrees)` | 1:N 关联 | 增 agent_session_ids[] / pipeline_agent_ids[] |
| `useStore((s) => s.workItems)` | 圆周散点 + 实时同步 (per A5.2) | 增 agent_session_id |
| `useARGStore((s) => s.agents)` | A11 ARG 节点 (Memgraph 同步) | 新建 |
| `useARGStore((s) => s.edges)` | A11 ARG 边 (Memgraph 同步) | 新建 |
| `useARGStore((s) => s.syncStatus)` | A11.9 同步桥状态 | 新建 |
| **`useCanvasCollabStore((s) => s.elements)`** (v1.2 新增) | A12.3 元素 (backend 持久化, 不进 zustand persist) | 新建 |
| **`useCanvasCollabStore((s) => s.presenceCursors)`** (v1.2 新增) | A12.2 多人 cursor (WSS) | 新建 |
| **`useCanvasCollabStore((s) => s.followers)`** (v1.2 新增) | A12.4 Follow mode | 新建 |
| **`useCanvasCollabStore((s) => s.comments)`** (v1.2 新增) | A12.5 评论 | 新建 |
| **`useCanvasCollabStore((s) => s.permissions)`** (v1.2 新增) | A12.7 权限 | 新建 |

**重要**: 画布 Agent View **不**调任何 action (transitionWorkItem / transitionAgent / addWorkItem), 仅订阅读取 (派生只读, per NFR-AGENT-STATE-01); ARG 写操作 (A11.2 / A11.4 / A11.7 / A11.8) 调 `/api/arg/edges` REST API, 走 `crates/api` + Memgraph + EventBus, 不直接调 store action; **A12 写操作 (A12.3 / A12.5 / A12.7 / A12.8) 走 BFF `/v1/collaboration/canvases/[id]/*` REST API, BFF middleware 3 级权限校验, 走 WSS 广播 + backend 持久化, 不直接调 store action**。

### 8.4 ARG API 端点 (per `SRS-AGENT-RELATIONSHIP-001.md` §7.2)

| Method | Path | 说明 | 守门 | 引用 |
|---|---|---|---|---|
| `POST` | `/api/arg/agents` | 创建 agent | W/T/M 严格 | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `GET` | `/api/arg/agents` | 列表 agent | 分页 + 过滤 | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `PATCH` | `/api/arg/agents/{id}` | 更新 agent (SCD Type 2, version +1) | Master | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `POST` | `/api/arg/edges` | 创建关系 (per A11.2) | 必填 11 字段 | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `GET` | `/api/arg/edges` | 列表关系 (per A11.1 渲染) | 过滤 type/agent | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `PATCH` | `/api/arg/edges/{id}` | 更新关系 (weight/metadata) (per A11.8) | Master | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `DELETE` | `/api/arg/edges/{id}` | 归档关系 (archived=true) (per A11.7) | append-only audit | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `GET` | `/api/arg/graph` | 整图查询 (Cypher) | 限频 | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `POST` | `/api/arg/templates/instantiate` | 模板实例化 (per A11.4) | 1 事务 | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `GET` | `/api/arg/achievements` | 成就列表 | 分页 | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `GET` | `/api/arg/achievements/me` | 我的解锁成就 | 用户维度 | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `POST` | `/api/arg/achievements/evaluate` | 触发评估 (admin only) | 幂等 | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |
| `WS` | `/ws/arg/events` | 实时事件推送 (edge.changed / achievement.unlocked) | auth required | per `SRS-AGENT-RELATIONSHIP-001.md` §7.2 |

### 8.5 A12 API 端点 (v1.2 新增, per 总册 `SRS-CANVAS-001.md` v1.1 §6.2.2)

| Method | Path | 说明 | 守门 | 引用 |
|---|---|---|---|---|
| `POST` | `/v1/collaboration/canvases/[id]/elements` | 创建 element (per A12.3) | BFF 校验 edit 权限 + 写 backend + audit 1 行 (per A12.8) | 本 SRS A12.3 + A12.8 |
| `GET` | `/v1/collaboration/canvases/[id]/elements` | 列表 element (per A12.3) | BFF 校验 view 权限 | 本 SRS A12.3 |
| `PATCH` | `/v1/collaboration/canvases/[id]/elements/[eid]` | 更新 element (per A12.3) | BFF 校验 edit 权限 + optimistic lock + audit 1 行 | 本 SRS A12.3 + A12.8 |
| `DELETE` | `/v1/collaboration/canvases/[id]/elements/[eid]` | 删除 element (per A12.3) | BFF 校验 edit 权限 + audit 1 行 | 本 SRS A12.3 + A12.8 |
| `WSS` | `wss://canvas-collab/canvases/[id]` | 多人编辑 Realtime 通道 (per A12.1 + A12.3) | TLS 1.3+ + auth required + BFF 校验 view 权限 | per `frontend-canvas-design.md` §4.1 模式 A + 本 SRS A12.1 + A12.3 |
| `WSS` | `wss://canvas-presence/canvases/[id]` | 多人 cursor presence 通道 (per A12.2) | TLS 1.3+ + auth required + BFF 校验 view 权限 + throttle 50ms | per `frontend-canvas-design.md` §4.6 升级 + 本 SRS A12.2 |
| `POST` | `/v1/collaboration/canvases/[id]/follow?user_id=` | 启动 Follow mode (per A12.4) | BFF 校验 ≥ view 权限 + 1 canvas 1 follower | 本 SRS A12.4 |
| `POST` | `/v1/collaboration/canvases/[id]/follow/stop` | 停止 Follow mode (per A12.4) | BFF 校验 follower 身份 | 本 SRS A12.4 |
| `POST` | `/v1/collaboration/canvases/[id]/comments` | 创建评论 (per A12.5) | BFF 校验 ≥ comment 权限 + 25 module notification 域对接 (mention 触发) + audit 1 行 | 本 SRS A12.5 + A12.8 |
| `GET` | `/v1/collaboration/canvases/[id]/comments?element_id=` | 列表评论 (per A12.5) | BFF 校验 view 权限 | 本 SRS A12.5 |
| `POST` | `/v1/collaboration/canvases/[id]/comments/[cid]/thread` | 回复评论 (per A12.5) | BFF 校验 ≥ comment 权限 + audit 1 行 | 本 SRS A12.5 + A12.8 |
| `POST` | `/v1/notifications/@` | @ 提醒 (per A12.5) | 25 module notification 域对接 | 本 SRS A12.5 |
| `GET` | `/v1/collaboration/canvases/[id]/multi-user-audit` | 多人编辑 audit log 查询 (per A12.8) | BFF 校验 view 权限 + 100% RLS 13 类必携 | 本 SRS A12.8 |
| `GET` | `/v1/collaboration/canvases/[id]/permissions?user_id=` | 查询权限 (per A12.7) | BFF 校验 view 权限 | 本 SRS A12.7 |
| `POST` | `/v1/collaboration/canvases/[id]/permissions` | 授予权限 (per A12.7) | admin only (5 域 Lead 真人到位后启用) + 守门 #14 v2 真人到位后追溯 | 本 SRS A12.7 |

---

## §9 验收标准 (受入基準 / Acceptance Criteria)

### 9.1 功能验收 (Functional AC, **46 个**)

| AC | 描述 | 测量 | 引用 |
|---|---|---|---|
| AC-A1.1 | 画布上 agent 节点显示 7 字段 (avatar / name / role / kind / status / token_usage / started_at), 元素大小 220x110, 蓝底, 双击跳 `/agent?selected={id}` | 手动 / vitest | A1.1 |
| AC-A1.2 | 14 状态对应 14 色码, 跟 StatusPill 60+ 一致 | 手动 | A1.2 |
| AC-A1.3 | 双击 agent_node 跳 `/agent?selected={id}`, 配置 A9.1 后跳 `/agent-view?agent={id}` | 手动 | A1.3 |
| AC-A2.1 | agent handoff connector 显示时间 / 状态 / 备注 3 字段, bezier 曲线, 颜色按 handoff_status | 手动 | A2.1 |
| AC-A2.2 | 5 Frame 标题固定 player/economy/match/social/admin, 域内 agent 自动聚类, 跨域 handoff 用 connector 跨 Frame | 手动 | A2.2 |
| AC-A2.3 | 父子 agent 关系用 orthogonal connector 树形显示, parent 在上, child 在下, 多 child 自动展开 | 手动 | A2.3 |
| AC-A2.4 | agent pipeline horizontal 显示, A → B → C 直线, 共享 worktree 高亮 | 手动 | A2.4 |
| AC-A3.1 | agent 状态变化 ≤ 200ms (P95) 反映到 `agent_node` 色码 | 手动 / Prometheus | A3.1 |
| AC-A3.2 | agent 状态变化必记 audit, 100% 覆盖, 可追溯 | 手动 | A3.2 |
| AC-A3.3 | agent 状态变 failed 时, 通知到所属域 Lead, 画布上 `agent_node` 红色边框 + 抖动动画 (≤ 500ms) | 手动 | A3.3 |
| AC-A4.1 | 1 agent 关联 N worktree 在画布上 圆周散开, 双向 connector, worktree 状态实时联动 | 手动 | A4.1 |
| AC-A4.2 | worktree 状态变化 ≤ 200ms (P95) 反映到 agent_node 色码联动 | 手动 | A4.2 |
| AC-A5.1 | drag work-item 到 agent_node 周围, 1:N 关联, 双向 connector, work-item 状态实时联动 | 手动 | A5.1 |
| AC-A5.2 | work-item 状态变化 ≤ 200ms (P95) 反映到 agent_node 状态聚合 (badge: in_progress 3 / done 5 / blocked 1) | 手动 | A5.2 |
| AC-A6.1 | 右键 agent_node 弹菜单, 启停操作 权限校验 + 调用 API + 状态实时反映 (≤ 1s) | 手动 | A6.1 |
| AC-A6.2 | 右键菜单 → "重启" 操作成功, 状态从 running → spawning → initializing 反映 | 手动 | A6.2 |
| AC-A6.3 | 右键菜单 → "查看 logs" 跳 agent-runtime logs 页面 | 手动 | A6.3 |
| AC-A6.4 | 右键菜单 → "查看 settings" 跳 V0.1 `/agent-settings?selected={id}` | 手动 | A6.4 |
| AC-A7.1 | 点击 agent_node 弹 detail panel, 4 字段实时 (≤ 1s 刷新), 走 StatusPill 60+ 色码 | 手动 | A7.1 |
| AC-A7.2 | agent node badge 显示 `已用 / 预算` 比例, 超 100% 红色高亮 | 手动 | A7.2 |
| AC-A7.3 | token 超预算 / runtime 异常 → 通知 + 画布高亮 (≤ 500ms) | 手动 | A7.3 |
| AC-A8.1 | 顶部 dropdown "聚类: role" 选项, 画布上同 role agent 用 Frame 子分组 | 手动 | A8.1 |
| AC-A8.2 | 顶部 dropdown "排序: kind" 选项, 画布上 agent 按 kind ASC 稳定排序 | 手动 | A8.2 |
| AC-A8.3 | 顶部 dropdown "过滤: status/token/started_at" 多选, 画布上只显示符合条件 agent | 手动 | A8.3 |
| AC-A9.1 | 双击 agent_node 配置走 `/agent-view?agent={id}`, 双向跳成功 | 手动 | A9.1 |
| AC-A9.2 | 双击 agent_node → 切 tab 跳 `/agent-relationships?agent={id}`, 显示该 agent 所有关系 | 手动 | A9.2 |
| AC-A10.1 | 画布 detail panel 顶部 "Settings" tab, 集成 V0.1 `AgentSettingsTab.tsx`, 改 role → 画布自动重聚类 | 手动 | A10.1 |
| AC-A10.2 | 画布右键 → "编辑 settings" 改 role / kind / token_budget, 实时反映到画布聚类 / 仪表 | 手动 | A10.2 |
| **AC-A11.1** | **10 类关系边在画布上正确渲染, 颜色区分清晰, 边箭头方向按 direction (directed/undirected)** | 手动 | **A11.1** |
| **AC-A11.2** | **拖拽 2 agent → 弹 modal → 填 3 字段 → 提交 → Memgraph 持久化 + 画布实时反映 (≤ 500ms)** | 手动 | **A11.2** |
| **AC-A11.3** | **4 维度指示器在边 label 上正确显示, 数值走 Memgraph 实时读 (≤ 1s), 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.3 4 维度一致** | 手动 | **A11.3** |
| **AC-A11.4** | **5 模板 dropdown 可见, 1-click 实例化成功, 1 事务写 Memgraph, 画布实时反映 (≤ 1s), 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.5 + UC-05 一致** | 手动 | **A11.4** |
| **AC-A11.5** | **关系增删改 100% audit, version +1, 4 表 W/T/M 100% 覆盖 (per 守门 #13), 改关系可回滚** | 手动 + DB schema 验证 | **A11.5** |
| **AC-A11.6** | **边粗细按 weight 缩放, 颜色渐变灰→绿, 信任度动态调整 weight (成功 +0.01 / 失败 -0.05)** | 手动 | **A11.6** |
| **AC-A11.7** | **关系右键菜单 → "归档" 改 `archived=true`, 画布上变灰半透明, 不影响协作; "恢复" 改回 `archived=false`, 物理删除 0 次** | 手动 + DB schema 验证 | **A11.7** |
| **AC-A11.8** | **关系改一次 version 自动 +1, 并发冲突 UI 提示 + 不覆盖旧版本, 改关系可回滚 (per audit log)** | 手动 + 冲突测试 | **A11.8** |
| **AC-A11.9** | **画布右上角 status badge 4 状态正确显示, 离线降级 走 in-process 缓存, 跟 `SRS-AGENT-RELATIONSHIP-001.md` §4.4 + UC-01 一致** | 手动 + 离线测试 | **A11.9** |
| **AC-A11.10** | **画布 "Relationship" tab 切换成功, 跳 `/agent-relationships?from=canvas`, 3 ARG UI 组件跟本画布 `agent_node` 数据双向同步** | 手动 | **A11.10** |
| **AC-A12.1 (v1.2 新增)** | **10 个用户同时编辑同一 canvas, 元素增删改 ≤ 200ms (P95) 反映到其他用户, WSS 连接稳定, envoy 独立 deployment, BFF 推 element 增删改, 跟 `frontend-canvas-design.md` §4.1 模式 A + §6.2 端点 `wss://canvas-collab/canvases/[id]` 一致** | 手动 + WSS latency 监控 | **A12.1** |
| **AC-A12.2 (v1.2 新增)** | **10 个用户同时编辑, 每人 cursor ≤ 100ms 显示在其他用户 viewport 上, cursor 颜色按 user_id 哈希分配 (12 色调色板), 名字 + viewport + selection 同步, 跟 V0.1 §4.6 PresenceCursor 字段 + 总册 §6.2.2 端点一致** | 手动 + WSS latency 监控 | **A12.2** |
| **AC-A12.3 (v1.2 新增)** | **1 用户拖拽 element, 其他用户 ≤ 200ms (P95) 看到 element 移动, backend 持久化成功, F5 刷新保留, V0.1 localStorage 不冲突 (降级为 fallback), 跟 V0.1 §4.1 模式 A + 总册 §6.2.2 端点一致** | 手动 + DB schema 验证 | **A12.3** |
| **AC-A12.4 (v1.2 新增)** | **A 端点 B 端 "Follow" 按钮, A 端 viewport 实时跟 B 端同步 (≤ 100ms P95), 顶部 toolbar 显示 "Following: {B user_name}", "Stop Following" 按钮可退出, 跟总册 §6.2.2 端点 + GitHub Live Share UX 一致** | 手动 + WSS latency 监控 | **A12.4** |
| **AC-A12.5 (v1.2 新增)** | **element 上右键 → "添加评论" → 弹 thread 输入框 → 提交 → 评论显示在 element 旁边, V0.1 💬 emoji + badge 保留; thread 内回复 + @ 用户 → notification 域发到被 @ 用户 inbox, 跟 V0.1 `CanvasView.tsx` line 253-262 `comment_pin` 升级一致 + 总册 §6.2.2 端点 + 25 module notification 对接** | 手动 + notification 域验证 | **A12.5** |
| **AC-A12.6 (v1.2 新增)** | **2 用户同时拖拽同 1 element, 选型 CRDT 自动 conflict-free merge, 不需要锁, 不冲突, 选型后所有 element 自动 conflict-free, 跟 A11 同步桥设计同源** | 手动 + CRDT 选型决策 | **A12.6** |
| **AC-A12.7 (v1.2 新增)** | **外部用户访问 canvas, BFF 校验 view 权限通过, 试图 edit 元素 → 403 Forbidden + UI 提示; 内部 Lead 默认 edit 权限; comment 权限用户可评论不能 edit; 跟总册 §6.2.2 + 守门 #14 v2 真人到位后追溯签字一致** | 手动 + BFF middleware 验证 | **A12.7** |
| **AC-A12.8 (v1.2 新增)** | **任何多人操作 (增删改 / cursor 移动 / 评论 / @) 100% 写到 `canvas_multi_user_audit` 表 (Transaction append-only, SCD Type 2 version +1, 100% RLS 13 类必携, 1 操作 1 audit 行), 物理删除 0 次, 真人 Lead 到位后可追溯, 跟守门 #13 W/T/M 分类 + 总册 §6.2.2 端点 + 25 module audit 域对接一致** | 手动 + DB schema 验证 | **A12.8** |

**AC 数量统计**: **46 个** (≥ 46 满足), 涵盖 46 项需求 1:1

### 9.2 质量验收 (Quality AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-Q-1 | vitest 30+ pass (3+ 测试文件, layout / selectors / AgentCanvasView / ArgCanvasView / **MultiUserCanvasView v1.2 新增**) | `pnpm test --run src/lib/canvas src/components/canvas` |
| AC-Q-2 | typecheck 0 err (本 SRS 新增的 5+ 个文件) | `tsc --noEmit` |
| AC-Q-3 | 不引入新依赖 (除 ARG 4 文档链引用的 4 份不新增 dep; **A12 CRDT 选型拍板后可能新增 Yjs / Automerge / LWW 库**) | `package.json` diff (空) |
| AC-Q-4 | commit author = Ulysses | `git log --format='%an <%ae>' HEAD` |
| AC-Q-5 | 7 段报告落档 (per AGENTS.md §3) | `docs/reports/PHASE-CANVAS-AGENT-IMPL-REPORT.md` 存在 (实装阶段) |
| AC-Q-6 | 派生确定性 (NFR-AGENT-DET-01) | vitest "排序稳定" / "deterministic" 2 个测试 pass |
| AC-Q-7 | 派生纯函数 (NFR-AGENT-DET-01) | 5+ 个函数 vitest pass |
| AC-Q-8 | 守门 #13 4 表 W/T/M 100% 覆盖 (per A11.5) | DB schema 审查 |
| AC-Q-9 | ARG 10 类关系颜色规范一致 (per A11.1) | 人工 review |
| AC-Q-10 | 守门 #14 v2 5 域 Lead 真人未到位 Mavis 临时代签 + 修订历史追溯签字 | 修订历史表 |
| **AC-Q-11 (v1.2 新增)** | 守门 #13 A12 7 表 W/T/M 100% 覆盖 (per A12.8, 含新增表 `canvas_multi_user_audit` Transaction append-only + SCD Type 2 + 100% RLS 13 类) | DB schema 审查 |
| **AC-Q-12 (v1.2 新增)** | 守门 #13 + v0.63 反转行 显式标 (per 守门 #1 禁回溯叙事, §0.2 v1.2 + §4.12 A12 段头 + §7.3 + §12 修订历史 v1.2 row) | 修订历史表 + 文档 § 标识 |
| **AC-Q-13 (v1.2 新增)** | A12 BFF 走 envoy 独立 deployment (per 9/1 13:03+13:05 JST 偏好, TC-14) | k3s deployment yaml 验证 |
| **AC-Q-14 (v1.2 新增)** | A12 WSS connection 走 TLS 1.3+ + auth required (per SC-7) | WSS handshake 验证 |

### 9.3 文档验收 (Documentation AC)

| AC | 描述 | 测量 |
|---|---|---|
| AC-D-1 | 本 SRS (要件定義書) 落档 | `docs/requirements/SRS-CANVAS-AGENT-001.md` 存在 (v1.2) |
| AC-D-2 | BD (基本設計書) 落档 (P3-D 阶段) | `docs/design/BD-CANVAS-AGENT-001.md` 存在 (待落档) |
| AC-D-3 | DD (詳細設計書) 落档 (P3-D 阶段) | `docs/design/DD-CANVAS-AGENT-001.md` 存在 (待落档) |
| AC-D-4 | 実装報告 (7 段) 落档 (P3-D 阶段) | `docs/reports/PHASE-CANVAS-AGENT-IMPL-REPORT.md` 存在 (待落档) |
| AC-D-5 | self-review 落地报告 (P3-D 阶段) | `docs/reports/PHASE-CANVAS-AGENT-SELF-REVIEW.md` 存在 (待落档) |
| **AC-D-6 (v1.2 新增)** | A12 多人编辑 CRDT 选型决策文档落档 | `docs/design/CRDT-SELECTION-DECISION.md` 存在 (实装阶段) |

---

## §10 已知风险 / 未解決問題 (Known Issues / 缺口)

> per 守门 #11 缺标比错标, 显式列已知缺口, 不隐藏

| # | 风险 / 缺口 | 影响 | 缓解 / 后续 |
|---|---|---|---|
| **1** | A1.1 `agent_session.avatar_url` 字段当前 store 缺 | 完整卡头像暂时占位, 后续 DDD Review 加 | P3-D DDD Review 拍板 |
| **2** | A2.2 `agent_session.domain` 字段当前 store 缺, V0.1 仅 1:1 | 5 域 Frame 分组 暂时走 store 派生 (per `agent_session.role` 推断 domain) | P3-D DDD Review 加 domain 字段 |
| **3** | A2.3 `agent_session.parent_session_id` 字段当前 store 缺 | 父子关系暂时走 mock, 后续 DDD Review 加 | P3-D DDD Review 拍板 |
| **4** | A2.4 `worktree.pipeline_agent_ids[]` + A4.1 `worktree.agent_session_ids[]` (替代 V0.1 1:1) + A5.1 `work_item.agent_session_id` (per `SRS-AGENT-VIEW-001.md` §10 缺口 #4) 字段当前 store 缺 | 1:N 关联暂时走 mock, 后续 DDD Review 加 | P3-D DDD Review 拍板 (跟 `SRS-AGENT-VIEW-001.md` §10 缺口 #4 同步) |
| **5** | A3.1 实时状态同步 WebSocket 选型未拍板 (跟原 COLLAB 专题同源) | 暂走 polling 30s fallback, 状态变化延迟 P95 > 200ms | P3-D 拍板后启动 WebSocket 集成 |
| **6** | A3.3 / A6.1-6.2 / A7.3 / **A12.7 view/comment/edit 3 级权限** 5 域 Lead 真人未到位, 操作权限 / 通知路由 暂走 Mavis 临时代签 | 真人到位后追溯签字覆盖 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事) | 真人到位时追溯 |
| **7** | A7.2 `agent_session.token_budget` 字段当前 store 缺, V0.1 仅记录 `token_usage` | 预算对比暂时走 mock (默认 1.2M / SRE·周 per `STAR-OLU-001.md` v0.1) | P3-D DDD Review 加 token_budget 字段 |
| **8** | A9.2 / A11.10 `/agent-relationships` 路由待 ARG UI 实装阶段落档 (per `BD-AGENT-RELATIONSHIP-001.md` §2.2 Tier 1) | "Relationship" tab 暂时 disable + 占位提示 | P3-C ARG UI 实装阶段落档后激活 |
| **9** | A11.1 `arg_edge` element kind V0.1 schema 缺, 需新增 (per `BD-AGENT-RELATIONSHIP-001.md` §5.2) | ARG 边渲染暂时走 mock, 后续 ARG 实装阶段加 schema | P3-C ARG 实装阶段 |
| **10** | A11.2 / A11.3 / A11.4 / A11.6 / A11.9 ARG 后端 5 维度 effect tier 模块 (`ARGDispatchRouter` / `ARGContextInjector` / `ARGTrustEngine` / `ARGOutputEvaluator` / `ARGAchievementEngine` per `BD-AGENT-RELATIONSHIP-001.md` §2.2 Tier 5) 实装待 P3-C 阶段 | ARG 关系建边 / 4 维度影响 / 5 模板 / 信任度 / 同步桥 暂时走 mock, 后续 ARG 实装阶段激活 | P3-C ARG 实装阶段 |
| **11 (v1.2 新增)** | A12.1 多人同时编辑 WebSocket 选型未拍板 (候选 NATS JetStream / native WebSocket / Socket.IO), 跟 A3 + A11 同步桥同源, **P0 阻塞** (per A12.1 + A12.6) | A12.1 + A12.3 多人编辑 Realtime 通道实现依赖选型 | P3-C/D 阶段拍板, 拍板后落 `docs/design/REALTIME-CHANNEL-SELECTION-DECISION.md` |
| **12 (v1.2 新增)** | A12.2 PresenceCursor V0.1 design 已落档, A12 实装扩展待 P3-C/D 阶段 | 多人 cursor 暂时走 in-process mock | P3-C/D 实装阶段 |
| **13 (v1.2 新增)** | A12.3 V0.1 localStorage + zustand persist 跟多人编辑冲突, 实施时需重构持久化层, **P0** (per BR-14) | V0.1 持久化逻辑降级为离线 fallback, A12 主数据走 backend 持久化 | P3-C/D 实装阶段重构 |
| **14 (v1.2 新增)** | A12.5 V0.1 `comment_pin` 渲染保留, thread + @ 数据结构 + 25 module notification 域对接实装待 P3-C 阶段 | 评论暂时只支持 1 顶级, 不支持 thread + @ | P3-C 实装阶段 |
| **15 (v1.2 新增)** | A12.6 **CRDT 选型未拍板** (候选 Yjs / Automerge / LWW), 跟 ARG 同步桥同源, **P0 阻塞** (per A12.6) | 2 用户并发改同 1 element 暂时走 view-only 兜底 (per A12.7) | P3-C 阶段拍板, 拍板后落 `docs/design/CRDT-SELECTION-DECISION.md` |
| **16 (v1.2 新增)** | A12.7 具体权限矩阵 (哪个 user 哪个 role) 待 5 域 Lead 真人到位后决策, 拍板前走 view-only 兜底 (外部用户仅 view, 内部 Lead 默认 edit, per BC-9), 25 module 联动待 DDD Review 拍板 | view-only 兜底期间外部用户功能受限 | 真人到位时追溯 + 拍板 |
| **17 (v1.2 新增)** | A12 WSS 选型 + CRDT 选型 + 5 域 Lead 真人到位 + 25 module 联动 = 4 跨 session 续做 P0 阻塞 (per brief §3) | A12 跨 session 续做需先解决 4 阻塞 | P3-C/D 阶段拍板 |
| **18** | A11 跨专题 5 缺口 (per brief §3 + brief §7 返报 #6): 5 域 Lead 真人未到位 (per守门 #14 v2) + Memgraph 部署 (port 7687 Bolt + 7444 HTTP, 数据卷持久化, per `SRS-AGENT-RELATIONSHIP-001.md` §3.2 PR-4) + L0 ↔ L1 通信协议 + ARG 集成 (per `SRS-AGENT-RELATIONSHIP-001.md` G-3) + TMO 9 节点 边界 梳理 (per `SRS-AGENT-RELATIONSHIP-001.md` G-9) + 跟 TMO 9 节点边界 (per G-9) | A11 跨 session 续做 5 域 Lead + Memgraph 部署 + L0↔L1 通信 + TMO 边界待 DDD Review + P3-C ARG 实装阶段 | P3-C ARG 实装阶段 + DDD Review 拍板 |
| **19** | 当前 store 是 in-memory + zustand persist (localStorage); 多用户多 session 共享状态不可见 | 实际跨 session 协同走后端 (D.6+ backend); **A12 实施时 V0.1 持久化层重构 (per 缺口 #13)** | 当前 SPA 模式可接受, D.6+ 接入真实 data plane |

**DDD Review 必查**: 缺口 #1 + #2 + #3 + #4 + #7 (schema gap 5 字段) + #11 (A12 WSS 选型) + #15 (A12 CRDT 选型) + #16 (A12 权限矩阵) + #17 (A12 4 跨 session 阻塞) + #18 (A11 跨专题 5 缺口)

**已知缺口统计**: **19 个** (≥ 8 满足, 含 A11 跨专题 5 缺口 + **A12 多人编辑 8 缺口 #11-#18 加 1 个 #17 跨 session 总结 = 8 个**)

---

## §11 签字栏 (5 角色 per AGENTS.md §3 7 段结构 + 守门 #14 v4 反转 v0.62)

| 角色 | 签字 | 日期 | 备注 |
|---|---|---|---|
| **架构** | 🟢 Mavis 接手 (per DEC-008) | 2026-09-10 | 8/27 19:39 JST 用户授权代签 + 守门 #14 v3 Mavis 永久代签 + 守门 #14 v4 反转 v0.62 真人代签流程全部取消, 改为 Mavis 审核 author=Ulysses |
| **SRE Lead** | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| **平台** | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| **评审主持** | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |
| **PM** | 🟢 Mavis 接手 (per 守门 #3 反转 8/21 + 9/3 11:35 JST 拍板 B) | 2026-09-10 | 5 域真人 Lead 到位前 Mavis 临时代签 |

**真人到位后追溯签字覆盖** = 修订历史表 +1 行 (per §12 + 9/3 19:35 JST 拍板 D 维持 + 守门 #14 v2), **不沿用代签决策** (per 守门 #1 禁回溯叙事)

---

## §12 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v1.0** | 2026-09-10 17:17 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | 初版, 12 段 (文档信息/目的/用语/前提/业务需求/约束/场景/数据/接口/验收/风险/签字), 28 项 (A1-A10, 10 子能力), 聚焦 agent 管理域 | 2026-09-10 17:08 JST Ulysses 拍板"管理 agent 和游戏化, 避免过度冗余" |
| **v1.1** | 2026-09-10 17:22 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | **重写: 28 项 → 38 项, 新增 A11 ARG 图论构造 10 项** (引用 `SRS-AGENT-RELATIONSHIP-001.md` v0.1 §1-§8 + `BD-AGENT-RELATIONSHIP-001.md` v0.1 §1-§7 + `DD-AGENT-RELATIONSHIP-001.md` + `DDD-REVIEW-AGENT-RELATIONSHIP-001.md`), 38 项 1:1 展开 (FR-AGENT-A1.1-A11.10 + NFR-AGENT-12 项 + AC-AGENT-38 项), 25 用户故事 (≥ 23 满足), 12 已知缺口 (含 A11 跨专题 5 缺口), A11.5 必含 4 表 W/T/M 三類横展 (per 守门 #13, 100% 表覆盖), 守门 #1+#3+#5+#6+#7+#9+#10+#13+#14 v2+#14 v4+#15+#19+#23 v2+#1 v15 14 项全过 (文档工作, 守门 #1 v25 cargo test --workspace -j 4 不需要跑) | 2026-09-10 17:21 JST Ulysses 补充"画布内体现 agent 之间关系的图论构造" (ARG 落档触发重写) |
| **v1.2 (当前)** | 2026-09-10 17:34 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3 + 守门 #14 v4 反转 v0.62) | **v0.63 反转: 重写 38 项 → 46 项, 新增 A12 多人编辑 8 项** (FR-AGENT-A12.1-A12.8 + NFR-AGENT-PERF-04/05/06 + NFR-AGENT-MU-CONS-01 + AC-AGENT-A12.1-A12.8 + UC-A11/UC-A12/UC-A13/UC-A14/UC-A15 + 缺口 #11-#17 共 7 个 + 已知缺口统计 19 个 ≥ 8 满足), **撤回 2026-09-10 17:08 JST 砍多人编辑决定** (per 17:34 JST Ulysses 拍板"**多人编辑是要的**", v0.63 反转行**显式标** per 守门 #1 禁回溯叙事, 出现在 §0.2 v1.2 + §1.4 + §3.3 BR-12/BR-13 + §4.12 A12 段头 + §7.3 A12.8 + §10 #11-#17 + 本 §12 v1.2 row), 引用 `frontend-canvas-design.md` §4.1 模式 A Realtime 通道 (A12.1 + A12.3) + §4.6 PresenceCursor 升级 (A12.2) + `CanvasView.tsx` line 253-262 `comment_pin` (A12.5); A12.8 必含新增表 `canvas_multi_user_audit` 7 表 W/T/M 必含 (per 守门 #13, 含 `canvas_multi_user_audit` Transaction append-only + SCD Type 2 + 100% RLS 13 类); 32 用户故事 (≥ 28 满足, 46 × 60% = 27.6); 16 NFR (含 v1.2 新增 PERF-04/05/06 + MU-CONS-01); 46 AC (1:1 覆盖 46 项); 守门 #1+#3+#5+#6+#7+#9+#10+#13+#14 v2+#14 v4+#15+#19+#23 v2+#1 v15+#1 v25+#7 v3+#1 v26+#24 v2+#6 v2 19 项全过 (文档工作, 守门 #1 v25 cargo test --workspace -j 4 不需要跑); 5 域 Lead 真人未到位前 Mavis 临时代签 (per 守门 #14 v2 + 9/3 11:35 JST 拍板 B + 9/5 10:43 JST 拍板 D, **不沿用代签决策** per 守门 #1 禁回溯叙事) | 2026-09-10 17:34 JST Ulysses 拍板"**多人编辑是要的**" (v0.63 反转) + 总册 `SRS-CANVAS-001.md` v1.1 三次更新版同步 (v0.63 反转行 显式标 + 78 项 = 46 + 32) |
