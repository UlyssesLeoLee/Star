/*
@cypher schema=1 source_sha256=0000000000000000000000000000000000000000000000000000000000000000
MERGE (self:File {path:"docs/requirements/SRS-INBOX-001.md"})
@endcypher
*/
# SRS-INBOX-001 — 收件箱

> **2026-11-24 现行适用说明**：收件箱按当前 Project/Lane/资源授权聚合事件，已读状态须持久化且跳转时复验；历史种子数量和界面演示不代表生产闭环。 统合边界与状态说明见 [要件总索引](README.md)。

> **聊天交互补充**：IM 消息撤回、个人上下文排除、已生成内容失效和在途动作处理以 [SRS-CHAT-CONTEXT-CONTROL-001](SRS-CHAT-CONTEXT-CONTROL-001.md) 为专题要件；收件箱本身不拥有消息或 Agent 上下文权威事实。该专题未验收。

> **全局消息聚合 (通知/评论/审计/反馈/搜索/上下文)** (per docs/prototypes/3 · 收件箱-html mockup, 2026-11-15 全量审计后建档)
>
> - 状态: Requirements Baseline v1.0；commit 23d60e75 记录历史预览。
> - 权威入口: 本文件记录收件箱专题；相关需求、设计及验收须同步。
> - 实现状态: 2026-11-15 浏览器预览；授权事件聚合和已读状态持久化需另行验收。
> - 数据源: store (seed 预览模式) — 宿主接入后切换授权投影

---

## §1 设计要点 (mockup 提炼)

- 左: 项目导航及全局消息入口；计数来自当前获授权事实，未接通时明确展示状态。
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
| 2026-11-15 | v1.1 | 状态回写核对 | 本提交 |
