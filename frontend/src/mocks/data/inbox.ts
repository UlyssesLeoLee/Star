/*
@cypher schema=1 source_sha256=302eb1aec4dde7f469a437cc56c4377201df7e0b3c9135eac2b38969026be232
MERGE (self:File {path:"frontend/src/mocks/data/inbox.ts"})
MERGE (fixture:Symbol {id:"frontend/src/mocks/data/inbox.ts::MOCK_NOTIFS",kind:"constant"})
MERGE (self)-[:DEFINES]->(fixture)
@endcypher
*/
// frontend/src/mocks/data/inbox.ts
// MOCK_NOTIFS — 10 row fixture (per mock-data-isolation.md §2.1)
// per 2026-11-14 参考 mockup (Inbox.dc.html): 评审请求/评论/通知 三类消息
// 内容对齐 Branch 工作区语境 (feat/relation-bfs · TASK-142/143 · PR #44)

import type { MockNotif } from "@/mocks/schemas/inbox";

export const MOCK_NOTIFS: ReadonlyArray<MockNotif> = [
  { id: "n-001", kind: "review_requested",  subject: "PR #44 TASK-143 关系权重",        body: "请你评审 · Agent A 提交了关系权重实现，验证 2/2 通过，等待人工闸门放行", read: false, ago: "2 分钟前" },
  { id: "n-002", kind: "feedback_question", subject: "评论 TASK-142 BFS 环检测",         body: "Agent B：自环判定已补，需要确认是否输出完整环路径",                     read: false, ago: "15 分钟前" },
  { id: "n-003", kind: "merge_conflict",    subject: "通知 wt-context-cache 已回退",     body: "Worktree 已回退到 dev 基线",                                          read: false, ago: "1 小时前" },
  { id: "n-004", kind: "merge_conflict",    subject: "fix/cache-ttl 有 2 处冲突待解决",  body: "评审请求 · 与 dev 基线冲突，需要人工确认保留哪一侧",                 read: false, ago: "3 小时前" },
  { id: "n-005", kind: "agent_decision_required", subject: "LANE auth-refactor 进入 P6", body: "通知 Physis / GVPE · 测试阶段开始，请确认验收入口",                    read: false, ago: "昨天" },
  { id: "n-006", kind: "ci_failed",         subject: "CI failed: Star-frontend #117",    body: "typecheck exit 1 — see log",                                         read: true,  ago: "5 小时前" },
  { id: "n-007", kind: "budget_alert",      subject: "Agent 预算 80% 已用",              body: "ag-002 Physis-builder hit $4.00/$5.00 daily cap",                   read: true,  ago: "1 小时前" },
  { id: "n-008", kind: "policy_violation",  subject: "Policy violation: INV-FB-02",      body: "ag-005 reported feedback required, lease paused",                   read: true,  ago: "2 小时前" },
  { id: "n-009", kind: "review_requested",  subject: "Review: feat/w2-gantt → main",     body: "@Ulysses review required (5 files, +420 -118)",                     read: true,  ago: "23 分钟前" },
  { id: "n-010", kind: "budget_alert",      subject: "Weekly cost summary",              body: "Total $87.42 across 5 agents (down 12% vs last week)",              read: true,  ago: "2 天前" },
];

// FALLBACK alias for page SSR — page 改 useEffect+fetch 后, SSR 阶段用 FALLBACK 兜底
// (per mock-msw-handlers.md §2.4 + §4 #1 缺标)
export const MOCK_NOTIFS_FALLBACK: ReadonlyArray<MockNotif> = MOCK_NOTIFS;
