# PHASE-9F4C-D16-ADMISSION-QUARANTINE-REPORT

> v0.2 · 2026-10-12 · Ulysses — Mavis 接手审核
> 状态：provider-neutral PostgreSQL metadata receipt 已实现；Rust 1.98.1 诊断测试与 PostgreSQL 18.6 disposable schema-only migration/RLS/role/WORM gate 通过。Rust 1.99.0、SQLx transaction cases、production DB credentials/grants、生产接线、原始 wire 恢复和完整 Schedule/Run execution 仍开放。

## §0 目的

给 D14 的 broker-neutral poison receive port 接上可持久化的 admission quarantine metadata writer，并使其可在不启用 JetStream SDK 的 crate 构建组合中使用。此切片不实现 Rust-native message server，不将本地服务端代码整仓搬入 Star，也不关闭 9F4C 或 Agent/任务卡执行 phase。

## §1 改动矩阵

| 范围 | 改动 |
|---|---|
| Rust writer | 新增 `PgAdmissionReceiveQuarantine`，实现 `AdmissionReceiveQuarantine`；deployment/provider/log-generation/log/cursor 作稳定来源键；精确核对路由、fault、版本、wire bytes 与 SHA-256，精确重放幂等，冲突 fail closed |
| Cargo boundary | 新增 `durable-quarantine` feature；`jetstream-adapter` 组合启用它，`--no-default-features --features durable-quarantine` 不包含 `async-nats` |
| PostgreSQL | 新增 `platform_messaging.admission_receive_quarantine_receipt` append-only Transaction 表；W/T/M=0/1/0；deployment+tenant FORCE RLS、ingress owner SELECT/INSERT、WORM 更新/删除/截断保护 |
| Integration target | 扩展 `schedule_admission_pg`，包括 exact replay、wire digest/fault conflict、不同 provider namespace、cross-tenant conflict 与 receipt cardinality；本轮仅编译该 SQLx test target，未执行事务场景 |
| Schema test/runner | 增加 FORCE RLS、权限和 trigger assertions；runner 在共享 WORM function migration 后应用 D16 migration，并对完整 migration chain 做 schema-only 验收 |
| 源模块复用 | 更新 `DD-RUST-SERVER-REUSE-001` 与上位需求/基本/详细设计：E:/AssetsLake 和其他本机 Rust 服务可按模块直接参考/适配/复用；AGPL-3.0 不等于商业使用禁令，须履行具体交付方式适用义务。AssetsLake 当前 dry-run relay 没有真实 Kafka 投递，不作为生产 transport。本 D16 receipt 未复制外部源码，因为其 RLS/idempotency/WORM 契约由 Star 所有 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `cargo test --offline --locked --ignore-rust-version --no-default-features --features durable-quarantine -p platform-messaging-runtime --lib -j 4` | PASS，16/16；本机 Rust 1.98.1 诊断运行 |
| `cargo check --offline --locked --ignore-rust-version --no-default-features --features durable-quarantine -p platform-messaging-runtime --all-targets -j 4` | PASS；Rust 1.98.1 诊断运行 |
| `cargo check --offline --locked --ignore-rust-version -p run-owner-runtime --test schedule_admission_pg --all-targets -j 4` | PASS，测试目标编译；没有执行 PostgreSQL 测试 |
| `cargo tree --offline --locked -p platform-messaging-runtime --no-default-features --features durable-quarantine` | PASS；此 crate feature dependency tree 不含 `async-nats` |
| `phase9f4d_run_owner_consumer.py --schema-only`（PostgreSQL 18.6 disposable loopback） | PASS；完整 migration chain 已应用一次；D16 migration、RLS、ingress SELECT/INSERT 权限、WORM/TRUNCATE 与 9 个 quarantine trigger assertions 通过；D16 migrations 未在本次 runner 重复应用，幂等性未单独验证；audit `target/phase9f4d-run-owner-consumer/3c264b37-fd6f-479d-99f0-f6a97387ee12/audit.jsonl`；实例已停止清理，Multipass inventory 未改动 |
| D16 SQLx transaction cases / production DB credentials and grants | 未运行/未验证；schema-only 只证明 disposable catalog/权限后置条件，不证明 adapter 事务、production identity 或目标库 provisioning |
| Rust 1.99.0 正式门禁 | 未运行；当前只能使用 Rust 1.98.1 并带 `--ignore-rust-version` |
| `check_index_coverage` | MCP 工具当前未提供；已定向查询 CodeGraph 并同步新增/修改代码索引，不宣称覆盖率完整 |
| RTK wrapper / CodeRabbit | RTK 不在 PATH；CodeRabbit wrapper 既有调用失败为 Permission denied，未形成独立审查结果 |

本轮 `cargo fmt -p platform-messaging-runtime -- --check` 曾发现既有、未修改 `admission_delivery.rs` 格式差异；新模块已单独用 rustfmt 格式化。不得把整 crate fmt 标为全绿。没有运行实际 NATS/PostgreSQL end-to-end、workspace/release/doc/bench、完整应用依赖闭包或资源性能测量。

## §3 已知缺口

