# DD-RUST-SERVER-REUSE-001 — 自有 Rust 服务端复用与原生消息替换

> v1.8 · 2026-10-12 · Ulysses — Mavis 接手审核
> 状态：D19 server/auth 与 D20 native receiver slices 已实现；D20 在 Rust 1.98.1 `--ignore-rust-version` 下目标 crate check/Clippy 和 10 个定向单测通过。真实 TLS e2e、production identity/control plane、Profile→log provisioning、target DB grants、owner delivery、BI/performance 和 cutover 均未完成。

## 1. 范围与证据

用户再次明确允许从 `E:/AssetsLake` 与电脑内其他自有 Rust 服务端直接参考、适配或复用合适模块；不必为每个候选重复征求复用许可。该授权不自动覆盖第三方依赖许可、NOTICE、传递依赖义务或来源项目中的其他权利人代码。此次核验 AssetsLake、RustGameServer、ROPE_CS 的消息/Outbox 候选，并以仓库级元数据筛查 Ada、CATs、GitGit、Physis；后续又定向查看 `D:/DB_SERVER/db_api` 与 `C:/Users/leo19/actix-demo`。这不是对电脑内全部 Cargo 项目的穷尽审计。所有源仓库只读，其未提交内容和运行环境均保留。

| 来源 | 当前 HEAD / 工作区 | 许可证与实际实现 | 决定 |
|---|---|---|
| `E:/AssetsLake` | `0156aca56c0d7364120d7389ebe1ccf0cb64b750`；17 个脏文件 | 根 LICENSE 为 AGPL-3.0。目标 `event_publisher_service.rs` 的 SHA-256 与本表一致；它有 envelope/outbox 字段，但当前 relay 无 Kafka client，dry-run 或未配置 broker 时会标记 published，且 stage 独立使用 pool，不能证明与业务写入同事务 | 按模块允许参考、适配或直接复用；AGPL-3.0 不禁止商业使用，但需按具体链接/部署/网络提供方式履行源码、NOTICE 等义务并核验完整依赖闭包。该 relay 本身因未真实投递而不作为生产 transport；D16 未复制源码，因为本阶段只需 Star-owned PG receipt |
| `D:/RustGameServer` | `89458d887e8bfddf6637f9d80ea65f2091e8b6f2`；当前复核为干净 | 根 `LICENSE` 为 Apache-2.0；另有 `COMMERCIAL-LICENSE.md`，商业生产使用要求权利人事先书面授权。用户于 2026-10-12 明确书面授权本项目复用其本机 Rust 服务端模块；本次仅按该授权适配列明的小型 TLS helper。目标 `tls.rs` SHA-256 为 `675CBC3903775B69B7E878BA7820803EC03F962DC32E45FC8B94E68689C39164`。它是 gRPC 游戏服务端，不是通用消息 broker | 已选择模块级适配 TLS 文件加载与强制 mTLS 配置；保留 Apache 来源与本次授权记录。`channel.rs` 的明文 opt-out 不复制；outbox/relay/producer 仍只作语义参考 |
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
| SandTree `crates/event/src/router.rs` | `401B079ACCAB094B35EB22E6A250985F47215CC0235FA41883E90642170747A4` |
| SandTree `plugins/provider-multipass/src/provider.rs` | `B6821B23392ED40FE09C42A4BA6963C9B6514A64EA4C54EEF5148800132BBCD6` |
| SandTree `plugins/provider-multipass/src/cli.rs` | `922D6B89930DFE6F34F3113E80DEE61F2C8489036CEE861132047ADBE07ED542` |
| IDE1.0 `crates/ide-shell-protocol/src/lib.rs` | `B2C3817C2BFD5046947F930DF35DEC2740D9D4B251AC62C0DBEE0BC0B45F616A` |

## 2. 模块选择与必须改进的语义

