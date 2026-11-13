# W8 当前候选只读分诊（2026-10-30）

## 范围与基线

本报告只审阅当前本地 `dev@313e5741fb0dc557874e5a37bc05e8839211cd90` 与 brief 指定的三处候选快照。候选均保持只读；没有对候选分支、工作树或 `D:\Star` 执行写入。报告是集成建议，不代替运行验收。

## 候选状态与证据

| 候选 | HEAD / 分支 | 状态 | 差异与证据 |
|---|---|---|---|
| `C:\Users\leo19\.codex\worktrees\wbs-w8-transport-remediation\Star` | `0ae310440e446f530a210ec413efb526cb038d8f` / `codex/wbs-w8-transport-remediation-20261025` | clean | 相对当前 dev 的共同祖先 `32d9a3ad727ca81c51adc3ef8e1a29df79d00b10`，候选侧有 1 个独有提交 `0ae31044`。其改动跨 Cargo 锁文件、adapter、BI Inbox/receiver/quarantine、migration、PG 测试和阶段文档，不能视作单文件修复。adapter `src/lib.rs` SHA-256=`0FA2854A13E6483412C3A0A72E01E2CFD6B8DFA994659295D6BA5454E73FE06B`；source-origin migration SHA-256=`58A49E4B71E5057CE3FF62DEFA44C6ADB3B892001FA792F4FECF6CD334F90F1A`。|
| `C:\Users\leo19\.codex\worktrees\wbs-w8-bi-native-adapter-20261024\Star` | `b660fa52a23d352539123eafc9ef6b005a978d71` / detached HEAD | dirty | 已提交 HEAD 是当前 dev 的祖先，没有候选侧独有提交；有效改动均未提交。7 个 tracked 修改：`Cargo.lock`、`Cargo.toml`、`crates/bi-owner-runtime/src/task_event_inbox.rs`、`crates/bi-owner-runtime/src/task_event_inbox_postgres_tests.rs`、`crates/bi-owner-runtime/src/task_event_quarantine.rs`、`crates/bi-owner-runtime/src/task_event_receiver.rs`、`crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs`。12 个 untracked status 项（adapter 目录由 Git 折叠显示；展开后为 13 个 untracked 文件）：`crates/bi-native-event-adapter/`、`db/migrations/2026-10-24-bi-task-event-source-origin.sql`、`db/migrations/2026-10-24-platform-native-transport-lease-order.sql`、briefs `WBS-W8-BI-NATIVE-ADAPTER-20261024.md` / `WBS-W8-FINAL-PARENT-REVIEW-20261024.md` / `WBS-W8-INDEPENDENT-REVIEW-20261024-C.md` / `WBS-W8-INDEPENDENT-REVIEW-20261024.md` / `WBS-W8-LEASE-ORDER-FIX-20261024.md` / `WBS-W8-PARENT-REVIEW-20261024.md`、`docs/design/DD-BI-NATIVE-RECEIVER-001.md`、`docs/reports/PHASE-WBS-W8-BI-NATIVE-ADAPTER-20261024.md`、`docs/reports/WBS-DISPATCH-W8-INDEPENDENT-REVIEW-20261024.jsonl`。tracked diff 汇总为 `Cargo.lock +12`、`Cargo.toml +2`、Inbox `+196/-7`、Inbox PG tests `+61/-10`、quarantine `+87/-9`、receiver `+40/-93`、native-origin PG test `+109/-62`。Inbox SHA-256=`F9A3447259955C846955E1654397DECE538762884D223F0999C4023510E8435E`；detailed design SHA-256=`7497784644591DA05E70F9AE3EA794DC2A55E01191DDA77519F834E4FC020627`。|
| `D:\orcaWork\Star\w8-source-position-claim-fix-20261025` | `bd90aecc1c39bdf7e647cd9768d36d7db1d55d0c` / `UlyssesLeoLee/w8-source-position-claim-fix-20261025` | dirty | 2 个 tracked 修改：`crates/bi-owner-runtime/src/task_event_inbox.rs`、`crates/bi-owner-runtime/tests/source_position_arbitration_pg.rs`；1 个 untracked：`docs/briefs/WBS-W8-DECODE-FIX-20261026.md`。branch 相对共同祖先 `32d9a3ad727ca81c51adc3ef8e1a29df79d00b10` 有 7 个候选提交：`0ae31044`、`07a896e3`、`fd0472ff`、`7f96a066`、`824c76d0`、`cd89b894`、`bd90aecc`。脏 Inbox 改为从返回行读取 `source_sequence`，PG 断言期望 source origin sequence 为 17；二者均依赖 source-origin v3 返回契约与共享 position claim migration。Inbox SHA-256=`BB69349BD00BB60F0E752D42EA0713F089BD1E26ACDC0A073D74FEEB8A4352DA`；PG test SHA-256=`B513882E0FCC90930CE0E5FF2A27419C0C9D1DA9911BEC2A781342690DE2686E`；shared-claim migration SHA-256=`3E8DC424458E3FDADE98D21175EE214C85D63C8441A5A0A84E61170561771FE0`；untracked decode brief SHA-256=`EAF8CC7F2E10590CAF2A0564A2DD878930289D8ED778608A181D1FBD06C06EE9`。|

## 当前 dev 契约与 API 依赖

