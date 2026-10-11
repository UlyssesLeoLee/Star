# DD-RUST-SERVER-REUSE-001 — 自有 Rust 服务端复用与原生消息替换

> v0.2 · 2026-10-12 · Ulysses — Mavis 接手审核
> 状态：已完成有界来源/许可审查和模块选择；未迁移源码，未实现 native transport provider。

## 1. 范围与证据

用户授权从本机 Rust 服务端项目参考并复用代码。此次核验 AssetsLake、RustGameServer、ROPE_CS 的消息/Outbox 候选，并以仓库级元数据筛查 Ada、CATs、GitGit、Physis；不是对电脑内全部 Cargo 项目的穷尽审计。所有源仓库只读，其未提交内容和运行环境均保留。直接复用仍须满足源仓库许可证和版权授权；“在本机可读取”不等于有权再分发或部署。

| 来源 | 当前 HEAD / 工作区 | 许可证与实际实现 | 决定 |
|---|---|---|
| `E:/AssetsLake` | `0156aca56c0d7364120d7389ebe1ccf0cb64b750`；17 个脏文件 | 根 LICENSE 为 AGPL-3.0。目标 `event_publisher_service.rs` 的 SHA-256 与本表一致；它有 envelope/outbox 字段，但当前 relay 无 Kafka client，dry-run 或未配置 broker 时会标记 published，且 stage 独立使用 pool，不能证明与业务写入同事务 | 仅参考事件字段；不直接复制。若以后引入源代码，须完成 AGPL 对 Star 分发/网络使用的履约与许可证兼容审查，或先取得单独授权 |
| `D:/RustGameServer` | `89458d887e8bfddf6637f9d80ea65f2091e8b6f2`；干净 | Apache-2.0。已核对的 channel/outbox/relay/producer 文件 SHA-256 与下表一致。它是 gRPC 游戏服务端；channel 模块是 mTLS 客户端构造器，不是消息 broker | 唯一当前许可清晰的模块级代码候选；可适配 mTLS/超时/指标模式。须移除明文 opt-out，并修正 owner lease fence 与完整 ack deadline 后才能接入 |
| `E:/ROPE_CS` | `3f4e54c595f3c486fd895b1a1456727b08931f86`；138 个脏文件 | `server/rope_server/LICENSE` 声明 proprietary/internal use，并禁止再分发或公开部署，除非获得 Say Co., Ltd. 明确许可 | 排除代码复制、改编和部署；先前候选结论由本版撤销。后续只有权利人明确书面授权后才能重新评估 |

补充候选仅作仓库元数据与适配度筛查，不作代码迁移授权：

| 来源 | 当前 HEAD / 工作区 | 许可证与适配度 | 决定 |
|---|---|---|---|
| `D:/Ada` | `1bb8d3f8334ed98c3fb45cb853035eff1a87f0cb`；2 个未提交生成文件 | AGPL-3.0-or-later 另附“任何使用、修改、分发或商业利用需版权持有人事先书面同意” | 不复用代码，除非取得该书面同意或完成权利人单独再许可 |
| `D:/CATs` | `9b97d6bc5996cc025c7970bce9f49b5e4685df53`；干净 | 根 LICENSE 为 AGPL-3.0，workspace manifest 声明 Apache-2.0，许可声明不一致；Kafka consumer/sink 当前是 stub | 不复制；先解决许可冲突，且 stub 不构成可用消息实现 |
| `D:/GitGit` | `4c063e34994677287e4a331d636eaef52b311af8`；16 个脏文件 | AGPL-3.0；本地 Git HTTP server/凭据保险库与本阶段 durable message transport 不匹配 | 不迁移代码；仅在具体 Git provider 需求出现时单独评估许可义务 |
| `D:/Physis` | `72ac7a8e6e10a30fe0978e64ae90b6c493c3814f`；干净、detached HEAD | MIT OR Apache-2.0；物理引擎而非服务端，现有 scheduler 的实际 worker pool 为单线程顺序执行 | 可作轻量 Rust API/内存布局的概念参考；不作为多代理并行调度实现 |

