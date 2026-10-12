# PHASE-9F4C-D20-NATIVE-RECEIVER-REPORT

> **Status**: D20 首个 native receive adapter slice 已实现；D20 整体进行中，生产接入未开放。
> **Worktree**: `C:/Users/leo19/.codex/worktrees/phase9f4c-d20-consumers/Star`
> **Branch**: `codex/phase9f4c-d20-native-receiver-20261012`
> **Base**: `501c6cb5209a0f87df971cd0907a49f448118982`（本 worktree 本轮实际 HEAD 基线）

## §0 目的

接通 D19 Star-owned durable gRPC Pull/Acknowledge 与既有 broker-neutral `AdmissionReceiveTransport` port，为 D20 后续 owner Outbox/Inbox consumers、quarantine/replay、BI/Benchmark projections 建立 native 接收 adapter 切片。此报告只覆盖当前隔离实现，不表示 D20、NATS replacement 或 Schedule/Run execution 完成。

## §1 改动矩阵

| 区域 | 改动 | 状态 |
|---|---|---|
| `crates/platform-messaging-runtime/src/native_grpc.rs` | 增加 `NativeGrpcAdmissionReceiveTransport`/delivery，实现共享 receive port；固定 scope/Profile，调用 PullOne/AcknowledgeOne | 已实现 |
| `crates/platform-messaging-proto/proto/native_transport.proto` | 新增版本化 `VerifyOrigin` unary RPC，携带固定 Profile、source route、log/generation/sequence/message 与 exact-wire SHA-256 | 已实现 |
| `crates/platform-messaging-runtime/src/native_grpc.rs`, `native_grpc_server.rs` | 增加接收端 exact-wire 来源 verifier 与 bounded server handler；DB errors/false 结果均 fail closed | 已实现；无 live TLS/PG e2e |
| `db/migrations/2026-10-12-platform-native-origin-verification.sql` | authenticated Append 时由 trigger 写入不可变 source certificate fingerprint；增加 exact current-lease verifier，并让 authenticated ACK 在同事务复验 | 已写入；未执行 migration |
| `db/tests/platform-native-origin-verification.sql` | 覆盖旧 provenance NULL fail-closed、exact digest、tamper、ACK terminality 与 accepted-time revocation | 已写入；未运行 |
| `scripts/automation/phase9f4d_run_owner_consumer.py` | schema-only runner 双次应用 D20 migration 并运行来源验证 SQL contract | 已接入；环境缺 PostgreSQL server binaries |
| 同上 | 严格验证 sender/kind、log ID、generation、cursor、message/lease UUID、attempt/expiry、exact wire 长度与 SHA-256 | 已实现，定向单测覆盖 |
| 同上 | 公开 `native_provider_identity()` helper，派生 domain-separated SHA-256(provider ID/version/Profile digest/endpoint identity)；不把该键视为身份凭据 | 已实现，identity pin 测试覆盖 provider/version/Profile digest/endpoint |
| 同上 | 每 adapter 8 个并发 RPC + 8 个 delivery handle permit；handle drop 前持续占用 outstanding slot；无预取 queue | 已实现；无 live load/RSS 测量 |
| `docs/requirements.md`, `docs/basic-design.md` | 同步 D20 source certificate provenance、exact-wire verification、quarantine/ACK 复验与 SQL 阻塞 | 要件 v6.44；基本设计 v6.44 |
| `docs/design/DD-AUTOMATION-SCHEDULE-API-001.md`, `docs/design/DD-RUST-SERVER-REUSE-001.md` | 细化来源证明、数据库事务与本机服务候选复用边界；校正测试数并记载隔离 Clippy 结果 | 详细设计 v0.64、Rust 服务复用设计 v1.10 |
| `docs/plan/master-implementation-plan.md` | D20 标记为进行中并保留剩余验收项 | 已同步 |
| `AGENTS.md` | 未修改；本机 Rust server 的模块级选择性复用规则已存在 | 无需重复 |

## §2 验证摘要

