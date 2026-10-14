# PHASE-9F4C-D20 P2 JetStream / Native Route-Pin Diagnostics Report

> v0.1 · 2026-10-15 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> 状态：🟡 D20 P2 部分诊断切片通过；P2、生产启用及 NATS cutover 均未关闭，Run/Schedule execution 保持 fail closed。
> 依据：`docs/briefs/D20-P2-CROSS-PROVIDER-20261015.md`；实现合入 `dev` 的 commit `d15951f9d2c720127899ebdd03df868a205be968`。

## §0 目的

记录 JetStream 安全跳过 native-pinned Outbox event 时，通过受信 route snapshot 获取有限跨 provider route-pin diagnostics 的实现与数据库验证。报告区分可观测性代码切片、operator surface 和生产/恢复闭环。

## §1 改动矩阵

| 范围 | 实际交付 | 状态 |
|---|---|---|
| JetStream 诊断 | feature-enabled seam 接受 matching-tenant、resolver-produced `NativeInfrastructureDeliveryRoute` snapshot；结果区分 `Available` 与 `Unavailable`，不把缺少 snapshot/feature 当作零阻塞 | 已合入 |
| 投递隔离 | JetStream 继续排除 native-pinned event；诊断不 claim、不 publish 该 event，也不提供 provider fallback | 已验证 |
| 持久化/W/T/M | 不新增 migration；既有 Infrastructure owner 分类为 20 authored tables = 4 W / 10 T / 6 M，其中 D20 有三张 tenant Transaction 表；不是全仓 100-table 重审 | 保持现有分类 |
| operator 与恢复 | 授权 operator API/UI、verified operator role/tenant binding、recovery command、terminal disposition | 未实现，D20 P2 开放 |

## §2 验证摘要

- Rust 1.99 default `cargo check -p infrastructure --lib --offline --locked -j 4`：PASS，4m34s；3 条现存 `Nonce` deprecation warnings。
- Feature-enabled `cargo check --tests`：PASS。
- Infrastructure library tests：76/76 PASS。
- 两次 disposable PostgreSQL 18.6 runner：每次 audit 95/95 passed，quarantine migration 各应用两次。可核实调用记录为 `scripts/automation/phase9f4d_run_owner_consumer.py`、`--schema-only --native-result-tests`，输出目录分别为 `target/d20-cross-provider-pg-run` 与 `target/d20-cross-provider-pg-run-2`。原始完整 runner argv 未被 audit JSONL 持久化；上述命令信息依据当时 invocation record 与输出目录回溯，不声称从 audit 导出。
- 每次 runner 后执行 `cargo test --offline --locked -p infrastructure --test infrastructure_native_result_delivery_pg --features native-grpc-test-support -j 4 -- --ignored --test-threads=1`：退出码 0；每次 2/2 tests passed。用例覆盖 JetStream stale native pin diagnostic without claim/publish，以及 native route/receipt/RLS/WORM/replay/attempt-fence 场景。
- Formatting、`git diff --check`、Cypher Guardian：PASS（实现 lane 针对最终合入代码 SHA 的结果）。
- CodeRabbit CLI：WSL permission denied，未得到 review result；不得称其通过。
- 上述是 slice-specific evidence；没有运行 20/20 全累计 migration gate、生产 target DB/grants、NATS broker、native server append、live TLS 或 production traffic。

## §3 已知缺口

1. `Available` 依赖启用 feature 与匹配 tenant 的 trusted resolver route snapshot；缺失、错配或 feature disabled 时返回 `Unavailable`。调用方不得将其解释为 0 blocked events。
2. JetStream 继续 skip native-pinned events；不 claim、不 publish，也不 fallback。跨 provider diagnostics 是有限 seam，不是完整 operator observability。
3. 当前没有 Infrastructure operator API/UI；`AuthUser.scope` 由 token issuance 复制请求 scope，exact-token check 本身不能证明授权来源。未来必须绑定 verified client/user role provisioning 或独立 authenticated internal operator identity，并由可信 resolver 确定 tenant。
4. recovery、terminal disposition、operator capability separation、owner-local append-only transition、actor/audit 与 lease/attempt fencing 仍未接入或验收。
5. 目标 DB/runtime role grants/RLS 部署、production startup/Profile resolver、live TLS/revocation、BI/Benchmark、load/RSS/fairness、NATS rollback 均开放。Infrastructure 的 20 表分类不是全仓 live-schema 重审，也不是 20/20 累计迁移门通过。

## §4 子代理失败接手清单

实现 lane 的两次 disposable PG18.6 runner 与 SQLx 结果由实现 owner提供；本 docs lane 未重跑这些运行门。Runner 完整原始 argv 未在 audit JSONL 持久化，报告只写调用记录可核实的 script、flags、output directory 和 SQLx 命令。CodeRabbit 权限错误由实现 lane 报告，不能记作 clean review。

## §5 守门规则

1. diagnostics 必须由同 tenant、trusted resolver-produced snapshot 绑定；不接受 caller 伪造 tenant/route。
2. 缺失 snapshot/feature 明确返回 `Unavailable`，不能折叠成成功的零结果。
3. JetStream 永不 claim/publish native-pinned event，不得 fallback、retarget 或 dual-publish。
4. operator scope 字符串不能单独构成授权；必须验证权限来源及 tenant binding。
5. recovery 只恢复原 provider/Profile revision/digest/endpoint 与 exact wire；无法恢复时只能走未来授权、审计的 terminal disposition。
6. 此 slice 不关闭 D20 P2、production traffic、NATS cutover 或 Schedule/Run execution。
7. 不把 95/95 与 SQLx 2/2 写成全量 20/20 或生产数据库验收。

## §6 签字栏

| 角色 | 审阅状态 |
|---|---|
| 架构 | Mavis 接手审核；cross-provider seam 与 immutable route pin 边界一致 |
| SRE Lead | Mavis 接手审核；两次 disposable PG18.6 证据范围已明确 |
| 平台 | Mavis 接手审核；Rust/check/test 与 SQLx 逐项区分，CodeRabbit 无结果 |
| 评审主持 | Mavis 接手审核；P2 gates 与 authorization gaps 保持开放 |
| PM | Mavis 接手审核；未将 operator recovery 或 production readiness 标为完成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容/触发 |
|---|---|---|---|
| v0.1 | 2026-10-15 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 D20 JetStream cross-provider route-pin diagnostics 的有限语义、Rust/PG/SQLx 实证、argv 可追溯性限制与未关闭的 operator/恢复/生产门；实现合入 dev 后同步文档 |
