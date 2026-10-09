# PHASE-9F4C-D5 — Transactional Task/Worktree currentness

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> Status: 🟡 Task binding prerequisite verified; complete admission/execution remains open.

## §0 Purpose

Schedule 新尝试必须在 Run-owner 同一事务内复验 Task/Worktree 的当前 canonical 归属：tenant、Project、repository、Branch、Engineering Run、Task、Worktree 精确一致；Worktree 未归档且有效，Project binding、Run binding、Task metadata、Task–Worktree relation 均为当前有效行。`revalidate_task_binding` 先复验固定创建者及锁定的 infrastructure binding，再调用固定路径 native SQL capability，持有五张目标表的 FOR SHARE 直到事务结束。缺失、归档或解绑返回 TargetUnavailable；权限/binding 拒绝保留原 typed code，数据库错误不得放行。

## §1 Change matrix

| Area | Change | Boundary |
|---|---|---|
| Native capability | verify_schedule_task_current, scoped static join, locks all five target rows | No new table or direct runtime write grant |
| Rust | revalidate_task_binding composes directory/binding then target gate | Profile/Hook/lifecycle/quota and Run writer remain separate |
| Runner | Apply existing task metadata/owner prerequisites and new migration twice; enforce least privilege | Disposable-only legacy audit stub and replica fixture |
| Docs | Requirements6.18/basic6.15, Schedule SRS0.39/BD-DD0.38, Data2.7, plan6.54, automation2.37/registry0.84 | Earlier evidence preserved |

## §2 Verification summary

验证：PostgreSQL18.6/Rust1.99.0 runner 40/40，SQLx binding/admission targets2/2；库12/12、包级 all-targets/no-deps Clippy -D warnings、格式/Python/图头/diff 通过。实际覆盖目标缺失、Task 身份不匹配、当前有效、Worktree 归档、关联关闭以及五类目标行并发关闭 SQLSTATE55P03。Audit：`target/phase9f4d-run-owner-consumer/857a6fbe-6bc7-4978-b070-bab871067cf7/audit.jsonl`。隔离 admin fixture 跳过生命周期触发器，不作为生产授权写入/审计证据。零 VM；生产数据库未变。CodeRabbit 因 /root/.local/bin/coderabbit: Permission denied 无独立通过结果。

Final runner failure short-circuit review: stop immediately when either currentness migration fails. Final schema-only rerun38/38; audit `target/phase9f4d-run-owner-consumer/7b824a59-1d4e-49ce-8cee-e7f03740a1a2/audit.jsonl`. Deterministic Cypher guard checks three Rust/Python source files; SQL manifest was manually reconciled against the fixed function joins.

## §3 Known gaps

仅新增受限 EXECUTE 能力；SECURITY DEFINER 使用 search_path=pg_catalog、静态限定 schema，PUBLIC execute 撤销，runtime 无目标表写权限。目标环境 function owner/RLS/底层行锁权限仍须验收。Profile、原生 HookSet、任务生命周期可执行状态、预算/reservation、非执行态 Run writer、owner delivery/consumer、sandbox 与真正执行闭环仍未完成；primitive commit_result 不是完整准入入口，不启用执行。

Broader workspace build/test/release/doc/bench and production deployment were not run or claimed. Existing domain-kms dependency deprecation warnings remain. RTK absent; direct PowerShell commands used. No indexed graph project is available; bounded named source/config reads were used. No cleanup of source lanes or unrelated dirty files.

## §4 Subagent failures and takeover

No subagent dispatched in this slice. Root implemented and reviewed the scoped changes. Independent CodeRabbit review unavailable as recorded above.

## §5 Guard rules

| # | Guard |
|---|---|
| 1 | Project → Branch → Run → Worktree remains canonical. |
| 2 | Fixed creator is reauthorized for each fresh attempt. |
| 3 | Exact historical replay remains a recovery fact. |
| 4 | Target scope includes tenant/project/repository/branch/run/task/checkout. |
| 5 | No Worktree ownership inference from labels or paths. |
| 6 | All five target rows remain locked until transaction ends. |
| 7 | Archived or invalid Worktree rejects admission. |
| 8 | Missing or closed task association rejects admission. |
| 9 | SQL capability uses fixed schema and search_path. |
| 10 | PUBLIC execute revoked; Run owner only. |
| 11 | Runtime gains no target table write capability. |
| 12 | Missing/integrity/storage failure never enables execution. |
| 13 | Profile/Hook/lifecycle/quota/reservations/readiness remain independent gates. |
| 14 | Multipass inventory unchanged, zero VM creation. |
| 15 | Selectively integrate verified content into clean local dev; preserve other lanes. |

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: canonical scope and same-owner transaction. |
| SRE Lead | Mavis review: isolated PostgreSQL checks, production gates remain open. |
| Platform | Mavis review: fixed capability with no runtime mutation. |
| Review chair | Mavis review: real locking and rejection cases, independent tool unavailable. |
| PM | Mavis review: documents synchronized; full product not marked complete. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis review | Implement and verify locked Task/Worktree prerequisite. | Continue recommended complete admission sequence after D4. |
