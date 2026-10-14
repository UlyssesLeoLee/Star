# PHASE-9F4C-D20-NATIVE-PUBLISHER-REVIEW

> **状态**：独立只读源码审查；仅新增此报告。审查基线 `61ee9c0f69313d45f6259cfd565df20c8fe688ef`。不代表 D20 或 native transport 已生产就绪。

## §0 目的

核查 D20 exact-append client、Infrastructure native Result publisher、PostgreSQL route/wire pin 与 receipt、lease fencing、JetStream 边界和 Run-owner Inbox/ACK 顺序。设计基线为 [实施计划 §6.138](../implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md#6138--phase-9f4c-d20-native-exact-append-and-owner-publisher-source-slice) 与 [D20 Native Receiver 报告](PHASE-9F4C-D20-NATIVE-RECEIVER-REPORT.md)。

本次结论只针对基线中的源码和迁移文本，不验证 PostgreSQL 实例、生产身份/Profile resolver、TLS、部署启动、BI/Benchmark 或运行期恢复。

### CodeGraph 与源码覆盖

父会话的图查询报告索引状态为 up-to-date（1,837 files），但精确 native publisher 符号未返回；本 lane 的 CodeGraph MCP 返回目标 worktree 没有 `.codegraph/`，且当前没有 `check_index_coverage` 工具。因此不声称图覆盖完整；按限定路径用 `rg` 与逐段源码读取复核，迁移文件也逐段读取。负向结论只限于本报告列出的仓库范围。

## §1 审查矩阵

| 范围 | 源码证据与结论 |
|---|---|
| Native exact append | `NativeAppendTransport` 只接受 Result append；客户端限制 wire 为 16 KiB、拒绝 nil identity、对精确 bytes 计算 SHA-256，并要求响应 digest、log id、generation、sequence 合法。[native_append.rs:34-82](../../crates/platform-messaging-runtime/src/native_append.rs) [native_grpc.rs:1062-1122](../../crates/platform-messaging-runtime/src/native_grpc.rs) |
| 首次 route/wire 持久化与重试 | `get_or_pin_native_wire` 先核验当前 lease fence；首发时在同一数据库事务写入 route/profile/recipient/result 和 exact payload/SHA，commit 后才把 bytes 返回给调用方。重试读取已 pin 的 bytes，并复算 SHA、核对固定 route。[infrastructure_owner_outbox_delivery.rs:535-639](../../crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs) |
| Receipt 与 attempt fence | publisher 核对 profile、SHA、log id 格式、非空 generation、正 sequence；delivery store 再核对 pin 的 profile 与 digest，在同一事务插入 append receipt 并按 lease owner、attempt_count 和未过期 lease 更新 delivered。[publisher.rs:257-266,285-301](../../crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs) [outbox_delivery.rs:644-752](../../crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs) migration 的 receipt trigger 复验当前 route 和 lease/attempt，表启用 FORCE RLS 并用 WORM trigger 阻止更新/删除/截断。[migration.sql:81-133,159-247](../../db/migrations/2026-10-14-infrastructure-native-result-delivery.sql) |
| JetStream compatibility boundary | JetStream 在 claim 事务内 pin `nats-jetstream`；native claim 只选未 pin 或完全匹配当前 native route 的行，NATS claim 排除 native pin。两路没有跨 provider fallback。[outbox_delivery.rs:278-350,447-514](../../crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs) |
| Run-owner Inbox/ACK | Run-owner 先对 exact origin 做授权验证，持久化 Inbox 结果，再复验 origin，最后 ACK；冲突只有先完成 durable quarantine、复验 origin 后才 terminal disposition。[native_capacity_result_consumer.rs:263-299,353-427](../../crates/run-owner-runtime/src/native_capacity_result_consumer.rs) [capacity_result.rs:532-579](../../crates/run-owner-runtime/src/capacity_result.rs) |
| 日志与错误边界 | 本次核查的 append client、publisher、delivery store、wire builder 与 JetStream publisher 中没有 payload/secret 日志调用；native append error 为不含 wire 或 credential 的固定错误值。[native_append.rs:54-69](../../crates/platform-messaging-runtime/src/native_append.rs) |
| fail-closed / 未启用项 | publisher 是显式调用、无后台任务；模块只在 `native-grpc-adapter` feature 下暴露，限定 `crates/` 搜索未发现生产启动/装配调用。两路 wire builder 都拒绝 `capacity_released`；本切片未实现 BI/Benchmark 写入。[publisher.rs:124-160](../../crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs) [lib.rs:405-415](../../crates/infrastructure/src/lib.rs) [wire.rs:52-91](../../crates/infrastructure/src/infrastructure_owner_outbox_wire.rs) [jetstream_publisher.rs:173-190](../../crates/infrastructure/src/infrastructure_owner_jetstream_publisher.rs) |

## §2 Findings

### [P1] Native route 没有绑定批次 tenant，错配调用会跨 tenant 投递

`NativeInfrastructureDeliveryRoute` 固定 profile、sender、recipient，但没有 tenant/deployment 身份。[outbox_delivery.rs:127-141](../../crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs) `publish_due_for_tenant` 则把独立传入的 `tenant_id` 用于 claim，却把该 claim 的 wire 交给固定 route 的 recipient；方法没有验证这两者属于同一 tenant。[publisher.rs:153-178,210-219](../../crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs) [wire.rs:77-86](../../crates/infrastructure/src/infrastructure_owner_outbox_wire.rs)

Native server 的 tenant 来自 mTLS certificate identity，随后只在该身份的 tenant 下解析 profile 和 route；append RPC 本身不携带 claim tenant。[native-server-auth.sql:322-361](../../db/migrations/2026-10-12-platform-transport-z-native-server-auth.sql) [native_grpc_server.rs:169-205](../../crates/platform-messaging-runtime/src/native_grpc_server.rs) 因此若调用方把 tenant A 的 Outbox claim 交给 tenant B 的已连接 route，wire 会被写入 B 的 native log。Run receiver 会在读取后发现 envelope tenant 不匹配并 quarantine，但数据届时已进入 B 的收件日志。[native_capacity_result_consumer.rs:253-260](../../crates/run-owner-runtime/src/native_capacity_result_consumer.rs)

**影响**：启用该 slice 前必须保证 publisher 的批次 tenant 与证书身份、profile、sender/recipient route 一致。目前代码没有强制这一约束；由于没有生产启动装配，尚无证据表明当前生产流量可触发它。

**建议**：把 tenant（以及部署身份，如适用）纳入不可变 route snapshot，由可信 resolver 构造；在 claim 前拒绝 `tenant_id != route.tenant_id`，并确保 endpoint/profile resolver 的 tenant 与 route pin 一致。补充跨租户负例与实际受限身份数据库验证后再接入启动路径。

### [P2] 已 pin 的 native route 与当前 route 不匹配时，事件静默滞留

`claim_native_due` 通过 `NOT EXISTS` 只挑选未 pin 或与当前 route 完全匹配的记录；route/profile mismatch 记录被过滤，不会返回 `RouteConflict`。[outbox_delivery.rs:447-477](../../crates/infrastructure/src/infrastructure_owner_outbox_delivery.rs) publisher 的 batch outcome 只反映返回的 claim 数量，没有 mismatch/blocked 计数或诊断事件。[publisher.rs:96-111,169-182](../../crates/infrastructure/src/infrastructure_owner_native_result_publisher.rs)

当部署只装配了新 Profile revision/endpoint 时，指向旧 pin 的 pending/retry 或过期 leased event 无法由这个 publisher 领取，也不会被本路径推进到 dead/quarantine；外部只会看到空批次。拒绝跨 provider 重发是正确的安全边界，但当前 API 未解析原 pin，也未显式呈现阻塞原因。

**建议**：按 AC-D20-INFRA-ROUTE-PIN-OBS-06/RECOVERY-07/DISPOSITION-08 落地跨 provider 盘点所有 native-pinned event、区分 empty-eligible 与 route-mismatch-blocked、bounded tenant/opaque-route aggregates、分页 event status 和新的 authenticated operator API/UI。operator permission 必须来自 verified client/user role provisioning 或独立认证的内部 operator identity；不能只相信请求 scope 字符串。tenant 必须经 verified user membership or registered client/service-identity tenant-binding resolver 推导；新增分离的 read/recover/dispose least-privilege capabilities。当前 star-api-rest 无对应 Infrastructure route/scope。数据库 quarantine table 或 RLS read seam 不足以关闭 P2。优先复验并恢复原 pinned Profile revision/endpoint，再以相同 message identity/exact bytes 重试；无法恢复则保持 operator-blocked 并等待 owner-authorized audited terminal disposition。禁止改 pin、跨 provider fallback 或 dual publish；替代业务结果必须由 origin owner 创建新 event/new pin。

## §3 已知缺口与限制

1. 新增 migration 没有在本审查 lane 执行；另一独立 lane 正在做 SQLx/migration 验证，本报告不引用或覆盖其结果。计划文档明确既有 83/89 步 evidence 不覆盖这两张新表。[实施计划 §6.138](../implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md#6138--phase-9f4c-d20-native-exact-append-and-owner-publisher-source-slice)
2. 未验证 production identity/Profile-to-endpoint resolver、mTLS live handshake/revocation、runtime grants、数据库 RLS 实例行为、crash/replay/recovery、性能/RSS/fairness、BI/Benchmark parity 或 NATS rollback。
3. 没有运行测试、编译或数据库命令；此 lane 被限定为只读独立审查并且不可运行测试。源码里的单元测试只作为设计线索，不视为本次通过证据。
4. CodeGraph 未能证明目标符号覆盖完整；采用限定源码 fallback，不据此作全仓 negative/exhaustive claim。
5. 本报告检查了 identity/route 的代码传递边界，但不证明任何生产配置目前实际跨 tenant 错配；P1 是启用前必须由实现消除的源代码级边界缺口。

## §4 子代理失败接手清单

本审查 lane 未再派生子代理；无子代理失败或产出需要接手。数据库/SQLx 验证为独立 lane，本报告没有改动其文件或结论。

## §5 守门规则

| # | 守门项 | 本次结果 |
|---:|---|---|
| 1 | Provider route 在首发前固定，retry 沿用相同 route 与 exact bytes | 源码路径通过；route mismatch 的可见性见 P2 |
| 2 | Receipt 覆盖 profile、log identity、generation、sequence、digest | 客户端与 Infrastructure store 做格式/值检查，SQL trigger 复验 route 与 fence；未执行数据库行为 |
| 3 | stale lease/attempt 不得 ACK 或标 delivered | SQL 条件同时校验 owner、attempt、未过期时间；未执行 race/DB 行为测试 |
| 4 | Native 与 JetStream 不互为 fallback | source claim filters 与 delivery guards 均保持 provider pin 边界 |
| 5 | 跨 tenant route 必须 fail closed | **未通过**；见 P1，production assembly 保持关闭 |
| 6 | Profile 变更/撤销不能静默丢失已 pin event | **需补强**；见 P2 |
| 7 | 无 payload/secret 日志 | 限定源文件未发现日志调用；未做运行期 telemetry 审计 |
| 8 | capacity_released、BI/Benchmark、execution readiness 不得冒充已接通 | 两路 builder 拒绝 capacity_released；BI/Benchmark 和 production wiring 仍未实现 |

## §6 审查签字

| 角色 | 结果 |
|---|---|
| 架构 / 安全边界 | Mavis 接手审核：P1 tenant binding 阻断 native publisher 生产接入 |
| SRE Lead | Mavis 接手审核：P2 route mismatch 可观测性需解决 |
| 平台 / 评审主持 | Mavis 接手审核：未执行测试或数据库验收，不据此宣称可生产使用 |
| PM | Mavis 接手审核：D20 仍进行中；NATS 保留，Run/Schedule execution 保持 fail closed |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.2 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 P2 recommendation 收敛为跨 provider bounded observability、append-only blocked evidence、原 route 恢复和 owner-governed terminal disposition 合同；状态仍为未实现/生产阻断，未添加测试或运行证据 | D20 P2 docs lane 根据当前 source/review 证据补齐处置合同 |
| v0.1 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 独立审查 D20 native exact append/publisher；记录 tenant-route P1 与 pinned-route 可观测性 P2，并说明未做 migration/runtime 验证 | 用户要求并行 Worktree lane 审查 D20 publisher |

## §8 后续处置与复验（2026-10-14）

| Finding | 状态 | 处置证据 |
|---|---|---|
| P1 tenant-route mismatch | 已修复 | `8506766e` 将 deployment、tenant、Profile ID/revision 和 endpoint identity 绑定到 resolver-produced `ResolvedTransportEndpoint`；publisher 仅接收与该 snapshot 匹配的 `NativeGrpcAppendTransport`，claim 不再接受调用者单独指定 tenant。跨租户错配负例通过，Infrastructure library tests 76/76 通过。 |
| PostgreSQL owner-store acceptance | 已验证，范围受限 | 随后的 disposable PostgreSQL 18.6 runner 为 93/93，owner-store SQLx test 为 1/1；验证 migration 双次应用、RLS/WORM、attempt fence 与 receipt/delivery 原子性。该测试使用 resolver/profile fixture 和合成 receipt，不调用 publisher 或 native server；详见 [PG 验收报告](PHASE-9F4C-D20-NATIVE-PUBLISHER-PG-REPORT.md)。 |
| P2 stale route-pin observability | 基线未修复；production wiring blocker。目标合同已在本次 docs lane 明确，implementation lane 尚待复验 | 当前 native claim 会过滤与当前 route 不匹配的旧 pin；只有调用 native publisher 才能看到其零 claim，而 JetStream 会安全跳过 native-pinned event 且没有 operator-visible cross-provider status。star-api-rest 当前没有 Infrastructure operator route/scope；且 requested scope token 不能替代已验证的 client/user role provisioning。目标为新增可信 operator identity、verified user membership or registered client/service-identity tenant-binding resolver、分离的 least-privilege read/recover/dispose capabilities 与 owner-authorized idempotent append-only blocked 状态；只有原 route 恢复并重新授权后允许 same-route exact-wire retry。无法恢复必须由明确 terminal disposition 结束旧事件，不能标 delivered 或 retarget；新业务结果使用新 event identity/new immutable pin。即使 quarantine table/RLS read seam 已新增，也需 cross-provider diagnostics 和 authorized operator surface 验收。本次文档更新不是实现或运行证据。 |

本 addendum 只更新 finding disposition；不扩大原审查范围。目标数据库部署、真实 native append/TLS/revocation、production Profile resolver/startup、BI/Benchmark、recovery、资源与 NATS rollback 仍未验收。CodeRabbit 因 CLI 权限问题未运行成功，不记为独立 clean review。
