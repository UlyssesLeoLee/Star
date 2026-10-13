# PHASE-9F4C-D20-NATIVE-RECEIVER-REPORT

> **Status**: D20 native receive/origin-proof、Run-owner Schedule admission/Result、Infrastructure demand receive、restricted grants/RLS contract 与 guarded SQLx origin target 已合并到本地 `dev`。原始 SQLx target commit `646eaa34` 已在 dev；本轮仅将数据库隔离/写前预检补丁 `b995636c` rebase 后快进合入。PostgreSQL 18.6 WSL ext4 cumulative schema-only runner 为 89/89；guarded SQLx target 编译通过但仍 ignored，未执行数据库行为。清理后本地 Git 仅保留 `main`/`dev` 两个分支与 worktree；D20 整体进行中，生产接入未开放；未 push。
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
| `db/migrations/2026-10-12-platform-native-origin-verification.sql`, `db/tests/platform-native-origin-verification.sql` | Fail-closed 要求预置且受限的 owner/runtime roles；runtime 仅获 authenticated append/pull/verify/ack function allowlist，不可直读写 transport tables、不可执行 trigger capture function；检查 PUBLIC 拒绝、forced RLS、非空跨 tenant 反例、untrusted verify/ACK 拒绝及 BYPASSRLS mutation guard | WSL ext4 disposable PostgreSQL 18.6 累计 runner 89/89；仅 schema/contract，不代表生产 provisioning/grants |
| `crates/platform-messaging-runtime/tests/native_origin_verification_pg.rs` | SQLx origin contract 覆盖 exact replay/digest、来源 scope/generation/sequence/message/digest、证书/lease/ACK 与 rollback/retry；测试现在要求 loopback、非默认端口及精确 `star_d20_origin_test`，并在 fixture 写入前预检角色/成员资格、所需表/函数及 enabled+forced RLS | guarded target 在 Rust 1.98.1 `--ignore-rust-version` 下 `--no-run` 编译通过；仍是 ignored，未执行 SQL；同名本机数据库仍是隔离残余风险 |
| `docs/briefs/D20-origin-grants.md`, `docs/briefs/D20-origin-sqlx.md` | 分别记录两条 worktree 的任务范围和验证证据 | 已随各 lane 纳入 dev |
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
| requirements/basic/Schedule DD/Rust reuse DD/implementation plan | 同步 D20 SQLx 写前安全预检、编译与未执行边界、production gates 和 fail-closed 状态 | requirements/basic v6.52、Schedule DD v0.73、Rust reuse DD v1.15 与 plan 已同步；dev 文档同步保留原有 D19 变更 |
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
| D20 restricted role/RLS follow-on | PostgreSQL 18.6 on WSL ext4: cumulative runner **89/89 steps passed**, covering migration replay, restricted owner/runtime roles, runtime function allowlist, PUBLIC and untrusted verify/ACK denial, forced RLS/cross-tenant isolation, BYPASSRLS role-mutation rejection/recovery and shutdown. Tracked evidence: [runner result](evidence/PHASE-9F4C-D20-PG18.6-20261014/runner-result.json) SHA-256 `CBD897B80ABA98A8EAC02422ED2BC613D5E93A56AC12D1195E1AC5E094271085`; [audit JSONL](evidence/PHASE-9F4C-D20-PG18.6-20261014/audit.jsonl) SHA-256 `6218847B147E2FABB20BB16A987912F6D9A6996B1C4CE458DF54A1658A08BB40`. This is disposable WSL/Linux schema/contract evidence only; no NATS or Multipass instance was started. |
| SQLx origin integration target | Rust 1.98.1 `--ignore-rust-version` 下 guarded `native_origin_verification_pg` target 使用隔离 target dir 执行 `--no-run` 编译，退出码 0；该测试仍 `#[ignore]`，没有数据库 URL，真实 SQL/事务场景未执行。改动加入 loopback、拒绝 5432、精确 `current_database()` 名称检查，并在任何 fixture 写入前检查 owner/runtime role membership、完整表/函数集合及 enabled+forced RLS。独立审查接受该修复；残余是假如本机另一个非默认端口 loopback 数据库使用相同精确名称，测试自身无法区分。 |
| Follow-on Git integration | 基础 SQLx commit `646eaa34` 已因 patch-equivalent 内容存在于 dev 而不重复合并；仅新增 guard commit `ab50f082` 经 `git rebase --onto dev 91fce4c3` 重放为 `b995636c`，随后 `git merge --ff-only` 合入 dev。变更范围只有该 SQLx integration test；未 push。 |
| Git cleanup | 本轮 D20 临时 worktree/branch 在 clean 与 dev-ancestor preflight 后移除；两个 Orca-managed D19 endpoint/E2E worktree 也在确认 checkout clean、提交已合入 dev、terminal 仅为空闲 PowerShell prompt 后通过 Orca CLI 移除。最终 Git branches/worktrees 仅 `main`/`dev`；被清理 lanes 的提交均可从 dev 到达，main 未改，未 push。 |
| D20 PostgreSQL runner selective integration | Plan recorded `main@608de5f1` (0 commits unique to `dev`, retained), `dev@c98a51d2` (target, initially +14 vs cached `origin/dev`), runner `08b2f6a4` (2 commits ahead, based on `b2dc0c68`) and docs `501b9540` (2 commits ahead, same `b2dc0c68` base). Docs lane rebased onto runner lane; runner lane rebased onto `dev`; `dev` fast-forwarded to delivery tip `8a6cae9d` without conflict. The clean code/docs/review worktrees were removed without force after ancestry checks; merged branches were deleted with `git branch -d` from `dev`. Final Git refs/worktrees: `main` and `dev` only. `dev` is +18 vs cached `origin/dev`; no push. The runner archive request returned `queued`; the app listing still shows all three identities as active, so no app archival completion is claimed. |
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
9. D20 origin-only PostgreSQL gate 为 83/83；后续 cumulative restricted role/grants/RLS runner 为 89/89。SQLx target 经本轮加固后只编译通过，仍 ignored，SQL/事务行为未运行。guard 检查 loopback、非默认端口、精确测试库名以及写前 role/schema/function/FORCE RLS；第二个同名本机数据库仍须由环境管理隔离。生产 grants/identity、TLS 与恢复保持未验收，切流和执行继续 fail closed。
10. Grants lane `a9bc2681` 与 SQLx baseline `646eaa34` 已在 dev；本轮 guard `b995636c` 仅包含测试文件。D20 临时 checkout/branch 与两个已合入的 D19 Orca worktree/branch 均已安全清理；终端移除前确认为空闲 shell。`main` 保持 `608de5f1`，本地 Git 只剩 `main` 与 `dev`；requirements/basic/Schedule DD/reuse DD/plan/report 已同步至 dev，`origin/dev` 未更新，未 push。D20 生产验收仍开放。