1. 新 migration 与真实 SQLx transaction、role grants、双重 RLS scope、WORM triggers、concurrent exact replay/conflict 尚未在 PostgreSQL 执行。
2. Rust 1.98.1 `--ignore-rust-version` 是诊断证据，不替代 workspace 要求的 Rust 1.99.0 验收。
3. Writer 尚未接 production source identity verifier 或 receiver composition；当前没有生产策略注册和可信 log-generation resolver。
4. Receipt 不保存 payload body；无法单凭该表恢复原始 wire。若产品要求 poison body replay，需单独设计加密、有额度和 TTL 的 Work storage、访问审计与安全恢复流程。
5. Durable receipt 成功之后的 terminal disposition、ACK 丢失/重启恢复、跨 provider redelivery 和 operator replay 尚未验证；source authentication/revocation 仍必须前后复验。
6. BI/Benchmark event projection/outbox 未接线；不要跨 owner 直接查询或修改业务表来替代授权接口/版本化事件。
7. `durable-quarantine` 只证明 `platform-messaging-runtime` crate 组合隔离；full application 可能仍经其他 crate 依赖 NATS，NATS 仍是当前默认兼容 adapter。
8. 本阶段不实现 native Rust provider、服务身份、生产 profile resolver、bounded multi-tenant fairness/throughput/RSS/内存预算实测、Multipass/K3s provider 或 executor readiness；Run execution 仍 fail closed。
9. D16 migrations 在 schema-only runner 中各应用一次；迁移重跑幂等性和 D16 SQLx transaction scenarios 仍需单独验证。

## §4 子代理失败接手清单

本切片未派发子代理；无 worker branch、未回收 patch 或失败 handoff。工作位于隔离 worktree `codex/phase9f4c-admission-quarantine`。`E:/AssetsLake`、`D:/RustGameServer` 等本机源项目保持只读；CodeGraph 本地索引目录不纳入交付。

## §5 守门规则

1. Worktree/Project/Run owner 边界与本机 Multipass/K3s 拓扑不变；当前测试 inventory 最多两条。
2. Quarantine source key 包含 deployment、provider、可信 generation、log 和 cursor，不能跨 generation/provider 比较 cursor。
3. Exact replay 仅接受完整不可变字段一致；未知或冲突结果 fail closed。
4. Tenant 与 deployment 两个 RLS scope 均需匹配；ingress runtime role 仅获得 SELECT/INSERT。
5. Receipt 为 append-only Transaction；没有 Work payload、TTL purge 或 Master policy。
6. 元数据摘要不等于加密原始 payload，也不提供可重放 body。
7. Source identity 必须由受信 verifier 独立认证，不能信任请求体或注入 UUID。
8. 数据库错误、权限缺失和 commit 结果未知时，源 delivery 不 terminally disposition。
9. Disposable schema-only gate 通过仅证明已执行 migration 与 schema assertions；不得据此宣称 SQLx transaction cases、production credentials/grants 或目标库 rollout 通过。
10. Rust 1.98.1 `--ignore-rust-version` 检查不替代 Rust 1.99.0 正式门禁。
11. 单 crate Cargo tree 不含 NATS，不证明完整应用分发闭包不含 NATS。
12. 本机 Rust 模块可直接复用，但先核验确切文件、源状态、权属/许可证、NOTICE、依赖闭包和语义差异；源工作区保持只读。
13. AGPL-3.0 不视为商业用途禁令；必须按具体修改、链接、部署/网络提供方式履行许可证义务。
14. dry-run、Noop、stub、内存队列和编译通过不作为生产消息交付证明。
15. 本地实例/VM/transport readiness 不能替代独立 Agent Runtime sandbox、原生 Hook、取消恢复与验证门。
16. D16 是局部切片，不把它或本阶段报告标为 9F4C/产品全目标完成。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis：provider-neutral receipt、owner 与 transport 边界已按实际代码审核；disposable PostgreSQL schema gate 通过，production DB 接线仍开放 |
| SRE Lead | Mavis：有界 metadata-only 写入与冲突 fail-closed 已实现；schema/WORM assertion 通过，恢复/容量/性能未验收 |
| 平台 | Mavis：crate 级无 NATS feature 组合与 PostgreSQL schema-only gate 通过；生产授权、完整 app 闭包及 Rust 1.99.0 未验收 |
| 评审主持 | Mavis：静态审查、限定 Cargo 检查及 PG schema-only runner 完成；SQLx transaction 与独立 CodeRabbit review 未通过/未形成结果 |
| PM | Mavis：持续推进全部 Worktree-first 产品 phases，不缩减为 D16 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 / 触发 |
|---|---|---|---|
| v0.1 | 2026-10-12 | Ulysses — Mavis 接手审核 | D16 provider-neutral PostgreSQL metadata quarantine、Cargo 隔离、源模块复用策略与未验收缺口 |
| v0.2 | 2026-10-12 | Ulysses — Mavis 接手审核 | 修复 schema test 的 9-trigger assertion 并记录 PostgreSQL 18.6 disposable schema-only migration/RLS/role/WORM gate 通过；注明 D16 migrations 单次应用且未验证重跑幂等性；保留 SQLx transaction、Rust 1.99.0 与生产集成缺口 |