| 守门 | 实际结果 |
|---|---|
| 目标 crate all-targets check | `cargo check --ignore-rust-version -p platform-messaging-runtime --features native-grpc-adapter --all-targets -j 4` 退出码 0 |
| 定向 lib tests | `cargo test --ignore-rust-version --offline --locked -p platform-messaging-runtime --features native-grpc-adapter --lib native_grpc::tests -j 1` 12/12 通过（Rust 1.98.1 诊断） |
| 本轮完整 lib tests | `cargo test --ignore-rust-version --offline --locked -p platform-messaging-runtime --features native-grpc-adapter --lib -j 4` 41/41 通过（Rust 1.98.1 诊断） |
| Clippy driver / all-targets | `cargo check --ignore-rust-version --offline --locked -p platform-messaging-runtime --features native-grpc-adapter --all-targets -j 4`，设置 `RUSTC_WORKSPACE_WRAPPER` 为 Rust 1.98.1 的 `clippy-driver.exe`；退出码 0。目标 crate 未输出 Clippy warning；未修改的 `star-context`、`domain-automation` 有既有 warning。直接 `cargo clippy` 因 1.98.1 `rustc` 与 1.99.0 clippy driver 混用报 `E0514`，该次不计为 lint 结果 |
| Rust 格式 | 三个修改的 runtime 实现文件 `rustfmt --edition 2024 --check` 退出码 0；`schedule_admission_pg.rs` 全文件检查报告大量既有格式差异，本轮只修改 trait 方法签名，未格式化整份无关测试文件 |
| Diff whitespace | `git diff --check` 退出码 0；Git 提示该 Rust 文件下次触碰时 CRLF 转 LF，不影响本次检查 |
| CypherGraph Guardian | `cgg_guard.py --changed --stamp` 通过，1 个 Rust 源文件 |
| 正式 Rust 版本 | 未通过正式门禁：本机使用 Rust 1.98.1 的 `--ignore-rust-version` 诊断运行；Rust 1.99.0 未安装/未运行 |
| 工具调用方式 | `cargo`/`rustc` 不在 PATH；实际通过 Rust 1.98.1 安装目录下的 `cargo.exe` 与显式 `build.rustc` 配置运行上述诊断命令 |
| D20 PostgreSQL origin migration/contract | 未运行：`psql`、`initdb`、`pg_ctl` 不在 PATH；PostgreSQL 15/17 安装目录无 `initdb.exe`。SQL 语法、触发器/锁、RLS、grants 和重复 migration 行为均未验证 |
| Native gRPC live e2e / Multipass/K3s | 未运行；本切片没有启动 NATS、PostgreSQL 或 Multipass 实例 |
| 独立 CodeRabbit review | 未运行成功：包装器返回 `/root/.local/bin/coderabbit: Permission denied`；本轮由实现者自审，不能称作独立 review 通过 |

## §3 已知缺口

1. 只有 provider adapter；没有把实际 Schedule/Run 或 Infrastructure owner Outbox/Inbox consumer 接到该 native adapter。
2. 真实 TLS client/server handshake、证书轮换/撤销、生产 Profile endpoint resolver、production composition 和目标 PostgreSQL runtime grants 未验收。
3. native provider 的 quarantine receipt/replay 再投递与 digest/fault 冲突尚未做端到端 DB 验收。共享 receiver 的代码顺序是 commit quarantine 后 terminal disposition；本 slice 未证明 native server/DB 联动。
4. BI/Benchmark event projections、NATS/native 双 provider contract parity、双跑差异对账和逐 profile rollback/cutover 未实现。
5. 未测 crash/restart recovery、lease reclaim、内存 RSS、p95/p99、全局多 adapter concurrency、跨 tenant fairness、队列 drain 或性能容量。
6. Rust 1.99.0 正式质量门未运行。Rust 1.98.1 `--ignore-rust-version` 的 check/test/Clippy 仅为诊断证据。
7. D20 整体仍进行中；native provider 不可用于产品流量，NATS 继续保留，Schedule/Run execution 仍受原有授权、预算、Reservation、Hook、sandbox、取消恢复和独立验证门控制。
8. 用户确认可直接参考、适配或按模块复用 AssetsLake 与其他本机 Rust server；本 slice 未搬入源代码。当前 AssetsLake event publisher 在 pipeline disabled 时只返回生成 UUID，relay 在 dry-run/no-broker 时会标记 published、配置 broker 时因无 Kafka client 标记失败，因此不适合作为当前 durable transport；它的 envelope、trace、causation 字段仍可作契约参考。RustGameServer TLS helper 是既有已审查适配；不据此推断整仓依赖、商业分发/SBOM 或其他模块已验收。
9. 本轮 D20 source-verification SQL contract 尚未执行；Rust 诊断通过、源码存在和自动化 runner 接入都不能替代 PostgreSQL 18.6 disposable schema test。切流和执行继续 fail closed。

## §4 子代理失败接手清单

