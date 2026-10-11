# PHASE-9F4C-D7 — Current catalogs and parallel preflight

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses（一人公司12角色per DEC-008）— Mavis 接手审核
> Status: 🟡 Current catalog preflight verified; complete admission/execution remains open.

## §0 Purpose

D7 增加 `ScheduleAdmissionTransaction.revalidate_execution_catalogs` 与私有 `ScheduleExecutionSnapshot`。同一 Run-owner 事务先通过 D4–D6 门，再由 native reader 仅读取 Profile 引用的当前 Provider/Skill 和当前 GrantSet：Provider≤5、Skill≤128、每项 capability≤64，Skill capability 预聚合预算384 KiB、返回 envelope≤512 KiB；Rust 复用 CurrentExecutionCatalogSnapshot/ExecutionProfileResolver，核对当前版本、摘要、availability、capability、完整 GrantSet 身份/版本/标签/到期时间，持有5秒目录 fence并在使用时要求至少1秒剩余。SQL 使用 C collation 对齐 Rust 的标识符排序。缺失、漂移、计数不一致、Grant变化或过期、fence到期均返回 TargetUnavailable，数据库失败不放行。

## §1 Change matrix

| Area | Implementation | Boundary |
|---|---|---|
| Native reader | Scope-authorized selected Provider/Skill/Grant, count/heap/wire bounds and row locks | No catalog write grant or executor permit |
| Publication/ACL locks | Shared native auth/catalog reads versus exclusive scoped writes; before-SCD trigger | Legacy Schedule API authorization helper stays conservative/exclusive |
| Rust | Shared domain snapshot/resolver,5-second fence, use-time expiry check | Private snapshot is no standalone authority after origin transaction ends |
| Verification | Real simultaneous same-Project preflight; normal ACL/catalog publish timeout, current dependency drift, real expiry | Synthetic registration/variation, no production writer proof |
| Docs | Requirements6.20/basic6.17, Schedule SRS0.41/BD-DD0.40, Data2.9, plan6.56, automation2.39/registry0.86 | Native Hook/readiness and execution still open |

## §2 Verification summary

验证：隔离 PostgreSQL18.6/Rust1.99.0 runner47/47，SQLx binding/admission targets2/2；包级all-targets check、库12/12、all-targets/no-deps Clippy -D warnings、包级fmt/Python/图头/diff。Audit `target/phase9f4d-run-owner-consumer/ee331ea1-5cde-4005-8f86-80cc630c1dce/audit.jsonl`。实际覆盖缺失目录、精确4 Provider/1 Skill/Grant、同Project两个不同Task/Worktree预检并行、正常ACL和目录publication55P03、Provider unavailable、Skill digest drift、Grant expiry drift、capability count mismatch及真实5秒fence到期拒绝。仅验证预检并行，不证明Run/executor并行吞吐；未测最大负载RSS/延迟。Fixtures初始注册/变异为隔离admin/replica，不证明生产publisher授权。零VM与生产DB变更。

First isolated runner failed at new parallel fixture registration because both checkouts reused the name fixture in one repository, violating worktree_canvas_worktree_repo_name_unique. Failure audit `target/phase9f4d-run-owner-consumer/9a4cb131-138c-4d85-a2e3-6c9df2452463/audit.jsonl`. Fixed generated fixture names to include their checkout UUID. Subsequent full runner47/47 passed; final run above includes actual fence expiry. All owned disposable PostgreSQL instances stopped. CodeRabbit attempted once and failed `/root/.local/bin/coderabbit: Permission denied`; no independent pass.

## §3 Known gaps

Native read 与 Run-owner 授权复验使用共享 Project advisory locks；目录 parent publication 和 canonical ACL mutation 仍使用相同 scope 的独占写锁，并持有当前目录/Grant row locks直到事务结束。reader SECURITY DEFINER/search_path=pg_catalog、PUBLIC execute 撤销、仅 Run owner EXECUTE，无目录直接写权限。发布 trigger 排序早于既有 SCD2 guard；capability 保持既有 same-transaction birth/immutable/count 契约。Snapshot 的 lifetime 校验不能替代原 owner 事务存活或执行前重新读取。目标 DB/function owner/RLS/应用身份装配仍未验收；任务 lifecycle、Project/Run预算预留、非执行态Run writer、owner delivery/confirmation、Runtime sandbox与独立验证仍开放，执行保持关闭。

调用时点须遵守已批准的基础设施准备模型：先创建等待基础设施预留的非执行态 Run，结果接受只表示准备请求已登记；实际启动/恢复 executor 前，必须复验 Host/cluster reservations与binding、当前权限/目录/策略，并以可信 Runtime health/readiness 和任务生命周期事实执行 Rust-native BeforeRunAdmission Hook。不得默认 health=true，不得以 UI health、VM/K3s就绪或仅文档校验代替 native Hook求值，也不新增弱化原生安全守门的 phase。当前本阶段未实现或声称通过该实际执行门。

Maximum-cardinality heap/RSS/throughput benchmarks remain unverified; source and schema impose limits, not measured performance. Final `cargo fmt --all -- --check` failed on pre-existing workspace formatting differences; no unrelated source formatted. `cargo fmt -p run-owner-runtime -- --check` and final package all-targets Clippy passed. Whole-workspace compile/test and release/doc/bench gates not run. Existing domain-kms3 deprecation warnings remain. RTK unavailable; direct PowerShell used. No matching indexed graph/tools; named bounded source/config fallback used. No new third-party dependency/version; full distribution SBOM/license closure remains release work.

## §4 Subagent failures and takeover

No subagent dispatched in this slice. Root implemented, reviewed and verified; source lanes and user dirty data preserved. External review failure disclosed in §2.

## §5 Guard rules

| # | Guard |
|---|---|
| 1 | Canonical Project → Branch → Run → Worktree scope preserved. |
| 2 | Fixed creator and Task binding revalidated in origin transaction. |
| 3 | Read only selected Profile references, never whole catalogs. |
| 4 | Provider5/Skill128/capability64 limits remain fixed. |
| 5 | Skill preaggregation heap384KiB/envelope512KiB bounds. |
| 6 | C collation matches Rust identifier ordering. |
| 7 | Current availability/digest/version/capability are authoritative. |
| 8 | Exact GrantSet and expiry verified independently of sealed declarations. |
| 9 | Five-second fence and one-second remaining threshold gate later use. |
| 10 | Shared Project auth reads still block exclusive ACL mutations. |
| 11 | Catalog publication lock runs before SCD guard. |
| 12 | Fixed search_path/schema; PUBLIC execute revoked; runtime no direct writes. |
| 13 | No runtime healthy=true fallback or weakened native Hook phase. |
| 14 | Non-executing preparation never counts as executor acceptance. |
| 15 | Selective local dev integration; zeroVM/production changes. |

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: scoped shared domain resolver and preparation/execution distinction. |
| SRE Lead | Mavis review: actual same-Project preflight concurrency and publication exclusion. |
| Platform | Mavis review: bounded selected data, native capabilities only. |
| Review chair | Mavis review: drift/expiry/count cases; independent tool unavailable. |
| PM | Mavis review: synchronized artifacts; complete product still open. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis review | Verify current bounded catalogs and shared authorizers. | Continue complete Schedule preparation after D6. |