| 模块 / 精确证据 | 选择 | Star 接入要求 |
|---|---|---|
| RGS `channel.rs:238–280` build_channel/retry_backoff | 参考 fail-closed 默认、连接/请求超时、错误分类退避与 HTTP/2 通道复用；源代码含 `require_tls=false` 明文 bypass | 不复制明文 bypass；Star client 强制 `https://`、mTLS identity 与 server name 校验，再独立绑定 tenant/owner/recipient 授权 |
| RGS `crates/shared-platform/src/tls.rs` | D19 已将 PEM 读取、client/server TLS config 形状适配至 Star Tonic 0.14；添加文件大小上限、服务器域名验证和显式 `client_auth_optional(false)` | 仅 TLS 配置适配；Star 仍须从 peer certificate 映射到 active service identity，并逐次验证 tenant/profile/route。模块见 `crates/platform-messaging-runtime/src/native_grpc.rs` |
| RGS `outbox.rs:480–554` list_pending/mark_sent/mark_failed | 参考事务内 SKIP LOCKED 与崩溃后租约接管 | settle 必须核对 worker lease token/generation；源 mark_sent 仅按 id 更新，不能直接沿用。保留 Star 已实现的 owner-local RLS、容量、永久回执和 stale-worker fence |
| RGS `outbox_relay.rs:185–316` tick/run/publish_entry | 参考事件年龄、批量、重试和 cycle 指标 | 不从 NATS subject 推断领域身份；改用 typed owner/Run/event。补取消/drain、批量硬上限及消息字节预算；不引入整套游戏域、Wasm/网关依赖 |
| RGS `producer.rs:293–340` publish_bytes | 参考 traceparent 与错误退避；不直接复用现有发布实现 | 当前 timeout 只包裹得到 ack future，后续 ack_future.await 无同一 timeout；还会每次重试 clone payload。Star 必须对完整持久化回执等待设 deadline、验证 log identity/cursor，并保持有界字节驻留 |
| AssetsLake `event_publisher_service.rs:83–145,219–296` DomainEventInput/stage/relay_once | 参考版本、trace/causation、幂等键字段 | stage 当前通过 pool 单独 INSERT，不能证明与业务同事务；disabled 返回新 UUID，dry-run/未配 Kafka 会 mark_published，真实 Kafka client 未链接。不得作为可用消息服务端或 durable acceptance 复用 |
| `E:/ROPE_CS` | 不保留源文件级复用建议 | Proprietary/internal-use LICENSE 禁止再分发或公开部署，除非取得 Say Co., Ltd. 明确许可；本设计不复制或部署该服务 |

## 3. 目标接入架构

复用以独立模块为单位；用户授权本机 Rust 服务端可作为直接复用/适配来源，逐项审查许可证、权属、源状态、依赖闭包和实现缺口仍是门槛。D19 已按用户当前书面授权适配 RustGameServer 的 Apache-2.0 TLS helper；该源仓额外 `COMMERCIAL-LICENSE.md` 的商业生产书面授权条件也须保留在来源记录中。`channel.rs` 的 plaintext opt-out 未复制。AssetsLake 模块同样可以直接评估和复用，AGPL-3.0 本身不禁止商业使用，需按具体交付方式履行许可证义务并审查完整依赖闭包；其当前 relay 未真实投递且不能证明与业务写入同事务，因此不作为生产 transport。ROPE_CS、Ada 与许可冲突的 CATs 不得进入构建或部署。现有 `PlatformPoisonQuarantine` 保持 JetStream `Message`/source identity 专用；D16 新增独立 `PgAdmissionReceiveQuarantine`，通过 `durable-quarantine` feature 写入 provider-neutral、deployment/tenant RLS 保护的 metadata-only Transaction 收据。它复用 Star 自有 PostgreSQL owner-role、幂等和 WORM 机制，不从本机候选项目拷入消息逻辑，因为这段存储语义已由 Star 当前契约直接规定。该 receive port 尚未接入生产 source verifier/composition，也不保留可重放原始 payload；不能据此宣称完整 poison recovery 上线。RustGameServer 本身不是通用消息 broker，Star 仍需实现自己的 Rust transport server。保持现有 `AdmissionTransport` 与 owner-local Outbox/Inbox；native 服务端仅拥有自身 transport persistence，不拥有 Schedule/Run 的业务写权限。不得跨 owner 共享写事务。

版本化 TransportProfile 固定 provider/version/digest、endpoint/server identity、协议与权限范围。发布确认必须来自已持久化的日志及有效 receipt；接收确认必须晚于目标 owner 的 Inbox 与业务结果提交。确认丢失后以稳定 message identity 重放，不能重新执行业务副作用。provider cursor 不跨端点/generation 混用；不能为 native server 强制模拟 JetStream。

接收侧分离 transport pull/ack、授权 verifier 和 owner handler：每次检查 tenant、发送/接收服务、kind 与 typed wire；handler 返回已提交事实，确认前复验授权。DB 失败、撤权、取消、未知身份都不得 ACK。毒消息先写受控持久化 quarantine，再按明确策略终结；不存在 quarantine provider 时保持失败关闭。

性能沿用 Star 的有界调度：发布 batch ≤8 串行、接收先以一条 in-flight 实现，wire 字节上限、连接数、租户并发、持久化容量、缓存和指标标签基数均显式预算；跨租户公平调度，取消停止新 claim、drain 在 deadline 内结束。Desktop 只消费分页投影与有界事件，不在 UI 保存完整消息日志；连接/runtime 不按任务卡重复创建。

Multipass/K3s 和最多两个测试 inventory 实例约束不变；本轮没有启动/修改虚拟机。native 服务可部署到明确绑定的 cluster，服务 readiness 不代表 Agent sandbox 或 executor 就绪。

## 4. 实施顺序和验收

