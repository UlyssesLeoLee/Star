# PHASE-9F4C-D20-NATIVE-RECEIVER-REPORT

> **Status**: D20 native receive/origin-proof、Run-owner Schedule admission/Result、Infrastructure demand receive 切片已合并到本地 `dev`；D20 origin migration/SQL contract 已在 disposable PostgreSQL 18.6 通过 83/83 runner steps；D20 整体进行中，生产接入未开放.
> **Initial native receiver slice**: `C:/Users/leo19/.codex/worktrees/phase9f4c-d20-consumers/Star`, branch `codex/phase9f4c-d20-native-receiver-20261012`, base `501c6cb5209a0f87df971cd0907a49f448118982`.
> **Run-owner composition slice**: branch `codex/d20-native-schedule-admission-20261013` at `C:/Users/leo19/.codex/worktrees/phase9f4c-d20-consumer-native/Star`, rebased onto `dev` at `4ffb0b9dc7d8bdcf0ec23c181f93c19a857167e6` and merged by `78c7211ac7d08011db5adcaa9c378fab38b28193`. After bounded manual review, its clean worktree was archived by Codex (archive artifact `01a135a5-85ba-7dc1-b8b6-76af953174cd`) and the local source branch was safely deleted. Post-merge Rust 1.99.0 package check/Clippy/formatting passed; the shared-target test link failed with LNK1104, then the isolated-target retry passed 17/17.
> **Infrastructure demand consumer lane**: branch `codex/d20-infrastructure-demand-receiver`, rebased and merged by `2b7531746d044e45e0835cf6fa9a3b21e27cfa3e`.
> **Run-owner capacity Result lane**: branch `codex/d20-native-capacity-result`, rebased and merged by `1cc94614a59ef687624e31744334716a30c9bbde`. Both clean source worktrees were removed after merge-ancestry preflight and their branches were deleted with `git branch -d`. Codex app archive requests returned `queued`; artifact listing still reports the two old identities as active, so the app metadata is not recorded as archived.

## §0 目的

接通 D19 Star-owned durable gRPC Pull/Acknowledge 与既有 broker-neutral `AdmissionReceiveTransport` port，为 D20 后续 owner Outbox/Inbox consumers、quarantine/replay、BI/Benchmark projections 建立 native 接收 adapter 切片。此报告只覆盖当前隔离实现，不表示 D20、NATS replacement 或 Schedule/Run execution 完成。

## §1 改动矩阵

