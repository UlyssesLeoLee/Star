# PHASE-9F4C-D16-ADMISSION-QUARANTINE-REPORT

> v0.3 · 2026-10-12 · Ulysses — Mavis 接手审核
> 状态：provider-neutral PostgreSQL metadata receipt 已实现；Rust 1.98.1 诊断测试与 PostgreSQL 18.6 disposable schema、migration replay 和 SQLx receipt transaction gate 通过。Rust 1.99.0、production DB credentials/grants、生产接线、原始 wire 恢复和完整 Schedule/Run execution 仍开放。

## §0 目的

给 D14 的 broker-neutral poison receive port 接上可持久化的 admission quarantine metadata writer，并使其可在不启用 JetStream SDK 的 crate 构建组合中使用。此切片不实现 Rust-native message server，不将本地服务端代码整仓搬入 Star，也不关闭 9F4C 或 Agent/任务卡执行 phase。

## §1 改动矩阵

| 范围 | 改动 |
|---|---|
| Rust writer | 新增 `PgAdmissionReceiveQuarantine`，实现 `AdmissionReceiveQuarantine`；deployment/provider/log-generation/log/cursor 作稳定来源键；精确核对路由、fault、版本、wire bytes 与 SHA-256，精确重放幂等，冲突 fail closed |
| Cargo boundary | 新增 `durable-quarantine` feature；`jetstream-adapter` 组合启用它，`--no-default-features --features durable-quarantine` 不包含 `async-nats` |
| PostgreSQL | 新增 `platform_messaging.admission_receive_quarantine_receipt` append-only Transaction 表；W/T/M=0/1/0；deployment+tenant FORCE RLS、ingress owner SELECT/INSERT、WORM 更新/删除/截断保护 |
| Integration target | 扩展 `schedule_admission_pg`，覆盖 exact replay、wire digest/fault conflict、不同 provider namespace、cross-tenant conflict 与 receipt cardinality；修复 runner prerequisite 后实际执行 `admission_receipt_replay_conflict_rollback_and_scope`，1/1 通过 |
| Schema test/runner | 增加 FORCE RLS、权限和 trigger assertions；runner 加载 D15 transport-profile prerequisite，重跑两份 D16 quarantine migrations，并双次应用 D15 migration |
| 源模块复用 | 更新 `DD-RUST-SERVER-REUSE-001` 与上位需求/基本/详细设计：E:/AssetsLake 和其他本机 Rust 服务可按模块直接参考/适配/复用；AGPL-3.0 不等于商业使用禁令，须履行具体交付方式适用义务。AssetsLake 当前 dry-run relay 没有真实 Kafka 投递，不作为生产 transport。本 D16 receipt 未复制外部源码，因为其 RLS/idempotency/WORM 契约由 Star 所有 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `cargo test --offline --locked --ignore-rust-version --no-default-features --features durable-quarantine -p platform-messaging-runtime --lib -j 4` | PASS，16/16；本机 Rust 1.98.1 诊断运行 |
| `cargo check --offline --locked --ignore-rust-version --no-default-features --features durable-quarantine -p platform-messaging-runtime --all-targets -j 4` | PASS；Rust 1.98.1 诊断运行 |
| `cargo check --offline --locked --ignore-rust-version -p run-owner-runtime --test schedule_admission_pg --all-targets -j 4` | PASS，SQLx 测试目标编译；实际 PostgreSQL 执行结果见下表 |
| `cargo tree --offline --locked -p platform-messaging-runtime --no-default-features --features durable-quarantine` | PASS；此 crate feature dependency tree 不含 `async-nats` |
| `phase9f4d_run_owner_consumer.py --schema-only --admission-tests`（PostgreSQL 18.6 disposable loopback） | PASS，68/68 steps；D16 两份 quarantine migrations 重跑通过，D15 transport-profile migration 双次应用通过，RLS/ingress SELECT+INSERT/WORM/TRUNCATE 与 9 个 quarantine trigger assertions 通过。最终 audit `target/phase9f4d-run-owner-consumer/0f8b23cf-1901-4a76-804b-50123914320c/audit.jsonl`；实例已停止且数据目录移除，Multipass inventory 未改动 |
| `schedule-admission-postgres-sqlx-integration` | PASS，`admission_receipt_replay_conflict_rollback_and_scope` 1/1；本机 Rust 1.98.1 诊断 shim 注入 `--ignore-rust-version`；不替代 Rust 1.99.0 正式验收 |
| 首次 SQLx runner attempt | FAIL，audit `target/phase9f4d-run-owner-consumer/2856e5d7-6b77-4cbc-94f9-8591f04036c7/audit.jsonl`；缺少定义 `automation.claim_admission_delivery_v2(...)` 的 D15 prerequisite migration（SQLSTATE 42883）。修正 runner 后完整重跑通过 |
| Rust 1.99.0 正式门禁 | 未运行；当前只能使用 Rust 1.98.1 并带 `--ignore-rust-version` |
| Cypher graph guard | `cgg_guard.py --root <repo> --changed --stamp` PASS，runner Python source header 已刷新；SQL header 另以 skill validator 定向检查 3/3，因为原生 `--changed` 不包含 SQL 扩展 |
| `check_index_coverage` | MCP 工具当前未提供；不宣称 CodeGraph 覆盖率完整 |
| RTK wrapper / CodeRabbit | RTK 不在 PATH；CodeRabbit wrapper 既有调用失败为 Permission denied，未形成独立审查结果 |