AssetsLake、RustGameServer、Ada、CATs、GitGit、Physis 未发现 `.codegraph` 索引；ROPE_CS 有既有图索引。本次对其指定符号运行了一次 CodeGraph 查询并直接读了目标源码；当前工具未提供 `check_index_coverage`，因此不声明图覆盖完整。未运行任何源项目测试，也不把源码中测试存在等同于通过。没有修改任何源仓库。

### 实际工作文件 SHA-256

| 文件（相对各源仓库） | SHA-256 |
|---|---|
| AssetsLake `backend/src/services/event_publisher_service.rs` | `3A2114863827CB8CE893AD5A24090D827F89B7752C6FF23546DBDB4DC3B8A4A5` |
| RustGameServer `crates/shared-platform/src/channel.rs` | `8949F57996706547BC8E60022101B8B4AB854D3D0B4AB07CB9BA4252A7433061` |
| RustGameServer `crates/shared-platform/src/outbox.rs` | `43DA6FFDC5F1AFDAF892F1FF6D95DD01A0FBA699F9FB58289CC4DDD12040D274` |
| RustGameServer `crates/shared-platform/src/outbox_relay.rs` | `71530C8E84B0698BA31AFC877533CEDF37696F810A19C57D318EADF3ECBF2F06` |
| RustGameServer `crates/shared-platform/src/producer.rs` | `052EBBE4AB012E37F38DD32D9072F686FFE29E5B33071EDCD55ED39096131D53` |

## 2. 模块选择与必须改进的语义

| 模块 / 精确证据 | 选择 | Star 接入要求 |
|---|---|---|
| RGS `channel.rs:238–280` build_channel/retry_backoff | 优先适配候选：mTLS 默认拒绝无 TLS、连接/请求超时、错误分类退避、HTTP/2 通道复用 | 独立小模块；生产禁止明文 bypass；mTLS 身份再绑定 tenant/owner/recipient 授权，不能以 TLS 就绪代替业务授权 |
| RGS `outbox.rs:480–554` list_pending/mark_sent/mark_failed | 参考事务内 SKIP LOCKED 与崩溃后租约接管 | settle 必须核对 worker lease token/generation；源 mark_sent 仅按 id 更新，不能直接沿用。保留 Star 已实现的 owner-local RLS、容量、永久回执和 stale-worker fence |
| RGS `outbox_relay.rs:185–316` tick/run/publish_entry | 参考事件年龄、批量、重试和 cycle 指标 | 不从 NATS subject 推断领域身份；改用 typed owner/Run/event。补取消/drain、批量硬上限及消息字节预算；不引入整套游戏域、Wasm/网关依赖 |
| RGS `producer.rs:293–340` publish_bytes | 参考 traceparent 与错误退避；不直接复用现有发布实现 | 当前 timeout 只包裹得到 ack future，后续 ack_future.await 无同一 timeout；还会每次重试 clone payload。Star 必须对完整持久化回执等待设 deadline、验证 log identity/cursor，并保持有界字节驻留 |
| AssetsLake `event_publisher_service.rs:83–145,219–296` DomainEventInput/stage/relay_once | 参考版本、trace/causation、幂等键字段 | stage 当前通过 pool 单独 INSERT，不能证明与业务同事务；disabled 返回新 UUID，dry-run/未配 Kafka 会 mark_published，真实 Kafka client 未链接。不得作为可用消息服务端或 durable acceptance 复用 |
| `E:/ROPE_CS` | 不保留源文件级复用建议 | Proprietary/internal-use LICENSE 禁止再分发或公开部署，除非取得 Say Co., Ltd. 明确许可；本设计不复制或部署该服务 |

## 3. 目标接入架构

复用以独立模块为单位。RustGameServer 的 Apache-2.0 通道代码是目前可评估的直接适配来源，但其明文 opt-out 必须删除；AssetsLake 仅提供 envelope 字段参考，若迁移代码先满足 AGPL/Star 许可证兼容要求。ROPE_CS、Ada 与有许可证冲突的 CATs 不得进入构建或部署。RustGameServer 本身不是通用消息 broker，Star 仍需实现自己的 Rust transport service。保持现有 `AdmissionTransport` 与 owner-local Outbox/Inbox；native 服务端仅拥有自身 transport persistence，不拥有 Schedule/Run 的业务写权限。不得跨 owner 共享写事务。