| 区域 | 改动 | 状态 |
|---|---|---|
| `crates/platform-messaging-runtime/src/native_grpc.rs` | 增加 `NativeGrpcAdmissionReceiveTransport`/delivery，实现共享 receive port；固定 scope/Profile，调用 PullOne/AcknowledgeOne | 已实现 |
| `crates/platform-messaging-proto/proto/native_transport.proto` | 新增版本化 `VerifyOrigin` unary RPC，携带固定 Profile、source route、log/generation/sequence/message 与 exact-wire SHA-256 | 已实现 |
| `crates/platform-messaging-runtime/src/native_grpc.rs`, `native_grpc_server.rs` | 增加接收端 exact-wire 来源 verifier 与 bounded server handler；DB errors/false 结果均 fail closed | 已实现；无 live TLS/PG e2e |
| `db/migrations/2026-10-12-platform-native-origin-verification.sql` | authenticated Append 时由 trigger 写入不可变 source certificate fingerprint；增加 exact current-lease verifier，并让 authenticated ACK 在同事务复验 | PostgreSQL 18.6 disposable runner 中双次应用通过 |
| `db/tests/platform-native-origin-verification.sql` | 覆盖旧 provenance NULL fail-closed、exact digest、tamper、ACK terminality 与 accepted-time revocation；使用固定 SHA-256 fixture 并在撤销事务设置 scoped actor | PostgreSQL contract 通过 |
| `scripts/automation/phase9f4d_run_owner_consumer.py` | schema-only runner 双次应用 D20 migration 并运行来源验证 SQL contract；PostgreSQL 仅监听 loopback 并禁用 Unix socket | 83/83 steps 通过；audit 已保存 |
| 同上 | 严格验证 sender/kind、log ID、generation、cursor、message/lease UUID、attempt/expiry、exact wire 长度与 SHA-256 | 已实现，定向单测覆盖 |
| 同上 | 公开 `native_provider_identity()` helper，派生 domain-separated SHA-256(provider ID/version/Profile digest/endpoint identity)；不把该键视为身份凭据 | 已实现，identity pin 测试覆盖 provider/version/Profile digest/endpoint |
| 同上 | 每 adapter 8 个并发 RPC + 8 个 delivery handle permit；handle drop 前持续占用 outstanding slot；无预取 queue | 已实现；无 live load/RSS 测量 |
| `docs/requirements.md`, `docs/basic-design.md` | 同步 D20 source certificate provenance、exact-wire verification、quarantine/ACK 复验与 SQL 阻塞 | 要件 v6.44；基本设计 v6.44 |
| `docs/design/DD-AUTOMATION-SCHEDULE-API-001.md`, `docs/design/DD-RUST-SERVER-REUSE-001.md` | 细化来源证明、数据库事务与本机服务候选复用边界；校正测试数并记载隔离 Clippy 结果 | 详细设计 v0.64、Rust 服务复用设计 v1.10 |
| `docs/plan/master-implementation-plan.md` | D20 标记为进行中并保留剩余验收项 | 已同步 |
| crates/run-owner-runtime | 新增 opt-in native-grpc-adapter feature 和 NativeGrpcScheduleAdmissionConsumer，固定 native transport + origin verifier + Run-owner store；拒绝 Result route，逐次消费一条 | 已实现；Rust 1.99.0 package all-targets check/test/Clippy 与变更文件 rustfmt 通过 |
| requirements/basic/detailed design | 同步 Run-owner composition 的 route、取消/quarantine 与 fail-closed 边界及正式 1.99.0 验证状态 | 已同步；需求/基本设计 v6.46，详细设计 0.67 |
| `crates/infrastructure/src/native_demand_consumer.rs` | 新增默认关闭 native Infrastructure demand consumer；固定 Request route，校验 typed demand、trusted current binding/placement 与 Host/cluster generation，Infrastructure owner Inbox commit + source recheck 后 ACK，durable quarantine 后 terminal disposition | 已合并；无 schema 变更；Rust 1.99.0 package tests 71/71 |
| `crates/run-owner-runtime/src/native_capacity_result_consumer.rs`, `capacity_result.rs` | 新增 opt-in provider-neutral Run-owner Result consumer/strict parser；核对原始 demand 与 binding，Run-owner Inbox commit + source recheck 后 ACK，poison 要独立 durable quarantine | 已合并；Run-owner library tests 22/22 |
| requirements/basic/Schedule SRS/BD/DD/Infrastructure DD/Group DD/reuse DD/implementation plans | 同步两个 consumer 的 owner boundaries、ACK 顺序、feature gates、合并后证据与未关闭门 | requirements/basic v6.48、SRS v0.60、Schedule BD v0.58/DD v0.69、Infrastructure DD v0.35、Group DD v4.83、reuse DD v1.11、plan v6.76 |
| `AGENTS.md` | 未修改；本机 Rust server 的模块级选择性复用规则已存在 | 无需重复 |

## §2 验证摘要

