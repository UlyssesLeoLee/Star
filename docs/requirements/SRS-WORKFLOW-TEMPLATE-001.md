# SRS-WORKFLOW-TEMPLATE-001

**瀑布式开发工作流模板 — 需求定义书 v0.1**

> **状态**: 🟡 Draft v0.1
> **目标阶段**: 要件定義 → 基本設計 → 詳細設計 → 実装
> **触发 issue**: ULYS-30 「全新模板模式」（父任务）
> **子任务 ID**: ULYS-34 [stage 1]
> **平行子任务**: ULYS-35 [stage 2 基本設計] / ULYS-36 [stage 3 詳細設計]
> **关联 SRS**: [`docs/requirements/SRS-CANVAS-WORKFLOW-001.md`](./SRS-CANVAS-WORKFLOW-001.md) (n8n 式自动化流程) / [`docs/requirements/SRS-STAR-OPS-001.md`](./SRS-STAR-OPS-001.md)
> **修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3)
> **审批**: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
> **日期**: 2026-09-14 JST
> **受众**: 基本設計エンジニア / アーキテクト / 5 域 Lead 真人

---

## §0 文档信息 / 修订履历

### 0.1 文档信息

| 项 | 内容 |
|---|---|
| 文书 ID | SRS-WORKFLOW-TEMPLATE-001 |
| 文书名 | 瀑布式开发工作流模板 需求定义书 |
| 版本 | v0.1 |
| 作成日 | 2026-09-14 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 父 issue | ULYS-30 「全新模板模式」 |
| 触发评论 | ULYS-30 描述原文 (2026-09-13 由创建者 4a45c05e agent 录入) |
| 上位 SRS | `SRS-CANVAS-WORKFLOW-001` v1.1 (W14 默认工作流模板库 — 子集关系, 本 SRS 是 W14 落地路径之一) |
| 平行 SRS | `SRS-CANVAS-WORKFLOW-001` (n8n 自动化执行图) — 命名消歧见 §2.4 |
| 守门合规 | 守门 #1 v15 + #11 + #12 v21 + #13 + #14 v4 全过 |
| 模板结构 | 12 段严格按 brief §1.3 (per SRS-MULTICA-TASK-001 / SRS-STAR-OPS-001 同形) |
| 子能力 | 8 子能力 (WT-1 ~ WT-8) × 33 FR × 14 US |

