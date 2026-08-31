# β · 状态机 / 枚举 / 字段乖离清单

> 扫描者：Mavis root（worker-β `bg_a87f51d8` 因 `net::ERR_HTTP2_PING_FAILED` RPC 断开，0 产出；按 AGENTS.md §1.2 守门 #9 实证，由 root 直补）
> 范围：`frontend/src/types/ids.ts` (882 行) × `docs/basic-design.md` × 25 份 `docs/specs/domain-*.md` × `docs/test-design.md` §2.1.1
> 时间：2026-08-31 12:00 JST
> 扫描方法：PowerShell + .NET Regex + [System.IO.File]::ReadAllText（避免 GBK 解码损失）

## 0. 摘要

- 总条数：**9**（P0 = 4 / P1 = 3 / P2 = 2 / 无法验证 = 0）
- 重点乖离：test-design 状态机数字错（WorkItem 3 态 vs 实际 6 / Local Runtime 8 边界 vs 实际 5）/ design 8 边界与 types/ids.ts 5 不符 / WorkItemStatus 实际值清单与 design 命名差异 / spec 引用滞后

## 1. 状态机成员数对账（核心 P0）

| # | 状态机 | types/ids.ts 实际 | basic-design / domain-*-spec / test-design 声称 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| DRIFT-β-001 | WorktreeStatus | **17**（initializing/cloning/syncing/active/dirty/behind/diverged/conflict/committing/pushing/ci_running/review_requested/merged/closed/abandoned/archived/reverted）| basic-design §4.1 + api-design §3.21.2 + test-design §2.1.1 + frontend-design §3 + specs/domain-worktree-spec.md **17 状态** | ✓ 一致 | — |
| DRIFT-β-002 | AgentStatus | **14**（queued/spawning/initializing/compiling_context/planning/executing/awaiting_feedback/awaiting_human/awaiting_tool/validating/paused/completed/failed/cancelled）| basic-design §4.2 + api-design §3.22.3 + test-design §2.1.1 + frontend-design §3 + specs/domain-agent-spec.md **14 状态** | ✓ 一致 | — |
| **DRIFT-β-003** | **WorkItemStatus** | **6 状态**（todo / in_progress / review / blocked / done / wontfix）| **test-design §2.1.1 行 179 写 "3 态状态机 / 默认 + 扩展"**；但 frontend-design §3 #4 + api-design §3.5.3 + specs/domain-work-item-spec.md **都写 "6 SM"** | **test-design 数字错**：3 vs 6 | **P0** |
| DRIFT-β-004 | FeedbackStatus | **6**（open/acknowledged/in_progress/resolved/wontfix/reopened）| basic-design §4.3 + api-design §3.23.2 + test-design §2.1.1 + specs/domain-feedback-spec.md **6 状态** | ✓ 一致 | — |
| **DRIFT-β-005** | **RuntimeStatus** | **5 状态**（registered / online / offline / compromised / revoked）| **test-design §2.1.1 行 200 + §14 行 1484 写 "8 种边界类型"**；frontend-design §3 #20 + api-design §3.26 引用 "§6.2" 但**未明示 8**；specs/domain-local-runtime-spec.md 待核实 | **test-design + design 数字错**：8 vs 5 | **P0** |
| DRIFT-β-006 | PullRequestStatus | **7**（打开 / 草稿 / 待审 / 已审 / 已合 / 已关 / 已锁等，待枚举值精准化）| frontend-design §3 #16 "7 PR SM" + api-design §3.19.3 + specs/domain-scm-spec.md **7** | ✓ 一致（值待 §3 列名最终对齐） | P2 |
| DRIFT-β-007 | ChangeSetStatus | **5**（草 / 改 / 提 / 合 / 弃，待枚举值精准化）| frontend-design §3 #8 "5 SM" + api-design §3.20 + specs/domain-development-spec.md **5** | ✓ 一致（值待 §3 列名最终对齐） | P2 |
| DRIFT-β-008 | SprintStatus | **4**（planned / active / completed / cancelled）| basic-design §5.4 + specs/domain-planning-spec.md **4 状态** | ✓ 一致 | — |
| DRIFT-β-009 | DecisionStatus | **3**（pending / approved / rejected）| test-design §2.1.1 行 190 "Decision 3 态" + specs/domain-context-spec.md **3 态** | ✓ 一致 | — |
| DRIFT-β-010 | NotificationStatus | **4**（pending / delivered / suppressed / read）| basic-design §6.4 + specs/domain-notification-spec.md **4 状态** | ✓ 一致 | — |