| 守门 | 实际结果 |
|---|---|
| 目标 crate all-targets check | `cargo check --ignore-rust-version -p platform-messaging-runtime --features native-grpc-adapter --all-targets -j 4` 退出码 0 |
| 定向 lib tests | `cargo test --ignore-rust-version --offline --locked -p platform-messaging-runtime --features native-grpc-adapter --lib native_grpc::tests -j 1` 12/12 通过（Rust 1.98.1 诊断） |
| 本轮完整 lib tests | `cargo test --ignore-rust-version --offline --locked -p platform-messaging-runtime --features native-grpc-adapter --lib -j 4` 41/41 通过（Rust 1.98.1 诊断） |
| Clippy driver / all-targets | `cargo check --ignore-rust-version --offline --locked -p platform-messaging-runtime --features native-grpc-adapter --all-targets -j 4`，设置 `RUSTC_WORKSPACE_WRAPPER` 为 Rust 1.98.1 的 `clippy-driver.exe`；退出码 0。目标 crate 未输出 Clippy warning；未修改的 `star-context`、`domain-automation` 有既有 warning。直接 `cargo clippy` 因 1.98.1 `rustc` 与 1.99.0 clippy driver 混用报 `E0514`，该次不计为 lint 结果 |
| 新增 Run-owner composition（Rust 1.99.0） | cargo +1.99.0 check --offline --locked -p run-owner-runtime --features native-grpc-adapter --all-targets -j 4 exit 0；cargo +1.99.0 test --offline --locked -p run-owner-runtime --features native-grpc-adapter --lib -j 4 为 17/17；cargo +1.99.0 clippy --offline --locked -p run-owner-runtime --features native-grpc-adapter --all-targets -j 4 exit 0；changed module rustfmt 与 lib.rs（skip_children=true）exit 0。依赖 crate 有既有 warnings |
| Post-merge Run-owner package gates (Rust 1.99.0) | all-targets check 0 (46.60 s), all-targets Clippy 0 (10.36 s), changed-file rustfmt 0; isolated-target library tests 17/17 (0.02 s). Shared target link first failed with Windows LNK1104; isolated retry succeeded. |
| Follow-on combined dev package check (Rust 1.99.0) | `cargo +1.99.0 check --offline --locked -p infrastructure -p run-owner-runtime --features infrastructure/native-grpc-adapter,run-owner-runtime/native-grpc-adapter --all-targets -j 4` exit 0. |
| Follow-on combined library tests (Rust 1.99.0) | `infrastructure` 71/71 and `run-owner-runtime` 22/22 passed, including both new consumer modules. |
| Follow-on strict target Clippy and format | `cargo +1.99.0 clippy --offline --locked --no-deps -p infrastructure -p run-owner-runtime --features infrastructure/native-grpc-adapter,run-owner-runtime/native-grpc-adapter --all-targets -j 4 -- -D warnings` exit 0; changed Rust modules and lib files rustfmt check exit 0. Existing `domain-kms` deprecation warnings remain outside the selected target crates. |
| D20 PostgreSQL 18.6 schema-only origin gate | WSL disposable PostgreSQL 18.6: runner **83/83 steps passed**, including D20 origin migration applied twice and `native-origin-verification-contract`; `postgres-stop` passed. Audit: `target/phase9f4c-d20-origin-wsl-20261014-final-83steps-v3/audit.jsonl` (SHA-256 `CE8FB01327E7BA181E22099399BD1C5FB20BB0D860204F3A8B7F6ED91BA69E80`). No NATS process started; Multipass instance count/inventory unchanged. This does not run Windows PostgreSQL runtime, SQLx tests or validate production grants/identity. |
| Follow-on Git integration | Infrastructure source commit `36680a392293540142eb1eb95ff730c66d354438` rebased and merged as `2b7531746d044e45e0835cf6fa9a3b21e27cfa3e`; Run-owner source commit rebased to `8cf669ee8bcb433c90e08b0cfa22c08479784663` and merged as `1cc94614a59ef687624e31744334716a30c9bbde`. No conflicts; current dev clean before docs; no push. |
| Git cleanup | Clean-ancestry preflight passed for both source commits. `git worktree remove` (without force) removed both clean checkouts and `git branch -d` deleted both merged refs; local `git branch` and `git worktree list` now contain only `main` and `dev`. Codex archive calls returned `queued`, and app artifact listing still shows the two stale identities as active; no archive completion is claimed. |
| Run-owner Clippy-driver / all-targets | 使用独立 target dir 与 Rust 1.98.1 `clippy-driver.exe` 设置 `RUSTC_WORKSPACE_WRAPPER`，`cargo check --ignore-rust-version --offline --locked -p run-owner-runtime --features native-grpc-adapter --all-targets -j 4` 退出码 0；`run-owner-runtime` 未产生 Clippy warning，依赖 crate 存在历史 warning。普通 `cargo clippy` 因共享 target 混有 Rust 1.99.0 元数据报 `E0514`，不作为结果 |
| GitConverge 到 `dev` | D20 已由 merge commit `78c7211ac7d08011db5adcaa9c378fab38b28193` 合入；清理时 `dev` 为 `4c0de14d9e2de669044dc47ac4559348b65a6805`，本地 ahead `origin/dev` 6 个提交，未 push。Run-owner worktree 已归档、分支已安全删除；本地只剩 `main` 与 `dev` 两个 branch ref、两处 worktree。Codex 应用登记仍有两个已不存在的旧 worktree 标为 active；归档请求被其他 task ownership gate 拒绝，未触碰其记录。完整收敛记录见 `GIT-CONVERGE-DEV-20261013.md` |
| Rust 格式 | 三个修改的 runtime 实现文件 `rustfmt --edition 2024 --check` 退出码 0；`schedule_admission_pg.rs` 全文件检查报告大量既有格式差异，本轮只修改 trait 方法签名，未格式化整份无关测试文件 |
| Diff whitespace | `git diff --check` 退出码 0；Git 提示该 Rust 文件下次触碰时 CRLF 转 LF，不影响本次检查 |
| CypherGraph Guardian | `cgg_guard.py --changed --stamp` 通过；本轮更新的 Python runner source hash 已刷新 |
| 正式 Rust 版本 | Run-owner composition 的 Rust 1.99.0 package all-targets check/test/Clippy 与定向 rustfmt 通过；native receiver 的既有验证仍是 Rust 1.98.1 诊断结果，workspace-wide release gates 未运行 |
| 工具调用方式 | rtk 不在 PATH；使用 E:\DevCache\cargo\bin 下的 cargo/rustfmt shim 与显式 +1.99.0 toolchain 运行本轮 gates |
| D20 PostgreSQL origin migration/contract | PostgreSQL 18.6 WSL disposable schema-only runner **83/83 通过**，包含 D20 migration 双次应用和来源验证 SQL contract；不含 SQLx target、生产身份或目标库 grants 验收 |
| Native gRPC live e2e / Multipass/K3s | 未运行 live gRPC/TLS e2e；本轮 schema-only gate 未启动 NATS、未创建 Multipass instance、未改变 inventory；临时 PostgreSQL 已停止 |
| 代码审查 | 对新增 consumer 与共享 receiver 调用路径做了有界手工复查：Result route 在连接前拒绝；只接受 Request/Resolution；单次至多消费一条并遵守取消/10 秒 deadline；owner commit 与来源复验先于 ACK；永久 poison 必须配置 durable quarantine；未发现新增执行绕过。dev worktree 的 CodeGraph 索引报告 up-to-date，但本次查询未返回新增 consumer；`check_index_coverage` MCP 工具不可用，因此从提交 diff 与精确源码回退审查，不作全图覆盖声明。当前修复的独立只读 agent review 已完成，未发现 P0-P3；CodeRabbit 未能启动：Windows npm shim 调用 `/root/.local/bin/coderabbit` 时返回 `/bin/bash: line 1: /root/.local/bin/coderabbit: Permission denied`，因此不记为独立 CodeRabbit 通过 |

