/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-PROTOTYPE-001.md"})
@endcypher
*/
# SRS-PROTOTYPE-001 — 原型生成器

> **1D-5D 原型生成与推演 (Branch 工作区页签)** (per docs/prototypes/2d · 原型-html mockup, 2026-11-15 全量审计后建档)
>
> - 状态: Requirements Baseline v1.0 — 已实现并实测 (commit 23d60e75)
> - 权威入口: **本文件是该功能的唯一权威文档** — 修改此功能只改本文件, 并在 §修订履历 加行
> - 实现状态: ✅ 已闭环 (浏览器实测 2026-11-15)
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影

---

## §1 设计要点 (mockup 提炼)

- 左: 生成类型 (1D~5D/UI/动效/图片视频) + 引用上下文 (文件/截图/任务卡)
- 中: 描述你要的原型 prompt + 生成/从模板 + 流水线 (建模→校验→推演→构建→验证→导出 = GemArc tools) + 导出/分享
- 右: 交互预览 (UI 关系查询 / 2D 节点关系 / 3D 机房→机柜→服务 / 4D 事件随时间传播 / 5D 反事实对照)

## §2 功能需求清单 (FR)

| FR | 描述 | 状态 |
|---|---|---|
| FR-PT-001 | 生成类型 (1D-5D) | ✅ |
| FR-PT-002 | 引用上下文 | ✅ |
| FR-PT-003 | 流水线 (GemArc tools 映射) | ✅ |
| FR-PT-004 | 1D-5D 预览 | ✅ |
| FR-PT-005 | 导出/分享 | ✅ |

## §3 实现状态与文件映射

| 文件 | 职责 |
|---|---|
| frontend/src/app/(app)/prototype/page.tsx | /prototype 页 |
| frontend/src/lib/nav/registry.ts | module prototype |

## §4 修订履历

| 日期 | 版本 | 内容 | commit |
|---|---|---|---|
| 2026-11-15 | v1.0 | 建档: mockup 设计提炼 + FR + 实现状态 (全量审计) | c0f149f2 |