## §4 子代理失败接手清单

本轮由 grants 与 SQLx 两条独立子代理/worktree lane 并行完成。Grants source commit `a9bc2681`、SQLx source commit `954a4673`（加入 brief 后 amend）依次 rebase 到 `dev`；SQLx 最终提交 `646eaa34` 与 grants commit 均已进入本地 dev。两条 lane 分别提供 `docs/briefs/D20-origin-grants.md` 与 `docs/briefs/D20-origin-sqlx.md`。合并前后对 role guard、RLS/non-vacuous cross-tenant fixture、SQLx ignored gating 和事务回滚路径做了有界手工复核，未发现已确认问题；CodeRabbit CLI 因 permission denied 无法启动，不记为独立 review 通过。来源 worktree clean/ancestor preflight 后非强制删除，源分支已删除；Codex archive API 仍返回 queued，附件列表仍显示 stale active identities，故不宣称 Codex archive 完成。已合入的代码提交保留在 dev 历史中，没有未提交内容遗失。此前其他 follow-on 的 brief 流程偏差见历史报告版本，不将其追溯补成事前记录。

本轮继续工作记录：独立只读 reviewer 对数据库身份限制与完整写前预检给出接受建议；CodeRabbit CLI 启动返回 `/bin/bash: /root/.local/bin/coderabbit: Permission denied`，不记为 CodeRabbit 通过。已用 SQLx target `--no-run` 编译、rustfmt、CypherGraph Guardian 与 diff whitespace 检查复核；没有真实 PostgreSQL SQLx 执行。
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
| 17 | Rust 1.99.0 正式工具链、PostgreSQL migration/RLS contract 与独立性能验收不得由诊断结果替代 | 既有 owner crates 的 Rust 1.99.0 check/tests/target Clippy/formats 与 PostgreSQL 89/89 disposable contract 证据仍有效；guarded SQLx target 使用 Rust 1.98.1 `--ignore-rust-version` 编译，仍 ignored 且 SQL 行为未运行；workspace-wide gates 与独立性能验收未运行，D20 保持开放 |

## §6 签字栏

| 角色 | Mavis 审核记录 |
|---|---|
| 架构 | `架构师 (Mavis 接手 agent per DEC-008)`：检查 provider/owner 边界与用户确认的本机模块复用规则；D20 保持进行中 |
| SRE Lead | Mavis review：未验收；没有恢复、负载、RSS、tail latency 或公平性证据 |
| 平台 | Mavis review：确认 grants/RLS contract 和 SQLx test target 已与文档对齐；生产 role provisioning/deployment grants、resolver、server deployment 和 owner producer wiring 未验收 |
| 评审主持 | Mavis review：独立只读审查接受 D20 SQLx 测试库身份与写前预检补丁，未确认目标库误写或 migration partial-write 路径；同名本机数据库仍是环境隔离残余。CodeRabbit 因 CLI 权限错误未完成，不冒称通过 |
| PM | Mavis review：阶段报告与 plan 均记录未关闭项；NATS/native cutover 不开放 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v1.7 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录仅新增 D20 SQLx database guard commit `b995636c` rebase/快进合入 dev；确认 baseline test commit 已 patch-equivalent 存在，避免重复合并；补录 loopback/非默认端口/精确库名与完整写前预检、no-run 编译及同名库残余风险；在确认干净、已合并和空闲终端后清理 D20/D19 worktrees 与额外分支，最终仅留 main/dev | 用户要求并行 worktree 结果合入 dev 并清理；独立审查指出误连库与 partial fixture 风险 |
| v1.6 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 grants/RLS 与 SQLx 两条独立 lane rebase/快进合入本地 dev、PostgreSQL 89/89、SQLx 1 ignored 未运行、tracked evidence hashes 与 Git worktree/branch 清理结果；注明 Codex archive API 仍 queued/active，不冒称归档完成；保持 production/execution gates 开放 | 用户要求并行 worktree 完成后 rebase/merge 到 dev；两条 lane 合并完成 |
| v1.5 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 D20 PostgreSQL runner 修复及需求/基本/详细设计 rebase、快进合入本地 dev 的准确 delivery tip `8a6cae9d`；clean code/docs/review worktrees 与已合入 refs 已清理；runner archive request 仍 queued、Codex app 仍列出 active identities，不冒称归档完成；未 push，D20 执行与生产门保持开放 | 用户要求将完成的并行 worktree 结果 rebase/merge 到 dev 并清理 |
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