## §3 已知缺口

1. 当前已接入 Run-owner Schedule admission/Result 与 Infrastructure demand 的 opt-in/default-off consumers，但尚未完成所有 owner Outbox/Inbox producer/consumer 迁移。Rust package gates 与 D20 disposable PostgreSQL origin migration/SQL contract 已通过；SQLx target、TLS e2e、production assembly/grants 与 runtime deployment 仍未验收。
2. 真实 TLS client/server handshake、证书轮换/撤销、生产 Profile endpoint resolver、production composition 和目标 PostgreSQL runtime grants 未验收。
3. native provider 的 quarantine receipt/replay 再投递与 digest/fault 冲突尚未做端到端 DB 验收。共享 receiver 的代码顺序是 commit quarantine 后 terminal disposition；本 slice 未证明 native server/DB 联动。
4. BI/Benchmark event projections、NATS/native 双 provider contract parity、双跑差异对账和逐 profile rollback/cutover 未实现。
5. 未测 crash/restart recovery、lease reclaim、内存 RSS、p95/p99、全局多 adapter concurrency、跨 tenant fairness、队列 drain 或性能容量。
6. Run-owner composition 的 Rust 1.99.0 package check/test/Clippy 与改动文件格式门通过；workspace-wide release gates 与 D20 native receiver 全量复验尚未运行。
7. D20 整体仍进行中；native provider 不可用于产品流量，NATS 继续保留，Schedule/Run execution 仍受原有授权、预算、Reservation、Hook、sandbox、取消恢复和独立验证门控制。
8. 用户确认可直接参考、适配或按模块复用 AssetsLake 与其他本机 Rust server；本 slice 未搬入源代码。当前 AssetsLake event publisher 在 pipeline disabled 时只返回生成 UUID，relay 在 dry-run/no-broker 时会标记 published、配置 broker 时因无 Kafka client 标记失败，因此不适合作为当前 durable transport；它的 envelope、trace、causation 字段仍可作契约参考。RustGameServer TLS helper 是既有已审查适配；不据此推断整仓依赖、商业分发/SBOM 或其他模块已验收。
9. D20 source-verification migration 与 SQL contract 已在 disposable PostgreSQL 18.6 通过 83/83 runner steps；这不覆盖 SQLx integration target、目标库 runtime grants、生产身份/TLS 或故障恢复。切流和执行继续 fail closed。
10. 两条 follow-on lane 已 rebase 并合并到本地 `dev`，merge commits 为 `2b7531746d044e45e0835cf6fa9a3b21e27cfa3e` 与 `1cc94614a59ef687624e31744334716a30c9bbde`。两个 source checkout 均通过 clean/ancestor preflight 后用非强制 Git worktree remove 清理，两个分支用 `git branch -d` 删除；本地 refs/worktrees 只剩 main/dev。Codex app archive API 返回 queued，artifact listing 仍显示旧 identity active，未声称已归档。`origin/dev` 未更新，未 push。D20 整体仍进行中，PostgreSQL/RLS/TLS/production、全量 owner consumers、性能与 BI/Benchmark 验收仍开放。

