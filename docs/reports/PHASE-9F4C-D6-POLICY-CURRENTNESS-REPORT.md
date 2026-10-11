# PHASE-9F4C-D6 — Native Profile/Hook currentness

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> Status: 🟡 Policy currentness verified; complete admission/execution remains open.

## §0 Purpose

D6 为 Schedule 增加 `revalidate_profile_and_hooks` 与私有字段 `SchedulePolicySnapshot`：沿用 D4/D5 同事务授权、Binding 与任务归属门，原生固定路径 SQL reader 至多读取三份 72 KiB envelope；Rust 复用 domain-agent 的 Profile 校验和 domain-hook 的原生策略校验。精确核对 Profile ID/version/digest/schema/scope，Project baseline 与 Worktree overlay 的 ID、继承父版本和规则、有效 HookSet ID/version/digest，以及 Profile 与 RunInfrastructureBinding 的全部八项资源预算。缺失、版本/摘要/继承漂移或文档篡改返回 TargetUnavailable；预算与 binding 不一致返回 BindingUnavailable；存储异常不得放行。

## §1 Change matrix

| Area | Implementation | Boundary |
|---|---|---|
| Native reader | Three bounded envelopes, current authorized scope, row locks | No target table writes or execution permission |
| Native publication fence | Shared Project policy read/exclusive before-mutation fence, ordered before inheritance guard | Production owner/RLS/rollout unverified |
| Rust | Private verified snapshot, shared domain Profile/Hook verification, all8 budget comparisons | Current catalogs, native lifecycle evaluation and quota remain open |
| Dependencies | Two existing workspace path dependencies, Cargo.lock two dependency edges only | No external version update; no live LLM/runtime constructed |
| Docs | Requirements6.19/basic6.16, Schedule SRS0.40/BD-DD0.39, Data2.8, plan6.55, automation2.38/registry0.85 | Historic evidence preserved; partial current status explicit |

## §2 Verification summary

验证：Rust1.99.0 包级 all-targets check、库12/12、all-targets/no-deps Clippy -D warnings、fmt/Python/图头/diff；隔离 PostgreSQL18.6 runner44/44、SQLx binding/admission targets2/2。Audit `target/phase9f4d-run-owner-consumer/2a610d58-4b81-43d6-bc8a-4e8dc0c3e793/audit.jsonl`。实际验证 baseline 成功、缺失 overlay 的正常发布被 55P03 阻挡、事务结束后的 overlay 发布使旧 pin 失效、新 overlay pin 成功、Profile 版本漂移、预算 mismatch 和摘要篡改拒绝。Fixtures 的初始注册与 Profile 修订由隔离 admin/replica 完成，不证明生产授权 publisher；overlay 并发 INSERT 使用正常 publication trigger。未创建 VM、未改变 Multipass inventory/生产 DB。

Initial candidate package check rejected unsigned Profile versus signed binding comparisons; checked conversion/widening fixed them. First isolated runner stopped at binding-target compilation43 steps; failed audit `target/phase9f4d-run-owner-consumer/1fe2e141-648c-4941-ac78-5369a2d85220/audit.jsonl`; final complete runner44/44 passed. Owned disposable PostgreSQL stopped on both paths. No failed gate is represented as a pass.

## §3 Known gaps

Profile/Hook 当前事实保持 FOR SHARE 至事务结束；Project-scoped Hook 共享 advisory read fence 与 BEFORE publication 独占锁保护“当前没有 overlay”的读空隙，trigger 名称排序在现有继承 guard 之前。Native reader SECURITY DEFINER/search_path=pg_catalog，PUBLIC execute 撤销、仅 Run owner EXECUTE，不授予目录/任务/Profile/Hook 表写权限；目标部署须复核 function owner、RLS、底层读取/行锁权限与 migration 装配。此快照不是执行许可证：仍须当前 Provider/Skill/Grant catalog、任务生命周期、原生 Hook RunAdmission 求值与事实生产者、Project/Run quota/reservation、非执行态 Run writer、owner delivery/confirmation、Runtime sandbox 与独立验证。commit_result primitive 仍不是完整准入入口；执行保持关闭。

Current catalogs and grants expiry are not proved by a sealed Profile. Native policy verification is not lifecycle Hook evaluation. Whole-workspace/release/doc/bench gates and end-to-end memory/concurrency measurements were not run; existing domain-kms three deprecation warnings remain. Existing authorization lock serializes same-Project authorization checks; parallelism within a Project requires later lock/admission budget review. No new third-party version introduced; complete distribution SBOM/license closure remains a product release gate. CodeRabbit independently attempted once and failed `/root/.local/bin/coderabbit: Permission denied`; no independent pass claimed. RTK absent; direct commands used. No indexed graph project available; named bounded source/config fallback used.

## §4 Subagent failures and takeover

No subagent dispatched; root implemented and verified this slice. External review failure disclosed above. No user workspace overwritten or source branch deleted.

## §5 Guard rules

| # | Guard |
|---|---|
| 1 | Project → Branch → Run → Worktree canonical scope preserved. |
| 2 | Fixed creator reauthorized in owner transaction. |
| 3 | Exact current Task and checkout prerequisite required. |
| 4 | Native policy reader uses fixed schema/search_path. |
| 5 | PUBLIC execute revoked; Run owner only. |
| 6 | No runtime Profile/Hook mutation capability. |
| 7 | Reader returns at most3 envelopes, each72KiB. |
| 8 | Canonical Profile document/schema/scope/pinned identity verified. |
| 9 | Effective overlay matches current baseline identity/version/rules. |
| 10 | Native digest verified; Python input never becomes authority. |
| 11 | Publication fence protects missing overlay and avoids inheritance lock inversion. |
| 12 | All8 budgets match current fixed binding without unchecked narrowing. |
| 13 | Current catalogs/native evaluation/quota/readiness still gate execution. |
| 14 | Zero VM, no daemon/socket/host-shell exposure or production DB change. |
| 15 | Selective local dev integration preserves source and user data. |

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: shared domain rules, scoped same-owner transaction. |
| SRE Lead | Mavis review: isolated44/44, actual publication concurrency, target deployment remains open. |
| Platform | Mavis review: bounded payloads and no runtime policy mutation. |
| Review chair | Mavis review: digest/drift/budget/overlay checks, independent tool unavailable. |
| PM | Mavis review: synchronized documents; complete product not marked achieved. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis review | Verify locked Profile/effective HookSet and budget consistency. | Next recommended admission prerequisites after D5. |