1. D16 已加入 broker-neutral metadata receipt writer；继续完成 D14 接收闭环：生产 source identity/composition、source 授权复验、poison terminal disposition/redelivery 与 ACK 丢失后的幂等恢复。测试授权 fixture 不替代生产身份提供器；如需保留原始 wire replay，另需通过显式加密 retention/KMS 设计。
2. 原生 server provider：按 Apache-2.0 要求从 RustGameServer 独立适配符合条件的 mTLS/client helper；补 authenticated durable append/pull/ack、租户隔离、重启恢复和持久化故障证明，执行同一 transport 契约套件。不得导入其游戏业务域、NATS producer 或未带 owner fence 的 Outbox 更新。
3. provider 依赖隔离（部分完成）：`jetstream-adapter` 已可与 `durable-quarantine` 分开构建；D16 的 no-default + durable-quarantine 包级依赖不引入 NATS。完整应用依赖闭包、capacity-result poison/quarantine 与生产装配仍待实现/核验。native 构建闭包最终不得要求 NATS，业务 ABI 和 owner 数据事实保持稳定。
4. 故障/性能与商业分发验收：完整 ack deadline、租约接管 fence、容量拒绝、取消 drain、恢复后 cursor、资源/标签基数与并行公平性；生成实际分发闭包 SBOM/NOTICE。未通过前不得宣称原生服务器替换完成或任务执行可用。

v0.2 审查只更新了设计与来源证据，没有新增运行时实现/测试通过声明；D13 已有验证不能代替 D14/native server 验收。v0.3 的 Cargo feature seam 属于后续独立实施增量。

## 5. Cargo provider boundary update

`platform-messaging-runtime` now has an optional `jetstream-adapter` feature, default-enabled for compatibility. JetStream publisher/receiver modules and JetStream-specific quarantine are gated by that feature; the provider-neutral admission ports remain available in a package build with `--no-default-features`. Rust 1.99.0 package checks passed with and without the default feature, the no-default dependency tree excludes `async-nats`, and package format check passed. The default build reports three deprecation warnings in unmodified `domain-kms`.

This establishes crate-level compile seams only. The NATS feature is still default-enabled, other application crates may still pull NATS, and there is no native server provider. D16 supplies an admission poison metadata receipt, but not capacity-result quarantine, encrypted replay body retention, production source verification/composition or ACK-loss recovery. Do not remove the default adapter or claim replacement until the full application dependency closure and native provider contract are verified.

D19 keeps protobuf code generation in `crates/platform-messaging-proto`, excluded from the root workspace and reached only through the optional `native-grpc-adapter` dependency. Normal workspace builds therefore do not compile Tonic/Prost codegen; enabling the native feature resolves the packages in the shared lockfile and compiles the isolated proto crate. Release SBOM/license review must still include this feature's full build and runtime dependency closure.

## 6. 本机候选扩展评估（v0.8）

本轮扩大有界扫描，新增审查本机 E:/CATEST、E:/OpenAIActixMCP、E:/SandTree 与 E:/CATLLM。这不是对全盘 Rust 仓库的穷尽审计。源仓库保持只读，工作区状态与目标文件 SHA-256 按本轮实测记录；没有运行这些源项目的测试。

