# PHASE-9F4D-1 — Multipass/K3s Readiness Gate Report

> **报告版本**：v0.1 · 2026-10-06
> **状态**：🟢 typed readiness gate 与 focused Rust gates 完成；Phase 9F4D 生产 admission/provider 仍开放并 fail-closed
> **范围**：Host reservation acceptance、Manager receipt 与 Multipass/K3s/runtime/workspace readiness evidence
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> **架构更正（2026-10-07）**：本报告记录的是 V1/单 guest readiness 切片当时的验证快照；当前产品设计已更正为 Multipass 多实例/多 K3s cluster、Run 精确 pin cluster ID/generation、Host+cluster 双层容量，见 [`PHASE-9F4D-MULTICLUSTER-CAPACITY-REPORT.md`](PHASE-9F4D-MULTICLUSTER-CAPACITY-REPORT.md)。本报告不再代表当前 wire schema 或拓扑基线。

## §0 目的

把用户确定的 Multipass 架构落实到 Schedule executor 准入前的 Rust 领域证据校验：Multipass 管理本地/CI Linux guest，K3s 在 guest 内运行；Manager daemon、owned VM、K3s、CRI、Agent Runtime sandbox 与 Worktree workspace bridge 是独立 readiness 层。Remote K3s 是单独 provider，并显式标记不适用的 Multipass 层。

本 slice 只验证版本化 receipt/evidence 的范围、期限、新鲜度和 readiness 状态。它不授予 run-as 权限、不读取数据库、不创建 VM/K3s、不启用 worker 或 executor。

## §1 改动矩阵

| 区域 | 完成内容 | 状态与边界 |
|---|---|---|
| Rust contract | 新增 InfrastructureReadinessStateV1、HostCapacityReservationAcceptanceV1、InfrastructureReadinessEvidenceV1 与 validate_infrastructure_execution_readiness | 🟢 精确 demand/occurrence/Run/binding/digest/reservation/Host generation 校验；仅为纯领域 gate |
| Multipass/K3s readiness | 绑定 Manager operation receipt；Multipass provider 要求 daemon 与精确 owned guest VM Ready；Remote K3s 要求这两层显式 NotApplicable；两者均要求 K3s API、profile 最低 Ready node 数、CRI、独立 sandbox、Worktree bridge Ready | 🟢 分层且 fail-closed；没有 provider evidence producer 或真实 VM/K3s probe |
| 时间和租约 | Reservation lease 与 admission deadline 必须有效；readiness 不得早于 acceptance 或晚于当前时间，最大年龄 30 秒 | 🟢 typed validator 拒绝过期、未来、错代或 scope 不一致证据 |
| 身份和授权边界 | Run owner 在 start/resume transaction 仍须重验固定 Rule creator run-as、实时 Project/Run ACL、occurrence lease/fence、quota 与当前 binding | 🟡 validator 不替代任何授权或 owner transaction |
| 文档追溯 | 同步 requirements v5.81、basic design v5.78、Group DD v4.56、Schedule SRS/BD/DD v0.15、Infrastructure DD v0.11、实施计划 v6.09、Automation Design v2.11 与 registry v0.61 | 🟢 文档保留 provider/store/worker/生产执行未完成状态 |

## §2 验证摘要

| 门禁 | 命令/证据 | 结果 |
|---|---|---|
| Rust focused gates | phase9f4d_infrastructure_contract.py；Rust 1.98.1；短 target C:\t\star9f4d-20261006 | 🟢 4/4：rustfmt 检查 lib.rs 与 readiness.rs、locked all-target check、warnings-as-errors Clippy、locked all-target tests |
| Domain tests | domain-infrastructure all-target tests | 🟢 15/15；其中 readiness 模块 7 项，覆盖 binding drift、scope/generation、reservation expiry、provider-specific NotApplicable、每层 readiness 与 stale/future evidence |
| Windows linker | 首次 worktree-local target 在测试链接阶段报 MSVC LNK1104；短 target 路径复跑完整 runner | 🟢 短路径复跑四项 gates 全通过；首次失败保留为路径环境证据，不是 Rust 诊断 |
| Automation registry | python scripts/automation/registry_check.py | 🟢 exit 0；0 errors；197 existing unregistered-script warnings |
| Cypher header | cgg_guard.py --changed --stamp | 🟢 PASS；3 source files checked |
| Diff whitespace | git diff --check | 🟢 no whitespace errors；CRLF conversion warning only |
| 独立 CodeRabbit review | 当前会话没有可用的 CodeRabbit review 工具；本轮执行本地 source/diff 自审 | ⚪ 未完成独立 CodeRabbit review |

