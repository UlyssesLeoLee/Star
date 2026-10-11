# PHASE-9F4C-D15-TRANSPORT-PROFILE-REPORT

> v0.1 · 2026-10-12 · Ulysses — Mavis 接手审核
> 状态：Rust transport profile pinning 与 provider-neutral receipt v2 已实现为一个可编译切片；22/22 包级单测通过。PostgreSQL migration/owner gates 未运行，目标 Rust 1.99.0 未验收；native provider、生产 profile resolver 与完整 9F4C 仍开放。

## §0 目的

为未来由 Star 自有 Rust message server 替换 NATS 建立可迁移的持久化交付身份：首次 claim 固定 provider profile，重试保持同一 profile，终态 receipt 保存 provider-neutral durable acceptance proof。业务 Outbox/Inbox、owner 授权、幂等事实与 BI 契约继续由 Star 管理；本切片不实现 message server 或生产 provider selection。

## §1 改动矩阵

| 范围 | 实际变更 |
|---|---|
| Rust transport contract | 增加 `AdmissionTransportProfile`；delivery claim 返回 owner Work row 首次固定的 provider/version/digest/endpoint identity；`AdmissionTransportReceipt` 记录 log ID/generation、opaque cursor 与 exact-wire SHA-256；publisher 和 owner settlement 双重核验 profile 与 wire hash |
| JetStream adapter | 显式使用 `nats-jetstream` profile；把可信 stream generation 和正 sequence 映射为 generic receipt 字段；NATS subject、PubAck 与序号语义仍限制在 adapter |
| Owner SQL | 新 migration 将旧 receipt 列改名为 legacy evidence，增加 v2 transport fields；用 profile-aware `claim_admission_delivery_v2` / `settle_admission_delivery_v2` 替代旧入口；SQL 已写但未执行 |
| Tests | 添加 profile identity 与 exact-wire receipt 单测；PostgreSQL target 增加 profile mismatch 与 v2 receipt assertions，但本轮仅编译该 target |
| Local Rust reuse | 源仓保持只读。复核 RustGameServer Apache-2.0 候选模块和 AssetsLake AGPL-3.0 event envelope；不把 RustGameServer NATS producer/缺少 Star lease fence 的 settlement 或 AssetsLake 无 Kafka publisher 的 dry-run relay 当成可用 native transport。完整 source hashes 与边界见 `docs/design/DD-RUST-SERVER-REUSE-001.md` |
| Documentation | 更新 Schedule SRS/BD/DD、主基本设计、阶段计划、`AGENTS.md` 与本报告；区分实现、诊断编译和未执行数据库验收 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `cargo test --offline --locked --ignore-rust-version -p platform-messaging-runtime --lib -j 4` | PASS，22/22；Rust 1.98.1 诊断运行 |
| `cargo check --offline --locked --ignore-rust-version --no-default-features -p platform-messaging-runtime --all-targets -j 4` | PASS，Rust 1.98.1 诊断运行 |
| `cargo check --offline --locked --ignore-rust-version -p run-owner-runtime --test schedule_admission_pg --all-targets -j 4` | PASS，目标仅编译，未运行 PostgreSQL 测试 |
| `cargo tree --offline --locked -p platform-messaging-runtime --no-default-features` | PASS；依赖树没有 `async-nats` |
| `rustfmt --edition 2024 --check crates/platform-messaging-runtime/src/admission_delivery.rs` | PASS；未宣称整仓或测试文件格式全过 |
| `git diff --check` | PASS |
| `cgg_guard.py --changed --stamp` | PASS，检查 2 个源码文件 |
| `cargo clippy ... --all-targets -- -D warnings` | 未形成 lint 结果。共享 target 和阶段专用 target 两次均发生 E0514 compiler metadata mismatch；本机 `rtk`、Rust 1.99.0 完整工具链不可用，没有清理共享 Cargo 缓存 |
| `phase9f4d_run_owner_consumer.py --pg-bin 'C:\Program Files\PostgreSQL\17\bin' --schema-only` | BLOCKED；缺少 `initdb.exe`、`pg_ctl.exe`、`psql.exe`，迁移没有执行 |
| CodeRabbit `coderabbit review --agent -t uncommitted` | 未完成；已安装入口转调 `/root/.local/bin/coderabbit` 时返回 Permission denied，未获独立审查结论 |

Cargo/rustfmt 通过本机已安装工具链的绝对路径运行，因为 `rtk` 和 Cargo 均不在 PATH。Rust 1.98.1 配合 `--ignore-rust-version` 只提供诊断证据，不能替代 workspace 要求的 Rust 1.99.0 验收。`check_index_coverage` MCP tool 当前不可用；涉及的 Star 符号由 CodeGraph 定向探索并以当前源码复核，不声明索引覆盖完整。RustGameServer/AssetsLake 未运行测试，也未修改其工作区。

