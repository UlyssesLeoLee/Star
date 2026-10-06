# PHASE-9F4D-0 — Infrastructure Binding Contract Report

> **报告版本**：v0.1 · 2026-10-06
> **状态**：🟡 当前 Rust 领域契约切片完成；Phase 9F4D 及 Schedule 生产执行仍开放
> **范围**：RunInfrastructureBinding / Host demand v1 typed contract 与其 focused quality gates
> **修订人**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

把用户确定的 Multipass/K3s 架构落实到 Run-facing 类型边界：Multipass 管理本地/CI Linux guest，K3s 运行于 guest 内；Run 只 pin 授权后的 provider/profile/binding 与资源预算，不携带 Multipass 命令、instance name、daemon credentials 或宿主路径。Run owner 与 Infrastructure owner 通过版本化 demand 消息异步交接，各自在本 owner 的事务中处理容量与事实。

本报告只验收 `domain-infrastructure` 的纯 Rust v1 值对象和 focused gates。它没有实现 Multipass Manager/provider、Run admission writer、owner persistence、worker 或生产执行闭环。

## §1 改动矩阵

| 区域 | 完成内容 | 状态与边界 |
|---|---|---|
| Rust contract | 新增 workspace crate `crates/domain-infrastructure`；定义 `ProviderKind::MultipassK3sGuest` / `RemoteK3s`、DNS label、resource budget、不可变 Run binding snapshot、Host demand、明确 schema version、全字段确定性 digest 与 `(tenant_id, occurrence_id)` 幂等键 | 🟢 类型/校验实现；无生产 caller，不代表 provider ready 或已授权 |
| Schedule / owner boundary | demand 固定 tenant/project/branch/Engineering Run、TaskExecutionRun、Rule revision、occurrence、binding、预算与 deadline；Run reservation 和 Host reservation 保持不同 owner | 🟡 契约已有；Directory ACL resolver、Run writer、双 owner Inbox/Outbox、Host capacity reservation 与 acknowledgement 未实现 |
| Multipass/K3s 决策 | Multipass 是本地/CI guest VM 的首选管理路径；远端 Linux K3s 是独立 provider；按 Host/environment 共享按需 VM/control plane，不按 Worktree/Agent 复制 | 🟡 需求和设计已定；Star 原生 Manager、平台 capability probe、lifecycle/readiness adapter 未实现 |
| 文档追溯 | 同步 requirements v5.77、basic design v5.74、Worktree Group DD v4.52、Schedule SRS/BD/DD v0.14、Infrastructure DD v0.10、实施计划 v6.05 §6.86、automation design v2.10 与 registry v0.60 | 🟢 当前设计均区分 typed contract、宿主 PoC 与未实现产品 provider |
| 可重放验证 | 新增 `scripts/automation/phase9f4d_infrastructure_contract.py`；stdout/stderr、JSONL audit 与 Cargo target 分开落在忽略目录；Windows 默认 Cargo target 使用较短路径 | 🟢 四项 focused gates 通过；不运行 DB、worker、VM、K3s 或 production admission |

## §2 验证摘要

| 门禁 | 命令/证据 | 结果 |
|---|---|---|
| Rust focused gates | `python scripts/automation/phase9f4d_infrastructure_contract.py --toolchain-bin C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin` | 🟢 4/4：rustfmt、locked all-target check、warnings-as-errors Clippy、locked all-target tests；8/8 tests 通过 |
| Runner 路径 | 默认路径 `target/9f4d-cargo-target`；日志与 audit 在 `target/phase9f4d-infrastructure-contract` | 🟢 默认入口复跑通过。此前更深 Cargo target 路径的 test link 报 LNK1104；显式短路径复跑及采用短路径后的默认入口均通过 |
| Automation registry | `python scripts/automation/registry_check.py` | 🟡 0 errors、197 warnings；仓库内未登记脚本的历史 warnings 仍在，新增 runner 已纳入 registry |
| Cypher header | `cgg_guard.py --changed --stamp` | 🟢 PASS，检查 2 个 source files |
| Diff whitespace | `git diff --check` | 🟢 无 whitespace error；Git 提示的 CRLF/LF 转换警告不属于 diff error |
| CodeRabbit | 本地 CodeRabbit shim 无法启动，返回 WSL `/bin/bash: /root/.local/bin/coderabbit: Permission denied` | ⚪ 未完成独立 CodeRabbit review；已做本地 source/diff 自审，工具失败不视为 review 通过 |

## §3 已知缺口

