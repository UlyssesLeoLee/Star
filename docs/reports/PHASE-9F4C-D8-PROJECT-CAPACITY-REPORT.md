# PHASE-9F4C-D8 — Current Project capacity gate

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses — Mavis接手审核
> Status: 🟡 Capacity gate verified; complete preparation/execution remains open.

## §0 Purpose

Schedule owner 必须在同一 owner 事务内完成固定 run-as 与 canonical Task/Worktree、Profile/Hook、Provider/Skill/Grant 及绑定复验，再比较当前 Project 配额与 Run 的全部八项 ResourceBudget 上限。不得以成功预检代替实际 reservation、非执行态 Run 创建或 executor 启动许可。缺失/禁用/未来生效配额、预算超限、计数/算术溢出与过期目录 fence 必须 fail closed。

## §1 Change matrix

| Area | Change | Boundary |
|---|---|---|
| Native SQL | Canonical task authorization, existing allocation epoch, current quota and aggregated ledger | No Run/reservation insert |
| Rust | Same-transaction catalog gate then quota comparison of all8 dimensions and slot count | Private snapshot no independent reservation authority |
| Verification | Missing/disabled quota, exact fit, normal epoch/quota locks, stale snapshot, lease states | Fixtures not production publishers/writers |
| Documentation | Requirements6.21/basic6.18, SRS0.42/BD-DD0.41, Data2.10, plan6.57, automation2.40/registry0.87 | Full execution open |

`revalidate_project_capacity` 使用 native `read_schedule_project_capacity_current`，复用手动 Run 分配路径的 Project allocation epoch（UPSERT 实际写入），串行同 Project 的容量分配。读取当前有效 quota，持有 quota publication 同键共享 advisory lock及quota行锁；REPEATABLE READ 下过时 epoch 必须产生40001而不能用旧 reservation 视图放行。统计同 Project 所有 active reservation，以及 lease 未过期的 pending reservation；过期 active 仍计数，released与过期 pending不计数。CPU/运行时长按单Run ceiling核对，RSS/子进程/并行工具/provider calls/输出/event buffer 按共享剩余总量核对，另核对Run数量。SQL仅返回一个≤4KiB envelope，Rust checked-add核算且在等待容量锁后重验5秒目录fence；无整表载入Rust缓存。数据库SUM仍需扫描Project相关ledger，未声称已验证最大规模查询/RSS性能。

## §2 Verification summary

验证：隔离 PostgreSQL18.6/Rust1.99.0 runner49/49，binding/admission SQLx targets2/2、库15/15、包级all-targets check/Clippy与fmt、Python编译、图头和diff检查。Audit `target/phase9f4d-run-owner-consumer/d1e13219-1fbf-4f85-b7c1-e3604fa548d7/audit.jsonl`。实际覆盖缺失/禁用quota、精确预算匹配、同Project容量预检与正常手动epoch写入锁互斥（55P03）、quota关闭等待、drop回滚epoch、REPEATABLE READ过时allocation snapshot拒绝40001，以及未过期pending/过期pending/过期active/released计数。初始quota和reservation状态使用隔离admin/replica fixtures，不证明生产quota publisher、Run/reservation writer或真正executor并发。零VM/productionDB变更。

Initial package Clippy found15 unnecessary casts/conversions in newly added fixture code. Removed same-type conversions and made the provider-call narrowing conversion checked. A final extended runner attempt failed before executing admission tests due Windows linker LNK1104 opening the generated test executable; audit `target/phase9f4d-run-owner-consumer/83964c19-01de-4632-8538-762f21e82884/audit.jsonl`. The process was confirmed terminal and its owned PostgreSQL stopped before retry. No test assertion failure in that attempt. CodeRabbit version probe failed `/root/.local/bin/coderabbit: Permission denied`; no independent review pass.

## §3 Known gaps

reader SECURITY DEFINER固定search_path=pg_catalog，PUBLIC execute撤销，仅Run owner受限EXECUTE；不授予runtime直接quota/epoch/reservation写权限。仅现有Work allocation epoch在本事务中变化，drop回滚；没有写入Run、reservation或容量审计事实，也不表示Project容量已预留。新增W/T/M表0/0/0；现有epoch保留30天TTL。未来Run/reservation writer必须在该仍存活事务内再次确认fence并原子写入Run、reservation及成对审计/Run事件和Infrastructure demand Outbox；Host/cluster容量仍由Infrastructure owner自己的事务预留。原生执行Hook、Runtime健康/沙箱、取消恢复与独立验证仍为实际executor门，不制造健康事实。

Task lifecycle/complete task snapshot reader, occurrence deadlines/cancellation, actual resource reservations, non-executing Run writer, complete immutable profile/Hook/context snapshots, Run events/demand Outbox, authenticated production consumers/targetDB deployment, infrastructure reservations/readiness, trusted Manager/provider, sandbox/native Hook evaluation, independent validation and BI/Benchmark closure remain open. This gate is not a complete admission entry point; `commit_result` remains a storage primitive. No executor enabled. Shared physical Host/cluster capacity cannot be inferred from Project quota.

No maximum-ledger-size latency/RSS or actual executor concurrency measurement. No whole-workspace/release/doc/bench gates run; previous workspace fmt check failed existing unrelated differences. Existing domain-kms deprecations remain. RTK unavailable; PowerShell fallback. No matching available graph tools/index; bounded named source fallback. Existing table schema/owner roles are tested only in disposable PostgreSQL; production function owner/RLS/underlying privileges need provisioning review. No new dependency version; release SBOM/license closure open.

## §4 Subagent failures and takeover

No subagent dispatched. Root implemented and verified. User/source worktrees preserved. External review limitation disclosed in §2.

## §5 Guard rules

| # | Guard |
|---|---|
| 1 | Canonical Project/Branch/Run/Worktree scope. |
| 2 | Fixed creator reauthorized before fresh capacity observations. |
| 3 | Current Profile/Hook/catalog checks in same owner transaction. |
| 4 | Exact eight Run ceilings; no default quota. |
| 5 | Same Project allocation epoch as existing manual writer. |
| 6 | Stale repeatable-read snapshot rejects, never overcommits. |
| 7 | Current quota and publisher lock retained until commit/rollback. |
| 8 | Active remains counted even after expiry. |
| 9 | Expired pending and released excluded. |
| 10 | CPU/runtime per Run, other maxima shared sums. |
| 11 | Checked Rust arithmetic; database overflow fails. |
| 12 | One≤4KiB envelope; no Rust ledger materialization. |
| 13 | Catalog fence rechecked after lock waits. |
| 14 | No direct ledger mutation grants, no cross-owner writes. |
| 15 | Observation is not reservation/Run creation/execution permission. |

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: existing ledgers and owner boundaries. |
| SRE Lead | Mavis review: shared allocation lock and fail-closed contention. |
| Platform | Mavis review: native capability with bounded response. |
| Review chair | Mavis review: package/PG evidence and honest missing writer. |
| PM | Mavis review: synchronized partial scope, full phases remain active. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis review | Add current Project capacity gate before actual preparation writes. | Continue Schedule owner closure after D7. |