| 来源 | HEAD / 工作区 | 相关模块与证据 | 复用决策 |
|---|---|---|---|
| E:/CATEST | 03d9364f6880d42dcbb48164dd6ea99066daa667；干净 | AGPL-3.0。crates/stream-events/src/producer.rs SHA-256 78A06DF9EFDABC12E53B844AAA41896326AA0B5306EB1ED2B143108F275B9ED1；consumer.rs SHA-256 098EA862D616A0981E933A4FDDF21F92708F8569E6D027D9EB811E5D65C78A17。Producer 使用 rdkafka 0.36、acks=1、5 秒发送等待；consumer 打开 enable.auto.commit=true，handler 失败只记录日志。workspace 还启用 cmake-build/ssl-vendored | 可参考 typed event/envelope 和 Kafka adapter 的薄封装。当前 producer/consumer 不满足 Star 的 owner commit-before-ACK、持久化回执、exact replay 与租户授权契约；不作为 native server 或生产接收器直接复用。AGPL-3.0 不禁止商业使用，若复制/链接/分发/网络提供须按适用方式履行许可证义务并审查 rdkafka/librdkafka 依赖闭包 |
| E:/OpenAIActixMCP | 0818cc6cdf0243288cc50804cea2e75b21f8621c；干净 | Unlicense/public-domain 授权。src/main.rs SHA-256 052BB1BED126EBF844EA37B6A0D9B028BFD662053AF2C223054465BB6A75B213。Actix + rmcp 0.8 Streamable HTTP MCP 示例，含可选 bearer token、TLS 与本地 session manager；实际工具是 echo/UTC | 可参考 MCP HTTP/TLS 装配，不是 durable message server。Star 已有 star-mcp，不得无具体缺口再复制完整 MCP 服务或引入第二套 session owner |
| E:/SandTree | 940edc3a8110d6edffc6224f1dbf044b6259d2cf；14 个工作区改动 | workspace manifest 声明 MIT OR Apache-2.0，本轮可见根级 LICENSE 为 Apache-2.0。`crates/ipc/src/transport.rs` 当前 SHA-256 `4477C390BD4C719DBBC16A89C6C960D9F50D9459A3605C8AFC87F4E684765CFE` 且文件有未提交改动。它实现带 framing、按 SID 命名/ACL 的 Windows named-pipe 本地 IPC，不包含跨主机 durable log/cursor | 可作为 Star desktop/host 与隔离 helper/plugin worker 间本地 IPC 的参考候选；不能替代 NATS 或跨 VM/Run transport。若复用，先固定并审阅精确 worktree hash/差异、license 文件和完整依赖；当前 source worktree 不改动 |
| E:/CATLLM | d4f06e1fedf56f42fa463d6be44b2d50f7cb2c5e；Cargo manifest、Rust crates、lock 与部署文件均未跟踪 | 根目录有 Apache-2.0 LICENSE；当前 crates/gateway/src/main.rs SHA-256 4A5DE949F5EE145BF1549511607F1F3CDF919E826C5052ACE10B77F26FE44C2F，使用 rskafka producer，但其源代码和 manifest 尚无可审查的 Git 基线 | 仅供当前文件级阅读参考；不复制、不进入 Star 构建，不宣称完成许可闭包或验证。形成可复现提交、锁定依赖/完整传递许可证并通过对应测试后再评估 |
| E:/SandTree (EventRouter/Multipass modules) | `940edc3a8110d6edffc6224f1dbf044b6259d2cf`；仓库其它路径有脏改动，以下目标文件干净 | workspace 声明 MIT OR Apache-2.0，已见根级 Apache-2.0 LICENSE。EventRouter 为有界进程内 fan-out；Multipass provider 使用 argv/name/output 限制，但同步 `Command::output()` 忽略传入 timeout | EventRouter 可参考为提交后易失 UI/投影通知，不提供 durable log、ACK、replay 或恢复；Multipass adapter 可复用校验模式，接入前需实现 deadline、取消、子进程回收，并沿用 Star Rust Host Manager、Run binding、双层 reservation 和两实例 inventory 上限 |
| E:/IDE1.0 | `f44479f145248bad808ea04831bf4ff5c652501f`；`main` 落后 `origin/main` 38 commits，仓库有未跟踪 feedback 文档，目标文件干净 | workspace 声明 MIT OR Apache-2.0；`ide-shell-protocol` 提供 HTTP/WebSocket/stdio server，但 event subscribers 使用无界 Tokio channel | 可参考任务卡 CLI 的协议适配面；不能直接引入无界 channel，须提供 bounded stream/backpressure、Run/Worktree/Task 授权、cancel/recover 与 output cap |
| D:/DB_SERVER/db_api | `master` 无 commit；Cargo manifest/lock/source 未跟踪；`src/main.rs` SHA-256 `A3B7BFBF3BFBA8783A1F9D1301C67BBA90C866B301CDC79DE526D07E7CBB5AA8` | Axum 0.7 + deadpool-postgres 的单路由健康检查样例；无根级 LICENSE，连接配置含硬编码凭证并使用 NoTls | 用户已允许本机项目作为参考；仅可参考限量连接池的概念。来源无可复现提交和许可声明，且配置不符合凭证/TLS要求，不复制源码或配置；凭证值不记录于本文 |
| C:/Users/leo19/actix-demo | `master` 无 commit；工作区含未跟踪文件；`src/main.rs` SHA-256 `D974724CE8DC4BF83D2DDD6E084F6638F67F930782824004708806EF467368F7` | Actix 4 + rustls 0.22 静态文件 HTTPS 示例；无根级 LICENSE，配置使用 `NoClientAuth`，不包含 durable messaging | 用户已允许本机项目作为参考；仅说明基础 listener/TLS 装配方式。没有 mTLS、消息持久化或 Star 授权契约，不复制到 NativeTransport；来源项目保持只读 |

本轮结论：CATEST 与 AssetsLake 都有可参考的事件 envelope/producer 经验，但都没有可直接替换 NATS 的 Star-compatible durable server；CATEST consumer 的自动提交尤其不能用于 Star owner transaction。OpenAIActixMCP 与 SandTree 分别服务于 MCP HTTP endpoint 和本机进程 IPC，属于独立 app/runtime 边界的素材，不属于消息 broker。RustGameServer 的 mTLS client helper仍是候选小模块，但该游戏服务端本身也不是通用 broker。Star native provider 仍需拥有自己的认证身份、durable append/pull、owner-independent cursor receipt、重试/恢复、资源限额与故障契约。

本轮再对 SandTree `EventRouter`、`provider-multipass` 与 IDE1.0 `ide-shell-protocol` 做文件级补充审查。EventRouter 默认每 subscriber queue 有界（256）并合并 `ResourceChanged`，只适合已提交事实之后可丢失的进程内通知。Multipass runner 同步阻塞且不执行传入 timeout，不能直接放进异步桌面/Host Manager 路径。IDE1.0 的多协议 CLI server 可参考协议形状，但无界 subscriber channel 不符合 Star 内存与背压预算。源文件哈希已记录；这些源仓仍只读，未运行源项目测试，未复制源码。未穷尽本机所有 Rust 项目；每次迁入仍须复核届时 HEAD、工作区、目标文件哈希、权属/许可证、NOTICE、传递依赖与产品语义。

CodeGraph 未能提供这些来源仓库的索引覆盖审计；本节结论来自当前源码与 Git 状态的定向核对，不代表全盘审计或源项目测试通过。源项目只读；这次没有复制 DB_SERVER 或 actix-demo 代码。