本阶段未派发子代理，无失败或散落产出待接手。CodeRabbit 工具执行受到 permission denied 阻断，按独立 reviewer 未完成处理，不将其归入通过项。

## §5 守门规则（17 项）

| # | 守门规则 | 本阶段状态 |
|---:|---|---|
| 1 | 以 Project/Branch/Run owner 边界决定职责 | 通过；adapter 不写 owner 事实 |
| 2 | source reuse 先逐模块适配，不整仓搬迁；用户已授权合适自有模块直接复用 | 通过；权限已确认，D20 当前 AssetsLake relay 语义不合格故未迁入 |
| 3 | 对许可、NOTICE、权属和完整依赖闭包逐版本审查 | 仍需每次新增源文件/发布时单独验收 |
| 4 | mTLS 证书身份是 peer 来源认证基础 | 继承 D19；本轮无 live handshake |
| 5 | Profile digest 与 endpoint/profile revision 必须固定 | 代码通过；production resolver 未接入 |
| 6 | recipient 来自服务端已认证 principal，不信 wire claim | proto/server 路由契约遵守；无 e2e |
| 7 | owner commit 和 commit 后授权复验先于 ACK | 共享 receiver 已定义；本轮无 production handler 组合 |
| 8 | permanent ACK 前先提交独立 durable quarantine | 共享 receiver 已定义；native DB 联测未做 |
| 9 | ACK 绑定 generation、sequence、message、lease 和 exact digest | client request 固定并发送这些字段；server live 验证未做 |
| 10 | digest 必须对原始 wire bytes 计算 | 代码已校验 exact SHA-256；10 项定向测试通过 |
| 11 | payload、frame 与身份字段有界 | payload 16 KiB、frame 32 KiB，字段有界校验 |
| 12 | 并行任务使用有界队列/背压/取消 | 8 RPC + 8 outstanding handle permits，无预取 queue；系统级预算未验收 |
| 13 | 未知/超时/错误时 fail closed | RPC/ACK 错误返回未确认；集成重试/取消未测 |
| 14 | provider-specific cursor 不泄漏为业务身份 | sequence/log/lease 留在 adapter delivery origin |
| 15 | NATS 不得在 replacement gates 前移除 | 通过；NATS 保留 |
| 16 | 需求、基本设计、详细设计与实现状态同步 | 通过；已同步 D20 origin proof 实现与 PostgreSQL blocker |
| 17 | Rust 1.99.0 正式工具链、PostgreSQL migration/RLS contract 与独立性能验收不得由诊断结果替代 | 未通过/未运行；PG binary 缺失，禁止宣称完成 |

## §6 签字栏

| 角色 | Mavis 审核记录 |
|---|---|
| 架构 | `架构师 (Mavis 接手 agent per DEC-008)`：检查 provider/owner 边界与用户确认的本机模块复用规则；D20 保持进行中 |
| SRE Lead | Mavis review：未验收；没有恢复、负载、RSS、tail latency 或公平性证据 |
| 平台 | Mavis review：仅确认 Rust 接收 adapter 切片与文档对齐；生产 grants、resolver 和 server composition 未验收 |
| 评审主持 | Mavis review：独立 CodeRabbit review 因权限错误未完成，自审不冒充独立 review |
| PM | Mavis review：阶段报告与 plan 均记录未关闭项；NATS/native cutover 不开放 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.4 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 校正 library tests 为 41/41、记录 Rust 1.98.1 Clippy-driver 检查与 integration test 文件格式现状；保持 PostgreSQL 合约未验证状态 | 隔离 Clippy-driver 检查完成并复核 D20 验证记录 |
| v0.3 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 exact-wire VerifyOrigin、append-time source certificate evidence、同事务 ACK 复验和 SQL runner 接入；把未运行的 PostgreSQL contract 明确记为阻塞，并复核 AssetsLake/RustGameServer 来源状态 | D20 source-verification 实装；用户重申可参考/复用自有 Rust 服务 |
| v0.2 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确本机 Rust server 可直接模块复用，解释 AssetsLake 当前 relay 不满足 durable delivery；记录公开 canonical provider identity helper 与实际工具链调用方式 | 用户再次确认 AssetsLake/本机 Rust 服务复用授权，并要求为 Star 自有服务替换 NATS 做准备 |
| v0.1 | 2026-10-12 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 D20 native receive adapter 实现、真实验证命令、剩余生产门与来源复用边界 | 用户授权参考/复用 AssetsLake 与其他本机 Rust server，继续推进 native transport |