## §4 子代理失败接手清单

本次 follow-on 由 `/root/native_infrastructure_demand` 与 `/root/native_capacity_result` 两个子代理分别在独立 worktree 完成，均已提交、rebase 并合入 dev；未遗留未合并代码。流程偏差：派发前没有按项目 `AGENTS.md` 调用 `scripts/automation/dispatcher.py brief(...)` 落地 Python brief；这是未遵守的前置流程，本报告如实记录，不能追溯伪装为已完成。CodeRabbit 工具执行受到 permission denied 阻断，按独立 reviewer 未完成处理，不将其归入通过项。

## §5 守门规则（17 项）

| # | 守门规则 | 本阶段状态 |
|---:|---|---|
| 1 | 以 Project/Branch/Run owner 边界决定职责 | 通过；adapter 不写 owner 事实 |
| 2 | source reuse 先逐模块适配，不整仓搬迁；用户已授权合适自有模块直接复用 | 通过；权限已确认，D20 当前 AssetsLake relay 语义不合格故未迁入 |
| 3 | 对许可、NOTICE、权属和完整依赖闭包逐版本审查 | 仍需每次新增源文件/发布时单独验收 |
| 4 | mTLS 证书身份是 peer 来源认证基础 | 继承 D19；本轮无 live handshake |
| 5 | Profile digest 与 endpoint/profile revision 必须固定 | 代码通过；production resolver 未接入 |
| 6 | recipient 来自服务端已认证 principal，不信 wire claim | proto/server 路由契约遵守；无 e2e |
| 7 | owner commit 和 commit 后授权复验先于 ACK | Run-owner Result 与 Infrastructure demand consumers 均在各自 owner commit、source revalidation 后 ACK；目标 crate gates 通过，真实 TLS/PG e2e 与 production assembly 未验收 |
| 8 | permanent ACK 前先提交独立 durable quarantine | 共享 receiver 已定义；native DB 联测未做 |
| 9 | ACK 绑定 generation、sequence、message、lease 和 exact digest | client request 固定并发送这些字段；server live 验证未做 |
| 10 | digest 必须对原始 wire bytes 计算 | 代码已校验 exact SHA-256；10 项定向测试通过 |
| 11 | payload、frame 与身份字段有界 | payload 16 KiB、frame 32 KiB，字段有界校验 |
| 12 | 并行任务使用有界队列/背压/取消 | 8 RPC + 8 outstanding handle permits，无预取 queue；系统级预算未验收 |
| 13 | 未知/超时/错误时 fail closed | RPC/ACK 错误返回未确认；集成重试/取消未测 |
| 14 | provider-specific cursor 不泄漏为业务身份 | sequence/log/lease 留在 adapter delivery origin |
| 15 | NATS 不得在 replacement gates 前移除 | 通过；NATS 保留 |
| 16 | 需求、基本设计、详细设计与实现状态同步 | requirements/basic/SRS/BD/DD/Group/Infrastructure/reuse/implementation plan 已同步到两个 merged consumer slices；PostgreSQL、workspace-wide、TLS 与 production gates 仍开放 |
| 17 | Rust 1.99.0 正式工具链、PostgreSQL migration/RLS contract 与独立性能验收不得由诊断结果替代 | 两个目标 crate 的合并后 Rust 1.99.0 check/tests/target Clippy/formats 通过；D20 disposable PostgreSQL migration/SQL contract 83/83 通过；SQLx、workspace-wide gates 与独立性能验收未运行，D20 保持开放 |