## 7. 修订历史

| 版本 | 日期 | 修订人 | 改动 / 触发 |
|---|---|---|---|
| v0.1 | 2026-10-11 | Ulysses — Mavis 接手审核 | 用户授权参考复用本机 Rust 服务端；记录实际源版本、选择性适配与 D14/native 验收边界 |
| v0.2 | 2026-10-12 | Ulysses — Mavis 接手审核 | 扩展本机候选审查；按实际许可证排除 ROPE_CS/Ada，标出 CATs 许可冲突和 stub，并刷新 RustGameServer/AssetsLake 当前状态 |
| v0.3 | 2026-10-12 | Ulysses — Mavis 接手审核 | 实装 JetStream Cargo feature 隔离并记录 crate 级 no-default 构建证据及剩余 native/quarantine 缺口 |
| v0.4 | 2026-10-12 | Ulysses — Mavis 接手审核 | 明确技术模块复用授权与业务域隔离边界；区分 JetStream-specific poison store 与新 provider-neutral receive quarantine port，并保留未接线缺口 |
| v0.5 | 2026-10-12 | Ulysses — Mavis 接手审核 | 在通用来源身份中固定日志 generation，并明确 JetStream adapter 从可信配置接收该代次 |
| v0.6 | 2026-10-12 | Ulysses — Mavis 接手审核 | Add D16 metadata-only PostgreSQL quarantine and standalone no-NATS feature; separate implemented receipt persistence from production wiring and encrypted replay gaps |
| v0.7 | 2026-10-12 | Ulysses — Mavis 接手审核 | Clarify that AssetsLake and other local Rust modules may be directly reused after module-level license/ownership/dependency review; AGPL is not a commercial-use ban, while the current dry-run relay remains unsuitable as production transport |
| v0.8 | 2026-10-12 | Ulysses — Mavis 接手审核 | 新增 CATEST、OpenAIActixMCP、SandTree、CATLLM 的有界候选审查；记录 Kafka ACK/auto-commit 缺口及本地 IPC/MCP 边界，明确无可直接替换 NATS 的现成服务 | 用户授权参考复用本机 Rust 服务端 |
| v0.9 | 2026-10-12 | Ulysses — Mavis 接手审核 | 补充 SandTree 有界 EventRouter/Multipass provider 与 IDE1.0 CLI protocol 文件级审查及精确 hash；明确内存、阻塞/超时、授权和背压缺口，保留本机 Rust 服务可模块级参考/复用原则 | 用户再次明确 E:/AssetsLake 与本机其他 Rust 服务可直接作为参考和选择性复用来源 |
| v1.0 | 2026-10-12 | Ulysses — Mavis 接手审核 | 复验 AssetsLake 候选 event publisher 文件在 dirty source repository 内仍未修改且 SHA-256 与记录一致；明确可择取其 typed event metadata 进行适配，同时排除 dry-run relay 的 durable delivery 声明；映射 D17 对 mTLS/profile identity 的小模块接入边界 | 用户再次确认 E:/AssetsLake 和本机其它 Rust server 可参考复用 |

## 8. D17 source reuse recheck

The read-only recheck on 2026-10-12 found AssetsLake at `0156aca56c0d7364120d7389ebe1ccf0cb64b750` with 17 dirty/untracked paths. The selected `backend/src/services/event_publisher_service.rs` itself was clean, and its SHA-256 remained `3A2114863827CB8CE893AD5A24090D827F89B7752C6FF23546DBDB4DC3B8A4A5`. It is eligible for selective adaptation of typed event metadata fields (schema version, trace, causation and idempotency); its separate-pool staging and dry-run/no-broker relay behavior remain unsuitable as Star owner Outbox or production durable transport. No source file was copied or changed in this recheck.

The RustGameServer `channel.rs` candidate remains Apache-2.0 at SHA-256 `8949F57996706547BC8E60022101B8B4AB854D3D0B4AB07CB9BA4252A7433061`; its explicit `require_tls=false` plaintext branch must be removed before adapting the client connector. D17 may use this small helper for trusted transport authentication after retaining the applicable notice and wrapping the full publish/ack operation in Star's absolute deadline. It does not provide service authorization or a durable receipt. SandTree EventRouter, its blocking Multipass provider, and IDE1.0's unbounded CLI subscriber retain the v0.9 suitability limits.

This recheck does not assert that all local Rust projects are audited, that AGPL obligations are already fulfilled for a hypothetical distribution, or that any external module is production-integrated. For each copied file, pin the exact source state, retain license/NOTICE, inspect the build and distribution dependency closure, adapt the code to Star's owner/Run authorization and bounded resource contracts, and record focused evidence.

## 9. D18 source reuse and Rust-owned replacement path

D18 does not copy transport implementation from AssetsLake or another project because the storage semantics now required are Star-specific: deployment+tenant+service isolation, generation-pinned route identity, cross-generation exact idempotency, lease-fenced owner ACK, payload retention with permanent receipt, same-transaction metadata Outbox, and per-message producer identity. This is a fit decision for this storage slice, not a restriction on future direct module reuse.