## §3 已知缺口

- D15 SQL 语法、迁移双次应用、legacy receipt preservation、function grants、RLS、WORM 和两个 owner 的 SQLx 行为均未实证；不能把 Rust target 编译等同于 migration pass。
- 生产 Transport Profile master/resolver、可信 endpoint/service identity 与授权尚未接入；当前 profile 由受信 composition 参数提供。
- 原生 Rust provider/server 不存在；客户端服务、durable log、cursor、持久化 dedupe、append/pull/ack、restart recovery、认证和 tenant isolation 尚未验收。
- Capacity-result、poison quarantine/replay、Infrastructure publisher/consumer 仍有 NATS 直连或 provider-specific gaps；no-default 仅证明 `platform-messaging-runtime` crate dependency seam，不证明全应用已脱离 NATS。
- Rust 1.99.0、strict Clippy、实际 PostgreSQL/NATS integration、workspace/release/doc/bench 和完整 package/dependency/SBOM gates 未通过本轮验证。
- 本地项目源代码复用不由“可读取”自动授权：AssetsLake 根许可为 AGPL-3.0；其当前 relay 未链接 Kafka client，dry-run/no-broker 路径会标记 published。RustGameServer 为 Apache-2.0，但其 producer 绑定 async-nats；复用仍须模块级审查、保留 NOTICE 并通过 Star transport contract。
- D15 不关闭 D14 durable quarantine/replay、整个 Schedule execution、Multipass/K3s capacity reservation、native Hook、Runtime sandbox、任务卡 CLI 独立验证或 BI/Benchmark phase。

## §4 子代理失败接手清单

本切片未派发子代理；没有 worker branch、未回收 patch 或失败交接项。当前代码/文档在隔离 worktree 的 D15 lane，E:\AssetsLake 与 D:\RustGameServer 源仓只读。

## §5 守门规则

1. Project → Branch → Engineering Run → local Worktree 的主导航与 owner 边界保持不变。
2. admission business wire、request/message identity 和 owner Outbox/Inbox facts 不绑定具体 broker。
3. 每条 Work 在首次 claim 固定 provider、version、profile digest 与 endpoint identity。
4. retry、lease reclaim 与 uncertain acceptance recovery 只在原 profile 内进行。
5. provider 切换前必须 drain 或显式迁移旧 profile 的未完成 Work。
6. 不跨 provider、endpoint 或 log generation 比较 cursor、ACK 或 dedupe window。
7. v2 receipt 必须核验 profile、可信 log generation、opaque cursor 与 exact-wire digest。
8. durable acceptance receipt 不表示 receiver Inbox commit、Run start 或 task execution success。
9. legacy v1 receipt 保留其原值和证据版本，不回填推测的 provider 身份。
10. JetStream subject、`Nats-Msg-Id`、stream/domain、sequence 和 PubAck 只能由 JetStream adapter 解释。
11. native server 要通过 authenticated append/pull/ack、持久化 dedupe、restart/replay 和 commit-before-ack 验收后才可启用。
12. Native queue、batch、payload、lease、连接和内存必须有硬上限、背压、取消和 drain 语义。
13. owner procedure 继续实施 tenant RLS 与 least-privilege；不得跨 owner 共享写事务。
14. PostgreSQL migration/owner tests 未执行前，不声明 DDL、grants、RLS、WORM 或 mismatch gate 通过。
15. `--ignore-rust-version` 诊断构建不代替 Rust 1.99.0 正式门禁。
16. Unit tests、no-default crate check 或编译-only integration target 不代替生产 provider、workspace、性能或端到端验收。
17. 参考/复用本机 Rust 服务端时固定源 HEAD、dirty state、hash、许可证、权属、依赖闭包与实现缺口；源工作区保持只读。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis：profile pinning 与 receipt v2 边界已核对；SQL migration 未实跑，阶段未关闭 |
| SRE Lead | Mavis：有界 claim 与 provider cutover 顺序已记录；native restart/failover/backpressure 未验收 |
| 平台 | Mavis：no-default crate 不含 async-nats；生产 resolver、接收端通用化和全应用依赖隔离仍开放 |
| 评审主持 | Mavis：Rust 单测/格式/graph guard 通过；Clippy 与 PostgreSQL 门禁未形成通过结果 |
| PM | Mavis：保持完整 9F4C/产品 phases 目标，没有将 D15 切片当整体交付 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 / 触发 |
|---|---|---|---|
| v0.1 | 2026-10-12 | Ulysses — Mavis 接手审核 | D15 profile-pinned transport proof 实施、验证证据与剩余 PostgreSQL/native provider 缺口 |
