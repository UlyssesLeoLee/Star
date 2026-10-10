# DD-RUST-SERVER-REUSE-001 — 自有 Rust 服务端复用与原生消息替换

> v0.1 · 2026-10-11 · Ulysses — Mavis 接手审核
> 状态：已完成限定范围源码调查和设计选择；未迁移源码，未实现 native transport provider。

## 1. 范围与证据

用户授权参考复用本机其他 Rust 项目。此次限定于 AssetsLake、RustGameServer、ROPE_CS 的消息、Outbox、资源锁、通道与遥测模块；不是整机所有项目的穷尽审计。源仓库只读，其未提交内容和运行环境均保留。

| 来源 | 当前 HEAD | 许可证据与复用状态 |
|---|---|---|
| `E:/AssetsLake` | `0156aca56c0d7364120d7389ebe1ccf0cb64b750` | 根 LICENSE 为 AGPLv3；允许商业用途，复制/链接/分发仍需逐项确认权属、许可义务及依赖闭包。工作区有修改，因此以以下文件 hash 标识实际调查版本 |
| `D:/RustGameServer` | `d21d31a85653c5966184f0faaedfcdaaa57ea2f2` | 根 LICENSE 与 workspace 声明 Apache-2.0；复制时保留版权/NOTICE，并审查具体模块依赖。源仓库有其他脏文件 |
| `E:/ROPE_CS` | `3f4e54c595f3c486fd895b1a1456727b08931f86` | server manifest 声明 license-file；此次尚未验证对应许可证内容/分发闭包，暂作行为与设计参考 |

Star/AssetsLake/RustGameServer 未发现现有 CodeGraph 索引；ROPE 使用既有图查询，未覆盖的 message_bus 实现做定向源码补读。当前工具未提供 index_repository/check_index_coverage，不能声明图覆盖完整。未运行源项目测试，也不把源码中测试存在等同于通过。

### 实际工作文件 SHA-256

| 文件（相对各源仓库） | SHA-256 |
|---|---|
| AssetsLake `backend/src/services/event_publisher_service.rs` | `3A2114863827CB8CE893AD5A24090D827F89B7752C6FF23546DBDB4DC3B8A4A5` |
| AssetsLake `backend/src/services/resource_lock_service.rs` | `05D4EB1C000516FEB98C918A8029CD8B4F5392347BFC5F177F57911DBFCBCAFE` |
| AssetsLake `backend/src/services/rate_limit_service.rs` | `119125023E53B6BE7269AC515DAF98A963BB4117E2A801ECAB340D1D84FE3866` |
| RustGameServer `crates/shared-platform/src/channel.rs` | `8949F57996706547BC8E60022101B8B4AB854D3D0B4AB07CB9BA4252A7433061` |
| RustGameServer `crates/shared-platform/src/outbox.rs` | `43DA6FFDC5F1AFDAF892F1FF6D95DD01A0FBA699F9FB58289CC4DDD12040D274` |
| RustGameServer `crates/shared-platform/src/outbox_relay.rs` | `71530C8E84B0698BA31AFC877533CEDF37696F810A19C57D318EADF3ECBF2F06` |
| RustGameServer `crates/shared-platform/src/producer.rs` | `052EBBE4AB012E37F38DD32D9072F686FFE29E5B33071EDCD55ED39096131D53` |
| ROPE `server/rope_server/src/common/infrastructure/message_bus.rs` | `BAA9539CE34B224DF63E0AE461C1D78CD03A3B75E3148AB4852ABD5C2EC6B338` |
| ROPE `server/rope_server/src/services/telemetry_service.rs` | `D8511BA9C414E566937AD251CED566C8D96D16CA4BA8C11138FEE1DF4D5CB42A` |

## 2. 模块选择与必须改进的语义

