/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-INBOX-001.md"})
@endcypher
*/
# SRS-INBOX-001 — 收件箱

> **全局消息聚合 (通知/评论/审计/反馈/搜索/上下文)** (per docs/prototypes/3 · 收件箱-html mockup, 2026-11-15 全量审计后建档)
>
> - 状态: Requirements Baseline v1.0 — 已实现并实测 (commit 23d60e75)
> - 权威入口: **本文件是该功能的唯一权威文档** — 修改此功能只改本文件, 并在 §修订履历 加行
> - 实现状态: ✅ 已闭环 (浏览器实测 2026-11-15)
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影

---

## §1 设计要点 (mockup 提炼)

- 左: 项目折叠列表 (Physis/StarGate/Mobile) + 全局组 (收件箱·5 / Agent 中心 / 分析)
- 中: 消息列表 (kind/where/time/title)
- 右: 消息详情 + 在 Branch 中打开 + 标为已读/未读
- 收件箱 = 6 域聚合 (notification/comment/audit/feedback/search/context redirect sink)

## §2 功能需求清单 (FR)

| FR | 描述 | 状态 |
|---|---|---|
| FR-IN-001 | 消息列表 + 详情 | ✅ |
| FR-IN-002 | 在 Branch 中打开 | ✅ |
| FR-IN-003 | 标为已读/未读 | ✅ |
| FR-IN-004 | 6 域 redirect sink | ✅ |

## §3 实现状态与文件映射

| 文件 | 职责 |
|---|---|
| frontend/src/app/(app)/inbox/page.tsx | /inbox 页 |
| frontend/src/lib/redirects.ts | 6 域 → /inbox |

## §4 修订履历

| 日期 | 版本 | 内容 | commit |
|---|---|---|---|
| 2026-11-15 | v1.0 | 建档: mockup 设计提炼 + FR + 实现状态 (全量审计) | c0f149f2 |