## §3 已知缺口

1. 尚无 Infrastructure owner acceptance/readiness 持久化、Inbox、Host reservation ledger、Outbox consumer 或过期清理 worker。
2. Rust Host Infrastructure Manager、Multipass argv adapter、VM owner ledger、固定 image/cloud-init/K3s profile 与 capability probe 尚未实现。
3. 尚无可信 provider evidence producer；当前测试只调用 Rust validator，未启动 Multipass daemon、Linux guest 或 K3s。
4. Run admission/start writer 尚未原子写入 Project/Run reservation、TaskExecutionRun、RunEvent 与 Outbox；Run-as ACL/lease/quota/current binding 仍须由未来 owner transaction 复验。
5. 目标 PostgreSQL migration、RLS/runtime grants、双 owner rollback/idempotency、BI/Benchmark 与 Schedule worker 未在本 slice 验收。
6. Agent Runtime sandbox、原生 Hook、Worktree workspace bridge 的生产实现、取消/drain/恢复和结果回写未闭环。
7. 桌面与 VM/K3s 的 RSS/CPU/IO、1–8 Agent 并行公平、冷启动和故障恢复均未测；不据此声称性能目标完成。
8. 本 slice 只关闭 9F4D-1 evidence validator；总 Phase、Schedule Loop 与产品执行能力继续 fail closed。

## §4 子代理失败接手清单

本阶段未派发子代理；因此没有子代理 RPC 或输出需要接手。

## §5 守门规则（本阶段适用 16 项）

1. Project → Cloud Branch → Engineering Run → 本地 Worktree 的归属保持不变；Run 是 Schedule 与 Run Apps 的业务边界。
2. 本地/CI 首选 Multipass 管理 Linux guest，K3s 在 guest 内运行；生产/远端 K3s 走独立 provider。
3. 按 Host/environment 共享 VM/control plane，不按 Worktree 或 Agent 复制 VM。
4. Run owner 只预留 Project/Run capacity；Infrastructure owner 独立预留 Host capacity。
5. 禁止跨 owner 直接写表或使用分布式数据库事务。
6. 只有可信 Rust Infrastructure Manager 可以调用受限 Multipass adapter。
7. Agent、Plugin、Schedule worker、Task Card CLI 不得访问 Multipass daemon/socket、宿主 shell 或任意 guest exec。
8. provider 输入固定为类型化参数、profile、allowlist argv、deadline 和有界输出；拒绝任意 shell 拼接。
9. Reservation/evidence 必须匹配 tenant、Project、Branch、Engineering Run、TaskExecutionRun、Rule revision、occurrence、binding digest、Host generation 和 reservation。
10. Host reservation lease 与 admission deadline 失效后必须 fail closed。
11. Readiness evidence 不得早于 acceptance、不得来自未来，且最多新鲜 30 秒。
12. Multipass path 分别检查 Manager receipt、daemon 与 exact owned guest；daemon wait-ready 不代表 VM 或 K3s ready。
13. Remote K3s 只允许将 Multipass daemon/guest 层显式标成 NotApplicable；其他 readiness 层仍必需。
14. K3s API/node/CRI readiness 不替代独立 Runtime sandbox、native Hook 或验证门。
15. Run start/resume 前重新验证固定 run-as ACL、current binding、lease/fence 与 Project/Run quota。
16. typed contract 或宿主 PoC 不得标成生产 provider、数据库、worker、Agent 执行或性能验收。

## §6 签字栏

| 角色 | 审核状态 | 边界说明 |
|---|---|---|
| 架构师 | 🟢 Mavis 接手审核 | 确认 Multipass/K3s readiness 分层与双 owner 边界一致 |
| SRE Lead | 🟡 Mavis 接手审核 | 只确认运行缺口已登记；未作容量/可靠性生产验收 |
| 平台 | 🟢 Mavis 接手审核 | Rust focused gates 通过；Manager/provider 尚未实现 |
| 评审主持 | 🟡 Mavis 接手审核 | 已完成本地自审；没有独立 CodeRabbit review |
| PM | 🟢 Mavis 接手审核 | 只关闭 9F4D-1 typed readiness slice，不关闭总 Phase |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 Phase 9F4D-1 报告；记录 Multipass daemon/guest、K3s/runtime/workspace readiness validator、Rust 1.98.1 四 gates 与 15/15 tests，以及尚未实现的 provider/owner/worker 边界 | 将用户选定的 Multipass 管理 guest K3s 架构落实为可验证的执行前合同 |