E:/AssetsLake remains eligible for direct reuse of independently reviewed modules; its typed event metadata is already adapted while its current dry-run/no-broker relay remains unsuitable as production durable delivery. D19 has started by adapting the RustGameServer TLS config helper under the user's written authorization and retaining its Apache source attribution plus the additional commercial-use condition. The client-side bypass branch was not copied; peer certificate fingerprint extraction is present, while the production fingerprint registry, server handlers and per-request profile/route checks remain pending. Other local Rust server modules may also be adopted where behavior matches the contract. No source repository was modified by D18 or D19.

The replacement path is incremental: preserve the existing NATS adapter behind the same versioned provider interface while the Star-owned Rust server implements the PostgreSQL durable log; run the same contract suite against each provider; add service authentication, profile→log provisioning, receiver recovery, BI/Benchmark parity, bounded memory and concurrency gates; then dual-run/compare before profile-based cutover. NATS remains a supported provider until all production routes, replay/recovery, operational rollback and performance gates are accepted. D18 provides no provider client/server, so this plan remains future implementation work.

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v1.1 | 2026-10-12 | Ulysses — Mavis接手审核 | Record D18 as Star-owned durable storage, preserve module-level reuse eligibility, and outline a contract-compatible staged NATS replacement path |
| v1.2 | 2026-10-12 | Ulysses — Mavis接手审核 | 记录用户对 RustGameServer 模块的明确书面复用授权及额外商业条款；D19 首次适配 TLS helper 并加入 gRPC client/proto slice；标明服务端持久身份、Profile→log 与 Cargo/测试缺口 |
| v1.3 | 2026-10-12 | Ulysses — Mavis接手审核 | 记录用户对 AssetsLake 与本机其他 Rust 服务端的直接参考/模块复用授权；补充 DB_SERVER、actix-demo 的边界审查和 D19 proto codegen 隔离；登记 Rust 1.99.0 下目标 crate 编译/格式验证结果 |
| v1.4 | 2026-10-12 | Ulysses — Mavis接手审核 | 为 D19 加入 PEM 文件实际读取上限与每实例 8 路并发背压；补充 5/5 定向测试、目标 crate 编译和全 workspace 格式检查的真实结果 |
| v1.5 | 2026-10-12 | Ulysses — Mavis接手审核 | 将 native client 构造参数收敛为公开类型化配置、绑定 URI 主机与 TLS 校验域名并修复新增 Clippy 告警；更新 Rust 1.99.0 all-targets check、Clippy、6 个 lib 单测与 Windows package-test 链接失败的实测状态 |
| v1.6 | 2026-10-12 | Ulysses — Mavis接手审核 | 补录 Star-owned gRPC server、证书身份/撤销与 Profile→log-generation 数据库授权实现、80/80 PostgreSQL 和 35/35 Rust lib 验证；明确生产 API、TLS e2e、consumer 与 cutover 仍开放 |

## 10. D19 native gRPC adapter and server authorization slices

本节记录隔离 worktree `codex/native-transport-d19-20261012` 中尚未提交的阶段进展，不代表 D19 或整体 Worktree-first 任务完成。当前代码将 `D:/RustGameServer/crates/shared-platform/src/tls.rs`（源仓 HEAD `89458d887e8bfddf6637f9d80ea65f2091e8b6f2`，源文件 SHA-256 `675CBC3903775B69B7E878BA7820803EC03F962DC32E45FC8B94E68689C39164`）中的 PEM 加载与 Tonic TLS 配置做小范围适配；没有复制 `channel.rs` 的明文分支、游戏业务域或 Outbox 实现。源仓 `COMMERCIAL-LICENSE.md` 的商业生产书面授权条件由用户本轮明确的书面复用授权覆盖；代码保留 `SPDX-License-Identifier: Apache-2.0`、来源 commit/hash 与 `NOTICE.md` 记录。

新增的 `native_transport.proto` 定义版本化 unary Append/PullOne/AcknowledgeOne。ProfilePin 只传 32-byte canonical profile digest；该 digest 已提交 deployment、tenant、profile id、revision、lifecycle 和完整配置，因此服务端可在经证书认证的 deployment/tenant scope 内解析唯一 profile revision。Append 不接受客户端自报 sender identity；服务端必须从 mTLS peer certificate 得到 SHA-256 fingerprint，再查 active certificate→service identity registry。当前 `native_grpc.rs` 已提供强制客户端证书的 TLS config、peer certificate fingerprint 提取、通过 `Read::take` 最多读取 1 MiB+1 字节来拒绝超限 PEM，以及实现 `AdmissionTransport` 的多路复用 gRPC publisher。`NativeGrpcClientConfig` 将 owner、HTTPS endpoint、证书路径、deadline 与固定 Profile 作为单一构造输入。每个 adapter 实例通过共享 Semaphore 将并行请求限制为 8 个；取消 future 会释放许可。client 校验 HTTPS URI 主机名与 TLS server domain 一致、限制连接 deadline 至 30 秒和请求 deadline 至 10 秒、wire ≤16 KiB、gRPC frame ≤32 KiB，并在生成 receipt 前核验 exact wire SHA-256、log generation 与正 sequence。