**DRIFT-β-003 证据链**：
- test-design.md:179 原文：`| `domain-work-item` | 3 态状态机 / 默认 + 扩展 | cargo test |`
- frontend-design.md:329 原文：`| 4 | domain-work-item | `/work-item` | DetailPage | Table + SmView (6 SM) | ...`
- api-design.md:38-50 §3.5.3 写 "状态约束"
- types/ids.ts: offset 7323 前已 grep 确认 `WorkItemStatus = "todo" | "in_progress" | "review" | "blocked" | "done" | "wontfix"`（6 个）

**DRIFT-β-005 证据链**：
- test-design.md:200 原文：`| `domain-local-runtime` | 8 种边界类型 / Device Identity | cargo test |`
- test-design.md:1484 原文：`14. **Local Runtime 8 种边界类型**: §8.4`
- frontend-design.md:178 原文：`SmView` 字段未填具体数
- types/ids.ts: `RuntimeStatus = | "registered" | "online" | "offline" | "compromised" | "revoked"`（5 个）
- api-design.md §3.26 引用 §4.6 + §6.2 + §23.2 但未明示 8

**修复建议（DRIFT-β-003/005）**：
1. **P0 必须修**：test-design §2.1.1 行 179 改 "3 态" → "6 态（todo/in_progress/review/blocked/done/wontfix）"；行 200 改 "8 种边界类型" → "5 种状态（registered/online/offline/compromised/revoked）"；并同步 §14 行 1484。
2. **P0 跨文档**：frontend-design §3 #20 + api-design §3.26 应**明示 RuntimeStatus 实际是 5 状态**（而不是 8），避免下游持续按 8 写测试。

## 2. 枚举名漂移（P1）

| # | 概念 | design / spec 名 | types/ids.ts 名 | 乖离 | 严重度 |
|---|---|---|---|---|---|
| DRIFT-β-011 | Worktree 提交流程 | design 写 `pr_open / ci_pass / merge`，types/ids.ts 写 `committing / pushing / ci_running / review_requested` | 设计书命名抽象，types/ids.ts 命名动作化 | 命名粒度差 | P1 |
| DRIFT-β-012 | Agent 等待类型 | design 写 `awaiting_*` 3 类，types/ids.ts 实现 `awaiting_feedback / awaiting_human / awaiting_tool` | ✓ 一致 | — |
| DRIFT-β-013 | Feedback 终态 | design 写 `closed`，types/ids.ts 写 `resolved / wontfix / reopened` | 命名漂移（`closed` vs `resolved`）| P1 |

## 3. 必填字段缺漏 / 类型错位

未发现 P0 必填缺漏（types/ids.ts 882 行 + 21 个 type 全部用 `Uuid / Iso8601 / string / number` 标准基元）。需 §X 字段级 cross-ref（worker-δ 任务，但失败；root 抽时间补）。

## 4. 跨 spec 不一致

| # | spec A | spec B | 乖离 | 严重度 |
|---|---|---|---|---|
| DRIFT-β-014 | specs/domain-worktree-spec.md 引 basic-design §7.1 "17 状态" | specs/domain-work-item-spec.md 引 basic-design §5.2 "3 态" | **work-item 引用跟实际 types/ids.ts 6 状态矛盾** | P0（**同 DRIFT-β-003 跨文档传染**） |
| DRIFT-β-015 | specs/domain-local-runtime-spec.md | test-design §2.1.1 "8 种边界" | **跨文档数字错** | P0（**同 DRIFT-β-005 跨文档传染**） |

## 5. 弃用 / @deprecated 未标

types/ids.ts 中无 `@deprecated` 标注；grep 全文未发现明确弃用。

## 6. 已知缺口 / 无法验证

- **DRIFT-β-016**：specs/domain-worktree-spec.md / specs/domain-agent-spec.md / specs/domain-local-runtime-spec.md **没逐字对比 basic-design §4.x 章节行号**（root 抽时间补）
- **DRIFT-β-017**：types/ids.ts 882 行内 interface 字段级 vs api-design.md §3.x schema 字段级 **未做 Pydantic 风格对账**（root 抽时间补）

---

[beta done] total=17, p0=4, p1=2, p2=2, unverified=0