## §6 签字栏

| 角色 | Mavis 审核记录 |
|---|---|
| 架构 | `架构师 (Mavis 接手 agent per DEC-008)`：检查 provider/owner 边界与用户确认的本机模块复用规则；D20 保持进行中 |
| SRE Lead | Mavis review：未验收；没有恢复、负载、RSS、tail latency 或公平性证据 |
| 平台 | Mavis review：确认两个 owner-native receive slices 与文档对齐；生产 grants、resolver、server deployment 和 owner producer wiring 未验收 |
| 评审主持 | Mavis review：已手工审阅 Run-owner Result、Infrastructure demand consumer 与共享 receiver 的限界路径，未发现新增执行绕过；独立 CodeRabbit 因权限错误未完成，不冒充独立 review |
| PM | Mavis review：阶段报告与 plan 均记录未关闭项；NATS/native cutover 不开放 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.4 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | Remove the cross-platform quoting defect in the disposable PostgreSQL runner; record the independently reviewed 83/83 PostgreSQL 18.6 v3 audit and keep Windows/SQLx/production limits explicit | Reviewer identified -k single-quote portability risk; corrected runner gate passed
| v1.3 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复 WSL disposable PostgreSQL runner 的 Unix-socket 权限边界及 D20 SQL fixture 的固定摘要/scoped actor，记录 PG18.6 migration replay 与 source-verification contract 83/83 通过；production/TLS/SQLx/BI/recovery/performance 门保持开放 | D20 origin SQL gate 在 Multipass-free WSL PostgreSQL 18.6 中实跑 |
| v1.2 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更新两条已合入 lane 的 Git worktree/branch 清理结果，并如实注明 Codex app archive queue 尚未反映为归档状态；D20 剩余生产门不变 | 完成两条 clean merged worktree 与 source branch 清理 |
| v1.1 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Infrastructure demand 与 Run-owner Result consumer 的 rebase/merge、联合 Rust 1.99.0 gates、文档同步、lane cleanup 状态与派发 brief 流程偏差；保持 D20 生产门未关闭 | 两条 consumer worktree rebase 并合入本地 dev |
| v1.0 | 2026-10-13 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补录 Run-owner lane 已归档并删除、Git refs/worktrees 收敛为 main/dev，以及两个旧 Codex artifact 受其他 task owner 阻止归档；记录本次有界手工审查与 CodeRabbit 执行阻塞，D20 生产缺口保持开放 | 用户授权先归档本地内容再清理；复核当前合并与应用登记状态 |
| v0.9 | 2026-10-13 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 D20 Run-owner composition rebase/merge 到本地 `dev`、合并后 gates 与 GitConverge 清理阻塞；不宣称 D20 全量完成 | GitConverge merge commit `78c7211a` 已创建，合并后 check/Clippy/rustfmt 与 isolated-target 17/17 tests 复验完成 |
| v0.8 | 2026-10-13 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更新 Run-owner consumer 正式 Rust 1.99.0 check/test/Clippy/rustfmt 证据；仍标记 PostgreSQL、TLS、production 和全量 D20 验收未完成 | 1.99.0 package gates 全部通过，17/17 tests |
| v0.7 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补录 Result route 在 transport connect 前拒绝的回归测试；Run-owner library tests 更新为 17/17，定向测试 1/1 | Rust 1.98.1 隔离 target 下新增回归测试和完整测试集通过 |
| v0.6 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补录 GitConverge 因调用 worktree/目标 worktree dirty gate 与 Orca runtime EPERM 未执行的证据；保留 detached D20 改动，不清理用户文件或 worktree | 用户要求优先合并 `dev` 并清理；当前 Orca 状态接口拒绝访问且 GitConverge preflight gate 未满足 |
| v0.5 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 追加 opt-in Run-owner native Schedule admission composition、Result route pre-connect rejection、one-message cancellation/quarantine boundary；记录 Rust 1.98.1 诊断 check、16/16 tests 与隔离 Clippy-driver 结果，保留正式 1.99.0/PG/TLS/production blockers 且 D20 不关闭 | 延续 D20 实现并同步需求、基本/详细设计与 plan |
| v0.4 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 校正 library tests 为 41/41、记录 Rust 1.98.1 Clippy-driver 检查与 integration test 文件格式现状；保持 PostgreSQL 合约未验证状态 | 隔离 Clippy-driver 检查完成并复核 D20 验证记录 |
| v0.3 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 exact-wire VerifyOrigin、append-time source certificate evidence、同事务 ACK 复验和 SQL runner 接入；把未运行的 PostgreSQL contract 明确记为阻塞，并复核 AssetsLake/RustGameServer 来源状态 | D20 source-verification 实装；用户重申可参考/复用自有 Rust 服务 |
| v0.2 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确本机 Rust server 可直接模块复用，解释 AssetsLake 当前 relay 不满足 durable delivery；记录公开 canonical provider identity helper 与实际工具链调用方式 | 用户再次确认 AssetsLake/本机 Rust 服务复用授权，并要求为 Star 自有服务替换 NATS 做准备 |
| v0.1 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 D20 native receive adapter 实现、真实验证命令、剩余生产门与来源复用边界 | 用户授权参考/复用 AssetsLake 与其他本机 Rust server，继续推进 native transport |
