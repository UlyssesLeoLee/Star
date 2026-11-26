/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-KANBAN-001.md"})
@endcypher
*/
# SRS-KANBAN-001 — 任务 · IPA 看板

> **2026-11-24 现行适用说明**：任务位于 Branch 下，在 Lane Backlog 内以可关闭子窗口打开，CLI 使用当前获授权 Worktree；历史绿色标记只覆盖当时预览，当前接入和验收以 §7.1 缺陷专题为准。 统合边界与状态说明见 [要件总索引](README.md)。

> **任务卡全生命周期管理**：任务位于 Branch 下，在 Lane Backlog 内以可关闭子窗口打开（原型背景见 docs/prototypes/1b · 任务 · IPA 看板）。
>
> - 状态: Requirements Baseline v1.0；commit 9187f43d 的历史预览证据不代表当前生产验收。
> - 权威入口: 本文件记录看板现有需求；智能缺陷管理的新增专题需求见 [SRS-DEFECT-INTELLIGENCE-001](SRS-DEFECT-INTELLIGENCE-001.md)。涉及缺陷视图的变更须同步两份需求及对应设计、验收状态。
> - 实现状态: 历史浏览器预览（2026-11-15）；当前授权数据和写入闭环待验收。
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影
> - 2026-11-23 验收：上述 2026-11-15 状态为历史记录；Lane Backlog、独立看板和缺陷视图仍需完成同一授权数据/写入路径及端到端验收。当前缺口和提交基线见 [智能缺陷需求 §7.1](SRS-DEFECT-INTELLIGENCE-001.md#71-2026-11-23-模块整合验收与后续代码复核)。

---

## §1 设计要点 (mockup 提炼)

- 6 视图: Sprint / 任务看板 / IPA 流程图 / 按文件开卡 / 时间线 / 日历
- Sprint 视图: 冲刺卡 (名称/状态/日期/卡数/点数/目标) + 启动冲刺/完成冲刺 + Backlog 区 (从 IPA 流程建卡/按文件开卡快捷入口)
- 任务看板: 6 列 (todo/in_progress/review/blocked/done/wontfix) + 每列 +Add task → 可关闭 Drawer 子窗口 (status 预填) + 列增删/重命名/拖动重排
- 创建栏 6 控件: 标题 + IPA 流程(默认 ipa-s05-002) + 工作流 + 关联文件 + 估点(默认 3) + 开卡按钮
- 生命周期条 (真实 board.columns) + 预览数据标识 (seed 不冒充正式视图)

## §2 功能需求清单 (FR)

| FR | 描述 | 状态 |
|---|---|---|
| FR-KB-001 | 6 视图切换 | ✅ |
| FR-KB-002 | 创建栏 6 控件 (1b mockup 精确对齐) | ✅ |
| FR-KB-003 | 每列创建任务卡 (Drawer, status 预填) | ✅ |
| FR-KB-004 | 列增删/重命名/拖动重排 (兜底列保护) | ✅ |
| FR-KB-005 | 冲刺启动/完成 | ✅ |
| FR-KB-006 | Backlog 快捷建卡 (IPA/文件) | ✅ c0f149f2 |
| FR-KB-007 | 预览数据标识 | ✅ cf48593b |
| FR-KB-008 | IPA 流程图 (IpaSkillWorkflowBoard) | ✅ |
| FR-KB-009 | 时间线 (GanttView) / 日历 | ✅ |

## §3 实现状态与文件映射

| 文件 | 职责 |
|---|---|
| frontend/src/app/(app)/kanban/page.tsx | 页面壳 (6 视图/创建栏/生命周期/Backlog 按钮) |
| frontend/src/components/board/KanbanBoard.tsx | 看板列容器 (列操作/拖拽) |
| frontend/src/components/board/KanbanCard.tsx | 单卡渲染 (聊天入口) |
| frontend/src/components/board/WorkItemDetailDrawer.tsx | 可关闭任务详情子窗口 |
| frontend/src/components/sprint/SprintBoardView.tsx | Sprint 视图 |
| frontend/src/components/ipa-skills/IpaSkillWorkflowBoard.tsx | IPA 流程图 |
| frontend/src/components/gantt/GanttView.tsx | 时间线 |

## §4 修订履历

| 日期 | 版本 | 内容 | commit |
|---|---|---|---|
| 2026-11-15 | v1.0 | 建档: mockup 设计提炼 + FR + 实现状态 (全量审计) | c0f149f2 |
| 2026-11-15 | v1.1 | 状态回写核对 | 本提交 |
| 2026-11-22 | v1.2 | 挂接智能缺陷管理专题需求，专题为目标方案且尚未验收 | 工作区 |
| 2026-11-23 | v1.3 | 注明历史验收范围并链接当前缺陷与 Lane 整合缺口 | 工作区 |
