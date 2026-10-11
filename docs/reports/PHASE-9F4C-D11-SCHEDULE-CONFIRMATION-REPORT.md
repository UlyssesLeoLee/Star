# PHASE-9F4C-D11 — Schedule confirmation and infrastructure demand

> Version: v0.1 (2026-10-10)
> Reviewer: Ulysses — Mavis接手审核
> Status: isolated Run-owner confirmation verified (56/56); full phases remain open.

## §0 Purpose

Implement the Run-owner half of the accepted-result confirmation handshake and publish its own bounded demand atomically.

## §1 Change matrix

| Area | Change | Boundary |
|---|---|---|
| Rust | Trusted-source confirmation, current gates, exact replay | No Schedule producer or broker authentication |
| SQL | Native current quota/task/policy/binding checks; receipt/event/demand | No Host/cluster reservation or executor |
| Tests | Real owner confirm, expiry/revocation, rollback, late fence and replay | Canonical source producers are fixtures |
| Docs | Requirements/basic/detailed/data/phase/automation sync | Full product objective open |

`PostgresScheduleAdmissionStore.confirm_admission`严格解码有界Schedule确认命令，核验原始Schedule service identity、tenant、accepted result及request/attempt/result/correlation/occurrence/Run/binding digest。它在同Run-owner事务复验当前Project/Branch/Run授权、Worktree/Task绑定、当前Infrastructure binding、Profile/Hook、Provider/Skill/Grant目录及5秒目录fence。确认不是executor许可证。部署必须独立认证service；注入UUID不是transport认证证据。

受限native `confirm_schedule_preparation`串行request和既有Project allocation epoch；复核当前quota及全Project已占用总量，准备reservation已在总量中，不能重复加算其预算。它锁定未确认、未补偿、未执行Run与有效pending reservation，检查Task lifecycle仍in_progress、run-as/Worktree一致、非pending_review，当前metadata/TaskContract版本与准备快照相同，以及当前binding/Profile版本与有效Hook文档不变。原生Hook的实际执行、Runtime健康/sandbox与独立验证仍为后续门。

确认RunEvent、typed V2 Infrastructure demand和不可变confirmation receipt在一个Run-owner事务提交。demand截止时间由DB钳制为min(occurrence deadline,Project reservation lease)，payload/receipt wire≤16KiB，Profile envelope≤72KiB；late fence/deadline或任何写入失败回滚全部事实。receipt引用其demand，tenant FORCE RLS且append-only，无直接receipt/Run/event/reservation写权限。请求精确重放返回同一历史demand/event identity，不重建demand或激活预算；不同resolution bytes拒绝。已确认/demand发布后不能再走D10本地补偿，后续必须通过跨owner释放协议处理。

新增W/T/M表0/1/0：`multica.schedule_confirmation_inbox`为T，无TTL/物理删除；既有Project allocation epoch/reservation为W，RunEvent/demand为T；当前Task/Profile/Hook/binding/catalog为M只读。单事务复用有界快照和数据库锁，无跨owner写事务，无VM创建或宿主shell权限扩展；Host/cluster容量由Infrastructure owner另行核验/预留。

验证：隔离PostgreSQL18.6/Rust1.99.0 runner 56/56，binding/admission SQLx targets 2/2，库15/15及包级all-targets check、strict Clippy、fmt、Python编译、Cypher图头和diff检查通过。Audit `target/phase9f4d-run-owner-consumer/b0a6e67a-47c9-4f05-86ce-aad4ae1292a8/audit.jsonl`。真实Run-owner路径验证当前授权撤销/预留过期/quota禁用拒绝、最后receipt失败整笔回滚、写入后目录fence失效回滚、成功确认及同一demand精确重放、冲突resolution拒绝、跨tenant RLS、确认后D10补偿拒绝，以及demand deadline钳制至reservation lease；只有一条demand且零执行事件。权限/quota/时钟失效来源仍为隔离admin/replica fixtures；未证明真实Schedule producer、认证transport、跨进程竞争或最大规模性能。隔离PostgreSQL已停止，零Multipass inventory/生产DB变更。

