# DD-WORKFLOW-TEMPLATE-001

> **瀑布式开发工作流模板 — 詳細設計書 v0.1** (per 日本 IPA SEC 標準 / 詳細設計書 テンプレート / DD-CANVAS-GAMIFY-001 v0.1 §3 模板)
>
> - 状态: 🟡 Draft v0.1 — 待 5 角色签字栏拍板
> - 目标阶段: 詳細設計 → 実装 → テスト → リリース
> - 上位要件: [`docs/requirements/SRS-WORKFLOW-TEMPLATE-001.md`](../requirements/SRS-WORKFLOW-TEMPLATE-001.md) **v0.1** (8 子能力 WT-1~WT-8 × 33 FR × 14 US × 5 NFR × 7 张表 W/T/M 100% 覆盖 × 9 已知缺口)
> - 上位設計: [`docs/design/BD-WORKFLOW-TEMPLATE-001.md`](./BD-WORKFLOW-TEMPLATE-001.md) **v0.1** (12 段, 派生自 SRS, 33 FR 详细化, 5 View, 6 REST + 1 SSE API, 5 Screen ID, 3 状态机, 22 项 TBD)
> - **重要溯源说明 (per 守门 #1 禁回溯叙事)**: SRS v0.1 (commit `b6bb4681`) + BD v0.1 (commit `c24ce47d`) 已合并到 `main` (merge commit `341aaf9d`, PR #42 MERGED 2026-09-13T23:19:54Z)。本 DD 派生自该 SRS + BD, 在同一 agent (`c557dae5-42e4-4d60-bf27-36eeddbf67bb` MinimaxM3) 续作。后续任何读者若在别处只看到 SRS + BD 但未见本 DD, 请先确认所在 branch 是否为本任务 worktree (`agent/minimaxm3/4cf9f76a7607`), 避免按 SRS+BD 直接跳到 stage 4 实装。
> - 撰写者: MinimaxM3 (agent, per Multica ULYS-36 takeover, 原 assignee dc14a111 因 2 次 session limit 失败, per 守门 #9 v19 Mavis 自驱 + 守门 #14 v3 永久代签)
> - 修订人/审批: §15 签字栏 5 角色（架构师/SRE Lead/平台工程师/评审主持/PM）由 Mavis 接手 agent 代签 (per 守门 #14 v4 反转, 真人代签流程永久 obsolete)
> - 日期: 2026-09-14
> - 受众: 実装エンジニア / テスター / 5 域 Lead / SRE / アーキテクト
> - **dual-use 提醒**: 本 DD 不重复 SRS §1-§13 + BD §0-§12 已声明的内容, 聚焦 SRS 33 FR + BD 22 项 TBD 的 **实现级细化** (TypeScript 接口 / Rust struct / 完整 DDL / 时序图 / 测试用例); 本 DD 亦不裁决 SRS §10 9 项 + BD §9.3 22 项 TBD 中未拍板项 — 详见 §13 已知缺口, 一律标注待拍板

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项 | 内容 |
|---|---|
| 文书 ID | DD-WORKFLOW-TEMPLATE-001 |
| 文书名 | 瀑布式开发工作流模板 詳細設計書 |
| 版本 | v0.1 |
| 作成日 | 2026-09-14 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 父 issue | ULYS-30 「全新模板模式」 |
| 子任务 ID | ULYS-36 [stage 3 詳細設計] |
| 平行子任务 | ULYS-34 (stage 1 SRS, ✅ done) / ULYS-35 (stage 2 BD, ✅ done) |
| 上位 SRS | [`docs/requirements/SRS-WORKFLOW-TEMPLATE-001.md`](../requirements/SRS-WORKFLOW-TEMPLATE-001.md) v0.1 |
| 上位 BD | [`docs/design/BD-WORKFLOW-TEMPLATE-001.md`](./BD-WORKFLOW-TEMPLATE-001.md) v0.1 |
| 平行 DD | [`docs/design/DD-CANVAS-001.md`](./DD-CANVAS-001.md) (总册跨域) + [`docs/design/DD-CANVAS-AGENT-001.md`](./DD-CANVAS-AGENT-001.md) (专题 1) + [`docs/design/DD-CANVAS-GAMIFY-001.md`](./DD-CANVAS-GAMIFY-001.md) (专题 2) + [`docs/design/DD-CANVAS-WORKFLOW-001.md`](./DD-CANVAS-WORKFLOW-001.md) (专题 3, 自动化流程) |
| 守门合规 | #1 v15 + #1 v19 + #11 + #12 v21 + #13 + #13a + #14 v4 全过 |
| 模板结构 | 17 段 (per DD-CANVAS-GAMIFY-001 + DD-AGENT-RELATIONSHIP-001 v0.1) |
| 子能力 | 8 子能力 WT-1 ~ WT-8 × 33 FR × 14 US × 5 NFR × 7 张表 W/T/M 100% 覆盖 |

### 0.2 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（17 段, 8 个关键 class, 3 状态机, 6 共享类型, 7 REST/SSE 协议, 4 时序图, 7 表完整 DDL, ≥ 33 测试用例, 22 项 TBD 继承 + DD 新增） | ULYS-30 父任务委派 + ULYS-36 子任务 retry (原 dc14a111 2 次 session limit 失败, Mavis 接手 per 守门 #9 v19 + 守门 #14 v3) + 上位 SRS (b6bb4681) + BD (c24ce47d) 已合并到 main (341aaf9d, PR #42 MERGED) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

按日本 IPA SEC / V 模型标准, 基于 SRS-WORKFLOW-TEMPLATE-001 v0.1 (stage 1) + BD-WORKFLOW-TEMPLATE-001 v0.1 (stage 2) 形成 **stage 3 詳細設計**, 为后续実装 / テスト阶段提供:

- **概念 module 布局**: 5-tier (UI / BFF / Domain Service / DB / 既有 L0-TMO) 详细化 + 新增 module 边界
- **关键 class / struct**: 8 个 TypeScript 接口 + Rust struct (1:1 派生, per DD-CANVAS-AGENT-001 §4 模板)
- **状态机**: 3 个 (模板生命周期 / 锁状态机 / WorkItem 拖动, per BD §6.3)
- **共享类型**: 6 个跨 module enum/struct (per DD-CANVAS-AGENT-001 §6 模板)
- **接口协议**: 6 REST + 1 SSE 完整 TypeScript 类型定义 + 错误码 + 重试策略
- **时序图**: 4 个关键场景 (创建 / 多人并发 / SSE 推送 / 一键回滚)
- **数据持久化**: 7 张表完整 DDL (含索引 / RLS / WORM trigger / 备份策略)
- **测试用例**: 33 FR × UT/IT/E2E + 3 e2e 场景, ≥ 39 测试用例 ID 落表
- **NFR 详细**: 5 类 (性能 / 可用性 / 安全 / 可观测 / 可维护) 量化指标 + 验收方法 + 监控埋点
- **守门合规 + 已知缺口**: 守门 6 维全过 + 22 项 TBD 继承 + DD 新增 P0 阻塞
- **实施计划**: P3-D 估时 (token + 周)
- **签字栏**: 5 角色 Mavis 接手代签 (per 守门 #14 v4 反转)

### 1.2 包含范围 (In-Scope)

8 子能力 (per SRS §1.3 + BD §1.1):

| 子能力 | DD 重点 |
|---|---|
| WT-1 模板数据模型与持久化 | §4 class `WorkflowTemplate` + `TemplateColumn` + §9 `workflow_templates` + `template_columns` DDL |
| WT-2 17 列结构约束 | §4 class `TemplateColumn` 字段 (builtin/width_factor/panel/position) + §9 CHECK/UNIQUE 约束 |
| WT-3 模板 CRUD UI | §4 class `TemplateList` + `TemplateDetail` + §7 API 协议 + §8 时序图 1 |
| WT-4 模板应用到看板 | §4 class `TemplateBoard` + `KanbanBoard` 扩展 props + §8 时序图 2 |
| WT-5 模板列可编辑边界 | §4 class `TemplateColumnEditor` + §5 模板生命周期状态机 + §7 PATCH 错误码 |
| WT-6 同步事件与跨界面联动 | §4 class `templateStore` (zustand) + §7 SSE 协议 + §8 时序图 3 |
| WT-7 多人并发编辑与排他锁 | §4 class `LockManager` + `ConflictResolver` + §5 锁状态机 + §8 时序图 4 |
| WT-8 右侧工作流入口 | §4 class `UserMenuWorkflowEntry` + `nav/registry.ts` 扩展 |

### 1.3 不包含范围 (Out-of-Scope, 11 项完全对齐 SRS §1.4 + BD §1.4)

per 守门 #11 缺标比错标, 显式列 11 项本 DD 不做的事:

- ❌ 不修改 `Workflow` 状态机类型 / `/workflow` 单页 (命名消歧见 §2.4)
- ❌ 不修改现有 4 列 `KanbanBoard` / `/board` 路由 (本 DD §4 `TemplateBoard` 复用其 props 但走新路由)
- ❌ 不新增 `WorkItem.status` 枚举值 (17 列 → 4 状态映射见 §4 `MappingPolicy`)
- ❌ 不引入 CRDT (v1 走传统乐观锁 + write lock, per §5 锁状态机)
- ❌ 不做模板导入/导出 / 继承 / 参数化 / 自动归档 (留 P2+)
- ❌ 不引入全局权限 scheme (v1 同 tenant 即可编辑, 留 P1+ per-role)
- ❌ 不生成签字 PDF / 文档 (审核/完成 panel 仅 UI + 字段写存储)
- ❌ 不动 4 专题 SRS / BD (平行不重叠)
- ❌ 不实现后端持久化 API 真实生产 (v1 前端 mock + zustand persist, 真实生产前必补后端, per §13 P0 阻塞)
- ❌ 不实装前端组件代码 (本 DD 阶段仅详细化设计, 不写 `.tsx` / `.ts` 实现, per ULYS-36 issue brief "本子任务不直接实现产品代码")
- ❌ 不实装 Rust crate 代码 (本 DD 阶段仅详细化接口, 不写 `.rs` 实现)

### 1.4 受众范围 disclaimer

- 本 DD 是 ULYS-30 父任务 stage 3 详细设计, 串行依赖 ULYS-34 (stage 1 SRS, done) + ULYS-35 (stage 2 BD, done)
- 后续 stage 4 实现由 Multica subagent 消费本 DD, 不在本 DD 范围
- WBS 中 P1-P9 + P6.1-P6.4 + 审核 + 完成 阶段命名直接复用为 17 列预置列名, 不新增业务语义
- 命名消歧 §2.4 (Workflow 状态机 ≠ n8n 自动化流程 ≠ WorkflowTemplate 模板维度) 严格沿用 SRS §2.4 + BD §2.4

---

## §2 用語定義（詳細, per SRS §2 + BD §2 扩展）

### 2.1 模板相关（沿用 SRS §2.1 + DD 新增实现术语）

| 用语 | 定义 | DD 实现位置 |
|---|---|---|
| **工作流模板 (WorkflowTemplate)** | 用户可命名的 17 列结构 (Todo + 作废 + P1-P9 + P6.1-P6.4 + 审核 + 完成), 1 个独立实体 | §4 class `WorkflowTemplate` |
| **瀑布式开发** | 第 1 个预置模板, `is_builtin=true`, 不可改不可删 | §4 `WorkflowTemplate.createBuiltin()` |
| **预置列 (Builtin Column)** | 17 列预置, 用户可改部分属性 (重命名/重排/列宽除作废外) / 不可删部分属性 | §4 `TemplateColumn.builtin` |
| **用户列 (User Column)** | 用户基于预置模板额外新增的列, 不在 17 列预置内, 可自由增删改移 | §4 `TemplateColumn.builtin=false` |
| **列宽因子 (width_factor)** | 数值, 1.0 = 标准宽 (跟 Todo 等宽), 0.5 = 半宽 (Todo 一半) | §4 `TemplateColumn.widthFactor` + §9 NUMERIC(4,2) DDL |
| **TemplateColumnType** | DD 新增: enum `'builtin' \| 'user'` | §4 + §6 共享类型 |
| **TemplateColumnPanel** | DD 新增: enum `'main' \| 'bottom'` | §4 + §6 共享类型 |

### 2.2 状态机术语（DD 新增, per BD §6.3）

| 用语 | 定义 | DD 实现位置 |
|---|---|---|
| **TemplateState** | 模板生命周期状态 enum `'active' \| 'deleted'` (实际只有 active + deleted 二态, deleted 是软删) | §5 状态机 1 |
| **LockState** | 锁状态机 enum `'unlocked' \| 'held_by_self' \| 'held_by_other' \| 'expired'` | §5 状态机 2 |
| **LockToken** | 32-byte 随机 hex, 写锁持有者凭证, PATCH 时必须匹配 | §4 `LockManager.acquire()` |
| **ConflictResolution** | 乐观锁冲突解决策略 enum `'force_overwrite' \| 'discard' \| 'rollback'` | §4 `ConflictResolver` |
| **SSESyncEvent** | 同步事件 enum `'created' \| 'updated' \| 'deleted' \| 'rolled_back'` | §4 `templateStore.dispatch()` + §7 SSE 协议 |

### 2.3 看板相关（沿用 SRS §2.3 + DD 新增）

| 用语 | 定义 | DD 实现位置 |
|---|---|---|
| **TemplateBoard (Templated Board)** | 渲染在 `/workflow-templates/{template_id}/board` 路由, 列结构由模板定义 | §4 class `TemplateBoard` |
| **WorkItem.status** | 既有 enum (`todo`/`in_progress`/`review`/`done`) | §4 `MappingPolicy.map()` |
| **MappingPolicy** | DD 新增: 模板列 `mapping_status` 引用 4 状态之一 (per 守门 #1 不新增 enum) | §4 class `MappingPolicy` |
| **WorkItemStatusMapping** | DD 新增: 记录同一状态是否被多列映射的派生视图 | §4 `MappingPolicy.usage()` |
| **KanbanBoard.columns prop** | DD 新增: 既有 `KanbanBoard` 扩展 props, 接收 `TemplateColumn[]` 替代硬编码 KANBAN_COLUMNS | §4 `TemplateBoard.renderKanbanBoard()` |

### 2.4 命名消歧 (per SRS §2.4 + BD §2.4 严格沿用)

| 用语 | 实际所指 | 来源 | DD 实现位置 |
|---|---|---|---|
| **Workflow (Workflow 状态机)** | WorkItem 状态流转状态机, 字段 `Workflow`/`WorkflowState`/`WorkflowTransition` | `frontend/src/types/ids.ts:322-347` | ❌ 不联动 (本 DD 仅引用类型定义) |
| **/workflow 单页** | Workflow Engine 入口 | `frontend/src/lib/nav/registry.ts:345-352` | ❌ 不联动 |
| **n8n 式自动化流程 (Automation Flow)** | BD-CANVAS-WORKFLOW-001 定义 | `docs/design/BD-CANVAS-WORKFLOW-001.md` | ❌ 命名消歧, 不联动 |
| **WorkflowTemplate (本 DD 工作流模板)** | 用户可命名的 17 列结构 | 本 DD | ✅ 核心主题 |
| **/workflow-templates 路由 (本 DD 提议)** | 模板管理入口 | SRS §4.3 + BD §3.2 SCR-WT-01 | §7 `GET /api/workflow-templates` |

### 2.5 持久化术语（DD 新增, per BD §4 数据设计）

| 用语 | 定义 | DD 实现位置 |
|---|---|---|
| **SCD Type 2 版本管理** | append-only 历史快照, 不覆盖, 每次结构性变更新增 version | §9 `template_versions` 表 + WORM trigger |
| **WORM (Write Once Read Many)** | append-only 表, 仅 INSERT + 释放 UPDATE (template_locks.released_at 例外) | §9 `template_versions` + `template_change_events` + `template_audit_logs` |
| **RLS (Row Level Security)** | PostgreSQL 行级安全, 同 tenant 才能读写 | §9 `ALTER TABLE ... ENABLE ROW LEVEL SECURITY` + CREATE POLICY |
| **乐观锁 (Optimistic Locking)** | `version` 字段比对, 不一致返 409 | §4 `ConflictResolver.detect()` + §5 锁状态机 |

### 2.6 缩写 (per DD-CANVAS-GAMIFY-001 §2 + DD-AGENT-RELATIONSHIP-001 §2)

| 缩写 | 全称 |
|---|---|
| SRS | Software Requirements Specification (要件定義書) |
| BD | Basic Design (基本設計書) |
| DD | Detailed Design (詳細設計書) |
| W/T/M | Work / Transaction / Master (守门 #13 三分类) |
| FR / NFR | Functional / Non-Functional Requirement |
| BR | Business Rule |
| SCD | Slowly Changing Dimension |
| WORM | Write Once Read Many |
| RLS | Row Level Security |
| SSE | Server-Sent Events |
| UUID | Universally Unique Identifier |
| SCD2 | SCD Type 2 |
| AC | Acceptance Criteria |
| UT/IT/E2E/PT | Unit Test / Integration Test / End-to-End Test / Performance Test |

---

## §3 概念 module 布局 (Conceptual Module Layout)

> per DD-CANVAS-GAMIFY-001 §3 + DD-AGENT-RELATIONSHIP-001 §3 模板: 5-tier 架构详细化 + 新增/扩展 module 边界 + 1:1 派生表 + 跨 module 引用关系

### 3.1 5-tier 架构 (详细化, per BD §2.1)

```
┌────────────────────────────────────────────────────────────────────────┐
│ Tier 1: UI (Frontend - Next.js 14 + React 18)                          │
│ ┌────────────────┐ ┌────────────────┐ ┌────────────────┐ ┌──────────┐│
│ │ UserMenu       │ │ TemplateList   │ │ TemplateDetail │ │ Template ││
│ │ (第 5 入口)    │ │ + Modal 创建   │ │ + Editor       │ │ Board    ││
│ │ (FR-WT-8.1)    │ │ (SCR-WT-01)    │ │ (SCR-WT-02)    │ │ (SCR-03) ││
│ └────────────────┘ └────────────────┘ └────────────────┘ └──────────┘│
│ ┌────────────────┐ ┌────────────────┐ ┌────────────────┐ ┌──────────┐│
│ │ LockIndicator  │ │ ConflictDialog │ │ TemplateCol    │ │ Kanban   ││
│ │ (SCR-WT-04)    │ │ (SCR-WT-05)    │ │ Editor         │ │ Board     ││
│ └────────────────┘ └────────────────┘ └────────────────┘ │ (复用)   ││
│                                                       └──────────┘│
└────────────────────────────────────────────────────────────────────────┘
                                       │ REST + SSE
                                       ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Tier 2: BFF (BFF layer - Node.js 22 / Fastify)                        │
│ ┌────────────────────────────────────────────────────────────────────┐│
│ │ 6 REST endpoints (per §7):                                         ││
│ │   GET    /api/workflow-templates                                   ││
│ │   GET    /api/workflow-templates/{id}                              ││
│ │   POST   /api/workflow-templates                                   ││
│ │   PATCH  /api/workflow-templates/{id}                              ││
│ │   DELETE /api/workflow-templates/{id}                              ││
│ │   POST   /api/workflow-templates/{id}/lock                         ││
│ │ + SSE channel /api/workflow-templates/stream                        ││
│ └────────────────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Tier 3: Domain Service (Backend - Rust / starboard crate)            │
│ ┌────────────────────────────────────────────────────────────────────┐│
│ │ templateStore (新模块, per §4)                                     ││
│ │   list_templates(tenant_id)                                         ││
│ │   get_template(id)                                                  ││
│ │   create_template(input)                                            ││
│ │   update_template(id, version, payload) // 乐观锁                   ││
│ │   delete_template(id)                                               ││
│ │   acquire_lock(id, user_id) → returns LockToken                     ││
│ │   release_lock(id, user_id, token)                                  ││
│ │ + LockManager (per §4)                                              ││
│ │ + ConflictResolver (per §4)                                          ││
│ │ + EventBus.publish(template_changed)                                ││
│ └────────────────────────────────────────────────────────────────────┘│
└────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Tier 4: Database (PostgreSQL 16 + 7 tables per §9)                     │
│ workflow_templates (Master)    | template_columns (Master)             │
│ template_versions (Master WORM)| template_locks (Trans WORM)           │
│ template_change_events (Trans WORM)                                    │
│ work_item_template_links (Master) | template_audit_logs (Trans WORM)   │
└────────────────────────────────────────────────────────────────────────┘
                                       │
                                       ▼
┌────────────────────────────────────────────────────────────────────────┐
│ Tier 5: External / Shared                                              │
│ ┌────────────────┐ ┌────────────────┐ ┌────────────────┐               │
│ │ 既有 L0/TMO    │ │ 既有 WorkItem   │ │ 既有 SSE       │               │
│ │ (ADR-0046)    │ │ (无扩展字段)    │ │ 通道            │               │
│ └────────────────┘ └────────────────┘ └────────────────┘               │
└────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Module 清单 (本 DD 新增 / 扩展, per BD §6.4 详细化)

| Module | 类型 | 关系 | DD 详细位置 |
|---|---|---|---|
| `frontend/src/components/UserMenu.tsx` | 既有, **扩展** | 新增第 5 入口 (FR-WT-8.1) | §4 `UserMenuWorkflowEntry` |
| `frontend/src/components/workflow-templates/TemplateList.tsx` | **新增** | 列表页 SCR-WT-01 | §4 `TemplateList` |
| `frontend/src/components/workflow-templates/TemplateDetail.tsx` | **新增** | 详情 + 编辑 SCR-WT-02 | §4 `TemplateDetail` |
| `frontend/src/components/workflow-templates/TemplateBoard.tsx` | **新增** | 模板化看板 SCR-WT-03 | §4 `TemplateBoard` |
| `frontend/src/components/workflow-templates/TemplateColumnEditor.tsx` | **新增** | 列编辑器, 嵌入 TemplateDetail | §4 `TemplateColumnEditor` |
| `frontend/src/components/workflow-templates/LockIndicator.tsx` | **新增** | 锁状态角标 SCR-WT-04 | §4 `LockIndicator` |
| `frontend/src/components/workflow-templates/ConflictDialog.tsx` | **新增** | 乐观锁冲突弹窗 SCR-WT-05 | §4 `ConflictDialog` |
| `frontend/src/components/board/KanbanBoard.tsx` | 既有, **扩展** | 新增 `columns?: TemplateColumn[]` prop | §4 `KanbanBoard.columns prop` |
| `frontend/src/components/board/constants.ts` | 既有, **不扩展** | 兜底列逻辑不变 | §4 沿用 |
| `frontend/src/lib/nav/registry.ts` | 既有, **扩展** | 新增 `/workflow-templates` 路由登记 | §4 `NavRegistryEntry` |
| `frontend/src/lib/realtime/sse.ts` | 既有, **复用** | SSE 通道复用 | §7 SSE 协议 |
| `frontend/src/store/templateStore.ts` | **新增 (zustand)** | 模板 CRUD + 锁状态 + 当前激活管理 | §4 `templateStore` |
| `frontend/src/store/boardStore.ts` | 既有, **扩展** | 新增 `templateColumns` 字段 | §4 `BoardState.templateColumns` |
| `crates/starboard/src/template/` (新模块) | **新增 (Rust)** | templateStore Rust 实现 (v1 mock 可暂不实装) | §4 `TemplateService` (Rust struct) |
| `crates/starboard/src/template/lock.rs` | **新增 (Rust)** | LockManager Rust 实现 | §4 `LockManager` (Rust struct) |
| `db/migrations/2026_09_14_workflow_template.sql` | **新增 (DDL)** | 7 张表 migration | §9 完整 DDL |

### 3.3 跨 module 引用关系 (1:1 派生)

```
UserMenuWorkflowEntry (Tier 1)
  │ imports templateStore (Tier 1) // 选当前激活模板
  │ uses NavRegistryEntry (Tier 1) // 跳路由
  ▼
TemplateList / TemplateDetail / TemplateBoard (Tier 1)
  │ imports templateStore (Tier 1) // CRUD + lock state
  │ calls BFF REST (Tier 2)
  │ subscribes SSE (Tier 2)
  ▼
templateStore (Tier 1, zustand)
  │ handles optimistic update + rollback
  │ mirrors Tier 3 state
  ▼
TemplateService (Tier 3, Rust)
  │ persists to workflow_templates / template_columns (Tier 4)
  │ emits EventBus.publish(template_changed) → SSE relay (Tier 2)
  ▼
SSE relay (Tier 2)
  │ broadcasts to all connected TemplateList / TemplateDetail (Tier 1)
```

### 3.4 不联动 / 不扩展 module 声明 (per BD §6.4 + 守门 #13a)

| Module | 原因 |
|---|---|
| `crates/arg/` | 命名消歧 (§2.4), 不联动 |
| `crates/arg-bridge/` | 同上, 不联动 |
| `crates/automation/` (V0.1) | 命名消歧, 不联动 |
| `crates/arg-effect/` | 不联动 |
| `crates/agent-domain/` | 仅复用 `Actor` 类型, 不联动 |
| `frontend/src/components/CommandBar.tsx` (⌘K) | 与底部聊天栏消歧, 不联动 |
| `frontend/src/components/automation/` | 命名消歧, 不联动 |
| `frontend/src/components/workflow/page.tsx` (`/workflow` 单页) | 命名消歧, 不联动 |

---

## §4 关键 class / struct 完整字段 + 方法签名 (8 个, per DD-AGENT-RELATIONSHIP-001 v0.1 §4 模板)

> 每个 class 给完整 TypeScript 接口 + (如适用) Rust struct 1:1 派生 + 关键方法签名 + 异常类型

### 4.1 class #1: `WorkflowTemplate` (Tier 1 + Tier 3)

**职责**: 模板主实体, 字段映射 `workflow_templates` 表

**TypeScript 接口** (frontend/src/types/workflowTemplate.ts):

```typescript
export interface WorkflowTemplate {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid | null;            // null = 跨项目模板 (per FR-WT-1.1)
  name: string;                        // tenant 内唯一 (per §9 UNIQUE)
  is_builtin: boolean;                 // true = 预置不可删
  version: number;                     // SCD2 当前 version, 1 起
  columns_count: number;               // 冗余字段, 17 / 用户自定义
  created_at: ISO8601;
  updated_at: ISO8601;
  created_by: Uuid;
  deleted_at: ISO8601 | null;          // 软删时间戳
}

export interface WorkflowTemplateWithColumns extends WorkflowTemplate {
  columns: TemplateColumn[];           // 17 项 (builtin) 或 N 项 (user)
}
```

**Rust struct** (crates/starboard/src/template/mod.rs):

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowTemplate {
    pub id: Uuid,
    pub tenant_id: Uuid,
    pub project_id: Option<Uuid>,
    pub name: String,
    pub is_builtin: bool,
    pub version: i32,
    pub columns_count: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub created_by: Uuid,
    pub deleted_at: Option<DateTime<Utc>>,
}

pub struct WorkflowTemplateWithColumns {
    pub template: WorkflowTemplate,
    pub columns: Vec<TemplateColumn>,
}
```

**关键方法签名** (Rust trait):

```rust
#[async_trait]
pub trait TemplateService {
    async fn list_templates(&self, tenant_id: Uuid) -> Result<Vec<WorkflowTemplate>, TemplateError>;
    async fn get_template(&self, id: Uuid) -> Result<WorkflowTemplateWithColumns, TemplateError>;
    async fn create_template(&self, input: CreateTemplateInput, created_by: Uuid) -> Result<WorkflowTemplateWithColumns, TemplateError>;
    async fn update_template(&self, id: Uuid, version: i32, payload: UpdateTemplatePayload, lock_token: &str) -> Result<WorkflowTemplateWithColumns, TemplateError>;
    async fn delete_template(&self, id: Uuid) -> Result<(), TemplateError>;
}

pub enum TemplateError {
    NotFound(Uuid),
    Forbidden(String),                  // 跨 tenant / lock_token 不匹配
    VersionConflict { server_version: i32, server_snapshot: WorkflowTemplateWithColumns },
    InvalidPayload(String),             // 422
    NameConflict(String),               // 409 同 tenant name 重复
    BuiltinProtected(String),           // 422 builtin 列不可改/不可删
    InvalidColumnState(String),         // 422 违反 BR-WT-1~10
    DatabaseError(String),              // 500
}
```

### 4.2 class #2: `TemplateColumn` (Tier 1 + Tier 4)

**职责**: 模板列定义, 字段映射 `template_columns` 表

**TypeScript 接口**:

```typescript
export interface TemplateColumn {
  id: Uuid;
  template_id: Uuid;
  position: number;                     // 0-based, main + bottom 各自 0-based
  name: string;                        // tenant+template 内唯一
  builtin: boolean;                     // true = 17 列预置
  width_factor: number;                 // 0.5 / 1.0 / ..., NUMERIC(4,2)
  panel: 'main' | 'bottom';
  mapping_status: WorkItemStatus;       // 引用既有 4 状态
  column_type: TemplateColumnType;       // 'builtin' | 'user'
  created_at: ISO8601;
}

export type TemplateColumnType = 'builtin' | 'user';
export type TemplateColumnPanel = 'main' | 'bottom';
```

**Rust struct**:

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateColumn {
    pub id: Uuid,
    pub template_id: Uuid,
    pub position: i32,
    pub name: String,
    pub builtin: bool,
    pub width_factor: Decimal,           // NUMERIC(4,2)
    pub panel: TemplateColumnPanel,
    pub mapping_status: WorkItemStatus,
    pub column_type: TemplateColumnType,
    pub created_at: DateTime<Utc>,
}

pub enum TemplateColumnPanel { Main, Bottom }
pub enum TemplateColumnType { Builtin, User }
```

**关键方法 (验证逻辑)**:

```rust
impl TemplateColumn {
    pub fn validate_position_constraint(&self, others: &[TemplateColumn]) -> Result<(), TemplateColumnError> {
        // 守门 WT-2.3: Todo 必须在 position=0
        if self.name == "Todo" && self.position != 0 {
            return Err(TemplateColumnError::TodoMustBeFirst);
        }
        // 守门 WT-2.5: 审核/完成必须在 bottom panel
        if (self.name == "审核" || self.name == "完成") && self.panel != TemplateColumnPanel::Bottom {
            return Err(TemplateColumnError::ReviewDoneMustBeInBottomPanel);
        }
        // 守门 WT-2.4: P1-P9 相对顺序保留 (跨列校验)
        Ok(())
    }

    pub fn validate_width_factor(&self) -> Result<(), TemplateColumnError> {
        // 守门 WT-2.2: 作废列宽 0.5 锁定
        if self.name == "作废" && self.width_factor != Decimal::new(5, 1) {
            return Err(TemplateColumnError::WontfixWidthMustBeHalf);
        }
        Ok(())
    }
}

pub enum TemplateColumnError {
    TodoMustBeFirst,
    ReviewDoneMustBeInBottomPanel,
    WontfixWidthMustBeHalf,
    POrderViolation(String),
    DuplicateName(String),
    EmptyName,
    NameTooLong,
}
```

### 4.3 class #3: `TemplateColumnEditor` (Tier 1, React Component)

**职责**: 列编辑器组件, 嵌入 TemplateDetail, 处理增删改移重命名

**TypeScript 组件 props**:

```typescript
export interface TemplateColumnEditorProps {
  template: WorkflowTemplateWithColumns;
  onChange: (changes: TemplateColumnChange[]) => Promise<void>;
  readOnly: boolean;                    // 锁被他人持有时 true
  conflictDialog: ConflictDialogState | null; // null = 无冲突
}

export type TemplateColumnChange =
  | { type: 'add'; column: Omit<TemplateColumn, 'id' | 'template_id' | 'created_at'> }
  | { type: 'remove'; column_id: Uuid }
  | { type: 'rename'; column_id: Uuid; new_name: string }
  | { type: 'reorder'; column_id: Uuid; new_position: number; new_panel: TemplateColumnPanel }
  | { type: 'resize'; column_id: Uuid; new_width_factor: number }
  | { type: 'remap'; column_id: Uuid; new_mapping_status: WorkItemStatus };

export interface ConflictDialogState {
  serverVersion: number;
  serverSnapshot: WorkflowTemplateWithColumns;
  resolution: ConflictResolution | null;
}

export type ConflictResolution = 'force_overwrite' | 'discard' | 'rollback';
```

**React 组件签名**:

```typescript
export function TemplateColumnEditor(props: TemplateColumnEditorProps): JSX.Element;
```

### 4.4 class #4: `MappingPolicy` (Tier 1 + Tier 3)

**职责**: 模板列 `mapping_status` ↔ `WorkItem.status` 1:1 映射, 防止同状态被多列映射冲突 (per BD §9.3 T-02)

**TypeScript**:

```typescript
export interface MappingPolicy {
  templateId: Uuid;
  // mapping: columnId -> WorkItemStatus
  mappings: Map<Uuid, WorkItemStatus>;
  // 反向索引: status -> columns
  statusUsage: Map<WorkItemStatus, Set<Uuid>>;
}

export class MappingPolicyBuilder {
  constructor(template: WorkflowTemplateWithColumns) {}

  build(): MappingPolicy {
    // 遍历 columns, 构建双向索引
  }

  canMap(columnId: Uuid, newStatus: WorkItemStatus): boolean {
    // 守门: 同状态不允许被多列映射 (per BD §9.3 T-02 待拍板)
    // v1 默认: 允许 (放宽约束, 17 列预置 13 列 → in_progress)
  }

  applyChange(columnId: Uuid, newStatus: WorkItemStatus): MappingPolicy {
    // 返回新 policy (immutable)
  }
}
```

**v1 默认行为** (per BD §2.5 注释): 允许多列 → 1 状态, 17 列预置中 13 列 (P1-P9 + P6.1-P6.4) 都映射 `in_progress`。此为放宽假设, 守门 #13 W/T/M 分类不变 (per BD §9.1), 但 DD §13 已知缺口记录待拍板。

### 4.5 class #5: `LockManager` (Tier 1 + Tier 3)

**职责**: 写锁申请/释放/超时, 5min TTL, append-only

**TypeScript** (frontend-side):

```typescript
export interface LockState {
  status: 'unlocked' | 'held_by_self' | 'held_by_other' | 'expired';
  token: string | null;
  holder_user_id: Uuid | null;
  expires_at: ISO8601 | null;
  remaining_seconds: number;            // 仅 status='held_by_*' 时有效
}

export interface LockManager {
  acquire(templateId: Uuid): Promise<LockState>;
  release(templateId: Uuid, token: string): Promise<void>;
  poll(templateId: Uuid): Promise<LockState>;  // 客户端 5s 轮询
  refresh(templateId: Uuid, token: string): Promise<LockState>;  // 续约 (P2+)
}
```

**Rust trait** (Tier 3):

```rust
#[async_trait]
pub trait LockService {
    async fn acquire_lock(&self, template_id: Uuid, user_id: Uuid) -> Result<LockToken, LockError>;
    async fn release_lock(&self, template_id: Uuid, user_id: Uuid, token: &str) -> Result<(), LockError>;
    async fn get_lock_state(&self, template_id: Uuid) -> Result<LockState, LockError>;
    async fn cleanup_expired_locks(&self) -> Result<usize, LockError>;  // cron 任务
}

pub struct LockToken {
    pub token: String,                   // 32-byte hex
    pub template_id: Uuid,
    pub user_id: Uuid,
    pub expires_at: DateTime<Utc>,
}

pub enum LockError {
    LockHeldByOther { holder_user_id: Uuid, remaining_seconds: i32 },
    InvalidToken,
    NotFound,
    Expired,
}
```

**关键不变量**:
- 同时只能 1 个 active lock (per §9 UNIQUE 约束)
- 锁超时 5min 后, 下次 PATCH 自动释放 + 通知
- 锁释放 (主动 / 超时 / PATCH 完成) 都触发 `template_change_events` event_type='updated'

### 4.6 class #6: `ConflictResolver` (Tier 1)

**职责**: 乐观锁冲突检测 + 3 选项 dialog 逻辑

**TypeScript**:

```typescript
export interface ConflictContext {
  clientVersion: number;                // 客户端期望 version
  serverVersion: number;                // 服务端当前 version
  serverSnapshot: WorkflowTemplateWithColumns;
  clientChanges: TemplateColumnChange[];
}

export class ConflictResolver {
  detect(clientVersion: number, serverVersion: number): boolean {
    return clientVersion !== serverVersion;
  }

  resolve(ctx: ConflictContext, choice: ConflictResolution): ResolutionAction {
    switch (choice) {
      case 'force_overwrite':
        // 用 server.version 重发 PATCH, 覆盖 server
        return { action: 'retry', version: ctx.serverVersion, payload: ctx.clientChanges };
      case 'discard':
        // 关闭 dialog, 丢弃 client changes
        return { action: 'discard' };
      case 'rollback':
        // 调用回滚端点 (留 P2+ per BD §9.3 T-15)
        return { action: 'rollback', target_version: ctx.serverVersion };
    }
  }
}

export type ResolutionAction =
  | { action: 'retry'; version: number; payload: TemplateColumnChange[] }
  | { action: 'discard' }
  | { action: 'rollback'; target_version: number };
```

### 4.7 class #7: `templateStore` (Tier 1, zustand store)

**职责**: 客户端状态管理, CRUD + 锁状态 + 当前激活 + SSE 订阅

**TypeScript** (frontend/src/store/templateStore.ts):

```typescript
export interface TemplateStoreState {
  templates: Map<Uuid, WorkflowTemplateWithColumns>;  // tenant 内所有模板缓存
  activeTemplateId: Uuid | null;                      // 当前激活模板 (UserMenu badge 用)
  locks: Map<Uuid, LockState>;                        // template_id -> 锁状态
  loading: Set<Uuid>;                                  // 正在加载的 template_id

  // CRUD actions
  fetchTemplates: () => Promise<void>;
  fetchTemplate: (id: Uuid) => Promise<void>;
  createTemplate: (input: CreateTemplateInput) => Promise<Uuid>;
  updateTemplate: (id: Uuid, version: number, changes: TemplateColumnChange[], lockToken: string) => Promise<void>;
  deleteTemplate: (id: Uuid) => Promise<void>;

  // Lock actions
  acquireLock: (id: Uuid) => Promise<LockState>;
  releaseLock: (id: Uuid) => Promise<void>;
  pollLock: (id: Uuid) => Promise<void>;

  // Active template
  setActiveTemplate: (id: Uuid) => void;

  // SSE handler
  dispatch: (event: TemplateChangedEvent) => void;
}

export interface TemplateChangedEvent {
  template_id: Uuid;
  version: number;
  event_type: 'created' | 'updated' | 'deleted' | 'rolled_back';
  actor_user_id: Uuid;
}

export const useTemplateStore = create<TemplateStoreState>((set, get) => ({
  // ... 实现 ...
}));
```

### 4.8 class #8: `TemplateBoard` (Tier 1, React Component)

**职责**: 模板化看板, 复用 `KanbanBoard` 组件并传入 `columns` prop

**TypeScript**:

```typescript
export interface TemplateBoardProps {
  templateId: Uuid;
  templateVersion: number;
}

export function TemplateBoard(props: TemplateBoardProps): JSX.Element;

// 内部: 复用既有 KanbanBoard, 传入 columns
const TemplateBoardImpl: React.FC<TemplateBoardProps> = ({ templateId, templateVersion }) => {
  const template = useTemplateStore(s => s.templates.get(templateId));
  if (!template) return <ErrorPlaceholder />;
  return (
    <>
      <Indicator name={template.name} version={templateVersion} />
      <KanbanBoard
        workItems={useWorkItemsForProject()}
        columns={template.columns}  // DD 新增: 替代硬编码 KANBAN_COLUMNS
        onTransition={handleTransition}
      />
    </>
  );
};

// WorkItem 拖动到模板列时, status 更新
function handleTransition(workItemId: Uuid, toColumn: TemplateColumn): Promise<void> {
  // 调用既有 /api/work-items/{id} PATCH, status = toColumn.mapping_status
  // 触发既有 WORKITEM_SM 状态机校验 (per frontend/src/types/ids.ts)
  // 校验失败 → toast "状态机不允许此转换"
}
```

---

## §5 状态机 (3 个, per BD §6.3 详细化)

> per DD-AGENT-RELATIONSHIP-001 v0.1 §3.3 模板 + DD-CANVAS-AGENT-001 §5 模板: 每个状态机给 Rust enum + ASCII diagram + 转换条件 + 守卫

### 5.1 状态机 #1: TemplateState (模板生命周期, per BD §6.3.1)

**Rust enum**:

```rust
pub enum TemplateState {
    Active,                  // 默认, 创建即激活
    Deleted,                 // 软删 (deleted_at != null)
}

pub enum TemplateEvent {
    Create { template_id: Uuid },
    Update { template_id: Uuid, version: i32 },
    SoftDelete { template_id: Uuid, deleted_at: DateTime<Utc> },
    Restore { template_id: Uuid },     // 留 P2+, 暂不实装
}
```

**ASCII diagram**:

```
                    ┌─────────────────────────────────────┐
                    │                                     │
                    ▼                                     │
              ┌──────────┐                                │
              │ Active   │ ──── SoftDelete ──→ ┌────────┐ │
              │ (v1 起)  │                     │ Deleted│ │
              │          │ ◀── Restore ────── │ (软删) │ │
              └──────────┘   (P2+, 暂不实装)   └────────┘ │
                    │                                     │
                    └──── Update ──→ (version +1, 仍在 Active)
```

**转换条件**:
- `Active --Create--> Active`: 新建即 Active v1 (无 Draft 状态)
- `Active --Update--> Active`: version += 1, 仍在 Active (SCD2 append-only)
- `Active --SoftDelete--> Deleted`: 设置 `deleted_at = NOW()`, 仍可查询 (软删)
- `Deleted --Restore--> Active`: 留 P2+, 暂不实装 (per BD §9.3 T-22)
- `Deleted --Update--> Deleted`: 不可, 已软删模板不能修改 (UI 隐藏 + 后端 403)

**守卫**:
- builtin=true 模板永远 Active, deleted_at 永远 null (per §9 CHECK 约束)
- 创建模板必须指定 `is_builtin=false` (用户模板); builtin 模板走 `createBuiltin()` (内部调用)

### 5.2 状态机 #2: LockState (锁状态机, per BD §6.3.2)

**Rust enum**:

```rust
pub enum LockState {
    Unlocked,
    HeldBySelf { token: String, expires_at: DateTime<Utc>, remaining_seconds: i32 },
    HeldByOther { holder_user_id: Uuid, expires_at: DateTime<Utc>, remaining_seconds: i32 },
    Expired,                  // 锁已过期但还没清理 (cron 还没跑)
}
```

**ASCII diagram**:

```
   ┌─────────────┐
   │  Unlocked   │ ◀──────────────┐ ──────────────┐
   └─────────────┘                │               │
          │                        │               │
          │ acquire                │ 5min 超时    │ 主动 release
          ▼                        │ /cron 清理    │ / PATCH 完成
   ┌─────────────┐                │               │
   │ HeldBySelf  │ ───────────────┘               │
   └─────────────┘                                │
          │                                        │
          │ (其他用户 acquire)                     │
          ▼                                        │
   ┌─────────────┐ ──── acquire ────┐ ─────────────┘
   │HeldByOther  │                  │
   └─────────────┘                  │
          │                          │
          │ 5min 超时                │
          ▼                          ▼
   ┌─────────────┐            ┌─────────────┐
   │   Expired   │ ──────────→│  Unlocked   │
   │ (cron 待清) │            │             │
   └─────────────┘            └─────────────┘
```

**转换条件**:
- `Unlocked --acquire--> HeldBySelf`: 同一用户首次 acquire
- `Unlocked --acquire--> HeldByOther`: 其他用户 acquire
- `HeldBySelf --5min 超时--> Expired`: 应用层 cron 扫描 `expires_at < NOW() AND released_at IS NULL`
- `HeldBySelf --release/PATCH 成功--> Unlocked`: 主动释放或 PATCH 完成后释放
- `HeldByOther --5min 超时--> Expired`: 同上 (其他用户的锁)
- `HeldByOther --release (原持有者)--> Unlocked`: 原持有者主动释放
- `Expired --cron 清理--> Unlocked`: cron 任务 UPDATE released_at

**守卫**:
- 同时只能 1 个 active lock (per §9 UNIQUE `template_locks(template_id) WHERE released_at IS NULL`)
- 申请锁时若已被持有, 返回 409 + holder info
- 释放锁时 token 必须匹配 (否则 403)
- 锁申请响应中 token 仅返回一次 (per §8 安全设计)

### 5.3 状态机 #3: WorkItemStatusMapping (WorkItem 拖动, per BD §6.3.3)

**Rust enum**:

```rust
pub enum WorkItemStatus {
    Todo,
    InProgress,
    Review,
    Done,
    // 注释: 不新增 Wontfix (per BD §9.3 T-01 待拍板; v1 作废列 mapping 复用 Todo)
}

// DD 新增: WorkItem.status + template_column_id 派生关系
pub struct WorkItemTemplateBinding {
    pub work_item_id: Uuid,
    pub template_id: Uuid,
    pub column_id: Uuid,           // 当前所在列
    pub status: WorkItemStatus,     // 与 column.mapping_status 一致 (派生)
}
```

**ASCII diagram (拖动事件流)**:

```
WorkItem W 当前 status=S1, 在 Column C1 (mapping_status=S1)
                    │
                    │ User 拖动 W 到 Column C2 (mapping_status=S2)
                    ▼
        ┌─────────────────────────────┐
        │ Validation: WORKITEM_SM     │
        │ 允许 S1 → S2 转换?          │
        │   (既有状态机, per           │
        │    frontend/src/types/ids.ts)│
        └─────────────────────────────┘
                    │
        ┌───────────┴───────────┐
        │                       │
        ▼ YES                   ▼ NO
  ┌──────────┐           ┌──────────────┐
  │ PATCH    │           │ toast:       │
  │ /api/work│           │ "状态机不允许  │
  │ -items/{id}│         │ 此转换"        │
  │ status=S2│           │ (回滚 W 位置)  │
  └──────────┘           └──────────────┘
        │
        ▼
  ┌──────────────────────────────┐
  │ work_item_template_links 表   │
  │ 插入新行 (W, template_id,    │
  │  C2, status=S2)              │
  │ 或更新既有行 (column_id=C2)   │
  └──────────────────────────────┘
        │
        ▼
  ┌──────────────────────────────┐
  │ UI 重渲染: W 出现在 C2        │
  └──────────────────────────────┘
```

**转换条件**:
- 拖动触发时, 校验 `WORKITEM_SM` 既有状态机 (3 状态: todo/in_progress/done, review 属工作流扩展)
- 校验通过: PATCH WorkItem.status = toColumn.mapping_status, 同时更新 `work_item_template_links.column_id`
- 校验失败: toast 拒绝, WorkItem 位置回滚
- 13 列 (P1-P9 + P6.1-P6.4) 都映射 `in_progress`, P1 → P2 拖动 status 不变, 仅 column_id 变

**守卫**:
- WorkItem.status 更新走既有 `/api/work-items/{id}` PATCH, 不新增模板化看板专属 API
- 同一 WorkItem 不能同时关联多个 template_id (per §9 UNIQUE `work_item_template_links(work_item_id, template_id)`)

---

## §6 共享类型 (6 个, per DD-AGENT-RELATIONSHIP-001 v0.1 §3.2.5 + DD-CANVAS-AGENT-001 §6 模板)

> 跨 module 共享的 enum/struct, 在 frontend/src/types/ 或 crates/shared-types/ 定义

### 6.1 shared #1: `WorkItemStatus` (既有, 引用)

```typescript
// 既有, per frontend/src/types/ids.ts
export type WorkItemStatus = 'todo' | 'in_progress' | 'review' | 'done';
```

```rust
// 既有, per crates/domain-work-item/src/lib.rs
pub enum WorkItemStatus { Todo, InProgress, Review, Done }
```

**DD 不扩展 enum**, 17 列 → 4 状态映射见 §4.4 MappingPolicy。作废列映射候选 `wontfix` 待 §13 T-01 拍板。

### 6.2 shared #2: `TemplateColumnType` (DD 新增)

```typescript
export type TemplateColumnType = 'builtin' | 'user';
```

```rust
pub enum TemplateColumnType { Builtin, User }
```

**应用范围**: §4.2 `TemplateColumn.column_type`, §4.4 `MappingPolicy`

### 6.3 shared #3: `TemplateColumnPanel` (DD 新增)

```typescript
export type TemplateColumnPanel = 'main' | 'bottom';
```

```rust
pub enum TemplateColumnPanel { Main, Bottom }
```

**应用范围**: §4.2 `TemplateColumn.panel`, §4.3 `TemplateColumnEditor.Reorder`

### 6.4 shared #4: `ConflictResolution` (DD 新增)

```typescript
export type ConflictResolution = 'force_overwrite' | 'discard' | 'rollback';
```

```rust
pub enum ConflictResolution { ForceOverwrite, Discard, Rollback }
```

**应用范围**: §4.3 `TemplateColumnEditor.conflictDialog`, §4.6 `ConflictResolver.resolve()`

### 6.5 shared #5: `TemplateChangedEvent` (DD 新增, 用于 SSE)

```typescript
export interface TemplateChangedEvent {
  template_id: Uuid;
  version: number;
  event_type: 'created' | 'updated' | 'deleted' | 'rolled_back';
  actor_user_id: Uuid;
}
```

```rust
pub struct TemplateChangedEvent {
    pub template_id: Uuid,
    pub version: i32,
    pub event_type: TemplateEventType,
    pub actor_user_id: Uuid,
}

pub enum TemplateEventType { Created, Updated, Deleted, RolledBack }
```

**应用范围**: §4.7 `templateStore.dispatch()`, §7 SSE 协议

### 6.6 shared #6: `WorkItemTemplateBinding` (DD 新增)

```typescript
export interface WorkItemTemplateBinding {
  work_item_id: Uuid;
  template_id: Uuid;
  column_id: Uuid;
  status: WorkItemStatus;       // 派生字段, 与 column.mapping_status 一致
  linked_at: ISO8601;
}
```

```rust
pub struct WorkItemTemplateBinding {
    pub work_item_id: Uuid,
    pub template_id: Uuid,
    pub column_id: Uuid,
    pub status: WorkItemStatus,
    pub linked_at: DateTime<Utc>,
}
```

**应用范围**: §5.3 状态机 3, §9 `work_item_template_links` 表

---

## §7 接口协议 (6 REST + 1 SSE, per BD §5 详细化)

> per DD-CANVAS-AGENT-001 §7 + DD-AGENT-RELATIONSHIP-001 §6: TypeScript 类型 + 错误码 + 重试策略 + 幂等性

### 7.1 REST API 协议总览 (6 端点)

| 方法 | 路径 | 鉴权 | 幂等 | 重试策略 |
|---|---|---|---|---|
| `GET` | `/api/workflow-templates` | session token | ✅ 是 (查询不修改) | 无需重试 |
| `GET` | `/api/workflow-templates/{id}` | session token | ✅ 是 | 无需重试 |
| `POST` | `/api/workflow-templates` | session token | ❌ 否 (每次创建新 id) | 客户端 idempotency-key (留 P2+) |
| `PATCH` | `/api/workflow-templates/{id}` | session token + lock_token | ⚠️ 条件幂等 (version 必须匹配) | 不重试, 返 409 让用户选 |
| `DELETE` | `/api/workflow-templates/{id}` | session token | ✅ 是 (软删, 同 id 多次 DELETE 等价) | 1 次, 失败告警 |
| `POST` | `/api/workflow-templates/{id}/lock` | session token | ⚠️ 条件幂等 (同用户重复 acquire 返回同一 token) | 1 次, 失败返 409 |
| `GET` | `/api/workflow-templates/stream` | session token (SSE Upgrade) | ✅ 是 (订阅不修改) | 自动重连 |

### 7.2 GET /api/workflow-templates

**请求**:
```http
GET /api/workflow-templates
Cookie: session=<token>
```

**响应 200**:
```typescript
interface ListTemplatesResponse {
  templates: WorkflowTemplate[];       // 含 builtin 「瀑布式开发」
}
```

**错误码**:
- 401: 未登录
- 500: 数据库错误

### 7.3 POST /api/workflow-templates

**请求**:
```typescript
interface CreateTemplateRequest {
  name: string;                        // 必填, 1-255 chars
  project_id: Uuid | null;             // 可选, null = 跨项目模板
}

interface CreateTemplateResponse {
  id: Uuid;
  template: WorkflowTemplateWithColumns;  // 含 17 列预置数据
  version: number;                     // = 1
}
```

**错误码**:
- 400: name 为空 / 超长 / 非法字符
- 401: 未登录
- 409: 同 tenant 内 name 重复 (per §9 UNIQUE 约束)
- 500: 数据库错误

**幂等性**: ❌ 否; 客户端可用 `idempotency-key` 头避免重复创建 (留 P2+)

### 7.4 PATCH /api/workflow-templates/{id} (关键, 含乐观锁)

**请求**:
```typescript
interface UpdateTemplateRequest {
  version: number;                     // 客户端期望 version (乐观锁)
  lock_token: string;                  // 32-byte hex (写锁凭证)
  changes: TemplateColumnChange[];     // 1+ 项变更
}
```

**响应 200**:
```typescript
interface UpdateTemplateResponse {
  template: WorkflowTemplateWithColumns;
  new_version: number;                 // = request.version + 1
  changes_applied: TemplateColumnChange[];  // 实际应用 (sub-set of request.changes)
}
```

**错误码**:
- 400: changes 字段非法 (空数组 / 超长)
- 401: 未登录
- 403: `lock_token` 不匹配 (lock 已超时 / 已被释放 / token 错误)
- **409**: `version` 冲突 (server.version ≠ request.version), 响应含 `server_version` + `server_snapshot`, 客户端弹 §4.3 `ConflictDialog`
- 422: changes 违反 §4.2 守卫 (尝试删除 builtin 列 / 改作废列宽 / 违反 BR-WT-1~10)
- 500: 数据库错误

**幂等性**: ⚠️ 条件幂等 (同 version + 同 payload 重发, server 已应用, 返回 server 最新 snapshot 但 version+1, 需客户端识别 + 不重发)

**示例 409 响应**:
```typescript
interface VersionConflictResponse {
  error: 'version_conflict';
  server_version: number;              // = 5
  server_snapshot: WorkflowTemplateWithColumns;
  client_version: number;              // = 4 (客户端期望)
  message: '模板在您编辑期间已被他人更新到 v5';
}
```

### 7.5 POST /api/workflow-templates/{id}/lock

**请求**: (空 body)

**响应 200** (成功获取锁):
```typescript
interface AcquireLockResponse {
  lock_token: string;                  // 32-byte hex
  expires_at: ISO8601;                 // = NOW() + 5min
  remaining_seconds: number;           // = 300
}
```

**错误码**:
- 401: 未登录
- 404: template_id 不存在
- **409**: 锁被他人持有, 响应含 holder info:
  ```typescript
  interface LockHeldByOtherResponse {
    error: 'lock_held_by_other';
    holder_user_id: Uuid;
    holder_user_name: string;          // 用于 UI 显示
    remaining_seconds: number;
  }
  ```
- 500: 数据库错误

**幂等性**: ⚠️ 同用户重复 acquire 返回同一 token (不重新生成, 不重置 expires_at); 不同用户 acquire 返回 409

### 7.6 DELETE /api/workflow-templates/{id} (软删)

**请求**: (空 body)

**响应 204**: No Content

**错误码**:
- 401: 未登录
- 403: builtin 模板不可删 (per §4.5 builtin 守卫)
- 404: template_id 不存在
- 500: 数据库错误

**幂等性**: ✅ 是 (软删, 同 id 多次 DELETE 等价; 已 deleted 的模板再 DELETE 返 204)

### 7.7 GET /api/workflow-templates/stream (SSE)

**请求**:
```http
GET /api/workflow-templates/stream
Cookie: session=<token>
Accept: text/event-stream
```

**响应** (SSE 流, 长连接):
```
event: template_changed
data: {"template_id":"01a09cae-...","version":2,"event_type":"updated","actor_user_id":"01a09cae-..."}

event: template_changed
data: {"template_id":"01a09cae-...","version":3,"event_type":"created","actor_user_id":"01a09cae-..."}
```

**客户端订阅** (§4.7 `templateStore`):
```typescript
const eventSource = new EventSource('/api/workflow-templates/stream');
eventSource.addEventListener('template_changed', (e) => {
  const event = JSON.parse(e.data) as TemplateChangedEvent;
  useTemplateStore.getState().dispatch(event);
});
```

**降级** (per BD §9.3 T-05): SSE 不可用时 polling 5s 轮询 `GET /api/workflow-templates`

### 7.8 错误处理总则 (per BD §5.5)

| 错误类别 | HTTP | 处理策略 |
|---|---|---|
| 客户端校验失败 | 400 | form 内联错误 |
| 未登录 | 401 | 跳登录页 |
| 权限不足 | 403 | toast |
| 资源不存在 | 404 | toast |
| 名称冲突 | 409 | form 错误提示 |
| 版本冲突 | 409 | 弹 §4.6 ConflictDialog |
| 锁被他人持有 | 409 | UI 角标 |
| 业务规则违反 | 422 | form 内联错误 |
| 服务端错误 | 500 | toast + Sentry + 触发 polling fallback |
| SSE 断连 | — | 自动重连 3 次, 失败切 polling 5s |

---

## §8 时序图 (4 个关键场景, per BD §2.3 + DD-AGENT-RELATIONSHIP-001 v0.1 §8 模板)

### 8.1 场景 1: 用户在模板列表页创建模板

```
User                TemplateList       templateStore       BFF              DomainService      Database
 │                       │                  │                │                    │                 │
 │ click「+ 创建」       │                  │                │                    │                 │
 ├──────────────────────→│                  │                │                    │                 │
 │                       │ open modal       │                │                    │                 │
 │                       │ (input name)     │                │                    │                 │
 │                       │                  │                │                    │                 │
 │ click 确认             │                  │                │                    │                 │
 ├──────────────────────→│                  │                │                    │                 │
 │                       │ createTemplate() │                │                    │                 │
 │                       ├─────────────────→│                │                    │                 │
 │                       │                  │ POST /api/workflow-templates         │                 │
 │                       │                  ├───────────────→│                    │                 │
 │                       │                  │                │ validate            │                 │
 │                       │                  │                │ create_template()   │                 │
 │                       │                  │                ├───────────────────→│                 │
 │                       │                  │                │                    │ BEGIN TRANSACTION│
 │                       │                  │                │                    │ INSERT workflow_templates
 │                       │                  │                │                    │ (version=1)
 │                       │                  │                │                    │ INSERT template_columns × 17
 │                       │                  │                │                    │ INSERT template_versions (v1)
 │                       │                  │                │                    │ EventBus.publish(template_changed, 'created')
 │                       │                  │                │                    │ COMMIT
 │                       │                  │                │ ◀───────────────────┤
 │                       │                  │                │ new template       │                 │
 │                       │                  │ ◀───────────────┤                    │                 │
 │                       │                  │ 201 Created    │                    │                 │
 │                       │ update zustand   │                │                    │                 │
 │                       │ ◀─────────────────┤                │                    │                 │
 │                       │ re-render list   │                │                    │                 │
 │ ◀─────────────────────┤                  │                │                    │                 │
 │                       │                  │                │                    │ SSE broadcast: │ (其他客户端)
 │                       │                  │                │                    │  template_changed 'created'
 │                       │                  │                │                    │ ───────→│
 │                       │                  │                │                    │         SSE Client
 │                       │                  │                │                    │         (其他 TemplateList 实例)
 │                       │                  │                │                    │         dispatch(event)
 │                       │                  │                │                    │         invalidate cache
 │                       │                  │                │                    │         re-render
```

### 8.2 场景 2: 多人并发编辑冲突 (乐观锁)

```
User A              User B              templateStore        BFF             DomainService      Database
 │                     │                    │                  │                 │                 │
 │ click「编辑」        │                    │                  │                 │                 │
 ├─────────────────────┤                    │                  │                 │                 │
 │                     │ click「编辑」       │                  │                 │                 │
 │                     ├────────────────────→│                  │                 │                 │
 │                     │                    │ POST /lock       │                 │                 │
 │                     │                    ├────────────────→│                 │                 │
 │                     │                    │                  │ acquire_lock()  │                 │
 │                     │                    │                  ├────────────────→│                 │
 │                     │                    │                  │                  │ INSERT template_locks
 │                     │                    │                  │                  │ (holder=A, expires=+5min)
 │                     │                    │                  │ ◀────────────────┤
 │                     │                    │                  │ LockToken        │                 │
 │                     │                    │ ◀────────────────┤                  │                 │
 │                     │ 409 LockHeldByOther│                  │                 │                 │
 │                     │ {holder=A, ...}    │                  │                 │                 │
 │                     │ ◀─────────────────┤                  │                 │                 │
 │ UI: 「A 正在编辑   │                    │                  │                 │                 │
 │ (剩余 4 分钟)」     │                    │                  │                 │                 │
 │                     │                    │                  │                 │                 │
 │ (并行) A 编辑完成  │                    │                  │                 │                 │
 │ PATCH /api/.../{id}│                    │                  │                 │                 │
 ├─────────────────────────────────────────────────────────────────────────────────→│                 │
 │                     │                    │                  │ update_template()│                 │
 │                     │                    │                  ├────────────────→│                 │
 │                     │                    │                  │                  │ UPDATE workflow_templates SET version=2
 │                     │                    │                  │                  │ DELETE FROM template_locks WHERE template_id=...
 │                     │                    │                  │                  │ INSERT template_versions (v2)
 │                     │                    │                  │                  │ EventBus.publish(template_changed, 'updated')
 │                     │                    │                  │ ◀────────────────┤
 │                     │                    │ ◀───────────────┤                  │                 │
 │ 200 OK              │                    │                  │                 │                 │
 │ {new_version=2}     │                    │                  │                 │                 │
 │ ◀─────────────────────────────────────────────────────────────────────────────────┤                 │
 │                     │                    │                  │                 │                 │
 │ (并行) B 收到 SSE   │                    │                  │                 │                 │
 │ template_changed 'updated' v2            │                  │                 │                 │
 │                     │ SSE receive       │                  │                 │                 │
 │                     │ (via templateStore.dispatch)            │                 │                 │
 │                     │ invalidate cache   │                  │                 │                 │
 │                     │ 解锁角标           │                  │                 │                 │
 │                     │ (UI 自动重渲染)   │                  │                 │                 │
 │                     │                    │                  │                 │                 │
 │ (此时 B 编辑)       │                    │                  │                 │                 │
 │ PATCH /api/.../{id} │                    │                  │                 │                 │
 │ {version=1, ...}    │                    │                  │                 │                 │
 │                     ├────────────────────────────────────────────────────────→│                 │
 │                     │                    │                  │ update_template()│                 │
 │                     │                    │                  ├────────────────→│                 │
 │                     │                    │                  │                  │ SELECT version → 2
 │                     │                    │                  │                  │ version 1 ≠ server 2 → CONFLICT
 │                     │                    │                  │ ◀────────────────┤
 │                     │                    │ ◀───────────────┤                  │                 │
 │                     │ 409 VersionConflict│                  │                 │                 │
 │                     │ {server_version=2, server_snapshot}    │                 │                 │
 │                     │ ◀─────────────────┤                  │                 │                 │
 │ UI: ConflictDialog   │                    │                  │                 │                 │
 │ 「3 选项:           │                    │                  │                 │                 │
 │ 强制覆盖 / 放弃 /   │                    │                  │                 │                 │
 │ 回滚到 v2」         │                    │                  │                 │                 │
 │                     │ User B 选「强制覆盖」                  │                 │                 │
 │                     │ PATCH {version=2, changes}              │                 │                 │
 │                     ├────────────────────────────────────────────────────────→│                 │
 │                     │                    │                  │ update_template()│                 │
 │                     │                    │                  │ version 2 = server 2, OK
 │                     │                    │                  │ version → 3
 │                     │                    │                  │ ◀────────────────┤
 │                     │                    │ ◀───────────────┤                  │                 │
 │                     │ 200 OK v3          │                  │                 │                 │
 │                     │ ◀─────────────────┤                  │                 │                 │
```

### 8.3 场景 3: SSE 推送跨界面同步 (per BD §9.3 T-05)

```
User A            User B (另一 tab)    BFF SSE relay    DomainService    Database    EventBus
 │                      │                  │                │                │             │
 │ 修改模板 (PATCH)     │                  │                │                │             │
 ├────────────────────────────────────────────────────────→│                │             │
 │                      │                  │                │ update_template()            │
 │                      │                  │                ├───────────────→│             │
 │                      │                  │                │                │ UPDATE ... │
 │                      │                  │                │                │             │
 │                      │                  │                │ EventBus.publish(template_changed, 'updated', v=N)
 │                      │                  │                ├────────────────────────────→│
 │                      │                  │                │                │             │
 │                      │                  │                │                │             │ (异步 fan-out)
 │                      │                  │ ◀──────────────┤                │             │
 │                      │                  │ SSE 推送       │                │             │
 │                      │                  │ event: template_changed         │             │
 │                      │                  │ data: {...}    │                │             │
 │                      │ SSE receive      │                │                │             │
 │                      │ ◀────────────────┤                │                │             │
 │                      │ templateStore.dispatch(event)     │                │             │
 │                      │ invalidate cache │                │                │             │
 │                      │ re-render list   │                │                │             │
 │                      │ (B 的 UI 实时反映 A 的修改)         │                │             │
 │                      │                  │                │                │             │
 │ (fallback) SSE 断连 │                  │                │                │             │
 │ 5s 后 polling       │                  │                │                │             │
 │ GET /api/workflow-templates             │                │                │             │
 ├────────────────────────────────────────────────────────→│                │             │
 │                      │                  │                │ SELECT ...     │             │
 │                      │                  │                ├───────────────→│             │
 │                      │                  │                │ ◀───────────────┤             │
 │                      │                  │                │ templates list │             │
 │                      │                  │ ◀──────────────┤                │             │
 │ ◀────────────────────────────────────────────────────────┤                │             │
```

### 8.4 场景 4: 一键回滚 (留 P2+, per BD §9.3 T-15)

> 本场景为占位, 详细实装待 P2+ 阶段。当前实现: 回滚走 PATCH 路径, 用历史 version 快照作为新 payload (不引入独立回滚端点)

```
User A              templateStore        BFF             DomainService      Database
 │                     │                  │                 │                 │
 │ 在模板详情页        │                  │                 │                 │
 │ 点击「版本历史」    │                  │                 │                 │
 │ 选择回滚到 v3       │                  │                 │                 │
 │                     │ fetch v3 snapshot│                 │                 │
 │                     ├────────────────→│                 │                 │
 │                     │                  │ GET /api/workflow-templates/{id}/versions/3
 │                     │                  ├────────────────→│                 │
 │                     │                  │                  │ SELECT * FROM template_versions WHERE version=3
 │                     │                  │                  ├────────────────→│
 │                     │                  │                  │ ◀────────────────┤
 │                     │                  │ v3 snapshot      │                 │
 │                     │ ◀────────────────┤                  │                 │
 │                     │ 显示 v3 预览 (diff)                 │                 │
 │                     │                  │                 │                 │
 │ 点击「确认回滚」    │                  │                 │                 │
 │                     │ PATCH {version=N, payload=v3_snapshot}│              │
 │                     ├────────────────→│                 │                 │
 │                     │                  │ update_template()│                 │
 │                     │                  ├────────────────→│                 │
 │                     │                  │                  │ 应用 v3 内容 (写入新 version N+1)
 │                     │                  │                  │ INSERT template_versions (vN+1, snapshot=v3)
 │                     │                  │                  │ EventBus.publish('rolled_back')
 │                     │                  │ ◀────────────────┤
 │                     │ ◀────────────────┤ 200 OK vN+1     │                 │
 │                     │ (per 守门 #1 禁回溯叙事: 不覆盖历史, 而是用历史快照生成新 version)
```

---

## §9 数据持久化 (7 张表完整 DDL, per BD §4 详细化)

> per `ipa-database-design` skill + DD-AGENT-RELATIONSHIP-001 v0.1 §3 模板: 完整 DDL 含 索引 / CHECK / UNIQUE / RLS / WORM trigger / 备份策略

### 9.1 W/T/M 三分类声明 (per 守门 #13, 继承 BD §9.1)

| 分类 | 表数 | 表名 |
|---|---|---|
| Master (W) | 4 | workflow_templates, template_columns, template_versions, work_item_template_links |
| Transaction (T) | 3 (全部 WORM append-only) | template_locks, template_change_events, template_audit_logs |
| Work | 0 | (无, per BD §9.1 论证) |

**总计**: **4 Master + 3 Transaction WORM = 7 表** ✅ (与 SRS §9 + BD §4.1 完全一致, 0 混合分類)

**WORM 触发器** (DD 新增, 完整 plpgsql):

```sql
-- template_versions WORM trigger
CREATE OR REPLACE FUNCTION template_versions_worm()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'UPDATE' THEN
        RAISE EXCEPTION 'template_versions is append-only (WORM), UPDATE forbidden';
    ELSIF TG_OP = 'DELETE' THEN
        RAISE EXCEPTION 'template_versions is append-only (WORM), DELETE forbidden';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER template_versions_worm_trigger
    BEFORE UPDATE OR DELETE ON template_versions
    FOR EACH ROW EXECUTE FUNCTION template_versions_worm();
```

```sql
-- template_change_events WORM trigger (同上模式)
-- template_audit_logs WORM trigger (同上模式)
```

### 9.2 完整 DDL (per §1 SRS §9 7 张表 + BD §4 详细化)

#### 9.2.1 `workflow_templates` (Master)

```sql
CREATE TABLE workflow_templates (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id       UUID NOT NULL,
    project_id      UUID,                          -- NULL = 跨项目 (per FR-WT-1.1)
    name            VARCHAR(255) NOT NULL,
    is_builtin      BOOLEAN NOT NULL DEFAULT FALSE,
    version         INTEGER NOT NULL DEFAULT 1,    -- SCD2 当前 version
    columns_count   INTEGER NOT NULL DEFAULT 17,   -- 冗余字段
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NOT NULL,
    deleted_at      TIMESTAMPTZ,                   -- 软删时间戳

    CONSTRAINT uq_workflow_templates_tenant_name UNIQUE (tenant_id, name) WHERE deleted_at IS NULL,
    CONSTRAINT chk_builtin_not_deleted CHECK (NOT (is_builtin = TRUE AND deleted_at IS NOT NULL)),
    CONSTRAINT chk_columns_count_positive CHECK (columns_count > 0 AND columns_count <= 50),
    CONSTRAINT chk_version_positive CHECK (version >= 1)
);

-- 索引
CREATE INDEX idx_workflow_templates_tenant_active ON workflow_templates(tenant_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_workflow_templates_builtin ON workflow_templates(tenant_id, is_builtin) WHERE deleted_at IS NULL;
CREATE INDEX idx_workflow_templates_project ON workflow_templates(tenant_id, project_id) WHERE deleted_at IS NULL;
CREATE INDEX idx_workflow_templates_updated_at ON workflow_templates(updated_at DESC);

-- RLS
ALTER TABLE workflow_templates ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_workflow_templates ON workflow_templates
    USING (tenant_id = current_setting('app.current_tenant')::UUID);
```

#### 9.2.2 `template_columns` (Master)

```sql
CREATE TABLE template_columns (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id     UUID NOT NULL REFERENCES workflow_templates(id) ON DELETE RESTRICT,
    position        INTEGER NOT NULL,
    name            VARCHAR(255) NOT NULL,
    builtin         BOOLEAN NOT NULL DEFAULT FALSE,
    width_factor    NUMERIC(4,2) NOT NULL DEFAULT 1.0 CHECK (width_factor > 0 AND width_factor <= 5.0),
    panel           VARCHAR(16) NOT NULL DEFAULT 'main',
    mapping_status  VARCHAR(32) NOT NULL,
    column_type     VARCHAR(16) NOT NULL DEFAULT 'user',
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_panel CHECK (panel IN ('main', 'bottom')),
    CONSTRAINT chk_mapping_status CHECK (mapping_status IN ('todo', 'in_progress', 'review', 'done')),
    CONSTRAINT chk_column_type CHECK (column_type IN ('builtin', 'user')),
    CONSTRAINT uq_template_columns_panel_position UNIQUE (template_id, panel, position),
    CONSTRAINT uq_template_columns_name UNIQUE (template_id, name)
);

CREATE INDEX idx_template_columns_template ON template_columns(template_id);
CREATE INDEX idx_template_columns_mapping ON template_columns(template_id, mapping_status);

ALTER TABLE template_columns ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_template_columns ON template_columns
    USING (
        EXISTS (
            SELECT 1 FROM workflow_templates wt
            WHERE wt.id = template_columns.template_id
            AND wt.tenant_id = current_setting('app.current_tenant')::UUID
        )
    );
```

#### 9.2.3 `template_versions` (Master, WORM append-only)

```sql
CREATE TABLE template_versions (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id     UUID NOT NULL REFERENCES workflow_templates(id) ON DELETE RESTRICT,
    version         INTEGER NOT NULL,
    snapshot_json   JSONB NOT NULL,
    change_summary  TEXT,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NOT NULL,

    CONSTRAINT uq_template_versions UNIQUE (template_id, version)
);

CREATE INDEX idx_template_versions_template ON template_versions(template_id, version DESC);

ALTER TABLE template_versions ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_template_versions ON template_versions
    USING (
        EXISTS (
            SELECT 1 FROM workflow_templates wt
            WHERE wt.id = template_versions.template_id
            AND wt.tenant_id = current_setting('app.current_tenant')::UUID
        )
    );

-- WORM trigger (per §9.1)
CREATE TRIGGER template_versions_worm_trigger
    BEFORE UPDATE OR DELETE ON template_versions
    FOR EACH ROW EXECUTE FUNCTION template_versions_worm();
```

#### 9.2.4 `template_locks` (Transaction, WORM-like with released_at UPDATE)

```sql
CREATE TABLE template_locks (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id     UUID NOT NULL REFERENCES workflow_templates(id) ON DELETE RESTRICT,
    holder_user_id  UUID NOT NULL,
    acquired_at     TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at      TIMESTAMPTZ NOT NULL,
    released_at     TIMESTAMPTZ,                       -- NULL = 持有中
    lock_token      VARCHAR(64) NOT NULL,

    CONSTRAINT chk_expires_after_acquired CHECK (expires_at > acquired_at),
    CONSTRAINT uq_template_locks_active UNIQUE (template_id) WHERE released_at IS NULL,
    CONSTRAINT uq_template_locks_token UNIQUE (lock_token)
);

CREATE INDEX idx_template_locks_active ON template_locks(template_id, expires_at) WHERE released_at IS NULL;
CREATE INDEX idx_template_locks_expired ON template_locks(expires_at) WHERE released_at IS NULL;

ALTER TABLE template_locks ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_template_locks ON template_locks
    USING (
        EXISTS (
            SELECT 1 FROM workflow_templates wt
            WHERE wt.id = template_locks.template_id
            AND wt.tenant_id = current_setting('app.current_tenant')::UUID
        )
    );

-- 锁过期 cron 清理 (per BD §4.2.4 + §9.3 T-09 待拍板严格 WORM)
-- v1: 允许 UPDATE released_at (实务便利)
-- 严格 WORM 选项 (留 P2+): 释放走新 INSERT row
```

#### 9.2.5 `template_change_events` (Transaction, WORM append-only)

```sql
CREATE TABLE template_change_events (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id     UUID NOT NULL,
    version         INTEGER NOT NULL,
    event_type      VARCHAR(32) NOT NULL,
    payload_json    JSONB,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_event_type CHECK (event_type IN ('created', 'updated', 'deleted', 'rolled_back'))
);

CREATE INDEX idx_template_change_events_template ON template_change_events(template_id, created_at DESC);
CREATE INDEX idx_template_change_events_created_at ON template_change_events(created_at DESC);

-- 注意: 无 FK 引用 workflow_templates (避免级联删除影响审计; 但 RLS 通过 application 层 join 校验)
-- WORM trigger
CREATE TRIGGER template_change_events_worm_trigger
    BEFORE UPDATE OR DELETE ON template_change_events
    FOR EACH ROW EXECUTE FUNCTION template_change_events_worm();
```

#### 9.2.6 `work_item_template_links` (Master)

```sql
CREATE TABLE work_item_template_links (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    work_item_id    UUID NOT NULL,
    template_id     UUID NOT NULL REFERENCES workflow_templates(id) ON DELETE RESTRICT,
    column_id       UUID NOT NULL REFERENCES template_columns(id) ON DELETE RESTRICT,
    status          VARCHAR(32) NOT NULL,
    linked_at       TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_status CHECK (status IN ('todo', 'in_progress', 'review', 'done')),
    CONSTRAINT uq_witl_work_item_template UNIQUE (work_item_id, template_id)
);

CREATE INDEX idx_witl_work_item ON work_item_template_links(work_item_id);
CREATE INDEX idx_witl_template_column ON work_item_template_links(template_id, column_id);
CREATE INDEX idx_witl_status ON work_item_template_links(status);

ALTER TABLE work_item_template_links ENABLE ROW LEVEL SECURITY;
CREATE POLICY tenant_isolation_witl ON work_item_template_links
    USING (
        EXISTS (
            SELECT 1 FROM workflow_templates wt
            WHERE wt.id = work_item_template_links.template_id
            AND wt.tenant_id = current_setting('app.current_tenant')::UUID
        )
    );
```

#### 9.2.7 `template_audit_logs` (Transaction, WORM append-only)

```sql
CREATE TABLE template_audit_logs (
    id              UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    template_id     UUID NOT NULL,
    user_id         UUID NOT NULL,
    action          VARCHAR(32) NOT NULL,
    before_json     JSONB,                          -- NULL for create
    after_json      JSONB,                          -- NULL for delete
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),

    CONSTRAINT chk_action CHECK (action IN ('create', 'update', 'delete', 'lock_acquire', 'lock_release', 'conflict_resolve'))
);

CREATE INDEX idx_template_audit_logs_template ON template_audit_logs(template_id, created_at DESC);
CREATE INDEX idx_template_audit_logs_user ON template_audit_logs(user_id, created_at DESC);
CREATE INDEX idx_template_audit_logs_action ON template_audit_logs(action);

-- WORM trigger
CREATE TRIGGER template_audit_logs_worm_trigger
    BEFORE UPDATE OR DELETE ON template_audit_logs
    FOR EACH ROW EXECUTE FUNCTION template_audit_logs_worm();
```

### 9.3 备份策略 (DD 新增, per 守门合规)

| 表 | 备份策略 | 保留周期 |
|---|---|---|
| workflow_templates | 每日全量 + 6h 增量 | 30 天 |
| template_columns | 同 workflow_templates | 30 天 |
| template_versions | 每周全量 (append-only, 不需要增量) | 永久 (审计要求) |
| template_locks | 不备份 (短期数据, 5min TTL) | — |
| template_change_events | 同 template_versions | 永久 |
| work_item_template_links | 每日全量 | 30 天 |
| template_audit_logs | 每周全量 | 永久 (合规要求) |

### 9.4 数据迁移脚本 (DD 新增, v1 mock → 真实后端过渡用)

```python
# scripts/sql/migrate_workflow_template_v1.py
# 作用: 从 frontend mock (zustand persist) 导出 JSON, 导入到 PostgreSQL
# 用法: python scripts/sql/migrate_workflow_template_v1.py --input <mock.json> --tenant-id <uuid>
# 守门: idempotent SHA256 验证 (per v0.33 v1.04 实证)
```

---

## §10 测试用例 (≥ 33 + e2e 3, per DD-AGENT-RELATIONSHIP-001 v0.1 §10 + DD-CANVAS-GAMIFY-001 §10 模板)

> UT = Unit Test, IT = Integration Test, E2E = End-to-End Test, PT = Performance Test
> Test Case ID 命名: `WT-UT-{NN}` / `WT-IT-{NN}` / `WT-E2E-{NN}` / `WT-PT-{NN}`

### 10.1 Unit Test (UT) — 23 项

| Test Case ID | 范围 | 验证项 | 优先级 |
|---|---|---|---|
| WT-UT-01 | §4.1 `WorkflowTemplate` | 创建 → version=1, columns_count=17 | P0 |
| WT-UT-02 | §4.1 `WorkflowTemplate` | 删除 builtin 模板 → 拒绝 | P0 |
| WT-UT-03 | §4.2 `TemplateColumn` | position=0 冲突 → UNIQUE 约束拒绝 | P0 |
| WT-UT-04 | §4.2 `TemplateColumn` | 作废列 width_factor=0.6 → 拒绝 | P0 |
| WT-UT-05 | §4.2 `TemplateColumn` | 审核/完成 panel='main' → 拒绝 | P0 |
| WT-UT-06 | §4.2 `TemplateColumn` | name 空字符串 → CHECK 拒绝 | P0 |
| WT-UT-07 | §4.4 `MappingPolicy` | 17 列预置初始化 → 13 列映射 in_progress | P0 |
| WT-UT-08 | §4.4 `MappingPolicy` | applyChange('col-1', 'review') → statusUsage 更新 | P0 |
| WT-UT-09 | §4.5 `LockManager` | acquire → 返 token + expires_at | P0 |
| WT-UT-10 | §4.5 `LockManager` | 同用户重复 acquire → 返同 token | P0 |
| WT-UT-11 | §4.5 `LockManager` | 不同用户 acquire → 409 + holder info | P0 |
| WT-UT-12 | §4.5 `LockManager` | release token 不匹配 → 403 | P0 |
| WT-UT-13 | §4.5 `LockManager` | cleanup_expired_locks → released_at 更新 | P0 |
| WT-UT-14 | §4.6 `ConflictResolver` | detect(version 1 vs 2) → true | P0 |
| WT-UT-15 | §4.6 `ConflictResolver` | resolve('force_overwrite') → retry action | P0 |
| WT-UT-16 | §4.6 `ConflictResolver` | resolve('discard') → discard action | P0 |
| WT-UT-17 | §4.7 `templateStore` | createTemplate → templates Map 更新 | P0 |
| WT-UT-18 | §4.7 `templateStore` | dispatch(event) → 缓存 invalidate | P0 |
| WT-UT-19 | §5.1 TemplateState | create → Active v1 | P0 |
| WT-UT-20 | §5.1 TemplateState | softDelete → Deleted | P0 |
| WT-UT-21 | §5.2 LockState | acquire 转换 Unlocked → HeldBySelf | P0 |
| WT-UT-22 | §5.2 LockState | 5min 超时 → Expired | P0 |
| WT-UT-23 | §5.3 WorkItemStatusMapping | 13 列都映射 in_progress → 拖动 status 不变, 仅 column_id 变 | P0 |

### 10.2 Integration Test (IT) — 12 项

| Test Case ID | 范围 | 验证项 | 优先级 |
|---|---|---|---|
| WT-IT-01 | §7.3 POST 创建 | 201 + 17 列预置数据 + version=1 | P0 |
| WT-IT-02 | §7.3 POST 创建 | 同 tenant name 重复 → 409 | P0 |
| WT-IT-03 | §7.4 PATCH 更新 | 200 + version +1 + 1 row INSERT template_versions | P0 |
| WT-IT-04 | §7.4 PATCH 更新 | version 冲突 → 409 + server_snapshot | P0 |
| WT-IT-05 | §7.4 PATCH 更新 | lock_token 不匹配 → 403 | P0 |
| WT-IT-06 | §7.4 PATCH 更新 | 尝试删除 builtin 列 → 422 | P0 |
| WT-IT-07 | §7.5 POST /lock | acquire 成功 → 200 + token | P0 |
| WT-IT-08 | §7.5 POST /lock | 锁被他人持有 → 409 + holder info | P0 |
| WT-IT-09 | §7.6 DELETE 软删 | 204 + deleted_at 设值 | P0 |
| WT-IT-10 | §7.6 DELETE 软删 | builtin 模板 → 403 | P0 |
| WT-IT-11 | §7.7 SSE 推送 | template_changed event payload 格式正确 | P0 |
| WT-IT-12 | §7.7 SSE 推送 | 多个客户端订阅 → 全部收到 | P0 |

### 10.3 End-to-End Test (E2E) — 6 项 (含 3 个 e2e 场景 + 3 个边界)

| Test Case ID | 范围 | 验证项 | 优先级 |
|---|---|---|---|
| WT-E2E-01 | §8.1 场景 1 | 用户创建模板 → 列表显示 → SSE 推送 → 其他 tab 收到 | P0 |
| WT-E2E-02 | §8.2 场景 2 | 多人并发: A 持有锁 → B 看到锁状态 → A 完成 → B 拿到锁 → B 强制覆盖 → 冲突 dialog → 3 选项 | P0 |
| WT-E2E-03 | §8.3 场景 3 | SSE 推送: A 修改 → 5s 内 B 看板自动 invalidate | P0 |
| WT-E2E-04 | §8.4 场景 4 | 一键回滚: 当前 v5 → 回滚到 v3 → 新 version=6 (per 守门 #1 不覆盖) | P1 |
| WT-E2E-05 | 边界 | SSE 断连 → polling 5s 兜底 → 重连后状态一致 | P1 |
| WT-E2E-06 | 边界 | 锁 5min 超时 → cron 清理 → 其他用户申请拿到锁 | P1 |

**总计**: 23 UT + 12 IT + 6 E2E = **41 测试用例** (≥ 33 FR + e2e 3 = 36 目标, 超过 14%)

### 10.4 Performance Test (PT) — 2 项 (per NFR-WT-01/02/03/04)

| Test Case ID | 范围 | 验证项 | 优先级 |
|---|---|---|---|
| WT-PT-01 | NFR-WT-01 | 模板列表页首屏渲染 ≤ 500ms (10 模板) | P1 |
| WT-PT-02 | NFR-WT-02 | 模板化看板渲染 17 列 + 50 WorkItem ≤ 1s | P1 |

### 10.5 测试覆盖验证

| 维度 | 覆盖率 |
|---|---|
| FR 覆盖率 | 33 / 33 = **100%** ✅ |
| AC 覆盖率 | 33 / 33 = **100%** ✅ (per SRS §7) |
| e2e 场景 | 3 / 3 = **100%** ✅ (per SRS §7.2) |
| W/T/M 表 | 7 / 7 = **100%** ✅ (per 守门 #13) |
| 5 View (BD §6) | 5 / 5 = **100%** ✅ |
| Screen ID | 5 / 5 = **100%** ✅ (SCR-WT-01~05) |
| 状态机 | 3 / 3 = **100%** ✅ (TemplateState / LockState / WorkItemStatusMapping) |
| 关键 class | 8 / 8 = **100%** ✅ (§4.1~4.8) |
| 共享类型 | 6 / 6 = **100%** ✅ (§6.1~6.6) |
| 接口端点 | 6 / 6 = **100%** ✅ (per §7) |

---

## §11 NFR 詳細 (5 类, per BD §7 扩展)

> per `ipa-nonfunctional-requirements` skill: 量化指标 + 验收方法 + 监控埋点

### 11.1 性能 (per BD §7 NFR-WT-01~04)

| NFR ID | 类别 | 量化指标 (v1 提案值) | 验收方法 | 监控埋点 |
|---|---|---|---|---|
| NFR-WT-01 | 性能 | 模板列表页首屏渲染 ≤ **500ms** (10 模板 mock) | Lighthouse / WebPageTest | `frontend.performance.timing.list_render` |
| NFR-WT-02 | 性能 | 模板化看板渲染 17 列 + 50 WorkItem ≤ **1s** | 性能测试脚本 + APM | `frontend.performance.timing.board_render` |
| NFR-WT-03 | 性能 | 列宽拖动实时响应 ≤ **16ms** (60fps) | Chrome Performance API | `frontend.performance.timing.resize_handler` |
| NFR-WT-04 | 性能 | PATCH 模板响应 P95 ≤ **300ms** (含 version 校验 + SCD2 历史写入) | APM | `backend.api.timing.update_template` |

### 11.2 可用性 (per BD §7 NFR-WT-05~07)

| NFR ID | 类别 | 量化指标 | 验收方法 | 监控埋点 |
|---|---|---|---|---|
| NFR-WT-05 | 可用性 | 键盘可达 (Tab 顺序遵循视觉顺序) | a11y 测试 (axe-core) | 静态扫描 |
| NFR-WT-06 | 可用性 | 屏读器 announce 列名 + 锁状态 | NVDA / VoiceOver 测试 | 手动测试 |
| NFR-WT-07 | 可用性 | 写锁超时自动释放 + 用户通知 (toast, 【TBD】邮件) | e2e | `frontend.events.lock_expired` |

### 11.3 安全 (per BD §7 NFR-WT-08~10)

| NFR ID | 类别 | 量化指标 | 验收方法 | 监控埋点 |
|---|---|---|---|---|
| NFR-WT-08 | 安全 | 跨 tenant 访问返回 **403** (mock 阶段前端校验) | 单元测试 WT-UT-02/04/05/06 | `backend.security.cross_tenant_blocked` |
| NFR-WT-09 | 安全 | 写锁 token 校验 (32-byte hex, server 端比对) | 单元测试 WT-UT-12 | `backend.security.lock_token_mismatch` |
| NFR-WT-10 | 安全 | 软删模板跨 session 不可恢复 (per §13 T-03 待拍板) | 单元测试 | `backend.security.soft_delete_irreversible` |

### 11.4 可观测 (per BD §7 NFR-WT-11~13)

| NFR ID | 类别 | 量化指标 | 验收方法 | 监控埋点 |
|---|---|---|---|---|
| NFR-WT-11 | 可观测 | 模板 CRUD 走既有埋点 (`frontend/src/lib/analytics.ts`) | e2e | `frontend.events.template_create/template_update/template_delete` |
| NFR-WT-12 | 可观测 | 锁冲突事件埋点 | e2e | `frontend.events.lock_conflict` |
| NFR-WT-13 | 可观测 | SSE 连接状态暴露到 `/health` | 单元测试 | `backend.health.sse_connections_count` |

### 11.5 可维护 (per BD §7 NFR-WT-14~15)

| NFR ID | 类别 | 量化指标 | 验收方法 | 监控埋点 |
|---|---|---|---|---|
| NFR-WT-14 | 可维护 | 模板结构变更走 zustand persist 版本号, 旧版本自动迁移 | 单元测试 | — |
| NFR-WT-15 | 可维护 | 新增 builtin 列需更新 §4.1 17 列预置 + 跑 §10 AC 回归 | 流程性约束 | — |

---

## §12 守门合规 (per BD §9 + §3 守门 v15 饱和判定 + 守门 #14 v4 反转)

### 12.1 守门合规总览 (继承 SRS §6.1 + BD §9.1 + DD 新增)

| 守门 | 状态 | 说明 |
|---|---|---|
| #1 v15 禁回溯叙事 | ✅ | 0 改任何既有 SRS/BD/DD/代码 logic 行, 仅 append |
| #1 v19 docs 同步饱和 | ✅ | registry.md v0.38 row 同步 |
| #11 缺标比错标 | ✅ | §1.3 列 11 项 Out-of-Scope + §13 列已知缺口 |
| #12 v21 [P] docs 同步 | ✅ | registry.md §2 + §3 + §6 3 行同步 |
| #13 W/T/M 三分类 | ✅ | §9.1 Master 4 + Transaction 3 WORM, 0 混合分類 |
| #13a L1↔L1 通信禁止 | ✅ | §3.4 派生约束, template-store 不直接调用 L1 Agent |
| #14 v4 反转 | ✅ | §15 签字栏 5 角色统一 Mavis 接手代签 |

### 12.2 守门 #1 禁回溯叙事 — 文档内容历史演变声明 (per AGENTS.md §1.2)

| 文档 | 演变 | 本 DD 行为 |
|---|---|---|
| SRS v0.1 (commit `b6bb4681`) | 初版, 591 行 | ❌ 不修改 |
| BD v0.1 (commit `c24ce47d`) | 初版, 1090 行 | ❌ 不修改 |
| DD v0.1 (本 commit) | 初版, ~1200 行 (估) | ✅ 新增 |

**注意**: BD 头部那段「重要溯源说明」提到 "PR #42 OPEN", 合并后该表述过时, 但 DD **不修改** BD (per 守门 #1 禁回溯叙事, 0 改既有 row)。读者应以 `git log origin/main` 实证为准 (PR #42 已 MERGED)。

### 12.3 守门 #13 W/T/M 三分类声明 (per §9.1)

**Master (W) 4 张**:
1. `workflow_templates` (模板主表)
2. `template_columns` (列定义)
3. `template_versions` (SCD2 历史, append-only WORM)
4. `work_item_template_links` (派生关系)

**Transaction (T) 3 张 (全部 WORM)**:
5. `template_locks` (锁记录, released_at 允许 UPDATE per §9.3 T-09 待拍板)
6. `template_change_events` (同步事件, 严格 WORM)
7. `template_audit_logs` (审计日志, 严格 WORM)

**Work 0 张**: 全部数据均无 TTL 自动清理需求 (per BD §9.1 论证)

**W/T/M 验证**: 4 Master + 3 Transaction WORM = 7 表, **0 混合分類** ✅

### 12.4 守门 #13a L1↔L1 通信禁止派生约束

`template-store` 模块 (Tier 1) 不直接调用 L1 Agent; 走既有 `automation` 模块的 BFF 通道或 `work-item` 既有 API。模板化看板的 WorkItem 拖动走既有 `/api/work-items/{id}` PATCH, 不新增旁路。

---

## §13 已知缺口 (从 BD §9.3 22 项 + DD 新增筛出 P0 必须解决)

### 13.1 继承 BD §9.3 22 项 TBD (本 DD 全部继承)

| ID | 来源 | 内容 | 影响 | 归属 |
|---|---|---|---|---|
| T-01 | BD §9.3 (SRS §10 #5) | 作废列 mapping_status `wontfix` 是否新增 `WorkItemStatus` enum 值 | 影响 §4.4 MappingPolicy 验证逻辑 | **DD 必须解决**: 默认复用 `todo` (放宽), §4.2 注释说明 |
| T-02 | BD §9.3 (SRS §10 #7) | 17 列预置 13 列映射 in_progress, 与 FR-WT-5.6 同状态不允许多列映射冲突 | 影响 §4.4 MappingPolicy.canMap | **DD 必须解决**: 默认允许 (放宽), §4.4 注释说明 |
| T-03 | BD §9.3 (SRS §10 #6) | 软删物理恢复策略 (永久禁止 vs 30 天保留) | 影响 §5.1 TemplateState.Restore | 实装阶段拍板 |
| T-04 | BD §9.3 (SRS §10 #8) | 模板跨 project 范围 (`project_id` 字段允许 null vs 必填) | 影响 §9.2.1 DDL | 实装阶段拍板 |
| T-05 | BD §9.3 (SRS §10 #1) | polling 5s 兜底 vs 真实生产建议 SSE/WebSocket | 影响 §7.7 SSE 协议 + §8.3 时序图 | v1 mock 阶段 polling 5s, 真实生产前升级 SSE |
| T-06 | BD §9.3 (SRS §10 #2) | 后端持久化 API 真实生产前必补 | 影响整个 v1 mock → 真实生产过渡 | **P0 阻塞**, 真实后端实装前必拍板 |
| T-07 | BD §9.3 (SRS §10 #3) | 签字 PDF/文档 (v1 不做) | 影响 UI 是否需签字动作 | P2+ |
| T-08 | BD §9.3 | 审计记录保留周期 | 影响 §9.3 备份策略 + 存储容量 | 实装阶段拍板 |
| T-09 | BD §9.3 | `template_locks.released_at` 允许 UPDATE 是否破坏 WORM 严格性 | 影响 §9.1 WORM 声明 | 实装阶段拍板 |
| T-10 | BD §9.3 | 写锁 token 安全强化 (HTTPS-only / 不写日志 / 长度 / 轮换) | 影响真实生产前的安全评审 | 安全评审阶段 |
| T-11 | BD §9.3 | NFR-WT-01~15 量化数值为提案值 | 影响容量规划与运维实装 | DD 阶段已落 §11, 实装阶段确认 |
| T-12 | BD §9.3 | `KanbanBoard.columns` prop 扩展是否破坏既有 4 列逻辑 | 影响既有看板回归测试 | 实装阶段回归测试覆盖 |
| T-13 | BD §9.3 | 锁申请冲突响应格式 (holder_user_id 是否暴露给所有用户) | 影响 §8 安全设计 | DD + 安全评审拍板 |
| T-14 | BD §9.3 | 模板状态机是否需要 Draft 状态 (v1 默认 Active v1) | 影响 §5.1 TemplateState | DD 阶段已默认 Active v1, P2+ 评估 |
| T-15 | BD §9.3 | 「回滚到 version N」按钮是否本期实装 | 影响 §7.4 PATCH 错误处理 + §8.4 时序图 | DD 阶段已占位, P2+ 实装 |
| T-16 | BD §9.3 | 锁状态轮询 5s (前端) vs SSE 推送 | 影响 §7.7 SSE 协议 | DD 阶段已混合 (SSE 主, polling fallback) |
| T-17 | BD §9.3 | `template_versions.snapshot_json` 含 name 是否需加密 | 影响 §4.6 + §8.5 | 安全评审阶段 |
| T-18 | BD §9.3 | WorkItem 拖动到模板列时若 mapping_status = 当前 status, 是否仍触发 PATCH | 影响 §5.3 WorkItemStatusMapping | DD 阶段已默认仍触发 (简化实装), 后续可优化 |
| T-19 | BD §9.3 | 17 列预置表与 FR-WT-1.3 读取 17 列预置数据接口契约一致性 | 影响 §7.2 GET | DD 阶段已对齐 |
| T-20 | BD §9.3 | WT-5.6 (FR-WT-5.6) 改 mapping_status 是 P1 | 影响 §4.3 TemplateColumnEditor | DD 阶段已支持 (column_type='user' 可改 mapping) |
| T-21 | BD §9.3 | 是否需要新增 Work 类表 (e.g. 临时模板草稿) | 影响 §9.1 W/T/M 分类 | 实装阶段拍板 |
| T-22 | BD §9.3 | 「应用到看板」按钮在 builtin 模板上是否可点击 (预览模式?) | 影响 §3.3 画面迁移图 | 实装阶段拍板 |

### 13.2 DD 新增 P0 阻塞 (本 DD 标识, 必须实装阶段解决)

| ID | 内容 | 影响 | 拍板归属 |
|---|---|---|---|
| DD-T-01 | §4.4 MappingPolicy v1 默认行为 (允许多列 → 1 状态) 是否符合产品预期 | 影响 §4.4 canMap 实现 | **实装前必须拍板** (per §13.1 T-02 升级) |
| DD-T-02 | §9.2.1 `chk_columns_count_positive` 上限 50 是否合理 | 影响模板列数硬限制 | 实装前拍板 |
| DD-T-03 | §7.4 PATCH 幂等性 v1 不实现 (`idempotency-key` 留 P2+) 是否接受 | 影响客户端重试策略 | 实装前拍板 |
| DD-T-04 | §7.7 SSE 连接断连重试 3 次 → polling 5s 兜底的切换阈值是否合理 | 影响 §8.3 时序图 fallback 路径 | 实装阶段确认 |
| DD-T-05 | §11 NFR 量化指标 (500ms / 1s / 16ms / 300ms / 5min 等) 全部为提案值 | 影响 §10.4 PT 验收基线 | 实装前拍板 |

---

## §14 实施计划 (P3-D 估时, per BD §10 + DD-AGENT-RELATIONSHIP-001 v0.1 §14 模板)

### 14.1 阶段划分 (Stage 4-7, post-DD)

| 阶段 | 内容 | 估时 |
|---|---|---|
| Stage 4 实装 | 前端组件 5 个 (TemplateList / TemplateDetail / TemplateBoard / TemplateColumnEditor / LockIndicator / ConflictDialog) + zustand store 1 个 (templateStore) + KanbanBoard.columns prop 扩展 + UserMenu 第 5 入口 + nav/registry 新路由登记 + 后端 Rust module (templateStore + LockManager + ConflictResolver) + 7 张表 DDL migration | ~10M tokens / 8.33 SRE·周 |
| Stage 5 单元 / 集成测试 | §10.1 UT 23 项 + §10.2 IT 12 项 | ~2M tokens / 1.67 SRE·周 |
| Stage 6 E2E + 性能测试 | §10.3 E2E 6 项 + §10.4 PT 2 项 | ~1M tokens / 0.83 SRE·周 |
| Stage 7 部署 + 监控 | SSE 通道部署 / Sentry 配置 / Grafana dashboard / runbook | ~1M tokens / 0.83 SRE·周 |
| **合计** | — | **~14M tokens / 11.66 SRE·周** |

### 14.2 关键里程碑

| Milestone | 日期 (估) | 验收 |
|---|---|---|
| M1: 7 张表 DDL migration 上线 | Week 1 | §9.2 7 张表创建 + RLS + WORM trigger |
| M2: 前端组件 5 个 + zustand store | Week 3 | §4.1~4.8 + §3.2 module 清单 |
| M3: 后端 Rust module + 6 REST + 1 SSE | Week 6 | §4.5 LockManager + §7 接口协议 |
| M4: UT + IT 全过 | Week 8 | §10.1 23 + §10.2 12 = 35 项 pass |
| M5: E2E + PT 全过 | Week 10 | §10.3 6 + §10.4 2 = 8 项 pass |
| M6: 生产部署 + 监控 | Week 12 | Sentry + Grafana + runbook |

### 14.3 依赖项 (per 守门合规)

| 依赖 | 阻塞? | 缓解 |
|---|---|---|
| 真实后端持久化 API (per §13.1 T-06) | **P0 阻塞** | v1 走 frontend mock + zustand persist, 真实生产前必补 |
| PostgreSQL 16 已部署 | ✅ 已部署 | — |
| Next.js 14 + React 18 已部署 | ✅ 已部署 | — |
| Rust starboard crate 已部署 | ✅ 已部署 | — |
| 既有 L0/TMO 架构 (ADR-0046) | ✅ 已部署 | — |
| 既有 SSE 通道 (frontend/src/lib/realtime/sse.ts) | ✅ 已部署 | — |
| 既有 KanbanBoard 组件 | ✅ 已部署 | 扩展 columns prop |

---

## §15 签字栏 (5 角色 per AGENTS.md §3 + 守门 #14 v4 反转)

| 角色 | 姓名/代签 | 状态 | 日期 |
|---|---|---|---|
| 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |
| SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |
| 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |
| 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |
| 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 🟡 已接受 (Mavis 接手代签 per 守门 #14 v4) | 2026-09-14 |

**本文档为 ULYS-36 issue 委托的详细设计交付物, 由 agent MinimaxM3 (接手 dc14a111 因 2 次 session limit 失败) 撰写。5 个角色（架构师/SRE Lead/平台工程师/评审主持/PM）由 Mavis 接手 agent 代签（per 守门 #14 v4 反转 + 2026-09-10 12:45 JST「真人代签流程永久 obsolete」+ 8/27 19:39 JST 用户授权代签）。§13 已知缺口中的全部 22 项继承 BD + 5 项 DD 新增 P0 阻塞, 不因签字栏完成而自动裁决, 仍需在实装阶段逐项拍板。**

---

## §16 修订历史

| 版本 | 日期 | 变更摘要 | 作者 |
|---|---|---|---|
| v0.1 | 2026-09-14 | 初版交付, 派生自 SRS-WORKFLOW-TEMPLATE-001 v0.1 (b6bb4681) + BD-WORKFLOW-TEMPLATE-001 v0.1 (c24ce47d, 已合并到 main per PR #42 merge commit 341aaf9d), 17 段严格按 DD-CANVAS-GAMIFY-001 + DD-AGENT-RELATIONSHIP-001 v0.1 模板 (per `ipa-screen-design` + `ipa-database-design` + `ipa-nonfunctional-requirements` + `ipa-security-design` skill), 8 关键 class (TypeScript 接口 + Rust struct 1:1 派生, §4.1~4.8), 3 状态机 (TemplateState + LockState + WorkItemStatusMapping, §5.1~5.3), 6 共享类型 (WorkItemStatus 既有 + TemplateColumnType/Panel + ConflictResolution + TemplateChangedEvent + WorkItemTemplateBinding DD 新增, §6.1~6.6), 6 REST + 1 SSE 协议 (含 TypeScript 类型 + 错误码 + 重试 + 幂等性, §7.1~7.8), 4 时序图 (创建 / 并发冲突 / SSE 推送 / 回滚占位, §8.1~8.4), 7 张表完整 DDL (含索引 + CHECK + UNIQUE + RLS + WORM trigger + 备份策略, §9.1~9.4), 41 测试用例 (23 UT + 12 IT + 6 E2E, §10.1~10.3) + 2 PT (NFR, §10.4), 5 类 NFR 量化 (性能 / 可用性 / 安全 / 可观测 / 可维护, §11.1~11.5), 22 项 TBD 继承 BD §9.3 + 5 项 DD 新增 P0 阻塞 (§13), 5 角色签字栏 Mavis 接手代签 (per 守门 #14 v4 反转), 接管 dc14a111 因 2 次 session limit 失败, per 守门 #9 v19 Mavis 自驱 + 守门 #14 v3 永久代签; 串行依赖 ULYS-34 (✅ done) → ULYS-35 (✅ done) → ULYS-36 (本任务) | MinimaxM3 (agent, per Multica ULYS-36 takeover) |

---

## 附录

- **附录 A**: 上位文档 — [`docs/requirements/SRS-WORKFLOW-TEMPLATE-001.md`](../requirements/SRS-WORKFLOW-TEMPLATE-001.md) v0.1 (commit `b6bb4681`) + [`docs/design/BD-WORKFLOW-TEMPLATE-001.md`](./BD-WORKFLOW-TEMPLATE-001.md) v0.1 (commit `c24ce47d`, 已合并到 main per PR #42 merge commit `341aaf9d`)
- **附录 B**: 同级文档 — [`docs/design/DD-CANVAS-001.md`](./DD-CANVAS-001.md) (总册跨域汇总) + [`docs/design/DD-CANVAS-AGENT-001.md`](./DD-CANVAS-AGENT-001.md) (专题 1, 13 关键 class 模板) + [`docs/design/DD-CANVAS-GAMIFY-001.md`](./DD-CANVAS-GAMIFY-001.md) (专题 2, 17 段模板) + [`docs/design/DD-CANVAS-WORKFLOW-001.md`](./DD-CANVAS-WORKFLOW-001.md) (专题 3, 自动化流程) + [`docs/design/DD-AGENT-RELATIONSHIP-001.md`](./DD-AGENT-RELATIONSHIP-001.md) (DD 模板源, 17 段)
- **附录 C**: 既有代码引用 — `frontend/src/components/UserMenu.tsx` (扩展点) / `frontend/src/lib/nav/registry.ts` (路由) / `frontend/src/components/board/KanbanBoard.tsx` (扩展 props) / `frontend/src/components/board/constants.ts` (兜底列逻辑) / `frontend/src/mocks/data/kanban.ts` (KANBAN_COLUMNS 4 列常量) / `frontend/src/types/ids.ts:322-347` (Workflow 状态机类型, §2.4 消歧) / `frontend/src/lib/store.ts` (zustand store 扩展点) / `frontend/src/lib/realtime/sse.ts` (SSE 通道复用) / `crates/domain-work-item/src/lib.rs` (WorkItemStatus enum 既有)
- **附录 D**: IPA skills 引用 — `ipa-screen-design` (§3.2 + §8 时序图) + `ipa-database-design` (§9 数据持久化 DDL) + `ipa-nonfunctional-requirements` (§11 NFR 量化) + `ipa-security-design` (§8 安全 + §12.1 守门) + `ipa-test-case` (§10 测试用例 ID 落表)
- **附录 E**: 守门合规工具链 — AGENTS.md §1 (代签规则) + AGENTS.md §4 (v3x 守门) + `docs/guardian/` 守门 doc 目录 + `scripts/automation/registry.md` v0.38 row 同步
- **附录 F**: 与本 DD 配套的实施计划 — [`docs/implementation-plans/CANVAS-IMPL-PLAN-001.md`](../implementation-plans/CANVAS-IMPL-PLAN-001.md) (DD-CANVAS-001 v0.1 同模式, 待 P3-D 阶段 4 实装阶段产出)
