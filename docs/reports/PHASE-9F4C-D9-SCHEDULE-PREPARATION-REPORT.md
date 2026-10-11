# PHASE-9F4C-D9 — Atomic Schedule Run preparation

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses — Mavis接手审核
> Status: 🟡 Preparation writer verified in isolated PostgreSQL (52/52); full product phases open.

## §0 Purpose

Schedule owner的fresh admission须经D4–D8当前授权、绑定、策略、目录和容量复验后，在Run owner本地事务原子创建非执行态TaskExecutionRun、pending Project reservation、成对reservation审计/RunEvent及处理后Inbox/admission-result Outbox。必须从canonical Task metadata、lifecycle和当前非空TaskContract读取有界完整快照，不接受调用者伪造任务内容。`started_at`与execution_state保持空，不产生run_started、VM请求或Task lifecycle写入；accepted只表示准备已登记。

## §1 Change matrix

| Area | Change | Boundary |
|---|---|---|
| Rust | Consuming prepare_and_commit, current gates, immutable typed result | No service/broker authentication composition or executor |
| SQL | Canonical lifecycle/contract/scope snapshot, quota recheck, native Run/reservation/event writer | Direct raw mutation privileges remain absent |
| T lineage | Request FK, binding snapshot/digest, deferred accepted result/pending reservation pair | Historic rows stay unchanged |
| Verification | Real owner preparation, source ineligibility, rollback/replay, no execution/demand | Admin fixture publishers are not production evidence |
| Docs | Requirements6.22/basic6.19, SRS0.43/BD-DD0.42, Data2.11, plan6.58, automation2.41/registry0.88 | Confirmation and real execution still open |

`ScheduleAdmissionTransaction.prepare_and_commit`消耗原事务，并复用`commit_result`的exact replay/pair持久化。当前准入拒绝可提交不可变rejected result；存储错误、写入后fence/deadline失效则整笔回滚，不把已暂存Run与rejected结果一起提交。受限native `prepare_schedule_execution_run`复验scope/当前Profile和binding、再次串行核算Project容量并拒绝同Task已有active/未过期pending reservation。Task eligibility与既有CLI一致：in_progress、固定run-as持有同Worktree且非pending_review；当前TaskContract必须存在且验收项非空。Task/验收快照各≤32KiB，描述≤16KiB、labels≤64、dependencies/验收项≤128；超限拒绝，不截断。完整typed Engineering Run snapshot保留全部25个identity/version/role/ref字段；Run另持久化不可变Infrastructure binding snapshot/digest和request关联。

## §2 Verification summary

验证：隔离 PostgreSQL18.6/Rust1.99.0 runner 52/52，binding/admission SQLx targets 2/2，库测试15/15，包级all-targets check、Clippy、fmt、Python编译、Cypher图头及diff检查通过。Audit `target/phase9f4d-run-owner-consumer/00ff7b51-16c4-4a62-a776-121f6ef33a47/audit.jsonl`。实际覆盖受限owner角色写入、生命周期/TaskContract拒绝、deadline拒绝、延迟配对门拒绝孤立提交、写入后目录fence失效整笔回滚、结果持久化失败回滚及同命令重试、exact replay和容量占用；未产生execution事件或Infrastructure demand。隔离PostgreSQL已停止，零Multipass inventory和生产DB变更。未完成workspace/release/doc/bench、生产交付或性能验收。

First integration compile failed on a new test SQL string escaping error and missing sqlx::Row import; repaired and all-targets check passed. Audit `target/phase9f4d-run-owner-consumer/3faab2c7-f08a-4a58-8f95-5f0f8cfdb0b5/audit.jsonl`. The next test exposed strict existing Engineering Run snapshot requirements (ck_task_run_engineering_run_snapshot_shape); writer was corrected to capture all canonical grant/revision/role/ref fields, without weakening the guard. Audit `target/phase9f4d-run-owner-consumer/bdf951b6-f50a-4588-aa73-07fdd8c94e5d/audit.jsonl`. First complete preparation pass:52/52, audit `target/phase9f4d-run-owner-consumer/235feb91-cee9-4b35-8cdb-cf51e17a3c97/audit.jsonl`. A later extended attempt hit Windows LNK1104 before binding integration execution, audit `target/phase9f4d-run-owner-consumer/b8be1812-54b3-472b-9a6b-30e8be4cf4a7/audit.jsonl`; terminal process and owned PostgreSQL stop confirmed before retry. A subsequent attempt hit LNK1104 for the admission executable, audit `target/phase9f4d-run-owner-consumer/2ec64ad1-d085-4bd6-bdb7-6e19fd8fd56a/audit.jsonl`. No relevant live build/test process was found; the isolated output directory write probe passed and the drive had approximately 822GB free. A serial admission target relink (`--no-run -j 1`) passed, then the complete extended runner passed 52/52. The transient linker root cause remains unproven; no ACL change or target cleanup was performed. CodeRabbit probe failed Permission denied; no independent review pass.