**实现边界**：Rust 1.99.0 下 `cargo check -p platform-messaging-runtime --features native-grpc-adapter --all-targets -j 4` 与目标 crate all-targets Clippy 均通过；Clippy 仅报告未修改的 `star-context`、`domain-automation`、`domain-kms` 警告，D19 文件无新增告警。`cargo test -p platform-messaging-runtime --features native-grpc-adapter --lib native_grpc::tests -j 1` 通过 6/6。未过滤 package tests 在链接 `transport_profile_pg` integration 与 `platform_messaging_runtime` lib-test executable 时遇 Windows `LNK1104`（无法打开目标 `.exe`）；单独的 integration target `--no-run` 随后成功，但完整 package test 未通过/未完成。定向 rustfmt check 与本轮 Cypher Graph Guardian 已通过；`cargo fmt --all -- --check` 因仓库内未修改文件的既有格式差异失败，未批量格式化无关代码。编译仍有 3 条未修改 `domain-kms` 的弃用 API 警告。Cargo.lock 已更新，但完整许可证/发布 SBOM 闭包仍未验收。上述 client 首片的边界记录的是 server/auth slice 到来前的状态；后续 D19 服务端授权首片已增加 Star-owned handlers、certificate identity registry 与 authenticated database wrappers，并由 Rust 按 exact wire bytes 验证 D18 durable receipt。仍没有 Profile endpoint identity→URI/TLS hostname 的可信解析、生产证书/绑定 provisioning API、production server composition、完整 owner consumer commit-before-ACK 闭环、双 provider 契约套件、性能/恢复测试。Native provider 仍不得用于产品流量。AssetsLake 目标文件当前仍保持原 SHA-256 `3A2114863827CB8CE893AD5A24090D827F89B7752C6FF23546DBDB4DC3B8A4A5` 且未复制；其 dry-run relay 仍不构成 durable transport。

## 11. D19 Star-owned gRPC server and mTLS authorization slice

Only one prior local source file is adapted into code: the bounded PEM/Tonic TLS setup helper from the user-authorized RustGameServer source, retained with source commit/hash, Apache-2.0 SPDX, applicable NOTICE, and the user's written authorization for its additional commercial-use condition. The new `native_grpc_server.rs`, wire protocol, certificate→service registry, Profile-route binding, SQL wrappers and migration are Star-owned implementations. No AssetsLake source or relay code was copied; its event envelope/causation metadata is adapted as a Star Outbox design idea. Source repositories remain untouched.

The server supports Append, PullOne and AcknowledgeOne through the versioned Proto contract. Tonic requires a client cert; Rust hashes the peer leaf certificate and passes that fingerprint into the authorization routines. The routine resolves the currently active certificate identity and active Profile revision, fixes directionality from authenticated service identity, checks provider/version/endpoint identity and allowed route/kind, then selects the exact bound log generation. For append it computes SHA-256 from exact wire bytes and validates the durable receipt; for pull it checks the stored digest against returned bytes. Runtime role can call only authenticated wrappers, never raw storage routines or table DML. All operations stay on the Star durable PostgreSQL log; they do not rely on NATS sequence semantics.

The module is strictly bounded: payload 16 KiB, gRPC frame 32 KiB, up to eight concurrent RPCs shared by service clones and eight per connection, each SQL operation deadline 10 seconds. These are memory/concurrency guardrails only; RSS, p95/p99 latency, fairness under multi-tenant load and crash/recovery behavior remain unmeasured.

The D19 runner re-applied `2026-10-12-platform-transport-z-native-server-auth.sql` twice and passed 80/80 disposable PostgreSQL 18.6 steps. The auth SQL contract checked current certificate/profile/route binding, exact generation, append replay, lease/Pull/Ack, unknown/revoked certificates, FORCE RLS, fixed-path routines, restricted runtime grants and audit facts. Its accepted Append fixture now carries the real SHA-256 for its exact test wire; the Profile digest itself remains a fixed 32-byte pin fixture, so this SQL contract proves lookup equality but not canonical digest calculation (covered separately by the D17 typed resolver unit test). Rust 1.99.0 target check and Clippy pass; native-feature library tests pass 35/35. The complete package test and live gRPC TLS handshake were not verified in this slice.

Current owner-level provisioning/revocation uses DDL owner database operations only for disposable tests; a typed service identity control-plane API is required before deployment. A trusted endpoint resolver must map the non-secret endpoint identity to HTTPS URI and TLS server-name pair. Production database role/grant deployment, certificate/key rotation, service lifecycle, per-owner consumer commit-before-ACK, quarantine/recovery, BI/Benchmark, Rust-native dual-provider parity, full NATS dependency removal, SBOM and performance gates remain open. Native transport is feature-gated and must remain disabled for production until these gates pass; Schedule/Run execution still requires its own budget, reservation, Hook, sandbox, cancellation and independent validation gates.

