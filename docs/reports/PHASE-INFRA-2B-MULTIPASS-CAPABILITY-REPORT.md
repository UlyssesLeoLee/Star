# PHASE-INFRA-2B — Multipass read-only capability facts

> 日期：2026-10-08
> 状态：🟢 本切片实现已合入本地 `dev`；INFRA-2 后续阶段仍开放
> 修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

落实用户选择的 Rust 1.99.0 workspace/toolchain 基线，在 INFRA-2A inventory adapter 上增加受界限的 Multipass CLI/daemon 版本、`local.driver` 与 Rust host target 事实探测。该切片保持只读，不启动 VM，不建立 Infrastructure owner/Manager authority，也不开放 Schedule 或 Task Card CLI 执行。

Rust 1.99.0 已于 2026-10-01 正式发布：[Rust 1.99.0 announcement](https://blog.rust-lang.org/2026/10/01/Rust-1.99.0/)；当前 `Cargo.toml` 的 workspace `rust-version` 与 `rust-toolchain.toml` 均为 1.99.0。

## §1 改动矩阵

| 范围 | 结果 | 边界 |
|---|---|---|
| Rust adapter | 增加 `MultipassCapabilityProbe` 和 `MultipassCapabilityObservation`；固定 `version`、`get local.driver` 两条只读 argv；记录 client/daemon version、driver、host OS/arch。 | 共用 inventory 的 absolute executable 与 fail-fast single-flight；每条命令 deadline 5 秒，stdout/stderr 各 4 KiB，串行 command budget ≤10 秒。 |
| 输出解析 | 要求 `multipass` 与 `multipassd` 各出现且只出现一次；driver 仅接受一行 bounded token，未知值原样保留。 | 不判定版本兼容性、driver 可用性、daemon readiness 或任何授权/容量事实。 |
| 测试 | 增加固定 argv/限额、版本格式、重复/缺失/畸形版本、未知 driver、畸形 driver 与 capability/inventory 共用 single-flight 测试。 | 所有测试为本地 fixtures，不运行 Multipass，不创建或修改 VM。 |
| 文档 | 同步 requirements v6.01、basic design v5.98、Infrastructure DD v0.27、Group DD v4.67、implementation plan v6.32，并新增本阶段报告。 | 无 DB migration、W/T/M 分类或产品导航改变。 |
| dev integration | No-FF 合入本地 `dev`：`24a4e94219d71fc41059ec8c013abe584e793402`；merge source 为 `44c4de6e4e51f65910569f5d162583611336440f`。 | 未 push；其余 20 个非 main 源分支已 contained，main 是 dev 祖先并保持不动。所有 21 个非 main source worktrees 都有 active-owner blocker，故未删除任何 branch/worktree。 |

## §2 验证摘要

以下定向命令均使用 Rust/Cargo 1.99.0、offline registry 与隔离 target directory `C:\t\star-infra2b-rust199-target`：

| 门禁 | 实测结果 |
|---|---|
| `cargo test --locked --offline -p multipass-adapter --lib -j 4` | 17 passed, 0 failed |
| `cargo clippy --locked --offline -p multipass-adapter --all-targets -j 4 -- -D warnings` | 通过，0 warning/error |
| `rustfmt --edition 2021 --check crates/multipass-adapter/src/lib.rs` | Rust 1.99.0 formatter 检查通过 |
| `git diff --check` | 通过 |
| `cgg_guard.py --root . --changed --stamp` | 通过；1 个 source file checked |
| Post-merge rerun on `dev` | `cargo test` 17/17、严格 Clippy、rustfmt 与 `git diff --check` 全部通过 |
| CodeRabbit review | 未能运行：本机 CLI 启动返回 `/root/.local/bin/coderabbit: Permission denied`；已完成本地 diff/source 自审，不将其视为独立审查通过 |

本报告不声称 workspace-wide build/test、生产 Multipass daemon/driver、跨平台 provider、虚拟机/K3s readiness、桌面 RSS 或性能测试通过。

## §3 已知缺口

1. 当前 probe 未在真实 Multipass CLI/daemon 上运行；CLI binary 解析与实际环境版本/driver 匹配仍未验收。
2. driver、client version 与 daemon version 是原始观察。兼容矩阵和 provider capability resolver 尚未实现；未知 driver 不能据此启动 guest。
3. HostEnvironment/instance/cluster/node ownership ledger、RunInfrastructureBinding resolver、Host/cluster 容量核算与 reservation、可信 Manager IPC/receipt、readiness producer、生命周期 UI 与目标 DB/RLS 均未完成。
4. 本切片未验证 Multipass Windows/Linux/macOS driver 差异、guest/K3s/CRI、Agent Runtime sandbox、Worktree bridge、取消恢复及 Hook/BI 联动。
5. 未运行 workspace-wide gates、RSS、UI responsiveness、公平并行或资源背压测量。
6. Multipass 测试阶段两条 inventory 记录（包含 Stopped/Deleted）的磁盘上限由已有受控 test/bootstrap guard 管理；本 adapter 不执行或替代该 guard。此切片创建 VM 数为 0。
7. Schedule/Task Card CLI 与 unattended Run execution 保持 fail-closed。Platform quarantine retention/fault taxonomy 的独立 9F4D-3D6 blocker 不由本阶段关闭。

## §4 子代理失败接手清单

本切片未派发子代理，因此没有子代理提交或 RPC 结果需要接手。实现、文档同步、定向验证与当前变更自审由本任务完成。

## §5 守门规则

| # | 守门规则 | 本阶段结果 |
|---:|---|---|
| 1 | 只允许 `version` 与 `get local.driver` 固定 argv | 已实现并有测试 |
| 2 | 可执行路径必须是受信绝对路径 | 复用 2A 构造门 |
| 3 | 禁止任意 argv、shell 与用户可控命令 | 已实现 |
| 4 | inventory/capability 共用 process-wide single-flight | 已实现并有跨 probe 测试 |
| 5 | 并发请求 fail-fast，不堆积等待队列 | 已实现 |
| 6 | 单命令最长 5 秒，probe 最多顺序执行两条命令 | 已实现 |
| 7 | 单条 stdout/stderr 各≤4 KiB | 已实现 |
| 8 | Malformed、duplicate、missing、oversized 或 non-UTF-8 输出必须 fail closed | parser 已实现并有负例测试 |
| 9 | 不保留原始 stdout/stderr 作为错误或日志 | 错误不包含原始输出 |
| 10 | 未知 driver 必须原样保留，不推断支持性 | 已实现并有测试 |
| 11 | version 不一致不由 probe 自动解释为兼容或不兼容 | 已实现并以不一致 fixture 验证 |
| 12 | OS/arch 是 target fact，不得伪装为 provider health | 仅作为观察字段输出 |
| 13 | 不执行配置写入、安装、VM 生命周期、mount 或 guest command | 固定命令列表无写操作 |
| 14 | 不推导 ownership、容量、readiness、授权或执行资格 | 需求/基本/详细设计均明确禁止 |
| 15 | fixture 验证必须不创建 VM | 17 项本地测试；本阶段未运行 Multipass |
| 16 | 测试阶段 inventory 仍最多两条 registered records，含 Stopped/Deleted | 未改动受控 bootstrap guard 或该上限 |
| 17 | Schedule/Task CLI 执行继续保持关闭，直至后续 owner/provider/runtime 门通过 | 保持 fail-closed |

## §6 签字栏

以下为本任务同一 Mavis 审阅者对五个职责视角的检查记录，不代表五名独立 reviewer：

| 职责 | 状态 |
|---|---|
| 架构 | Mavis 接手审核：只读 observation boundary 与 Run/Host owner 责任保持分离 |
| SRE Lead | Mavis 接手审核：命令/输出/deadline/concurrency bounds 明确；无真实 daemon/性能验收 |
| 平台 | Mavis 接手审核：Rust 1.99.0 toolchain 与 package gates 定向通过；全 workspace 未验收 |
| 评审主持 | Mavis 接手审核：需求 v6.01、基本设计 v5.98、Infrastructure DD v0.27、Group DD v4.67、计划 v6.31 与本报告版本/范围交叉核对；Rust 1.99.0 适用范围与 fail-closed 边界一致 |
| PM | Mavis 接手审核：本切片范围有限，INFRA-2 剩余事项与执行关闭状态明确 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 INFRA-2B 只读 Multipass capability-facts 实现、Rust 1.99.0 定向验证、跨层文档同步与未验收边界；初始验证占位项须在阶段收口前替换为实测结果。 | 用户选择 Rust 1.99.0，继续推进 Multipass/K3s 基础设施架构。 |
| v0.2 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 以实测 Rust 1.99.0 结果替换 rustfmt/CypherGraph 待执行项；记录 CodeRabbit CLI 权限错误与本地自审边界；同步实现计划验证状态，并核对需求/基本/详细设计及计划版本链。 | 合并前差异审查发现阶段报告仍保留过时的 pending 验证项。 |
| v0.3 | 2026-10-08 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 INFRA-2B no-FF 合入本地 `dev` 的 merge commit 与 source commit；确认 post-merge Rust 1.99.0 focused tests、Clippy、rustfmt、diff-check 通过；未 push，active-owner worktrees 保留。 | 用户要求将完成结果合入 dev 并继续后续阶段。 |
