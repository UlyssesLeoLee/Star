/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-SANDBOX-001.md"})
@endcypher
*/
# SRS-SANDBOX-001 — 沙盒

> **实体机/虚拟机/Docker 资源与观测 (Branch 工作区页签)** (per docs/prototypes/2c · 沙盒-html mockup, 2026-11-15 全量审计后建档)
>
> - 状态: Requirements Baseline v1.0 — 已实现并实测 (commit 304f3ce5)
> - 权威入口: **本文件是该功能的唯一权威文档** — 修改此功能只改本文件, 并在 §修订履历 加行
> - 实现状态: ✅ 已闭环 (浏览器实测 2026-11-15)
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影

---

## §1 设计要点 (mockup 提炼)

- 左资源树: 实体机/虚拟机/Docker + 观测方式四级 (原生→执行→探针→元数据, 来源/可信度/新鲜度, 离线主机仍列出)
- 主区沙盒详情: 类型/系统/资源/接入方式/网络出站 + 启动/停止/快照/销毁
- 已安装 Agent CLI + Worker 心跳/任务租约
- 新建沙盒/接入实体机按钮
- 数据源: sandtree-core/argo-core API 门面 (后端不可达回退本地 mock)

## §2 功能需求清单 (FR)

| FR | 描述 | 状态 |
|---|---|---|
| FR-SB-001 | 资源树 + 观测四级 | ✅ |
| FR-SB-002 | 沙盒详情字段 | ✅ |
| FR-SB-003 | 新建沙盒/接入实体机入口 | ✅ |
| FR-SB-004 | sandtree/argo API 接线 (失败回退) | ✅ |
| FR-SB-005 | Worker 心跳/租约 | ◐ 展示结构, 实时数据未接 |

## §3 实现状态与文件映射

| 文件 | 职责 |
|---|---|
| frontend/src/app/(app)/sandbox/page.tsx | /sandbox 页 |
| crates/sandtree-core/ | SandTree 拓扑/观测四级 |
| crates/argo-core/ | Argo 多机模型/CLI |
| crates/star-api-rest/src/routes/integrations.rs | /api/v1/sandtree + /argo 门面 |

## §4 修订履历

| 日期 | 版本 | 内容 | commit |
|---|---|---|---|
| 2026-11-15 | v1.0 | 建档: mockup 设计提炼 + FR + 实现状态 (全量审计) | c0f149f2 |