## 12. D20 Native gRPC receive adapter slice

隔离 worktree `codex/phase9f4c-d20-native-receiver-20261012` 在既有 `AdmissionReceiveTransport` 与 `AdmissionReceivedDelivery` 上实现 Star native Pull/Acknowledge client adapter。它连接 D19 mTLS client/TLS helper 和版本化 `PullOne`/`AcknowledgeOne` proto；调用者固定 `AdmissionReceiveScope`，Profile digest 固定 provider revision。`PullOneRequest` 携带 Profile pin、授权 sender service ID、message kind 与 bounded 30–120 秒 lease。recipient 由 server 根据已认证 mTLS service identity 解析，不信任 payload 或 caller body 声明。响应必须匹配固定 scope、sender/kind、合法 log ID、非零 log generation/message/lease UUID、正 sequence/attempt/lease timestamp、≤16 KiB wire 和 wire 的真实 SHA-256。

`AdmissionReceiveOrigin.provider_identity` 由 domain-separated SHA-256 导出，输入按 length-prefix 组合 provider ID、provider version、Profile digest 与 endpoint identity。该不透明值是稳定 quarantine/replay 来源键，不代替 TLS 身份或 `AdmissionReceiveAuthorization` 的独立来源/授权复验。确认请求复用 pull 返回的 log generation、sequence、message ID、lease ID 和 wire digest；`acknowledged=false` 或 RPC 错误都保持未确认。共享 `AdmissionReceiver` 只在 owner handler 报告 durable commit 并完成提交后授权复验后 ACK；永久 disposition 只有在 quarantine receipt 已持久化后才能调用，native adapter 以同一 lease-fenced `AcknowledgeOne` 完成来源日志终态处置。

并发内存边界分两层：每 adapter 最多 8 个 RPC 正在执行；另一个 8-permit semaphore 在 Pull 返回前取得并随 delivery handle 持有，只有 handle drop 才释放，因此慢 owner handler/遗留 delivery 无法无限累积租约或 wire buffer。每份 wire ≤16 KiB、Tonic frame ≤32 KiB；没有额外 prefetch queue。Profile/endpoint resolver、server certificate 生命周期与密钥轮换、target DB grants、production composition 和全局多 adapter/多 owner 预算仍属上层部署/调度责任。

本 slice 没有从 `E:/AssetsLake` 复制代码，不是因为缺少复用授权：当前 `event_publisher_service.rs`（SHA-256 `3A2114863827CB8CE893AD5A24090D827F89B7752C6FF23546DBDB4DC3B8A4A5`）在 pipeline disabled 时会返回未持久化的新 UUID；relay 在 dry-run 或无 broker 时会将事件标为 published，而配置 broker 时因二进制未链接 Kafka client 只会标记失败，因此不能直接作为 Star durable message transport。它的版本、trace、causation 与幂等字段仍可供 Star-owned event envelope 参考。RustGameServer TLS helper 是此前已逐模块审查并按用户授权适配的代码，来源 hash/NOTICE 见上表；各源仓代码和 dirty worktree 均未改动。

`native_provider_identity()` 是公开的 canonical identity helper，接收端的授权与 quarantine 实现可以共用其稳定派生规则。结果只标识固定 Profile/provider revision 的来源键，不是认证凭据或授权结论；调用方仍必须验证可信 Profile 与 mTLS peer identity。

Rust 1.98.1 `--ignore-rust-version` 下目标 crate all-targets check 通过，目标 crate Clippy 退出码 0、`native_grpc::tests` 10/10、定向 rustfmt 与 `git diff --check` 通过。因 cargo/rustc 不在 PATH，命令通过安装目录中的 1.98.1 `cargo.exe` 和显式 `build.rustc` 配置执行；这不构成 Rust 1.99.0 正式门禁。Clippy 仅报告未改动 `star-context`、`domain-automation`、`domain-kms` 的既有警告。未做真实 TLS server/client e2e、consumer/quarantine DB 联测、BI/Benchmark 对账、恢复、RSS、p95/p99、公平调度或 dual-provider contract suite。故该 native receiver 仍为 opt-in implementation slice，不算 D20 consumer migration/production-ready 或可替代 NATS。剩余 D20 scope 与不变的 fail-closed gates 见 [D20 阶段报告](../reports/PHASE-9F4C-D20-NATIVE-RECEIVER-REPORT.md)。

| Version | Date | Reviewer | Change |
|---|---|---|---|
| v1.8 | 2026-10-12 | Ulysses — Mavis接手审核 | 明确用户可直接复用本机 Rust 服务模块并说明 AssetsLake relay 的实际投递边界；公开 canonical provider identity helper，记录当前 Rust 1.98.1 诊断验证与未关闭的 D20 gates | 用户再次确认模块级直接复用，并要求为 Star 自有 Rust 服务替换 NATS 做准备 |
| v1.7 | 2026-10-12 | Ulysses — Mavis接手审核 | 记录 native gRPC receive port、canonical provider identity、leased-delivery memory cap 和真实诊断验证；明确 D20 consumer/BI/production gates 仍开放 |
