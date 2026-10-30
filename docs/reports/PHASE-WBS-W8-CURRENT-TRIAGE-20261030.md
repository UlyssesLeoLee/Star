# PHASE-WBS-W8-CURRENT-TRIAGE-20261030

## §0 目的

基于当前 `dev@313e5741fb0dc557874e5a37bc05e8839211cd90`，只读盘点三个既有 W8 候选的 HEAD、dirty/untracked 状态与 source-position/API 依赖，判定可安全摘取内容，澄清 WBS 过时状态，并给出下一步实现门禁。本阶段不修改源代码、WBS 或候选树，不执行任何测试。

## §1 改动矩阵

| 项目 | 结果 | 说明 |
|---|---|---|
| 当前 dev Inbox | 已核对 | `bi.commit_task_event_v2`；文件 SHA-256=`ACB299042F4994AC24BC5E9707A7449EEA85E0FCEB1C077BC87220DC74C2602C`。 |
| transport remediation 候选 | 已核对 | clean，HEAD `0ae310440e446f530a210ec413efb526cb038d8f`；分支有一个候选独有提交，涉及 v3 API 与 migration 依赖，未建议集成。 |
| detached native adapter 候选 | 已核对 | HEAD `b660fa52a23d352539123eafc9ef6b005a978d71`；7 个 tracked 修改、11 个 untracked Git 路径，未触碰。 |
| source-position claim 候选 | 已核对 | HEAD `bd90aecc1c39bdf7e647cd9768d36d7db1d55d0c`；Inbox 与 PG test 两个 tracked 修改、decode brief 一个 untracked 路径，未触碰。 |
| WBS | 只读对账 | 引用主 WBS §14.1、§20.1 和 §23.1 的确切行号；当前候选状态与历史文字不完全相同，建议另开 lane 更新 WBS。 |
| 文件 | 两份新增审查产物 | 本报告与 `docs/reviews/WBS-W8-CURRENT-TRIAGE-20261030.md`；不修改 WBS 或 source。 |

详细候选路径、散列、Keep/Adapt/Defer/Reject 与依赖理由见审查报告。没有候选文件进入本次修改集。

## §2 验证摘要

- **只读状态/差异复核**：三个候选均重新读取 `git status --short --branch` 与 `git rev-parse HEAD`；candidate-only commit 范围与关键 dirty paths 依据本 brief 指定候选记录。
- **内容定位**：确认当前 dev Inbox 调用 `bi.commit_task_event_v2`；候选 transport migration 声明 `bi.commit_task_event_v3` 并返回 `source_sequence`；source-position migration 定义共享 source position claim 表与触发器。
- **哈希**：对当前 dev Inbox、候选 Inbox/adapter/migrations/PG test/brief/详细设计执行 SHA-256，结果见审查报告。
- **WBS**：读取 `docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` 的 §14.1 行 436–448、§20.1 行 557–562 与 §23.1 行 631–638。
- **Cargo / Rust fmt / tests / clippy / PostgreSQL / RLS / mTLS / recovery / settlement / E2E**：**NOT_RUN**（只读审查 brief 明确禁止运行）。不得将旧报告记载的运行结果外推为本阶段通过。
- **W8 阶段门**：OPEN / FAILED-TEST；本次 audit 不解除失败状态，也不授权 rebase、merge、push 或候选清理。

## §3 已知缺口

1. 当前 dev v2 API 和候选 v3 migration/return schema 不匹配；尚无获批的兼容版本化迁移设计。
2. SQLx `source_sequence` 解码补丁依赖 v3 返回列及 shared-claim migration；只复制查询改动会造成 schema/API 不一致。
3. PG/RLS、目标应用角色 grants、并发双向冲突、隔离、exact replay、接收 ACK/quarantine 终态与恢复尚未在本阶段复测。
4. transport lane 和 detached adapter 中存在跨 API、migration、Cargo 和多文件测试的依赖；不能按单行或未经审查整支集成。
5. §14.1/§20.1 的旧 fingerprint、状态日期与“等待旧 lane”的文字属于历史记录；WBS 当前摘要需后续独立更新。
6. 本阶段未确认候选工作树 owner；只读状态不能证明没有其他未显示的外部使用者。

## §4 子代理失败接手清单

N/A：本 lane 未派生子代理；没有子代理失败需要接手。

## §5 守门规则

1. 只读审计不得改变候选树的 index、工作区或 ref。
2. 不覆盖、reset、clean、提交、删除或归档候选分支/工作树。
3. 不访问或更改 `D:\Star` 用户工作树。
4. 本 lane 不编辑 WBS 或任何产品源代码。
5. 本 lane 不运行 Cargo、Rust、数据库、mTLS、runtime 或 E2E 命令。
6. 不将 git clean 状态解读为 owner 已释放写入权。
7. 不将静态审查解读为 PostgreSQL/RLS/runtime 验收。
8. 不将数据库 migration apply 成功解读为并发、权限或 ACK 验收。
9. 不把 `source_sequence` 单行 SQLx 改动从 v3 候选摘入 v2 API。
10. 任何 v2→v3 变更必须版本化 schema/API/迁移，并定义兼容部署顺序。
11. Inbox 与 quarantine 的 source position 仲裁必须在同一持久 owner 契约内唯一。
12. 候选集成前需审查完整依赖闭包、Cargo 锁定、migration 权限、回执/幂等语义及失败恢复。
13. 持久化接收只有在 owner 事务提交后才能确认；传输 ACK 不能早于 durable commit。
14. PG 验收需使用目标应用角色验证 RLS/grants，superuser 单测不能代替。
15. 并发、exact replay、ACK/quarantine 终态、断线/重启恢复必须有真实可复现验证。
16. 仅在来源 owner/manifest 清晰、独立 review 通过、验证完整后，协调者可重新评估 rebase/快进。
17. W8 在所有适用门通过前保持 OPEN；本阶段不授权 merge、push、分支清理或产品闭环声明。

## §6 签字栏

| 角色 | 结果 |
|---|---|
| 架构 | 🟢 Mavis 接手终审：保留 v2 baseline；拒绝无依赖闭包的 query-only cherry-pick。 |
| SRE Lead | 审查已完成；本阶段未运行 PostgreSQL、mTLS、恢复或 settlement 验收。 |
| 平台 | 审查已完成；transport adapter 与 owner 持久事实边界需在实现 lane 中复核。 |
| 评审主持 | 🟢 Mavis 接手终审；候选状态与限制见 §2/§3。 |
| PM | 本阶段交付只读候选裁定；W8 未关闭，后续实现与运行验收仍需排期。 |

**修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。
**审批者**：架构师 (Mavis 接手 agent per DEC-008)。

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 对三个 W8 候选进行只读状态与 API 依赖分诊；指出 v2/v3 不匹配，保留 W8 OPEN/FAILED-TEST 并给出最小后续 lane 条件。 | 用户要求并行推进、验收后再合并到 dev；本子任务限定只读候选审计。 |
