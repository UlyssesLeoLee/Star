/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-OPS-CONSOLE-001.md"})
@endcypher
*/
# SRS-OPS-CONSOLE-001 — 运维控制台

> **2026-11-24 现行适用说明**：历史界面及模拟链路不是实际 K3s 运维验收；真实控制、授权、持久事实、部署和回滚分别取证，未接入项明确展示。 统合边界与状态说明见 [要件总索引](README.md)。

> **集群/告警/原子化 App 晋级 (仅管理员)** (per docs/prototypes/7 · 运维控制台（仅管理员）-html mockup, 2026-11-15 全量审计后建档)
>
> - 状态: Requirements Baseline v1.0 — 已实现并实测 (commit 23d60e75)
> - 权威入口: **本文件是该功能的唯一权威文档** — 修改此功能只改本文件, 并在 §修订履历 加行
> - 实现状态: ✅ 已闭环 (浏览器实测 2026-11-15)
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影

---

## §1 设计要点 (mockup 提炼)

- 环境切换 (staging ▾) + 仅管理员标识
- 集群与节点 + 告警列表
- 原子化 App 表: 每个 App 独立构建/独立晋级/独立回滚 (App/当前版本/可升级到/契约/健康/操作+回滚)
- 兼容性检查 (晋级前必过) + 晋级路径 + 渐进发布 (金丝雀)
- 详细设计: docs/detailed-design/OPS-DETAILED-DESIGN-001.md

## §2 功能需求清单 (FR)

| FR | 描述 | 状态 |
|---|---|---|
| FR-OP-001 | 集群与节点 | ◐ 集群更新端到端已接 (ClusterTab), 节点列表未实现 |
| FR-OP-002 | 原子化 App 表 + 回滚 | ✅ |
| FR-OP-003 | 兼容性检查 | ✅ |
| FR-OP-004 | 晋级/渐进发布 | ✅ |
| FR-OP-005 | 告警 | ❌ 无告警列表, 仅 MetricsTab 查询错误横幅 |

## §3 实现状态与文件映射

| 文件 | 职责 |
|---|---|
| frontend/src/app/ops/page.tsx | /ops 页 (5 tab) |
| docs/detailed-design/OPS-DETAILED-DESIGN-001.md | 运维详细设计 |

## §4 修订履历

| 日期 | 版本 | 内容 | commit |
|---|---|---|---|
| 2026-11-15 | v1.0 | 建档: mockup 设计提炼 + FR + 实现状态 (全量审计) | c0f149f2 |
| 2026-11-15 | v1.1 | 状态回写核对 | 本提交 |