当前 dev 的 `crates/bi-owner-runtime/src/task_event_inbox.rs` 使用 `bi.commit_task_event_v2(...)`，完整文件 SHA-256=`ACB299042F4994AC24BC5E9707A7449EEA85E0FCEB1C077BC87220DC74C2602C`。当前 `commit_task_event_v2` API 不接收候选 source-origin 参数，Inbox decoder 也未读取候选 v3 的 `source_sequence` 返回字段。transport candidate 的 Inbox 改为调用 `bi.commit_task_event_v3(...)`；`2026-10-24-bi-task-event-source-origin.sql` 定义该函数并返回 `source_sequence`。source-position candidate 再依赖 `2026-10-25-bi-task-event-shared-position-claim.sql` 的共享 Inbox/quarantine claim 表、触发器与 owner 权限。

因此，PG 失败中“把 `source_sequence` 加到 SQLx 解码”是候选 v3 契约的一部分，不能孤立地拷到当前 v2 实现。仅改查询而不引入并审查匹配的函数返回 schema、source identity 写入和 shared-claim migration，会留下 API/schema 不一致；对当前 v2 的真实解码缺字段修复，必须先以当前迁移和返回类型重新定界。

## Keep / Adapt / Defer / Reject

| 判定 | 内容 | 理由 / 下一步 |
|---|---|---|
| Keep | 当前 dev 的 v2 Inbox API 与其已提交数据契约 | 它是唯一与基线匹配的可审阅实现；继续保持执行 fail-closed，不把候选视为已集成。 |
| Adapt | source position 必须跨 Inbox 与 quarantine 共用唯一性仲裁的设计目标 | 先形成与当前 owner、API 版本、schema、迁移次序和权限边界匹配的独立设计，再实现；必要时显式设计 v3 迁移，禁止只复制 SQLx 一行。 |
| Defer | `0ae31044` transport adapter 整组、detached adapter dirty patch、source-position 分支提交及脏补丁 | 依赖链跨 Cargo、API、DB、权限、Inbox/quarantine 与测试；旧报告还记录候选数据库授权/历史 source position 风险。待来源 owner/claim 明确、候选经复核、版本化迁移对齐且 Rust/真实目标角色 PostgreSQL 门禁通过后再挑选。 |
| Reject | 只 cherry-pick 或手工复制 `source_sequence` 查询/解码行 | 它依赖 `commit_task_event_v3` 的返回列；current dev 使用 v2。单行修改不可构成正确修复。 |
| Keep as historical evidence | 本报告所引用的既有 W8 报告与 WBS 历史记录 | 保留失败、未运行和候选隔离证据；不把静态审查或 migration apply 成功改写为完整验收。 |

## WBS 对账与过时陈述

主 WBS 为 `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md`：

- §14.1 行 436 记录 source-position lane `bd90aec`、旧失败和旧 MAGOS fingerprint `b61e89c47f61`；行 444 与 §20.1 行 561 仍记录旧阻塞状态；其中 §20.1 行 561 记有旧 fingerprint `52c321488e77`，行 562 仍写着“先等占用 lane 被审查后集成或明确释放”。这些值和候选快照日期是历史状态，不能用作本次当前门禁或授权；本报告提供当前 HEAD/status/API 复核，但不改写 WBS 历史。
- §20.1 行 559 的 SQLx decode 失败和后续场景未运行仍是有效历史证据；行 560 的“候选仅增加 SELECT 和 PG 回归断言”需要补充：该 dirty 候选现可确认其依赖完整 v3 source-origin API 与 shared-claim migration，不能单独摘取。W8 仍 OPEN/FAILED-TEST。
- §23.1 行 631–638 是 W11 Multipass/Run infrastructure 与复用来源差距，不包含 W8 状态。本轮没有把 W8 内容挪入该节，也不以其任何项目作为 W8 验收证据。

后续 WBS 应新增一个带本次审计日期的当前状态条目，引用本报告并列出当前 v2 baseline、v3 dependency gap、未运行验证门和新的 owner/manifest 复核前置条件。该 WBS 编辑不在本 lane manifest 内，需由协调者在后续授权 lane 完成。

## 可执行的下一步

1. 协调者确认各候选的 owner/manifest 与可写状态；当前审计没有取得改动这些候选的授权。
2. 以最新 dev 单独登记实现 lane，先决定保持 v2 并修复其真实解码问题，或版本化迁移至 v3；manifest 同时涵盖 Rust API、schema/migration、Inbox/quarantine 仲裁和回归测试。
3. 通过 Rust 1.99 定向格式/编译门及真实 PostgreSQL 角色/RLS/grant、source-position 双向冲突、并发、exact replay、ACK/终态和恢复验证；仅 disposable superuser 或静态门不得替代目标角色门。
4. 独立 reviewer 复核 accepted HEAD、migration 向前兼容与回滚/恢复策略；通过后协调者再决定 rebase/快进到 dev。此报告不授权 merge、push、candidate 清理或关闭 W8。

## 限制

本审计仅复核了已列出的三个候选树、对应 Git 状态/差异、指定源文件与 migration 哈希，以及主 WBS 对应段落。没有审阅整个仓库、没有取得数据库运行日志的本轮复测、没有验证 Orca 活动 owner，也没有运行 Cargo、测试、PostgreSQL、mTLS、restart/recovery、settlement 或端到端任务。CodeGraph 对当前 dev 的父级证据显示 Inbox 使用 v2；候选 worktree 没有本地 `.codegraph` 索引。任何未在本报告列出的源码语义都不作穷尽性结论。