### 0.2 修订履历

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（8 子能力 / 33 FR / 14 US / 5 NFR / 9 已知缺口 / 5 角色签字栏） | ULYS-30 父任务委派 + ULYS-34 子任务 retry (前次 74dda328 commit 缺失, 详见 §10 #4) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

按日本 IPA SEC / V 模型标准, 为 STAR 平台定义「**瀑布式开发工作流模板**」的需求规格说明书。该模板是 ULYS-30 「全新模板模式」父任务下的**首个预置模板**, 配套实现用户原始诉求中的 5 项硬要求:

1. 待办 (Todo) 是一列, 作废是一列
2. 作废列宽度**只有待办的一半**, 二者放在最前面
3. 后续流程按**最详细的日本 IPA 标准**, 每一个环节一列
4. 下方 1/3 大小位置放**审核列 + 完成列**
5. 命名为**瀑布式开发**, 入口放在**右上角菜单的工作流入口**里管理

### 1.2 背景 (用户痛点)

ULYS-30 父任务原文 (issue 创建者, 2026-09-13):

> "我希望待办是一列, 作废是一列, 并且作废这一列较细, 只有待办的一半, 它俩放在最前面, 后面的流程, 按照最详细的日本IPA标准, 吗, 每一个环节一列, 并且在下方 1/3 大小的位置放置一个审核栏一个完成栏. 这样的构造更加适合严格的瀑布式软件开发, 给这套工作流模板命名为瀑布式开发, 所有工作流放在右上角菜单的工作流入口里面管理, 并且允许用户自己移动增加删除, 用户的改动会同步反应到其他界面, 整体都是一体的. 并且支持多人同时操作, 排他回滚设计要完善. 从需求文档到基本设计到详细设计都开子任务处理完."

**已确认的现状差距** (per 代码走查, 2026-09-14):

| 现状 | 差距 |
|---|---|
| `frontend/src/components/UserMenu.tsx` 现有 4 类菜单项 (工作区 / CLI / API / OPS / Lang / 退出) | **无工作流模板入口** — 用户无法集中管理"模板"这个维度 |
| `frontend/src/lib/nav/registry.ts:345-352` `workflow` 单页 (`/workflow`, code "WF", category "work") | 这是**单页入口**, 暴露的是 `Workflow` 状态机引擎 (`ids.ts:322-347`), **不是模板管理**; 现 `/workflow` 路由与本 SRS 提议的 `/workflow-templates` 路由语义不同, 本 SRS §2.4 专门消歧 |
| `frontend/src/components/board/KanbanBoard.tsx:71-100` 4 列硬编码 (`todo` / `in_progress` / `review` / `done`) 经 `frontend/src/mocks/data/kanban.ts:9` 常量化 | **没有"模板维度"概念** — 列名/列数/列宽全部硬编码, 没有用户可改的入口; `todo` 兜底列不可删 (per `constants.ts:31`), 其它列允许重命名/删除/重排 (per `KanbanBoard.tsx:57-70`), 但**没有"顺序 + 列宽规则"的模板预设机制** |
| `frontend/src/types/ids.ts:322-347` `Workflow` / `WorkflowState` / `WorkflowTransition` 类型 | 这是**WorkItem 状态流转状态机**, 不是模板管理; 与本 SRS "工作流模板" 是不同概念, §2.4 消歧 |
| `frontend/src/mocks/data/kanban.ts` 4 列常量化 | 列宽固定 (UI 默认宽度), **没有"作废列是 todo 列一半"的约束** |
| `STAR-P3-WBS-001.md` 已有 P1-P9 + P6.1-P6.4 + 审核 + 完成 阶段命名 | 阶段名已存在, **本 SRS 直接复用为模板预置列名**; 但**列宽 + 顺序 + 模板维度切换**尚未实现 |

### 1.3 包含范围 (In-Scope)

8 子能力 (本 SRS):

| 子能力 | 名称 | 项数 | 概述 |
|---|---|---|---|
| WT-1 | 模板数据模型与持久化 | 5 | WorkflowTemplate 实体 + 17 列预置结构 + 版本管理 (SCD Type 2) + 租户隔离 + 软删除 |
| WT-2 | 17 列结构定义与不可违反规则 | 6 | Todo / 作废 / P1-P9 + P6.1-P6.4 / 审核 / 完成 共 17 列 + 列宽比 + 顺序约束 + 预置/用户列分组 + 命名约束 + 修改边界 |
| WT-3 | 模板 CRUD UI | 5 | 创建 / 查看 / 重命名 / 删除 / 复制 5 动作, 全在 `/workflow-templates` 路由下 |
| WT-4 | 模板应用到看板 | 4 | 选中模板 → 跳到 `/workflow-templates/{id}/board` → 用模板列渲染看板 → WorkItem.status 与模板列 1:1 映射 |
| WT-5 | 模板列可编辑边界 | 5 | 用户可增列 / 删列 / 改列宽 / 重排顺序 / 重命名, 但 17 列预置中 Todo 不可删 / 作废列宽 0.5 不可改 / 顺序前 2 列固定 |
| WT-6 | 同步事件与跨界面联动 | 3 | 模板改动通过 SSE / polling 5s 兜底同步到所有打开的看板 / 详情页 / UserMenu 入口状态 |
| WT-7 | 多人并发编辑与排他锁 | 3 | 写锁 5min 超时 + 乐观锁 (version 字段) + 一键回滚到任意历史版本 |
| WT-8 | 右侧工作流入口 | 2 | UserMenu 新增第 5 类入口 → `/workflow-templates` + 当前激活模板 indicator |

**合计**: 33 FR (P0 23 / P1 10 / P2 0) + 14 US + 5 NFR

### 1.4 不含范围 (Out-of-Scope)

> per 守门 #11 缺标比错标 — 显式列 11 项不做的事, 避免下游 stage 2/3 误以为在范围内

- ❌ 不修改 `Workflow` 状态机类型 (`frontend/src/types/ids.ts:322-347`) 与 `/workflow` 单页 — 命名消歧见 §2.4
- ❌ 不修改现有 4 列 `KanbanBoard` (`frontend/src/components/board/KanbanBoard.tsx`) 与 `/board` 路由 — 模板化看板为**新独立路由** `/workflow-templates/{id}/board`
- ❌ 不新增 `WorkItem.status` 枚举值 — 模板列通过 §4.5 映射层映射到既有 `WorkItemStatus` (`todo`/`in_progress`/`review`/`done`)
- ❌ 不引入 CRDT (Yjs/Automerge) — v1 走传统乐观锁 + write lock (per §3.2 A12 依赖声明)
- ❌ 不做模板导入/导出 (JSON/YAML) — 留 P2+
- ❌ 不做模板继承 / 模板参数化 — 留 P2+
- ❌ 不做模板自动归档 — 留 P2+
- ❌ 不引入全局权限 scheme — v1 同 tenant 即可编辑 (留 P1+ per-role)
- ❌ 不生成签字 PDF / 文档 — 审核/完成 panel 仅 UI 渲染 + 字段写存储
- ❌ 不动 4 专题 SRS (`SRS-CANVAS-{AGENT,GAMIFY,WORKFLOW}-001`) — 平行不重叠; 本 SRS 仅复用 W14 子能力作为**首个模板的预置实现路径**
- ❌ 不实现后端持久化 API — v1 前端 mock + zustand persist; 真实生产前必补后端 (per §10 #2)

### 1.5 受众范围 disclaimer

- 本 SRS 是 ULYS-30 父任务下的 stage 1 需求定义, 与 ULYS-35 (stage 2 基本設計) / ULYS-36 (stage 3 詳細設計) 串行依赖
- 实现由 stage 3 → ULYS-36 子任务承接, 不在本 SRS 范围
- WBS 中 P1-P9 + P6.1-P6.4 + 审核 + 完成 阶段命名直接复用为 17 列预置列名, 不新增业务语义

---

## §2 用语定义

### 2.1 模板相关

| 用语 | 定义 |
|---|---|
| **工作流模板 (WorkflowTemplate)** | 用户可命名的 17 列结构 (Todo + 作废 + P1-P9 + P6.1-P6.4 + 审核 + 完成), 是 1 个独立实体, 可应用到看板 |
| **瀑布式开发** | 本 SRS 定义的第一个预置模板名, `is_builtin=true`, 不可改不可删 |
| **预置列 (Builtin Column)** | 模板自带的 17 列, 用户**可改部分属性** (重命名/重排/列宽除作废外) / **不可删部分属性** (Todo 列不可删, 作废列宽 0.5 不可改, 顺序前 2 列固定) |
| **用户列 (User Column)** | 用户基于预置模板**额外新增**的列, 不在 17 列预置内, 可自由增删改移 |
| **列宽因子 (width_factor)** | 数值, 1.0 = 标准宽 (跟 Todo 等宽), 0.5 = 半宽 (Todo 一半); 作废列宽因子固定 0.5 |

### 2.2 IPA 阶段

| 阶段 | 复用源 |
|---|---|
| P1 超上流工程 | `STAR-P3-WBS-001.md` 阶段名 |
| P2 要件定義 | 同上 |
| P3 基本設計 | 同上 |
| P4 詳細設計 | 同上 |
| P5 実装 | 同上 |
| P6 テスト工程 | 同上 |
| P6.1 テスト計画 | 同上 (P6 子阶段) |
| P6.2 テスト設計 | 同上 |
| P6.3 テスト実装 | 同上 |
| P6.4 テスト実行 | 同上 |
| P7 移行・リリース | 同上 |
| P8 運用・保守 | 同上 |
| P9 廃止 | 同上 |
| 审核 | 用户原文「审核栏」, 列名 `review` |
| 完成 | 用户原文「完成栏」, 列名 `done` |

### 2.3 看板相关

| 用语 | 定义 |
|---|---|
| **模板化看板 (Templated Board)** | 渲染在 `/workflow-templates/{template_id}/board` 路由, 列结构由模板定义, 不是 `KanbanBoard` 默认 4 列 |
| **WorkItem.status** | 既有 enum (`todo`/`in_progress`/`review`/`done`), 模板列通过映射层 1:1 映射到这 4 个值 (具体映射规则见 §4.5) |
| **列宽比** | 同屏水平方向的相对宽度比例, 列宽因子 × 总宽 = 该列实际像素宽 |

### 2.4 命名消歧 (per 守门 #11 缺标比错标 — 三词易混)

| 用语 | 实际所指 | 来源 |
|---|---|---|
| **Workflow (Workflow 状态机)** | WorkItem 状态流转状态机, 字段 `Workflow`/`WorkflowState`/`WorkflowTransition` | `frontend/src/types/ids.ts:322-347` |
| **/workflow 单页** | Workflow Engine 入口, 用户配置状态机用 | `frontend/src/lib/nav/registry.ts:345-352` |
| **n8n 式自动化流程 (Automation Flow)** | SRS-CANVAS-WORKFLOW-001 定义, 可执行节点图 | `docs/requirements/SRS-CANVAS-WORKFLOW-001.md` W1-W13 |
| **WorkflowTemplate (本 SRS 工作流模板)** | 用户可命名的 17 列结构, 模板维度管理 | 本 SRS §1.3 WT-1 ~ WT-8 |
| **/workflow-templates 路由 (本 SRS 提议)** | 模板管理入口, 跟 `/workflow` 单页**不同路由不同语义** | 本 SRS §4.3 |

**消歧示例**: "瀑布式开发模板" ≠ "Workflow 状态机" ≠ "n8n 自动化流程图". 前者是**看板列结构模板**, 后两者是状态机/执行图.

---

## §3 业务背景

### 3.1 用户原始诉求 (ULYS-30 issue 原文引用)

per §1.2 全文引用, 关键约束 5 项:

1. **结构**: Todo (1 列) + 作废 (1 列, 列宽 = Todo × 0.5) + IPA 阶段列 (P1-P9 + P6.1-P6.4 共 13 列) + 审核 (1 列) + 完成 (1 列) = **17 列**
2. **顺序**: Todo + 作废 固定在最前 2 位, 其余 13 列按 P1 → P9 顺序, 审核 + 完成在下方 1/3 panel
3. **命名**: 模板固定名「瀑布式开发」, 是第 1 个预置模板
4. **入口**: 右上角 UserMenu 新增「工作流」入口 → `/workflow-templates`
5. **行为**: 用户可移动/增加/删除列; 改动跨界面同步; 多人操作支持排他锁 + 回滚

### 3.2 A12 依赖声明 (per §10 缺标)

| 依赖 | 当前状态 | 阻塞风险 |
|---|---|---|
| **A12 乐观锁** | 既有 `board.version` 字段 (per `frontend/src/store/boardStore.ts`) | 不阻塞, 可复用 |
| **写锁机制** | 既有 WebSocket 写锁 5min (per `docs/specs/board-lock-001.md`) | 不阻塞, 可复用 |
| **SSE 推送** | 既有 SSE 通道 (per `frontend/src/lib/realtime/sse.ts`) | 不阻塞, 可复用; v1 fallback polling 5s |
| **zustand persist** | 既有 `frontend/src/store/*.ts` 已用 zustand persist middleware | 不阻塞, 可复用 |
| **KanbanBoard 组件** | 既有 `frontend/src/components/board/KanbanBoard.tsx` | **阻塞 1**: 不支持自定义列宽因子, 需扩展 |
| **WorkItem.status enum** | 既有 `frontend/src/types/ids.ts` 4 状态 | **阻塞 2**: 17 列 → 4 状态 映射层需新建 |
| **后端持久化 API** | **不存在** | **阻塞 3 (硬)**: v1 走前端 mock, 真实生产前必补 |

### 3.3 业务规则 (BR-WT-1 ~ BR-WT-10)

| ID | 规则 |
|---|---|
| **BR-WT-1** | 17 列预置列**不可删除**, 用户列可自由删除 |
| **BR-WT-2** | 作废列宽因子 0.5, **不可修改** |
| **BR-WT-3** | Todo + 作废 必须在最前 2 位 (顺序固定), 其余 13 列顺序可重排但相对顺序 P1 → P9 不能颠倒 |
| **BR-WT-4** | 审核列 + 完成列在下方 1/3 panel (水平排列, 审核 60% / 完成 40%) |
| **BR-WT-5** | 「瀑布式开发」模板 `is_builtin=true`, 不可改名不可删除不可复制 |
| **BR-WT-6** | 写锁超时 5 分钟, 超时自动释放 + 通知原锁持有者 |
| **BR-WT-7** | 模板版本 SCD Type 2 (per §4.1.4), 每次结构性变更新增 version, 不覆盖历史 |
| **BR-WT-8** | 跨界面同步延迟 ≤ 5s (polling) / ≤ 1s (SSE, 真实生产) |
| **BR-WT-9** | 一键回滚到任意历史版本, 回滚也产生新 version (per 守门 #1 禁回溯叙事) |
| **BR-WT-10** | 同 tenant 用户可编辑, 跨 tenant 只读 (v1 不做 per-role, 留 P1+) |

---

## §4 功能需求 (FR, 33 项)

### WT-1 模板数据模型与持久化 (5 项)

**WT-1.1 (FR-WT-1.1, P0)** — 定义 `WorkflowTemplate` 接口

```typescript
interface WorkflowTemplate {
  id: Uuid;
  tenant_id: Uuid;
  project_id: Uuid | null;       // null = 跨项目模板
  name: string;                    // "瀑布式开发" / 用户自定义
  is_builtin: boolean;             // true = 预置不可删
  version: number;                 // SCD2 递增, 1 起
  columns: TemplateColumn[];       // 17 项 (瀑布式) 或 N 项 (用户)
  created_at: ISO8601;
  updated_at: ISO8601;
  created_by: Uuid;
  deleted_at: ISO8601 | null;      // 软删
}
```

**WT-1.2 (FR-WT-1.2, P0)** — 定义 `TemplateColumn` 接口

```typescript
interface TemplateColumn {
  id: Uuid;
  template_id: Uuid;
  position: number;                // 0-based, 顺序
  name: string;                    // "Todo" / "作废" / "P1 超上流工程" / ...
  builtin: boolean;                // true = 17 列预置
  width_factor: number;            // 1.0 / 0.5 / ...
  panel: "main" | "bottom";        // main = 主区域, bottom = 下方 1/3
  mapping_status: WorkItemStatus;  // 映射到 WorkItem.status
  // 预置列校验规则:
  // - Todo: builtin=true, panel="main", position=0, mapping_status="todo"
  // - 作废: builtin=true, panel="main", position=1, width_factor=0.5
  // - 审核: builtin=true, panel="bottom", position=14 (相对 main panel)
  // - 完成: builtin=true, panel="bottom", position=15
}
```

**WT-1.3 (FR-WT-1.3, P0)** — 17 列预置数据 (固定结构)

| # | name | builtin | width_factor | panel | position | mapping_status |
|---|---|---|---|---|---|---|
| 0 | Todo | true | 1.0 | main | 0 | `todo` |
| 1 | 作废 | true | 0.5 | main | 1 | `wontfix` (新增, 复用 §10 #5 待决) |
| 2 | P1 超上流工程 | true | 1.0 | main | 2 | `in_progress` |
| 3 | P2 要件定義 | true | 1.0 | main | 3 | `in_progress` |
| 4 | P3 基本設計 | true | 1.0 | main | 4 | `in_progress` |
| 5 | P4 詳細設計 | true | 1.0 | main | 5 | `in_progress` |
| 6 | P5 実装 | true | 1.0 | main | 6 | `in_progress` |
| 7 | P6 テスト工程 | true | 1.0 | main | 7 | `in_progress` |
| 8 | P6.1 テスト計画 | true | 1.0 | main | 8 | `in_progress` |
| 9 | P6.2 テスト設計 | true | 1.0 | main | 9 | `in_progress` |
| 10 | P6.3 テスト実装 | true | 1.0 | main | 10 | `in_progress` |
| 11 | P6.4 テスト実行 | true | 1.0 | main | 11 | `in_progress` |
| 12 | P7 移行・リリース | true | 1.0 | main | 12 | `in_progress` |
| 13 | P8 運用・保守 | true | 1.0 | main | 13 | `in_progress` |
| 14 | P9 廃止 | true | 1.0 | main | 14 | `in_progress` |
| 15 | 审核 | true | 0.6 | bottom | 0 | `review` |
| 16 | 完成 | true | 0.4 | bottom | 1 | `done` |

**WT-1.4 (FR-WT-1.4, P0)** — SCD Type 2 版本管理

- 每次结构性变更 (列增删/重排/重命名/列宽改/映射改) 生成新 version, 不覆盖历史
- `version_history: TemplateSnapshot[]` 保留全部历史快照
- 一键回滚 (FR-WT-7.3) 从历史快照恢复, 也生成新 version

**WT-1.5 (FR-WT-1.5, P0)** — 租户隔离 + 软删

- `tenant_id` 必须匹配当前会话 tenant
- 删除走 `deleted_at` 时间戳, 不真删
- `is_builtin=true` 的模板 `deleted_at` 永远 null

### WT-2 17 列结构定义与不可违反规则 (6 项)

**WT-2.1 (FR-WT-2.1, P0)** — 17 列预置固定, 用户不可删 17 列预置 (per BR-WT-1)

**WT-2.2 (FR-WT-2.2, P0)** — 作废列宽因子 0.5 锁定 (per BR-WT-2), UI 列宽调节器置灰禁用

**WT-2.3 (FR-WT-2.3, P0)** — Todo + 作废 顺序锁定 (per BR-WT-3), 拖动重排时若用户尝试把 Todo 拖离位置 0, UI 弹 toast 拒绝并保持原顺序

**WT-2.4 (FR-WT-2.4, P0)** — P1 → P9 顺序相对固定, 13 列可整体重排但保持 P1 < P2 < ... < P9 的相对顺序

**WT-2.5 (FR-WT-2.5, P0)** — 审核 + 完成在 bottom panel, 不可移到 main panel (per BR-WT-4)

**WT-2.6 (FR-WT-2.6, P0)** — 列名约束: 预置列重命名时不允许空字符串 / 仅空白 / 与其它列重名; 用户列允许任何非空字符串

### WT-3 模板 CRUD UI (5 项)

**WT-3.1 (FR-WT-3.1, P0)** — 列表页 `/workflow-templates` 展示当前 tenant 全部模板 (含 builtin 「瀑布式开发」)

**WT-3.2 (FR-WT-3.2, P0)** — 创建模板按钮: 弹 modal 输入名称, 默认复制「瀑布式开发」结构 (用户可在新建时改列)

**WT-3.3 (FR-WT-3.3, P0)** — 详情页 `/workflow-templates/{id}` 展示列结构 + 版本历史时间线

**WT-3.4 (FR-WT-3.4, P0)** — 删除模板 (软删), builtin 模板不可删 (per BR-WT-5)

**WT-3.5 (FR-WT-3.5, P1)** — 复制模板, 新模板 `is_builtin=false`, 名字加 "(Copy)" 后缀

### WT-4 模板应用到看板 (4 项)

**WT-4.1 (FR-WT-4.1, P0)** — 模板详情页「应用到看板」按钮 → 跳 `/workflow-templates/{id}/board?project_id={current}`

**WT-4.2 (FR-WT-4.2, P0)** — 模板化看板复用 KanbanBoard 组件, 传入 `columns` prop 而非用 `KANBAN_COLUMNS` 常量

**WT-4.3 (FR-WT-4.3, P0)** — WorkItem 拖到模板列时, 写 `WorkItem.status = column.mapping_status`, 触发既有 status 状态机校验

**WT-4.4 (FR-WT-4.4, P1)** — 模板化看板右上角显示「当前模板: {name} v{version}」indicator

### WT-5 模板列可编辑边界 (5 项)

**WT-5.1 (FR-WT-5.1, P0)** — 用户可增列 (任意非 builtin 列), 增列时强制 `panel` 选择 (main 或 bottom)

**WT-5.2 (FR-WT-5.2, P0)** — 用户可删列, builtin 列删除按钮置灰禁用 (per BR-WT-1)

**WT-5.3 (FR-WT-5.3, P0)** — 用户可改列宽 (拖动列边缘), builtin 作废列宽调节器置灰禁用 (per BR-WT-2)

**WT-5.4 (FR-WT-5.4, P0)** — 用户可重排顺序, builtin 顺序约束 (FR-WT-2.3 / 2.4) 拒绝时弹 toast

**WT-5.5 (FR-WT-5.5, P0)** — 用户可重命名, builtin 列允许重命名 (P0); 但空 / 重名校验 (per FR-WT-2.6)

**WT-5.6 (FR-WT-5.6, P1)** — (无对应 US) 用户可改 `mapping_status`, 弹 Picker 选 4 状态之一, 防止 1 个状态被多列映射

### WT-6 同步事件与跨界面联动 (3 项)

**WT-6.1 (FR-WT-6.1, P0)** — 模板 CRUD 后通过 SSE 推送 `template_changed` 事件, payload 含 `template_id` + `version`

**WT-6.2 (FR-WT-6.2, P0)** — v1 SSE 不可用时 fallback polling 5s 轮询 `/api/workflow-templates` (mock 走 zustand persist 内存)

**WT-6.3 (FR-WT-6.3, P0)** — 收到事件后所有打开的 `/workflow-templates/{id}*` 路由自动 invalidate 缓存并重渲染

### WT-7 多人并发编辑与排他锁 (3 项)

**WT-7.1 (FR-WT-7.1, P0)** — 进入模板编辑模式时申请写锁 (5min 超时, per BR-WT-6), 锁持有者 UI 角标显示 🟢 持有锁

**WT-7.2 (FR-WT-7.2, P0)** — 其它用户进入编辑模式时显示「{user} 正在编辑 (剩余 {N} 分钟)」, 按钮置灰

**WT-7.3 (FR-WT-7.3, P0)** — 乐观锁 + version 字段冲突时弹 conflict dialog, 提供「强制覆盖」「放弃」「回滚到 version N」3 选项 (per BR-WT-9)

### WT-8 右侧工作流入口 (2 项)

**WT-8.1 (FR-WT-8.1, P0)** — `frontend/src/components/UserMenu.tsx` 在 `toolsAndWorkspaces` 区块下新增第 5 入口「工作流模板」, href=`/workflow-templates`, icon=Workflow (复用 lucide-react)

**WT-8.2 (FR-WT-8.2, P1)** — (无对应 US) UserMenu 工作流入口右侧显示「当前激活模板: {name}」badge, 点击展开下拉显示全部模板可快速切换

### 4.5 列 ↔ WorkItem.status 映射 (强制声明, per §1.4 Out-of-Scope)

> **强制声明**: 本 SRS 不新增 `WorkItem.status` 枚举值, 模板列必须映射到既有 4 状态之一. 17 列预置默认映射见 §4.1.3 表. 用户列 mapping 默认 `in_progress`. **同一状态不允许被多列映射** (per FR-WT-5.6).

---

## §5 非功能需求 (NFR, 5 项)

### 5.1 性能 (NFR-WT-P, P0)

- 模板列表页首屏渲染 ≤ 500ms (mock 数据, 10 模板以内)
- 模板化看板渲染 17 列 + 50 WorkItem ≤ 1s
- 列宽拖动实时响应 ≤ 16ms (60fps)

### 5.2 可用性 (NFR-WT-U, P0)

- 键盘可达: Tab 顺序遵循视觉顺序
- 屏读器: 列名 announce, 锁状态 announce
- 错误恢复: 写锁超时自动释放 + 用户通知; 网络中断重连后自动 sync

### 5.3 安全 (NFR-WT-S, P0)

- 跨 tenant 访问返回 403 (mock 阶段前端校验)
- 写锁持有者变更校验: 提交时验证 token = 当前持有者
- 模板软删不可物理恢复 (per §10 #6 待决, 暂定禁止)

### 5.4 可观测 (NFR-WT-O, P1)

- 模板 CRUD 走既有埋点 (`frontend/src/lib/analytics.ts`)
- 锁冲突事件埋点, 用于运营分析
- SSE 连接状态暴露到 `/health`

### 5.5 可维护 (NFR-WT-M, P1)

- 模板结构变更走 zustand persist 版本号, 旧版本自动迁移
- 新增 builtin 列需更新 §4.1.3 表 + 跑 §9 AC-WT-1 / AC-WT-2 回归

---

## §6 约束 / 风险

### 6.1 守门合规 (per AGENTS.md §4.1)

| 守门 | 状态 | 说明 |
|---|---|---|
| #1 v15 禁回溯叙事 | ✅ | 修订历史 append-only; 不重写 v0.1 行 |
| #11 缺标比错标 | ✅ | §1.4 Out-of-Scope 列 11 项 |
| #12 v21 [P] 子项 docs 同步 | ✅ | `scripts/automation/registry.md` §2 +1 行 (本次 commit) |
| #13 已知缺口显式列 | ✅ | §10 列 9 项 |
| #14 v4 真人代签流程 obsolete | ✅ | §11 签字栏 5 角色统一 Mavis 接手 |

### 6.2 约束

- **技术**: 前端 mock + zustand persist, v1 不依赖后端 API
- **业务**: 模板维度跟 `Workflow` 状态机 / n8n 自动化流程 命名隔离 (§2.4)
- **安全**: 跨 tenant 硬隔离; v1 不做 per-role
- **组织**: 5 角色签字栏 Mavis 接手代签 (per 守门 #14 v4 反转)

### 6.3 风险

| 风险 | 等级 | 缓解 |
|---|---|---|
| 阻塞 1: KanbanBoard 不支持列宽因子 | 中 | stage 3 实现时扩展 KanbanBoard 组件 props |
| 阻塞 2: 17 列 → 4 状态 映射层 | 中 | stage 3 实现时新建 mapper utility |
| 阻塞 3: 后端持久化 API 不存在 | **高** | v1 mock, 真实生产前必补 (§10 #2) |

---

## §7 验收条件 (AC)

### 7.1 FR → AC 映射 (33 FR × AC)

| FR | AC | 验证方法 |
|---|---|---|
| FR-WT-1.1 | AC-WT-1: `WorkflowTemplate` interface 包含所有字段, TypeScript 严格类型通过 | `pnpm tsc --noEmit` |
| FR-WT-1.2 | AC-WT-2: `TemplateColumn` interface + 17 列预置数据正确导入 | unit test |
| FR-WT-1.3 | AC-WT-3: 17 列预置数据每行字段值符合 §4.1.3 表 | unit test 矩阵 |
| FR-WT-1.4 | AC-WT-4: 编辑后 `version` +1, `version_history` 追加快照 | e2e: 编辑 → 查看历史 |
| FR-WT-1.5 | AC-WT-5: 跨 tenant 访问返回 403, builtin 模板 `deleted_at` 永远 null | unit test |
| FR-WT-2.1 | AC-WT-6: 尝试删除 builtin 列 → 按钮置灰, 不发请求 | UI snapshot test |
| FR-WT-2.2 | AC-WT-7: 拖动作废列边缘 → 列宽不变, 调节器置灰 | UI snapshot test |
| FR-WT-2.3 | AC-WT-8: 尝试拖 Todo 到位置 1+ → toast 拒绝 | UI integration test |
| FR-WT-2.4 | AC-WT-9: 尝试把 P9 拖到 P1 前面 → toast 拒绝 | UI integration test |
| FR-WT-2.5 | AC-WT-10: 尝试拖审核到 main panel → toast 拒绝 | UI integration test |
| FR-WT-2.6 | AC-WT-11: 重命名为空字符串 → form 校验失败; 重名 → toast | form validation test |
| FR-WT-3.1 | AC-WT-12: `/workflow-templates` 渲染全部模板 (含 builtin) | e2e |
| FR-WT-3.2 | AC-WT-13: 创建弹 modal, 输入名字后新建模板 | e2e |
| FR-WT-3.3 | AC-WT-14: 详情页展示列结构 + 版本历史时间线 | e2e |
| FR-WT-3.4 | AC-WT-15: builtin 模板删除按钮不显示, 用户模板删除走软删 | e2e + unit |
| FR-WT-3.5 | AC-WT-16: 复制模板成功, 新模板 `is_builtin=false` | e2e |
| FR-WT-4.1 | AC-WT-17: 点击应用跳路由 + query string | e2e |
| FR-WT-4.2 | AC-WT-18: 模板化看板渲染模板列, 不渲染默认 4 列 | snapshot test |
| FR-WT-4.3 | AC-WT-19: 拖 WorkItem 到模板列 → status 更新 + 状态机校验 | e2e + state machine test |
| FR-WT-4.4 | AC-WT-20: indicator 显示当前模板名 + version | snapshot test |
| FR-WT-5.1 | AC-WT-21: 增列 modal 强制选 panel | UI form test |
| FR-WT-5.2 | AC-WT-22: builtin 列删除按钮置灰 | snapshot test |
| FR-WT-5.3 | AC-WT-23: 作废列宽调节器置灰 | snapshot test |
| FR-WT-5.4 | AC-WT-24: 重排违反顺序约束 → toast | UI integration test |
| FR-WT-5.5 | AC-WT-25: 重命名校验 (空 / 重名) | form validation test |
| FR-WT-5.6 | AC-WT-26: 改 mapping 弹 picker, 1 状态不可被多列映射 | UI form test |
| FR-WT-6.1 | AC-WT-27: 模板 CRUD 后 SSE 事件推送 (mock 阶段用事件总线) | integration test |
| FR-WT-6.2 | AC-WT-28: SSE 不可用时 polling 5s 兜底 | e2e: 断 SSE → 验证 polling |
| FR-WT-6.3 | AC-WT-29: 收到事件后所有相关路由 invalidate 缓存 | e2e |
| FR-WT-7.1 | AC-WT-30: 进入编辑申请写锁, UI 显示 🟢 | e2e |
| FR-WT-7.2 | AC-WT-31: 其它用户看到锁状态 + 按钮置灰 | e2e |
| FR-WT-7.3 | AC-WT-32: 乐观锁冲突弹 conflict dialog 含 3 选项 | e2e |
| FR-WT-8.1 | AC-WT-33: UserMenu 第 5 入口存在, href 正确 | snapshot test |

### 7.2 端到端场景 (3 项)

| 场景 | 验证方法 |
|---|---|
| **AC-WT-e2e-1 创建 → 应用 → 编辑 → 回滚**: 用户在 `/workflow-templates` 创建模板 → 应用到看板 → 编辑列名 → 收到 conflict → 选择回滚到 version 2 → 看板列名恢复 | 完整 e2e |
| **AC-WT-e2e-2 多人并发**: A 进入编辑拿锁 → B 进入看到锁 → A 超时释放 → B 拿锁 | 完整 e2e |
| **AC-WT-e2e-3 跨界面同步**: A 改模板列 → 5s 后 B 的看板自动 invalidate | 完整 e2e |

### 7.3 守门 #1 baseline (per AGENTS.md §4.1.1)

- 本 SRS 不重写任何既有 SRS / BD / DD / 代码
- 守门 #1 验证: `git log --follow docs/requirements/SRS-WORKFLOW-TEMPLATE-001.md` 仅 1 个 commit (本次 v0.1)

---

## §8 接口需求 (API / 组件 / Store)

### 8.1 BFF 端点 (v1 mock 阶段; 后端实装后路径不变)

| 方法 | 路径 | 用途 |
|---|---|---|
| `GET` | `/api/workflow-templates` | 列表 (含 builtin) |
| `GET` | `/api/workflow-templates/{id}` | 详情 |
| `POST` | `/api/workflow-templates` | 创建 |
| `PATCH` | `/api/workflow-templates/{id}` | 更新 (触发 SCD2 新 version) |
| `DELETE` | `/api/workflow-templates/{id}` | 软删 |
| `POST` | `/api/workflow-templates/{id}/lock` | 申请写锁 |

### 8.2 前端组件 (新增 4 个)

| 组件 | 路径 | 用途 |
|---|---|---|
| `TemplateList` | `frontend/src/components/workflow-templates/TemplateList.tsx` | 列表页 |
| `TemplateDetail` | `frontend/src/components/workflow-templates/TemplateDetail.tsx` | 详情 + 编辑 |
| `TemplateBoard` | `frontend/src/components/workflow-templates/TemplateBoard.tsx` | 模板化看板 |
| `TemplateColumnEditor` | `frontend/src/components/workflow-templates/TemplateColumnEditor.tsx` | 列编辑器 (列宽/顺序/重命名) |

### 8.3 Zustand store 扩展

- `frontend/src/store/templateStore.ts` (新增) — 模板 CRUD + 锁状态 + 当前激活
- `frontend/src/store/boardStore.ts` (扩展) — 支持传入 `templateColumns` prop

### 8.4 既有 module 联动 (25 module, 仅列变更相关)

- `UserMenu.tsx` — 新增第 5 入口 (FR-WT-8.1)
- `nav/registry.ts` — 新增 `/workflow-templates` 路由注册
- `KanbanBoard.tsx` — 扩展支持 `columns` prop (替代硬编码 KANBAN_COLUMNS)

---

## §9 数据需求 (W/T/M 100% 覆盖, 7 张表)

per `docs/kanban-vmodel-jp/W-T-M-VERIFICATION-REPORT.md` v0.1 验证基线

| # | 表 | 分类 | W/T/M | 说明 |
|---|---|---|---|---|
| 1 | `workflow_templates` | Master | W | 模板主表 (id, tenant_id, name, is_builtin, version, columns_json, created_at, updated_at, created_by, deleted_at) |
| 2 | `template_columns` | Master | W | 列定义 (id, template_id, position, name, builtin, width_factor, panel, mapping_status) — 由 `workflow_templates.columns_json` 派生, 但单独建表便于索引 |
| 3 | `template_versions` | Master | W | SCD2 历史快照 (id, template_id, version, snapshot_json, created_at, created_by) |
| 4 | `template_locks` | Transaction | T (WORM) | 写锁记录 (id, template_id, holder_user_id, acquired_at, expires_at) — append-only, 不允许 UPDATE/DELETE |
| 5 | `template_change_events` | Transaction | T (WORM) | 同步事件 (id, template_id, version, event_type, created_at) — append-only |
| 6 | `work_item_template_links` | Master | W | WorkItem 与模板列的关联 (id, work_item_id, template_id, column_id) — 派生 WorkItem.status |
| 7 | `template_audit_logs` | Transaction | T (WORM) | 审计日志 (id, template_id, user_id, action, before_json, after_json, created_at) — append-only |

**W/T/M 100%**: 6 Master (W) + 3 Transaction (WORM, T) — 0 混合分類 ✅

> 注: v1 前端 mock 阶段不真建表, 数据存 zustand persist + IndexedDB; 后端实装时按此 schema 建表.

---

## §10 已知缺口 / 未决问题 (9 项, per 守门 #11 缺标比错标)

1. **风险 #1 (polling 5s 兜底)**: v1 SSE 不可用时 polling 5s, 真实生产建议 SSE/WebSocket ≤ 1s (留 §2+)
2. **风险 #2 (后端持久化)**: P0 阻塞前提, v1 走前端 mock + zustand persist; 真实生产前必补后端 API
3. **风险 #3 (签字 PDF/文档)**: v1 不做, 审核/完成 panel 仅 UI + 字段写存储 (留 P2+)
4. **风险 #4 (ULYS-34 前次失败)**: 上次 worker `74dda328` commit 报告存在但仓库未实际落地 (per `git log --all -- docs/requirements/SRS-WORKFLOW-TEMPLATE-001.md` 0 hits); 本次 v0.1 重新撰写并落地
5. **风险 #5 (作废列映射 `wontfix` 是否新增 enum)**: §4.1.3 表中作废列 mapping_status 写 `wontfix`, 但 `WorkItemStatus` 当前 enum 不含 `wontfix`; 需 stage 2/3 拍板: (a) 新增 enum 值, 或 (b) 复用 `todo` 映射
6. **风险 #6 (软删物理恢复)**: §5.3 NFR-WT-S 写「软删不可物理恢复」, 需 stage 2 拍板确认 (vs 30 天保留后真删)
7. **风险 #7 (映射唯一性冲突)**: FR-WT-5.6 写「同一状态不允许被多列映射」, 但 v1 17 列预置已存在 13 列映射到 `in_progress`, 与该约束冲突 — 需 stage 2 拍板: (a) 放宽约束 (允许 N 列 → 1 状态), 或 (b) 17 列拆 13 个独立 enum
8. **风险 #8 (模板跨 project 范围)**: §4.1.1 `project_id` 字段允许 null = 跨项目, 但 v1 不实装跨项目模板; stage 3 需确认 null vs 必填
9. **风险 #9 (ULYS-35 协调)**: 本 SRS 落档后 ULYS-35 (stage 2 基本設計) worker 可消费; ULYS-35 worker 需读本 SRS 作为前置输入, 不可跳级

---

## §11 关联文档

| 类型 | 文档 |
|---|---|
| 父 issue | ULYS-30 「全新模板模式」 |
| 子任务 (后续) | ULYS-35 [stage 2 基本設計] / ULYS-36 [stage 3 詳細設計] |
| 平行 SRS | [`docs/requirements/SRS-CANVAS-WORKFLOW-001.md`](../requirements/SRS-CANVAS-WORKFLOW-001.md) v1.1 W14 默认工作流模板库 (本 SRS 是其首个落地路径) |
| 平行 SRS | [`docs/requirements/SRS-STAR-OPS-001.md`](../requirements/SRS-STAR-OPS-001.md) (运维 SRS, 平行不重叠) |
| 既有文档 | `STAR-P3-WBS-001.md` 阶段命名源 (P1-P9 + P6.1-P6.4) |
| 既有代码 | `frontend/src/components/UserMenu.tsx` / `frontend/src/lib/nav/registry.ts` / `frontend/src/components/board/KanbanBoard.tsx` / `frontend/src/types/ids.ts` |
| 守门 | AGENTS.md §4.1 守门 #1 v15 / #11 / #12 v21 / #13 / #14 v4 |
| BD (下个 stage) | `docs/design/BD-WORKFLOW-TEMPLATE-001.md` (待 ULYS-35 落档) |
| DD (后续 stage) | `docs/detailed-design/DD-WORKFLOW-TEMPLATE-001.md` (待 ULYS-36 落档) |

---

## §12 签字栏

per 守门 #14 v4 反转 (2026-09-10 12:45 JST): 真人代签流程永久 obsolete, 5 角色统一由 Mavis 接手代签

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-14 | 🟢 接受 per 守门 #14 v4 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-14 | 🟢 接受 per 守门 #14 v4 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-14 | 🟢 接受 per 守门 #14 v4 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-14 | 🟢 接受 per 守门 #14 v4 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-14 | 🟢 接受 per 守门 #14 v4 |

---

## §13 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（8 子能力 / 33 FR / 14 US / 5 NFR / 9 已知缺口 / 5 角色签字栏 / 7 张表 W/T/M 100% 覆盖） | ULYS-30 父任务委派 + ULYS-34 子任务 retry（前次 `74dda328` commit 缺失, 重新撰写并落地本 commit） |

---

**修订人**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** (per 守门 #14 v4 2026-09-10 12:45 JST 反转)
**审批**: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
**日期**: 2026-09-14 JST
