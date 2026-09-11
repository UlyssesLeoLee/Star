# SRS-MULTICA-TASK-001

> **Multica Task Lifecycle 域要件定義書 v0.1** (per ADR-0026 v0.2 §2.1 模式 2, 跟 v33 候选对齐)
>
> - 状态: 🟡 Draft v0.1
> - 目标阶段: 要件定義 → 基本設計 → 詳細設計 → 実装
> - 关联 commit: (留空, root 统一 commit 时填)
> - 关联基本設計書: [`docs/design/BD-MULTICA-TASK-001.md`](../design/BD-MULTICA-TASK-001.md) (下个 turn 落档)
> - 关联 ADR: [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2
> - 关联 inventory: [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.2 (v33 候选)
> - 修订人: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 2026-08-27 19:39 JST 用户授权 + 守门 #14 v3)
> - 审批: 架构师 (Mavis 接手 agent per DEC-008) — per 守门 #14 v4
> - 日期: 2026-09-11 JST
> - 受众: 詳細設計エンジニア / アーキテクト / SRE / 5 域 Lead 真人

---

## §0 文档信息 / 修订履历

| 项目 | 内容 |
|---|---|
| 文书 ID | SRS-MULTICA-TASK-001 |
| 文书名 | Multica Task Lifecycle 域要件定義書 (v33 候选对齐) |
| 版本 | v0.1 |
| 作成日 | 2026-09-11 |
| 作成者 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per DEC-008) |
| 承認者 | 架构师 (Mavis 接手 agent per DEC-008) |
| 关联 ADR | ADR-0026 v0.2 §1.2 + §2.1 模式 2 |
| 关联 inventory | inventory §2.2 v33 候选 |
| 上位要件 | SRS-MULTICA-RUNTIME-001 v0.1 (Runtime Registry 是 Task Lifecycle 的前置) |
| 守门合规 | 守门 #1 + #5 + #6 + #9 + #11 + #12 v21 + #14 v4 全过 |
| 模板结构 | 12 段严格按 brief §1.3 |
| 子能力 | 5 子能力 (TK-1 ~ TK-5) × 22 项 (per §6) |

---

## §1 文档目的 / 适用范围

### 1.1 文档目的

本文档基于 ADR-0026 v0.2 §1.2 痛点 + §2.1 模式 2 (WBS 5 态状态机 + review gate), 定义 STAR 平台 **Multica Task Lifecycle 域** 的需求规格说明书。

**核心方向锚点 (per ADR-0026 v0.2 + 2026-09-11 20:10 JST Ulysses 拍板)**: WBS 现有 4 态 → 5 态扩展 (加 `claimed` + `failed`), subagent complete → Mavis review → 才进 done。**配套** (跟 SRS-MULTICA-POISON-001 强绑定) session poisoning 标记。

### 1.2 背景 (用户痛点)

STAR / Mavis 当前 root session 模型下, 3 类具体痛点 (per ADR-0026 §1.2 + 守门 #9 实证):

1. **Subagent "succeeded" 不可信** — 守门 #9 实证 10/10 `net::ERR_CONNECTION_CLOSED` 但 status 报 succeeded
2. **claim 跟 start 状态不显式** — WBS 现在 4 态, 多个 subagent 抢同一任务时竞争不显式
3. **Complete ≠ 入 done** — subagent 报 complete 直接进 done, 无 review gate 拦截质量
4. **Session 不可恢复无标记** — `(agent, issue) session` resume 必然坏的情况, 反复 resume 同一烂 session (per Multica poisoned.go:10-217)

### 1.3 包含范围 (In-Scope)

5 子能力 (per Multica client.go + poisoned.go line refs):

| 子能力 | Multica 源 | 关键 file:line |
|---|---|---|
| TK-1 5 态状态机扩展 | client.go:227-235 (ClaimTask) + task state machine | enqueued → claimed → started → completed / failed |
| TK-2 4 类 404 区分 | client.go:35-91 (isWorkspaceNotFoundError / isTaskNotFoundError / isRuntimeNotFoundError / isUnauthorizedError) | daemon 知道 server 端删除事件 |
| TK-3 Session poison 标记 (跟 SRS-MULTICA-POISON-001 配套) | poisoned.go:10-217 (FailureReason × 5) | 4 类原因 + GetLastTaskSession 过滤 |
| TK-4 Review gate | Multica 默认 subagent complete → 人工 review → done | subagent_review.py |
| TK-5 WBS row schema 升级 | WBS current 4 态 → 5 态 + 字段 | `STAR-P3-WBS-001.md` 状态定义 |

### 1.4 不含范围 (Out-of-Scope, per ADR-0026 v0.2 §2.2)

- ❌ WebSocket claim (Multica real-time push)
- ❌ Per-runtime claim + machine-level batch claim (per Multica 30s/5s timeout)
- ❌ ReclaimStaleDispatchedTasksForRuntimes (per Multica)
- ❌ 5/10 max task concurrency 限制

### 1.5 受众范围 disclaimer

- 本 SRS 跟 WBS 状态定义一对一映射, 跟 `SRS-STAR-AGENT-RUNTIME-001.md` 平行
- WBS row schema 升级由本 SRS 拍板, 实装由 `scripts/automation/wbs_migrate_v33.py` 落地

---

## §2 用语定义

| 用语 | 定义 |
|---|---|
| **Enqueued** | 任务已入 WBS, 等待被 claim (Mavis 自动派) |
| **Claimed** | subagent 已 claim 但未 start (防多 subagent 抢同一任务) |
| **In Progress** (原 started) | subagent 实际开始执行 |
| **Completed** | subagent 报 success + Mavis review 通过 |
| **Failed** | subagent 报 fail (自动) / Mavis 标记 fail (手动) |
| **Cancelled** | 人工取消 (跟 Failed 区分, per ADR-0026 §1.2) |
| **Session poisoned** | `(agent, issue) session` resume 必然坏, 标 `session_poisoned: bool = true` (per poisoned.go 4 类) |
| **Review gate** | subagent 报 complete → Mavis verify (commit hash, output, artifact) → 才进 done |
| **Stale dispatch** | subagent 报 succeeded 但实际 RPC failed (per 守门 #9 实证) — 不进 in_progress 也不进 completed, 标 stale_dispatch = true |

---

## §3 业务背景

### 3.1 Multica 实证 (per ADR-0026 v0.2 §1.3)

| 文件 | 关键 | 引用 |
|---|---|---|
| `client.go:227` | `ClaimTask` (per-runtime 30s timeout) | TK-1 |
| `client.go:294` | `ClaimTasks` (machine-level batch 5s timeout, MUL-4257) | TK-1 (本 SRS 不引入 batch) |
| `client.go:35-91` | 4 类 404 区分 (workspace / task / runtime / unauthorized) | TK-2 |
| `poisoned.go:10-217` | 4 类 session poison 原因 + 4 classify 函数 | TK-3 (跟 SRS-MULTICA-POISON-001 配套) |
| `runtime_sweeper.go:160-170` | 6 阶段 sweep pipeline | 配套 (runtime liveness 跟 task lifecycle 强绑定) |

### 3.2 拍板来源

- 2026-09-11 20:10 JST Ulysses "multica 的核心功能我原则上都要有"
- 20:43 JST ask_user 选项 form_opt2 (分专题) + inventory_opt1 + code_opt1

### 3.3 守门合规 (per AGENTS.md §4)

- 守门 #1 v15 (新事件触发)
- 守门 #5 (env 安全)
- 守门 #6 (PowerShell only)
- 守门 #9 v27 (RPC fallback) — 本 SRS 直接对齐
- 守门 #11 (缺标比错标) — §1.4 + §8 显式列
- 守门 #12 v21 ([P] docs 同步)
- 守门 #14 v4 (Mavis 审核)

---

## §4 功能需求 (FR, 22 项)

### TK-1 5 态状态机扩展 (FR-1 ~ FR-6)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-1 | WBS row `status` 字段加 `claimed` + `failed` 2 态, 总 6 态: pending / claimed / in_progress / completed / failed / cancelled | P0 |
| FR-2 | `claimed` 显式存在, subagent dispatcher 调用后立即标 claimed (claim → start 间隔期间) | P0 |
| FR-3 | `failed` 跟 `cancelled` 区分: failed = subagent 报 fail (自动) / cancelled = 人工取消 (手动) | P0 |
| FR-4 | 状态机迁移脚本 `wbs_migrate_v33.py` 自动改 41 子项 status 字段 (per 守门 #1 v15) | P0 |
| FR-5 | `stale_dispatch: bool` 字段加进 WBS row, 标守门 #9 v27 RPC 失败的 task | P0 |
| FR-6 | 状态机守门: claimed 后 30s 内必须 start, 否则 reaper 自动回 pending (per Multica 30s claim timeout) | P0 |

### TK-2 4 类 404 区分 (FR-7 ~ FR-10)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-7 | WBS row 加 `workspace_not_found_at: timestamp` 字段, server 端 workspace 删时标 | P0 |
| FR-8 | WBS row 加 `task_not_found_at: timestamp` 字段, server 端 task 删时标 | P0 |
| FR-9 | WBS row 加 `runtime_not_found_at: timestamp` 字段, server 端 runtime 删时标 | P0 |
| FR-10 | automation console 显示 4 类删除事件 + 影响 subagent 数 | P0 |

### TK-3 Session poison 标记 (FR-11 ~ FR-15, 跟 SRS-MULTICA-POISON-001 配套)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-11 | WBS row 加 `session_poisoned: bool` 字段, 标 `(agent, issue) session` resume 必然坏 | P0 |
| FR-12 | WBS row 加 `session_poison_reason: enum` 字段, 5 类原因: IterationLimit / AgentFallbackMsg / APIInvalidRequest / CodexSemanticInactivity / CodexResumeOversized | P0 |
| FR-13 | 标 session_poisoned 后, 下次 subagent 接到该 task 自动 fresh session 起步 (不 resume) | P0 |
| FR-14 | session_poisoned 标写入审计 log (per 守门 #1 v15) | P0 |
| FR-15 | automation console 显示 🔴 poisoned 标 + 5 类原因过滤 | P0 |

### TK-4 Review gate (FR-16 ~ FR-19)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-16 | subagent 报 complete → 不直接进 done, 进 `pending_review` 中间态 (或 `in_progress` 带 `awaiting_review: bool`) | P0 |
| FR-17 | Mavis review 3 件事: (a) commit hash 存在 / (b) output 落档 / (c) artifact 落档 | P0 |
| FR-18 | Mavis review 通过 → 进 `completed`; review 失败 → 进 `failed` + 标 review_failed_reason | P0 |
| FR-19 | Review gate 可配置 skip (per 守门 #9 v27 fallback) | P0 |

### TK-5 WBS row schema 升级 (FR-20 ~ FR-22)

| FR | 描述 | 优先级 |
|---|---|---|
| FR-20 | WBS row schema 升级, 5 态 status + session_poisoned + stale_dispatch + 4 类 404 timestamp + review gate 字段 | P0 |
| FR-21 | 守门 #13 W-T-M 100% 覆盖: pending/claimed = Work (短 TTL), in_progress/completed/failed = Transaction (append-only), cancelled = Work (短 TTL) | P0 |
| FR-22 | WBS row 加 `task_lifecycle_audit: jsonb` 字段, 每次状态变更 +1 审计 entry | P0 |

---

## §5 非功能需求 (NFR, 5 项)

| NFR | 指标 |
|---|---|
| NFR-1 性能 | 状态机迁移 41 子项 < 5s |
| NFR-2 可靠性 | subagent RPC 失败 → stale_dispatch=true, 不阻塞其他 task |
| NFR-3 可观测 | 每次状态变更写 audit log, console 显示状态机 transition 图 |
| NFR-4 易用 | 状态机守门 = 不可绕过, 但提供 force_skip 路径 (per 守门 #9 v27) |
| NFR-5 安全 | session_poisoned 标不删除, 等人工修 (per 守门 #11) |

---

## §6 约束 / 风险

| 约束 | 描述 |
|---|---|
| 守门 #9 v27 RPC fallback | stale_dispatch=true 跟 review gate 配套 |
| 守门 #11 缺标比错标 | session_poisoned 不删标 |
| 守门 #13 W-T-M | 5 态 status 严格分类 |
| 守门 #14 v4 Mavis 审核 | author=Ulysses |

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 41 子项 status 字段迁移漏改 | 中 | 中 | `wbs_migrate_v33.py` 自动改 + `registry_check.py` 校验 |
| Review gate 太严阻塞主流程 | 中 | 高 | 提供 force_skip 配置 (per 守门 #9 v27) |
| Session poison 5 类原因分类不够 | 中 | 中 | Multica 当前 5 类, 后续可扩 (per Multica `taskfailure.ReasonAPIInvalidRequest` 集中) |
| Stale dispatch 标了之后没法识别 | 低 | 中 | automation console 红 banner + 周报 |

---

## §7 验收条件 (AC)

| AC | 描述 |
|---|---|
| AC-1 | 41 子项 status 字段全部迁移, 无 pending 漏改 (registry_check 验证) |
| AC-2 | subagent complete → 必进 pending_review, 不直接进 done |
| AC-3 | session_poisoned 5 类原因枚举全 |
| AC-4 | stale_dispatch=true 时, console 红 banner 显示 |
| AC-5 | 守门 #13 W-T-M 100% 覆盖 (Work / Transaction / Master) |
| AC-6 | automation console 状态机 transition 图可视化 |
| AC-7 | 4 类 404 timestamp 字段全部落档 |
| AC-8 | audit log 每次状态变更 +1 entry |

---

## §8 已知缺口 (per 守门 #11)

| 缺口 | 优先级 | 阻塞 | 缓解 |
|---|---|---|---|
| #1 5 态 state machine 跟现有 WBS row 字段冲突 (e.g. `status` 当前是 enum 4 态) | P0 | 阻塞 AC-1 | `wbs_migrate_v33.py` 兼容迁移 |
| #2 Session poison 5 类原因 跟现有 session 概念区分 (Mavis 当前 session = Mavis root session, 不等于 `(agent, issue) session`) | P0 | 阻塞 AC-3 | 文档加 disclaimer, 实施时统一命名 |
| #3 Review gate 跟守门 #9 v27 fallback 协调 (v27 已是 fallback 路径) | P0 | 阻塞 AC-2 | force_skip 配置 |
| #4 stale_dispatch=true 时, subagent output 怎么保留? (per 守门 #9 v27) | P1 | 不阻塞 | 暂存 `<task_id>.stale.json` 24h |
| #5 4 类 404 timestamp 跟 automation-design §3.4 横向 audit log 范式协调 | P1 | 不阻塞 | 复用现有 audit_log 字段 |
| #6 状态机守门 "claimed 后 30s 必须 start" 跟守门 #9 v27 30s claim timeout 重复 | P1 | 不阻塞 | 复用同一 timeout 常量 |

---

## §9 关联文档

| 类型 | 文档 |
|---|---|
| ADR | [`docs/adr/0026-multica-patterns-borrow.md`](../adr/0026-multica-patterns-borrow.md) v0.2 §1.2 + §2.1 模式 2 |
| Inventory | [`docs/inventory/multica-gap.md`](../inventory/multica-gap.md) v0.1 §2.2 v33 候选 |
| 配套 SRS | [`docs/requirements/SRS-MULTICA-POISON-001.md`](../requirements/SRS-MULTICA-POISON-001.md) (Session Poison 配套) |
| 配套 SRS | [`docs/requirements/SRS-MULTICA-RUNTIME-001.md`](../requirements/SRS-MULTICA-RUNTIME-001.md) (Runtime Registry 前置) |
| BD | [`docs/design/BD-MULTICA-TASK-001.md`](../design/BD-MULTICA-TASK-001.md) (下个 turn 落档) |
| WBS | [`STAR-P3-WBS-001.md`](../../STAR-P3-WBS-001.md) 状态定义 |
| 守门 | AGENTS.md §4.1 守门 #1 v15 / #9 v27 / #11 / #13 |

---

## §10 签字栏

| # | 角色 | 姓名 | 签字日 | 结论 |
|---|---|---|---|---|
| 1 | 架构负责人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 2026-09-11 20:43 JST 拍板 |
| 2 | SRE Lead | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 3 | 平台工程师 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 4 | 评审主持人 | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |
| 5 | 项目负责人（PM） | 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-11 | 🟢 接受 per 守门 #14 v3 |

---

## §11 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-11 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 初版（22 FR / 5 NFR / 6 已知缺口 + 5 角色签字栏） | 2026-09-11 20:43 JST ask_user 选项 form_opt2 |