本阶段关闭Run-owner本地确认+demand写入，不关闭Schedule owner的result Inbox、lease/generation/deadline/cancellation复验与link/confirmation producer、生产认证/角色配置、可靠Outbox交付、过期/取消/跨owner补偿、Host Manager/sandbox/readiness/native Hook求值/独立验证、Agent/任务卡CLI完整闭环及BI/Benchmark。仍无生产executor、无完整workspace/release/doc/bench或规模性能验收。

## §2 Verification summary

验证：隔离PostgreSQL18.6/Rust1.99.0 runner 56/56，binding/admission SQLx targets 2/2，库15/15及包级all-targets check、strict Clippy、fmt、Python编译、Cypher图头和diff检查通过。Audit `target/phase9f4d-run-owner-consumer/b0a6e67a-47c9-4f05-86ce-aad4ae1292a8/audit.jsonl`。真实Run-owner路径验证当前授权撤销/预留过期/quota禁用拒绝、最后receipt失败整笔回滚、写入后目录fence失效回滚、成功确认及同一demand精确重放、冲突resolution拒绝、跨tenant RLS、确认后D10补偿拒绝，以及demand deadline钳制至reservation lease；只有一条demand且零执行事件。权限/quota/时钟失效来源仍为隔离admin/replica fixtures；未证明真实Schedule producer、认证transport、跨进程竞争或最大规模性能。隔离PostgreSQL已停止，零Multipass inventory/生产DB变更。

Initial integration attempt failed because the expired-reservation test fixture set lease_expires_at before the row's created_at, violating an existing check constraint. Adjusted the fixture's created_at and restored both timestamps afterward; did not weaken the production guard. Audit `target/phase9f4d-run-owner-consumer/d4be2213-644c-4976-aa5f-044a9ddf19c9/audit.jsonl`. The next runner passed 56/56 (`7633ceef-1b18-439e-bb49-079dfd2972e3`), before the final quota/RLS and profile-version additions. Final evidence appears above.

## §3 Known gaps

Schedule's result/link/confirmation producer and real transport identity/delivery are not implemented by this slice. After demand, cancellation/expiry must use coordinated owner releases; D10 intentionally refuses local-only compensation. Native execution Hook/readiness/sandbox/independent validation, physical provider reservations and performance/product acceptance remain open. No whole-workspace/release/doc/bench claim. Previous CodeRabbit probe failed Permission denied, no independent review pass. RTK and matching graph tools unavailable; bounded source fallback. No new dependency or VM/production DB change.

## §4 Subagent failures and takeover

No subagent dispatched; root owns this lane. Preserve source/user worktrees and the dev index.

## §5 Guard rules

1. Strict bounded command.
2. Tenant and original service identity.
3. Accepted-result exact lineage.
4. Original command hash integrity.
5. Current authorization and binding.
6. Current policy and bounded catalogs.
7. Catalog fence before/after writes.
8. Occurrence deadline before/after writes.
9. Pending unexpired reservation only.
10. Current quota, no double counting.
11. Current Task metadata/contract/lifecycle.
12. No compensation/execution/demand already recorded.
13. Atomic confirmation/event/demand/receipt.
14. Deadline capped by Project reservation lease.
15. Replay is history, no executor/VM permission.

## §6 Signatures

| Role | Review |
|---|---|
| Architecture | Mavis review: confirmation then own demand, owner isolation. |
| SRE Lead | Mavis review: transaction rollback and bounded deadlines. |
| Platform | Mavis review: current gates and native constrained capability. |
| Review chair | Mavis review: honest isolated evidence and remaining producer gaps. |
| PM | Mavis review: full execution and product phases remain active. |

## §7 Revision history

| Version | Date | Reviewer | Change | Trigger |
|---|---|---|---|---|
| v0.1 | 2026-10-10 | Ulysses — Mavis review | Run-owner confirmation and demand writer. | Continue after D10 integration. |