| 模块 / 精确证据 | 选择 | Star 接入要求 |
|---|---|---|
| RGS `channel.rs:238–280` build_channel/retry_backoff | 优先适配候选：mTLS 默认拒绝无 TLS、连接/请求超时、错误分类退避、HTTP/2 通道复用 | 独立小模块；生产禁止明文 bypass；mTLS 身份再绑定 tenant/owner/recipient 授权，不能以 TLS 就绪代替业务授权 |
| RGS `outbox.rs:480–554` list_pending/mark_sent/mark_failed | 参考事务内 SKIP LOCKED 与崩溃后租约接管 | settle 必须核对 worker lease token/generation；源 mark_sent 仅按 id 更新，不能直接沿用。保留 Star 已实现的 owner-local RLS、容量、永久回执和 stale-worker fence |
| RGS `outbox_relay.rs:185–316` tick/run/publish_entry | 参考事件年龄、批量、重试和 cycle 指标 | 不从 NATS subject 推断领域身份；改用 typed owner/Run/event。补取消/drain、批量硬上限及消息字节预算；不引入整套游戏域、Wasm/网关依赖 |
| RGS `producer.rs:293–340` publish_bytes | 参考 traceparent 与错误退避；不直接复用现有发布实现 | 当前 timeout 只包裹得到 ack future，后续 ack_future.await 无同一 timeout；还会每次重试 clone payload。Star 必须对完整持久化回执等待设 deadline、验证 log identity/cursor，并保持有界字节驻留 |
| AssetsLake `event_publisher_service.rs:83–145,219–296` DomainEventInput/stage/relay_once | 参考版本、trace/causation、幂等键字段 | stage 当前通过 pool 单独 INSERT，不能证明与业务同事务；disabled 返回新 UUID，dry-run/未配 Kafka 会 mark_published，真实 Kafka client 未链接。不得作为可用消息服务端或 durable acceptance 复用 |
| AssetsLake `resource_lock_service.rs:115–198` acquire | 参考 session/token hash、TTL、唯一约束争抢冲突 | Star 使用当前授权、owner/Run scope、事务与 CAS fencing；该读取/更新组合不是完整租约 fence 证明，不能直接替换 Worktree lock |
| AssetsLake `rate_limit_service.rs:122–159` check | 参考限流响应与时间窗 | mutex 中 retain 全 map，key 数量无硬上限且零配置跳过限流；Star 需 bounded key cardinality、公平预算、分片/增量清理，避免 UI/多 Agent 下全表扫描 |
| ROPE `telemetry_service.rs:229–323,326–354` new/ingest/replay_dead_letters | 参考 bounded queue、try_send、backlog/backpressure 指标 | register_dedup 先于 try_send，入队失败可能抑制重试；LPOP 后重新入队不能证明故障安全恢复。关键 Run/BI 事实以事务 Outbox/Inbox 接受成功为准，内存队列只作有界加速层 |
| ROPE `message_bus.rs:19–130` MessageBus/Noop/RedisStreamMessageBus | 参考 port 与 adapter 分离 | Noop 返回成功、Redis adapter 使用 unbounded channel，均不得进入 Star 权威消息路径；不把本机服务端等同于已有 Rust-native durable broker |

## 3. 目标接入架构

复用以独立模块为单位，先适配 RGS 通道/可观测模式，再实现 Star native provider。保持现有 `AdmissionTransport` 与 owner-local Outbox/Inbox；native 服务端仅拥有自身 transport persistence，不拥有 Schedule/Run 的业务写权限。不得跨 owner 共享写事务。

版本化 TransportProfile 固定 provider/version/digest、endpoint/server identity、协议与权限范围。发布确认必须来自已持久化的日志及有效 receipt；接收确认必须晚于目标 owner 的 Inbox 与业务结果提交。确认丢失后以稳定 message identity 重放，不能重新执行业务副作用。provider cursor 不跨端点/generation 混用；不能为 native server 强制模拟 JetStream。

接收侧分离 transport pull/ack、授权 verifier 和 owner handler：每次检查 tenant、发送/接收服务、kind 与 typed wire；handler 返回已提交事实，确认前复验授权。DB 失败、撤权、取消、未知身份都不得 ACK。毒消息先写受控持久化 quarantine，再按明确策略终结；不存在 quarantine provider 时保持失败关闭。

性能沿用 Star 的有界调度：发布 batch ≤8 串行、接收先以一条 in-flight 实现，wire 字节上限、连接数、租户并发、持久化容量、缓存和指标标签基数均显式预算；跨租户公平调度，取消停止新 claim、drain 在 deadline 内结束。Desktop 只消费分页投影与有界事件，不在 UI 保存完整消息日志；连接/runtime 不按任务卡重复创建。

Multipass/K3s 和最多两个测试 inventory 实例约束不变；本轮没有启动/修改虚拟机。native 服务可部署到明确绑定的 cluster，服务 readiness 不代表 Agent sandbox 或 executor 就绪。

## 4. 实施顺序和验收

1. D14：broker-neutral receiver + 两侧 owner handler；真实 DB 失败不 ACK、提交后 ACK、ACK 丢失幂等回放、撤权/取消/毒消息。测试授权 fixture 不替代生产身份提供器。
2. 原生 server provider：优先适配已核对的自有小模块；补 authenticated durable append/pull/ack、租户隔离、重启恢复和持久化故障证明，执行同一 transport 契约套件。
3. provider 依赖隔离：Cargo feature/独立 adapter crate；native 构建闭包不要求 NATS，业务 ABI 和 owner 数据事实保持稳定。
4. 故障/性能与商业分发验收：完整 ack deadline、租约接管 fence、容量拒绝、取消 drain、恢复后 cursor、资源/标签基数与并行公平性；生成实际分发闭包 SBOM/NOTICE。未通过前不得宣称原生服务器替换完成或任务执行可用。

本轮为设计与来源证据更新，没有新运行时实现/测试通过声明；D13 已有验证不能代替 D14/native server 验收。

## 5. 修订历史

| 版本 | 日期 | 修订人 | 改动 / 触发 |
|---|---|---|---|
| v0.1 | 2026-10-11 | Ulysses — Mavis 接手审核 | 用户授权参考复用本机 Rust 服务端；记录实际源版本、选择性适配与 D14/native 验收边界 |