本轮 `cargo fmt -p platform-messaging-runtime -- --check` 曾发现既有、未修改 `admission_delivery.rs` 格式差异；新模块已单独用 rustfmt 格式化。不得把整 crate fmt 标为全绿。SQLx 集成使用诊断 shim 和 `--ignore-rust-version`；没有完成 Rust 1.99.0、workspace/release/doc/bench、完整应用依赖闭包或资源性能测量。

## §3 已知缺口

1. Rust 1.98.1 `--ignore-rust-version` 是诊断证据，不替代 workspace 要求的 Rust 1.99.0 验收。
2. Writer 尚未接 production source identity verifier 或 receiver composition；当前没有生产策略注册和可信 log-generation resolver。
3. Receipt 不保存 payload body；无法单凭该表恢复原始 wire。若产品要求 poison body replay，需单独设计加密、有额度和 TTL 的 Work storage、访问审计与安全恢复流程。
4. Durable receipt 成功之后的 terminal disposition、ACK 丢失/重启恢复、跨 provider redelivery 和 operator replay 尚未验证；source authentication/revocation 仍必须前后复验。
5. BI/Benchmark event projection/outbox 未接线；不要跨 owner 直接查询或修改业务表来替代授权接口/版本化事件。
6. `durable-quarantine` 只证明 `platform-messaging-runtime` crate 组合隔离；full application 可能仍经其他 crate 依赖 NATS，NATS 仍是当前默认兼容 adapter。
7. 本阶段不实现 native Rust provider、服务身份、生产 profile resolver、bounded multi-tenant fairness/throughput/RSS/内存预算实测、Multipass/K3s provider 或 executor readiness；Run execution 仍 fail closed。
8. SQLx test 使用 disposable test identity/fixtures；它不证明生产数据库角色配置、授权 publisher、目标库 migration rollout 或 production receiver composition。

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
| 架构 | Mavis：provider-neutral receipt、owner 与 transport 边界已按实际代码审核；disposable schema、migration replay 与 SQLx 1/1 通过，production DB 接线仍开放 |
| SRE Lead | Mavis：有界 metadata-only 写入与冲突 fail-closed 已实现；schema/WORM 和 SQLx receipt transaction 通过，恢复/容量/性能未验收 |
| 平台 | Mavis：crate 级无 NATS feature 组合和 PostgreSQL schema/SQLx gate 通过；生产授权、完整 app 闭包及 Rust 1.99.0 未验收 |
| 评审主持 | Mavis：限定 Cargo 检查及 PG runner 68/68 完成；CodeRabbit 独立审查未执行成功（Permission denied），不计为通过 |
| PM | Mavis：持续推进全部 Worktree-first 产品 phases，不缩减为 D16 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 / 触发 |
|---|---|---|---|
| v0.1 | 2026-10-12 | Ulysses — Mavis 接手审核 | D16 provider-neutral PostgreSQL metadata quarantine、Cargo 隔离、源模块复用策略与未验收缺口 |
| v0.2 | 2026-10-12 | Ulysses — Mavis 接手审核 | 修复 schema test 的 9-trigger assertion 并记录 PostgreSQL 18.6 disposable schema-only migration/RLS/role/WORM gate 通过；注明 D16 migrations 单次应用且未验证重跑幂等性；保留 SQLx transaction、Rust 1.99.0 与生产集成缺口 |
| v0.3 | 2026-10-12 | Ulysses — Mavis 接手审核 | 修复 runner 缺失 D15 prerequisite migration；记录 68/68 PostgreSQL steps、D16 migrations reapply、D15 double-apply、SQLx receipt transaction 1/1 与清理证据；明确 1.98.1 diagnostic、Rust 1.99.0/生产门未关闭 |