版本化 TransportProfile 固定 provider/version/digest、endpoint/server identity、协议与权限范围。发布确认必须来自已持久化的日志及有效 receipt；接收确认必须晚于目标 owner 的 Inbox 与业务结果提交。确认丢失后以稳定 message identity 重放，不能重新执行业务副作用。provider cursor 不跨端点/generation 混用；不能为 native server 强制模拟 JetStream。

接收侧分离 transport pull/ack、授权 verifier 和 owner handler：每次检查 tenant、发送/接收服务、kind 与 typed wire；handler 返回已提交事实，确认前复验授权。DB 失败、撤权、取消、未知身份都不得 ACK。毒消息先写受控持久化 quarantine，再按明确策略终结；不存在 quarantine provider 时保持失败关闭。

性能沿用 Star 的有界调度：发布 batch ≤8 串行、接收先以一条 in-flight 实现，wire 字节上限、连接数、租户并发、持久化容量、缓存和指标标签基数均显式预算；跨租户公平调度，取消停止新 claim、drain 在 deadline 内结束。Desktop 只消费分页投影与有界事件，不在 UI 保存完整消息日志；连接/runtime 不按任务卡重复创建。

Multipass/K3s 和最多两个测试 inventory 实例约束不变；本轮没有启动/修改虚拟机。native 服务可部署到明确绑定的 cluster，服务 readiness 不代表 Agent sandbox 或 executor 就绪。

## 4. 实施顺序和验收

1. 完成 D14 接收闭环：broker-neutral durable quarantine/recovery、ACK 丢失后的幂等恢复、生产身份与授权复验、撤权/取消和 poison redelivery；测试授权 fixture 不替代生产身份提供器。
2. 原生 server provider：按 Apache-2.0 要求从 RustGameServer 独立适配符合条件的 mTLS/client helper；补 authenticated durable append/pull/ack、租户隔离、重启恢复和持久化故障证明，执行同一 transport 契约套件。不得导入其游戏业务域、NATS producer 或未带 owner fence 的 Outbox 更新。
3. provider 依赖隔离（部分完成）：`platform-messaging-runtime` 的 Cargo feature seam 已实现并通过 no-default package check；完整应用依赖闭包和 portable quarantine 仍待实现/核验。native 构建闭包最终不得要求 NATS，业务 ABI 和 owner 数据事实保持稳定。
4. 故障/性能与商业分发验收：完整 ack deadline、租约接管 fence、容量拒绝、取消 drain、恢复后 cursor、资源/标签基数与并行公平性；生成实际分发闭包 SBOM/NOTICE。未通过前不得宣称原生服务器替换完成或任务执行可用。

v0.2 审查只更新了设计与来源证据，没有新增运行时实现/测试通过声明；D13 已有验证不能代替 D14/native server 验收。v0.3 的 Cargo feature seam 属于后续独立实施增量。

## 5. Cargo provider boundary update

`platform-messaging-runtime` now has an optional `jetstream-adapter` feature, default-enabled for compatibility. JetStream publisher/receiver modules and JetStream-specific quarantine are gated by that feature; the provider-neutral admission ports remain available in a package build with `--no-default-features`. Rust 1.99.0 package checks passed with and without the default feature, the no-default dependency tree excludes `async-nats`, and package format check passed. The default build reports three deprecation warnings in unmodified `domain-kms`.

This only establishes a crate-level compile seam. The NATS feature is still the default, other application crates may still pull NATS, and there is no native server provider or portable quarantine implementation yet. Do not remove the default adapter or claim replacement until the full application dependency closure and native provider contract are verified.

## 6. 修订历史

| 版本 | 日期 | 修订人 | 改动 / 触发 |
|---|---|---|---|
| v0.1 | 2026-10-11 | Ulysses — Mavis 接手审核 | 用户授权参考复用本机 Rust 服务端；记录实际源版本、选择性适配与 D14/native 验收边界 |
| v0.2 | 2026-10-12 | Ulysses — Mavis 接手审核 | 扩展本机候选审查；按实际许可证排除 ROPE_CS/Ada，标出 CATs 许可冲突和 stub，并刷新 RustGameServer/AssetsLake 当前状态 |
| v0.3 | 2026-10-12 | Ulysses — Mavis 接手审核 | 实装 JetStream Cargo feature 隔离并记录 crate 级 no-default 构建证据及剩余 native/quarantine 缺口 |