## §3 Known gaps

准备阶段的Outbox是Run→Schedule admission-result，不提前发布Infrastructure demand。Schedule owner必须消费结果、复验本owner lease/generation/deadline/cancellation，提交链接及confirmation；Run owner在下一独立事务消费可信confirmation、复验当前授权/绑定/Project reservation后才发布自己的Infrastructure demand Outbox。Host/cluster容量由Infrastructure owner自己的事务预留。这维持DD §6.2既有两owner确认协议，并明确D8“原子Run/reservation与Outbox”中的Outbox时点，不能用准备accepted绕过confirmation。Task owner的自动分配/claim/in_progress生产者仍需接入，Schedule不可跨owner篡改lifecycle。

Project准备reservation的lease由数据库取min(occurrence deadline,当前时间+2分钟)，必须与目录5秒fence区分。等待确认期间预算已占用；超时/取消的补偿、release/expiry worker及后续激活仍开放，不把lease过期视作VM/Runtime已清理。新增W/T/M表0/0/0：现有reservation为W，Run/reservation events/RunEvent/Inbox/results为T，TaskContract与binding为M只读。新请求关联有deferred FK，native Run插入有deferred accepted-result/pending-reservation配对门，阻止孤立准备事实提交。SECURITY DEFINER固定search_path，PUBLIC execute撤销，仅可信Rust Run-owner角色EXECUTE，仍无直接Run/event/reservation/Task写权限。原生Hook实际求值、健康/sandbox/readiness、独立验证与executor门保持关闭；未把REST/UI或fixture作为生产闭环。

Production service/broker authentication, target database function owner/RLS/read privileges and role provisioning, command production/delivery, Schedule result/confirmation consumer, Run confirmation handler and demand publication, bounded compensation/release/expiry, Task automatic claim/assignment, trusted provider/Manager, physical host/cluster reservations/readiness, native execution Hook/sandbox/independent validation, cancellation/resume and BI/Benchmark remain incomplete. Preparation is an owner-local implementation, not a usable unattended Agent/CLI product closure.

No maximum-cardinality/RSS/latency or actual executor throughput benchmark; no whole-workspace/release/doc/bench pass. Previous workspace fmt differences remain. Existing domain-kms deprecations remain. RTK unavailable; direct PowerShell. Graph tools/matching index unavailable; named bounded source evidence. No new dependency/version; full distribution SBOM/license closure remains release work.

## §4 Subagent failures and takeover

No subagent dispatched. Root implemented/verified; user/source worktrees retained. External review unavailable as disclosed in §2.

## §5 Guard rules

| # | Guard |
|---|---|
| 1 | Canonical full Project/Branch/Run/Worktree identity. |
| 2 | Fixed run-as current authorization. |
| 3 | Current policy/catalog/binding and quota gates in one transaction. |
| 4 | Canonical eligible lifecycle, no arbitrary Task claim mutation. |
| 5 | Nonempty current bounded acceptance contract. |
| 6 | Complete typed directory grant/revision snapshot. |
| 7 | Immutable infrastructure identity/digest snapshot. |
| 8 | Same Project allocation epoch and exact resource ceilings. |
| 9 | Existing held Task reservation prevents duplicate preparation. |
| 10 | Pending lease≤2min and occurrence deadline. |
| 11 | Paired reservation/Run event, no fake runtime measurements. |
| 12 | Deferred request/result/reservation pairing forbids orphan commit. |
| 13 | Late fence/deadline and persistence failures rollback all writes. |
| 14 | Schedule confirmation precedes Infrastructure demand. |
| 15 | No run_started, execution state, VM or cross-owner writes. |

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: preparation/result versus later confirmation/demand. |
| SRE Lead | Mavis review: allocation serialization and atomic rollback. |
| Platform | Mavis review: native constrained writer and canonical snapshots. |
| Review chair | Mavis review: real SQLx writer, honest production and external review gaps. |
| PM | Mavis review: synchronized implementation state; full phases active. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis review | Implement atomic non-executing Run preparation without pre-confirmation demand. | Continue complete Schedule owner closure after D8. |
