/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-KANBAN-001.md"})
@endcypher
*/
# SRS-KANBAN-001 — 任务 · IPA 看板

> **任务卡全生命周期管理 (Branch 工作区固定页签: 任务)** (per docs/prototypes/1b · 任务 · IPA 看板-html mockup, 2026-11-15 全量审计后建档)
>
> - 状态: Requirements Baseline v1.0 — 已实现并实测 (commit 9187f43d)
> - 权威入口: **本文件是该功能的唯一权威文档** — 修改此功能只改本文件, 并在 §修订履历 加行
> - 实现状态: ✅ 已闭环 (浏览器实测 2026-11-15)
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影

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
