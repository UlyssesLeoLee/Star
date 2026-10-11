# PHASE-9F4C-D10 — Preparation compensation

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses — Mavis接手审核
> Status: isolated compensation verified (54/54); full product phases open.

## §0 Purpose

Close owner-local compensation for prepared Runs before confirmation, demand or execution.

## §1 Change matrix

| Area | Change | Boundary |
|---|---|---|
| Rust | Strict compensation consumer, trusted-source matching | No production service authentication |
| SQL | Native release, paired audit, immutable receipt, demand fence | No external owner writes |
| Tests | Real owner release, replay, rollback, RLS and demand rejection | Fixture producers only |
| Docs | Requirements/basic/detailed/data/phase/automation synchronized | Whole execution still open |

Run owner提供`PostgresScheduleAdmissionStore.compensate_admission`，消费有界严格`ScheduleRunAdmissionResolutionV1`补偿命令。必须绑定原始Schedule service identity、tenant及不可变accepted result的request/attempt/result/correlation/occurrence/Run/binding digest；错误来源、跨租户、伪造digest、Confirm误投及冲突重放均拒绝。可信transport认证仍须由部署组合，调用者传入UUID不是认证证据。

Native `compensate_schedule_preparation`在自身事务中串行原request及Project allocation epoch，仅对未确认、未执行、未发布Infrastructure demand且reservation仍pending的D9准备Run释放Project预算。它原子写入released状态、配对reservation/Run release audit、schedule_admission_compensated及exact-byte不可变receipt。最后receipt写入失败须回滚释放与全部事件。清理以原始owner事实为依据，不要求已撤权run-as重新授权；此权限不能创建或启动工作。历史accepted replay不改变released reservation。

新`guard_prepared_schedule_demand`对带D9 preparation lineage的Run复用同Project epoch和Run行锁，要求确认事件、未补偿和有效pending reservation，避免低层Outbox primitive绕过确认或与补偿竞争；旧无preparation lineage的Run保持既有契约。REPEATABLE READ过时epoch必须由数据库拒绝，未以Rust无界缓存观察竞争状态。

W/T/M新增0/1/0：`multica.schedule_compensation_inbox`为T（不可变exact-wire、无TTL/物理删除、FORCE tenant RLS）；既有reservation和allocation epoch为W，release/Run events及receipt为T；M只读。仅受限SECURITY DEFINER固定search_path capability执行写入，Run role无直接receipt/reservation/Task mutation。新表索引为tenant/request主键与唯一resolution身份，wire≤16KiB；函数有5秒lock/10秒statement timeout。

验证：隔离PostgreSQL18.6/Rust1.99.0 runner 54/54，binding/admission SQLx targets 2/2，库15/15及包级all-targets check、strict Clippy、fmt、Python编译、Cypher图头与diff检查通过。Audit `target/phase9f4d-run-owner-consumer/67cccbc7-e28b-4984-bd14-920afbc329f5/audit.jsonl`。真实owner角色验证释放/配对审计/receipt、exact replay、冲突重放、来源/tenant/digest误配、Confirm误投、active/confirmed拒绝、最终receipt失败整笔回滚、跨tenant RLS与补偿前后demand拒绝。Active/confirmed输入为隔离admin/replica fixtures，不证明生产确认/激活producer；未验证新的跨进程并发/过时snapshot竞争或最大规模性能。隔离PostgreSQL已停止，零VM/生产DB操作。

Schedule result/link/confirmation producer、confirmation消费与Infrastructure demand真正发布、全量超时release worker、跨Infrastructure owner补偿、生产service身份/DB grants、native Hook求值、Host/cluster reservations、Manager/sandbox/readiness、取消恢复及独立验证仍开放；本阶段不可宣称Agent/CLI闭环可用。零新依赖、零VM/生产DB操作。

## §2 Verification summary

验证：隔离PostgreSQL18.6/Rust1.99.0 runner 54/54，binding/admission SQLx targets 2/2，库15/15及包级all-targets check、strict Clippy、fmt、Python编译、Cypher图头与diff检查通过。Audit `target/phase9f4d-run-owner-consumer/67cccbc7-e28b-4984-bd14-920afbc329f5/audit.jsonl`。真实owner角色验证释放/配对审计/receipt、exact replay、冲突重放、来源/tenant/digest误配、Confirm误投、active/confirmed拒绝、最终receipt失败整笔回滚、跨tenant RLS与补偿前后demand拒绝。Active/confirmed输入为隔离admin/replica fixtures，不证明生产确认/激活producer；未验证新的跨进程并发/过时snapshot竞争或最大规模性能。隔离PostgreSQL已停止，零VM/生产DB操作。

First strict Clippy pass caught clone_on_copy in the new test budget fixture. Removed the redundant clone; strict Clippy, library tests and the complete PostgreSQL runner then passed. Initial passes: f067bbe0-8c87-4fef-866f-99427c08755b and 5e1cdc39-fc1a-4b05-a39c-22e22a312a50 (54/54 each); final audit above includes the corrected source. No suppression or dependency changes.

## §3 Known gaps

Confirmation producer/consumer and demand publication, expired preparation worker, external reservation release, production identity/database role composition, Host Manager and independent Runtime/Hook/readiness/validation remain open. Focused package checks cannot prove full-workspace, release, performance or product closure. CodeRabbit unavailable in preceding slice (Permission denied), no independent review pass. RTK/graph tools unavailable; bounded source fallback used.

## §4 Subagent failures and takeover

No subagents dispatched. Root owns this lane; source and user worktrees preserved.

## §5 Guard rules

1. Trusted tenant and original service identity.
2. Strict bounded wire.
3. Exact accepted result lineage.
4. Confirm messages rejected by compensation endpoint.
5. Same request lock.
6. Same Project epoch.
7. Only non-executing unconfirmed Run.
8. No existing Infrastructure demand.
9. Only pending Project reservation.
10. No cross-owner writes.
11. Release/audit/receipt commit together.
12. Exact replay does not reactivate budget.
13. Conflicting replay fails closed.
14. FORCE RLS and append-only receipt.
15. No executor/VM activation; whole goal remains active.

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: owner-local compensation boundary. |
| SRE Lead | Mavis review: atomic rollback and bounded lock waits. |
| Platform | Mavis review: native least privilege and tenant RLS. |
| Review chair | Mavis review: isolated evidence and explicit remaining gaps. |
| PM | Mavis review: full phases remain open. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis review | Preparation compensation and demand fence. | Continue after D9 integration. |