1. Run admission 尚未从 canonical Directory 当前读取 creator `run_as_actor_id` 权限；每次 trigger/retry/resume 的 Project/Run recheck、撤权竞态与 fail-closed writer 未实现。
2. `RunInfrastructureBinding` 尚无权威 resolver，也未原子写入 TaskExecutionRun snapshot；digest 只表示内容，不表示授权或真实性。
3. Infrastructure owner 的 Inbox、Host reservation ledger、Outbox、幂等 consumer、capacity acknowledgement 和清理/过期 worker 未实现。
4. Rust Host Infrastructure Manager 与受限 Multipass argv adapter 未实现；没有创建、启动、停止、删除 VM 或管理 K3s 的产品代码。
5. guest cloud-init/image/K3s digest、API/node/CRI probes、跨平台 Multipass driver capability 与用户自有远端集群 provider 尚未在 Star 内验收。
6. Worktree 路径授权 bridge、Agent Runtime sandbox、原生 Hook、取消/drain、独立验证与结果回写未闭环；K3s readiness 不代表代码执行 sandbox 安全。
7. 目标 DB schema/migrations、RLS/runtime role grants、双 owner 事务/回滚/idempotency、并行公平性、BI/Benchmark、桌面及 VM/K3s 峰值 RSS/CPU/IO 均未在本 slice 验证。
8. 当前只通过 `domain-infrastructure` focused gates；未声称全 workspace、全部 Group Apps phases 或 Schedule production capability 完成。Schedule 执行保持 fail closed。

## §4 子代理失败接手清单

本 slice 未派发子代理；因此没有子代理 RPC 或产出需要接手。后续即使启用 worker，也必须通过用户授权的 dispatch/verification 流程并验证真实输出，不能只依据 agent 状态标记。

## §5 守门规则（本阶段适用 16 项）

1. 主归属仍为 Project → Cloud Branch → Engineering Run；Worktree 是本地 checkout，不拥有 Run Apps。
2. 本地/CI K3s 运行在 Multipass 管理的 Linux guest；生产/远端 Linux K3s 使用独立 provider。
3. 默认按 Host/environment 共享 VM 与 control plane；不按 Worktree 或 Agent 创建 VM。
4. Run/Execution owner 只持有 Project/Run reservation；Infrastructure owner 单独持 Host reservation，不跨 owner 写表或共用业务事务。
5. 只有可信 Rust Infrastructure Manager 可以访问受限 Multipass adapter；Agent、Plugin、Schedule worker 和 Task CLI 不得访问 daemon/socket 或宿主 shell。
6. provider 调用须固定 argv allowlist、路径、deadline、输出上限、owner ledger 与幂等 operation；不拼接任意 shell。
7. binding 与 demand 带 schema version；未知版本、无效 scope、deadline 或预算一律拒绝。
8. digest 是内容指纹，不是权限、signature、readiness 或可信来源证明。
9. Schedule 规则创建者保持固定 run-as；每次 trigger/retry/resume 必须重验当前 Project/Run 权限。
10. Project/Run 执行预算与 Host 容量预算分开计量、分别预留、分别审计。
11. Host capacity 未确认前不得启动 executor；未知/不足容量必须拒绝或延迟。
12. Multipass daemon、VM、K3s API/node/CRI、Agent Runtime sandbox readiness 分层记录。
13. K3s namespace/RBAC/NetworkPolicy 不能替代独立 Runtime sandbox、native Hook 与验证门。
14. Worktree guest workspace 只通过授权、限路径、可审计的 bridge 提供，不能默认可写挂载整个用户目录。
15. 取消、drain、恢复、输出缓冲、并发和资源预算必须有界；未实测时不宣称性能达标。
16. 宿主 Multipass/GitOps PoC、Rust 类型或 UI 不能替代 Star provider、DB/RLS、真实任务执行及结果回写验收。

## §6 签字栏

| 角色 | 审核状态 | 边界说明 |
|---|---|---|
| 架构师 | 🟢 Mavis 接手审核 | 确认 Multipass/K3s 首选路径、双 owner 事务和 Run-facing contract 一致 |
| SRE Lead | 🟡 Mavis 接手审核 | 只确认容量/readiness/恢复缺口已写清；没有 SRE 运行容量或可靠性验收 |
| 平台 | 🟢 Mavis 接手审核 | focused crate gates 通过；Manager/provider 与宿主权限边界未实现 |
| 评审主持 | 🟡 Mavis 接手审核 | 做了本地自审；CodeRabbit WSL shim 未能运行，不能称为独立 review 通过 |
| PM | 🟢 Mavis 接手审核 | 本报告只关闭 9F4D-0 contract slice，不关闭 9F4D 或总 Phase |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-06 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 9F4D-0 报告；记录 Multipass 管理本地/CI guest K3s 的架构决定、Rust v1 类型契约、验证结果和未关闭的 owner/provider/执行门 | Rust `domain-infrastructure` 首个 binding/demand slice 完成定向验证 |
