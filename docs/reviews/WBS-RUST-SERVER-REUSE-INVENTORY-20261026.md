# 本机 Rust 服务端复用盘点（WBS）

- 日期：2026-10-26
- 范围：只读审查；本报告不授权复制、修改、部署任何来源仓库代码。
- 结论：目前没有模块达到“可直接复用并部署”的放行条件。唯一值得进入下一轮适配评估的是 `D:\RustGameServer` 的 PostgreSQL Outbox 存储边界；传输层仍直接依赖 NATS，且依赖与权属闭包未审完。Star 的 Project/Run 授权、持久化事实、预算、Outbox/Inbox、Multipass 管理与运行时隔离仍由 Star 自己负责。

## 1. 搜索边界与限制

- `E:\AssetsLake`：检查 Git 根、`backend` 下 Rust manifest、选定的授权/Outbox/验证服务文件、根许可证与后端锁文件；不查看环境文件、密钥或生成输出。
- `D:`：仅枚举即时子目录，排除 Star、backupProjects、Docker 数据、缓存等明确排除项；以 Git 根和 Cargo manifest 识别候选。对选中仓库递归列出 Cargo manifests（排除 `.git`、`target`、`node_modules`），随后只读相关模块。对 `D:\actix` 和 `D:\bevy` 仅检查一层子目录；未把非 Git 的 `D:\bevy\SayServer-0.1.0` 当作来源仓库。
- `C:\Users\leo19\source\repos`：检查即时项目目录，并在项目内递归列 Cargo.toml，排除 Git/生成/依赖目录；未发现新的 Rust 服务端 Git 来源候选。其下 `2DAction` 有 Cargo manifests 但不是 Git 根；`hlsl-grimoire-sample` 与 `NKGMobaBasedOnET` 是 Git 根但未找到 Cargo manifest。
- 未扫描整个用户配置文件夹。未读任何密钥、环境变量文件或 build output。未运行 build/test/benchmark；所有实现判断都是静态文件审阅，不能作为运行时质量证明。

## 2. 候选模块结论

### 2.1 `E:\AssetsLake` — Reference-only；代码复用仍需另行判定

- Git HEAD：`0156aca56c0d7364120d7389ebe1ccf0cb64b750`。
- Dirty：是。相关 manifest `backend/Cargo.toml` 已修改；`backend/rust-toolchain.toml` 是未跟踪文件，未读取。所审查的服务源码、`backend/Cargo.lock` 与根 `LICENSE` 在 scoped status 中干净。
- Rust/MSRV：当前工作文件写有 `rust-version = "1.98"`，但该行是未提交变更；HEAD 版本未声明 MSRV。Workspace 后续需要针对固定版本与 Star 1.99.0 实际核验。
- 许可证/权属：根 `LICENSE` 为 AGPL-3.0，SHA-256 `6F1E622C82A380075843BB084A7EC3B1F1D12A4A02526D75E78B0924A860AA75`。AGPL 允许商业使用；§§4–6 对源码/二进制传递及修改版源码有要求，若运行的是修改版，§13 要求远程交互用户可获得该版本的 Corresponding Source（本地文件 `LICENSE:196,233,540`）。根文件未提供经核验的代码权属链或独立 NOTICE；不能仅凭仓库路径断定每个文件及依赖的授权闭包。商业交付与网络服务的 AGPL 履行方式尚未由本盘点确认。
- 依赖闭包：当前工作 manifest SHA-256 `2DBE3B0C655C49BAD4041226ED4F8F438DE5617ACA7E7F382DE41880B0352AE2`，`backend/Cargo.lock` SHA-256 `99B6F312B8BFFC40617A11DE994825E64D147309505C32EB20B2B0BABDD1AA2F`。直接依赖包含 Actix、Tokio、SQLx/PostgreSQL、AWS S3、OpenTelemetry 等；没有完成锁文件传递依赖、feature、打包产物的 SPDX/SBOM 与许可证义务审查。
- 实现与耦合：
  - `backend/src/services/event_publisher_service.rs` SHA-256 `3A2114863827CB8CE893AD5A24090D827F89B7752C6FF23546DBDB4DC3B8A4A5`：Outbox 写入使用自己的 `PgPool`，未接收业务事务 executor，因此所查 API 本身不能证明业务更新与事件原子提交（101–145）。后台 relay 以定时轮询和 `FOR UPDATE SKIP LOCKED` 取批次；未配置 Kafka broker **或**启用 dry-run 都会把事件标为 published；broker 配置存在时，二进制仍明确报 Kafka client 未链接并记录失败（159–170、219–261）。这是轮询/演示语义，不是可移植的可靠传输实现。
  - `backend/src/services/authorization_service.rs` SHA-256 `132C8F4A745CB1986F77B56A63651735B787222995F0D948BF81A1DA1E21FA6D`：按 session 的全局 role 和 route permission 作检查；没有看到 Star 所需 Project/Run owner 与资源级当前授权绑定（144–202）。
  - `backend/src/services/verification_service.rs` SHA-256 `8BE44BFD67B50C2F030509124B28CDDB9D6B16292C265B39BA1DE846B5B15022`：验证/邮件短信领域 Outbox，不作为 Star 执行消息服务候选。
  - `backend/src/services/auth_service.rs` SHA-256 `E5AA3AAD0101557FFE21AC90A2D6CEC55819F2568EDC1F62E4E00F7F6F87C52A`；`backend/src/config.rs` SHA-256 `16E2D332BDF05BBCC9A217CD5332C886C429D16A526AE26EE65BE874CEC338B2`。当前配置还暴露 Kafka、dry-run 与 polling 参数。

**处置：**可借鉴其明确暴露的失败模式与 SQL 领用思路；目前按 reference-only 处理。若要复制/改编源代码，先确认 AGPL 对 Star 网络服务及发行形态的兼容方案、代码权属、全依赖 SBOM，并改为同一 owner 事务写业务事实与 Outbox；不得保留“事件管道关闭仍返回成功”或 dry-run 成功回执语义。

### 2.2 `D:\RustGameServer` — Outbox 可作为 adaptation candidate（仅概念候选）

- Git HEAD：`9997c9434af1aced6c96886f7bd82ff2bf8ce4fb`。
- Dirty：是，修改集中在 mock CI/UAT 脚本及其说明；本节所审源码与 manifest 在 scoped status 中干净。
- Rust/MSRV：workspace 声明 1.98.1；低于 Star 当前 1.99.0 pin，可作为待评估代码，但需在 Star 固定工具链和 feature 配置下重验。
- 许可证/权属：根 Cargo manifest 声明 Apache-2.0；根 `LICENSE` SHA-256 `BD3328238FE80F9A0F0BEA5469ED40022A31FCC1809BBBCA8FC28FBAF407988B`，未发现根 NOTICE。Apache-2.0 允许商业使用；分发需附许可证、保留适用版权/专利/商标/归属声明、标明修改文件，并在存在 NOTICE 时保留其归属（`LICENSE:89–118`）。本审计未建立版权权属链，也未确认传递/打包依赖全部满足 Star 分发要求。
- 依赖闭包：`Cargo.lock` SHA-256 `B1F189ECC7B5CDC2DFD1D08C25463D7D19858423850BA1518F04E27F3BFEC74C`；root `Cargo.toml` SHA-256 `A825A04AE96CF58ED64CACA39EAF7173DD9A82DDA8B9533C40B9E3017E4C1B1E`；`shared-platform/Cargo.toml` SHA-256 `D1181677F9393867018FCD738B8C3CE1D1F2CF43274135095EA918E68D619153`。直接闭包含 SQLx/PostgreSQL、async-nats、Tokio、Tonic/Prost、Serde、Tracing/OpenTelemetry；锁定版本已记录，但传递许可证、feature 与发行闭包尚未审核。
- 实现与耦合：
  - `crates/shared-platform/src/outbox.rs` SHA-256 `43DA6FFDC5F1AFDAF892F1FF6D95DD01A0FBA699F9FB58289CC4DDD12040D274`：PostgreSQL repository 的 append 接受 `PgExecutor`，可由调用方传入事务 executor；批量领取使用 `FOR UPDATE SKIP LOCKED`、30 秒 lease 和状态迁移；实现还同时包含 in-memory repository，不能把后者当作持久化验收。它提供值得评估的 DB 存储 seam，不证明 Star 当前 Outbox/Inbox 语义完整。
  - `crates/shared-platform/src/outbox_relay.rs` SHA-256 `5D0F78C3D04665FDB5548EECEFD8A163E3D5D4D6EE94A2732A88F715FFEADBD5`、`crates/shared-platform/src/producer.rs` SHA-256 `052EBBE4AB012E37F38DD32D9072F686FFE29E5B33071EDCD55ED39096131D53`、`crates/shared-platform/src/consumer.rs` SHA-256 `D1A1CF10C3DBF2EA350274642A3D818721230BDB63E5A58AD244B7A6D6F80919`：网络层直接引用 async-nats/JetStream、subject 与 NATS ack/nak；不可把这些 API、cursor 或确认语义泄漏进 Star 业务契约。后续 Rust-native provider 替换须沿 Star 自己的版本化 transport adapter 与 durable receipt 契约实施。
  - `crates/cluster-ops/Cargo.toml` SHA-256 `AF12F8D33EFA272B21F70E3E4206DD238167A70B64958DB684A463AF5A02F931`；`crates/cluster-ops/src/service.rs` SHA-256 `4A6519E7E683BD24208A94EA190F70817095C87582254DD74F3E62A55DFA8E21`；`crates/cluster-ops/src/repository.rs` SHA-256 `700C0969C10DE07FA109AD2FE55BAF59FA1500087196EE116B95CEB2514D511D`。这里的 ClusterNode/heartbeat 是游戏服务器节点管理，不能等同于 Multipass instance、HostEnvironment、K3s cluster 或 Star Host/cluster capacity reservation；同一仓库中还存在 in-memory repository。
  - 唯一检查到的 K3s scale 文件 `crates/cluster-ops/src/realm_lifecycle/operations/scale.rs` SHA-256 `FFA50F595E92F830C0140A1257F879E577414B3C06CEC3E1EA3FDC918AD9682F` 明确写明“trait + 占位”，`NoopScaleOperator` 返回 pending-impl 错误（50–55、109–130），不是可运行的 K3s provider。未找到可证明 Multipass 管理集成的实现。

**处置：**只将 PostgreSQL Outbox repository 的事务 executor、并发领用与 lease 作为适配评估对象；后续若采用，必须切断 NATS 业务耦合、重建 Star 的 owner-local 事务、Inbox、幂等、租约、取消/drain 与 durable receipt 契约。当前不直接移植整 crate、游戏领域或 NATS adapter。

### 2.3 `D:\Ada` — Excluded pending rights clarification

- Git HEAD：`2f8b07002d0da9e99cebe2c9b5712d1029be8e08`。
- Dirty：存在未跟踪的 prototype 文档/页面；本节模块源文件、Cargo manifest 与 lock 文件在 scoped status 中干净。
- Rust/MSRV：workspace 声明 Rust 1.98；根 `Cargo.lock` SHA-256 `ECBFB48A5B89794FEE68C05E8FFE282119127C85498CA5D3F79FCC1E2F47C925`。
- 许可证/权属：workspace Cargo metadata 声明 MIT（根 `Cargo.toml` SHA-256 `0B64CC9055EE037B275EC83FA5E561E01EB44318ACA5A7BE793DA1CB73DDD25B`），但根 `LICENSE` SHA-256 `23A543CCBA1D4C58616844D4505A5F101EE6187FD7F7F0C5A69FD4A36884D213` 另加条件，要求任何使用、修改、分发或商业利用先取得版权方书面许可（`LICENSE:5–12`）。两者互相冲突；不能把标准 AGPL 的商业使用许可解释成免除此额外条件。联系对象及权利链未经独立确认。
- 依赖闭包：根 `Cargo.lock` 已哈希；M-15、M-06、M-10 都依赖本仓 `ada-core` 和 Tokio/Serde 系列，M-10 未依赖 SQLx 且以 in-memory middleware 为当前实现；M-16 也是本 workspace 内模块。没有做 workspace feature-resolved transitive SPDX/SBOM 审查，license 闭包未知。
- 候选文件与实现：
- M-15 event bus：`crates/ada-m15-central-event-bus/Cargo.toml` SHA-256 `4EC7C7C37C3A9AD5DCEA358CB3D9B79AD1A8FCE7E5D8E068754324E8DA2C94F4`；`crates/ada-m15-central-event-bus/src/lib.rs` SHA-256 `208738F94B3929DF2B3ADA9673EB50743765ED9087DFB3EEAE570393250EF849`。README 明确当前为 Tokio in-process broadcast；PostgreSQL event log/Redis durable queue 属未来生产部署，存在 Lagged 语义。
- M-10 tenant middleware：`crates/ada-m10-tenant-middleware/Cargo.toml` SHA-256 `D283354F242852A01BCBEAF41F3FAC053EA52BDB8631D6AB0489EEDC93C469D2`；`crates/ada-m10-tenant-middleware/src/lib.rs` SHA-256 `5AB396B1A493ED676919FF9F9A0AD9E0DC1B5049DE2E0C837E277E2DD16E82C9`。源码自述是最小 skeleton；PostgreSQL session context 与 11 张表 RLS 尚未实现。
- M-06 plugin SDK：`crates/ada-m06-plugin-sdk/Cargo.toml` SHA-256 `ABA62C178B155C27001BCECDFB8E63F614AD7A765DDA284F8D9D2852F0C1310E`；`crates/ada-m06-plugin-sdk/src/lib.rs` SHA-256 `9FEBE1931F0C321B663F8C9768DE9DFB5901427A27E2242EB2073FAA907C937C`；`crates/ada-m06-plugin-sdk/src/sandbox.rs` SHA-256 `B644821D8BB13C5D38A501EB8A207046F00F195DCC1FE5021FA0A1BB177772F8`。sandbox 文件明确说 syscall/cgroup 等 enforcement 以后才实现，当前只是保存 capability 与资源上限字段；不能作为生产隔离或内存预算证明。
- M-16 cluster coordinator：`crates/ada-m16-cluster-coordinator/Cargo.toml` SHA-256 `4AED59E41B982E59E60FE3EBB89E7784880661CF07625AB91D5AAB6FCB6DF745`；`crates/ada-m16-cluster-coordinator/src/lib.rs` SHA-256 `0EB49CB82088A2E8A49D8BAD39EEAE688EF3CF5280BAC58D16B5B08525446E32`。当前是 in-process heartbeat/选举；数据库租约为后续设计。

**处置：**不复制、不部署。须先由权利方书面确认与澄清 manifest/LICENSE 冲突，再重新评估；以上代码状态也不足以满足 Star 的可靠消息、RLS、插件隔离或 VM/K3s 管理要求。

### 2.4 `D:\IM1.0` — Unknown; 不得使用 dirty candidate

- Git HEAD：`c1f0979bdad02172ffa01cc006ec71570a9f6dbf`。
- Dirty：是，`crates/im-protocol/src/ws_frames.rs` 有未提交修改，另有 README/test 与文档变更。候选协议文件工作副本 SHA-256 `C5C09BF836B5CA1DB42AC0001D5FC1942E63111CB96E3E7D5F553609F76128EC`；Git HEAD 中该文件 blob 为 `222a3670aecdfd3d20e6b3d399c7b1f9ef195c3e`。本次不把 dirty 内容视为可复用实现。
- Rust/MSRV：workspace 声明 1.80；低于 Star 1.99.0。实际兼容仍需在 Star 固定工具链验证。
- 许可证/权属：根 `Cargo.toml` SHA-256 `24F3015047208DD6E962A7EDCBDDD1940BB642EBA9094F5A409287FC9261D707` 声明 Apache-2.0；根 `LICENSE` SHA-256 `30A9820BE0A745AAC2277D1494136EF1144D348F89743999FBCA1711CC1E75EE` 是带有 Ulysses Leo Lee 版权行的 MIT 文本（`LICENSE:1–12`）。二者都是可商用许可，但授权文本不一致，需确认哪个适用于各源码及分发包；根目录未发现 NOTICE。不能自行挑选更方便的许可。
- 依赖闭包：`Cargo.lock` SHA-256 `AB498651A9CCDE8EDFF0784E9732D3920CA02A3C3C8FA753D30EDCD561CC2995`；module manifests：protocol `C04B7624A785952B2783FFE8DB09A802FE11FFAF371B1E6651EB4B3BDF4F2388`，core `4F30038801254069502AD88CD14E6F2049431576B1094DC6EB0D8608DB3B23F9`，extension runtime `ECC3B459295772C556F4E32000271174351F2061D208C2F4F5BC8DF1BC6A11C9`。core 直接耦合 identity/conversation/message 与 SQLx、Redis、async-nats；完整传递/打包许可证闭包未审。
- 实现与耦合：`crates/im-protocol/src/ws_frames.rs` dirty；干净的 `crates/im-core/src/lib.rs` SHA-256 `35870EC48EEFC29B3400E747664280A1061036450043C895A206595260BCA2AD` 明确把事件发布到 NATS；`crates/extension-runtime/src/lib.rs` SHA-256 `CC3570C9E9C6A3300D5C08F1DD4279EB3E32DC668C8C68D4965DB67293C5C0C4` 标注 V1+ 独立进程与 NATS 订阅为未来实现，当前仅有 placeholder。消息业务域和传输 API 都不应并入 Star 的 Run/owner 边界。

**处置：**暂不分类为可复用模块；先澄清 MIT/Apache 授权范围并审阅 clean HEAD；即使协议格式有参考价值，也只重新设计 Star 自己的版本化事件契约。

## 3. 其余边界发现（排除，不作为候选）

- `D:\CATs` HEAD `d422a66070ec4be2fedd8beceb0518bb74e14ad0`：有 Task/Project/Worker 服务，但属于翻译业务；workspace Cargo 声明 Apache-2.0、根 LICENSE 为 AGPL-3.0（同样需先解决冲突）。不引入其业务域。
- `D:\DOKIPON` HEAD `c7244797cf2b0562d44dd291f33bab245a29723c`：Commerce 业务，Apache-2.0/Rust 1.85；不引入其 commerce/RGS shim 或数据库域。
- `D:\dokipon-gmconsole-wt` HEAD `8cb5de7cb747e2f82f13728ec501d5dfe146d6ca`：`D:\DOKIPON\.git` 的 Apache-2.0/Rust 1.85 commerce-console worktree；只核对根 metadata，不展开业务源码或修改。
- `D:\tests-mock` HEAD `9f5e691f4f375bc27e65885d1a56d4f695ef3096`：仓库明确是 tests-mock adapter/fixtures workspace，MIT OR Apache；仅测试辅助，不是生产服务端实现。
- `D:\GitGit` HEAD `437e4c10b9709deefc9fc60619784eac12cfdba8`：AGPL-3.0、Rust 1.75 的桌面应用而非服务端来源。
- `D:\Physis` HEAD `bf5e3fdba1b34d7501d84c9e57351d4d01ccc1e7`：MIT OR Apache、Rust 1.75 的仿真/物理领域，不是 Star 服务端模块。
- `D:\Rigin` HEAD `d31549e986f0ed721940cb481b63bab2638707ec`：Cargo 声明 MIT OR Apache、Rust 1.98，但根 LICENSE 内容为 AGPL-3.0；有大量脏文档/源码，排除且不碰其脏内容。
- `D:\ROPE` HEAD `2f79b12353c51d044dbc080c6e9e5ceaea4c9480`：Rust/Bevy 游戏项目，root manifest/source 有脏改动及已删除的 `target` 产物；不读取、不恢复、不清理。
- `D:\actix\examples` HEAD `74788ca2b7a450dabc77d551942e7d298f5272e3`：Actix 示例仓而非 Star 业务服务；dirty 清单含 `https-tls/openssl/cert.pem`、`key.pem`，两者均未打开或读取。
- `D:\ex` HEAD `cd67bac544a874f668c706b837c8b3093dfb9545`：Bevy 引擎/示例源，不是 Rust 服务端。
- `D:\bevy\SayServer-0.1.0`：有 Cargo manifest 但未找到 Git 根，按搜索边界排除；没有读取内容。
- C 盘指定 source/repos 下没有新增 Git + Rust 服务端候选；不据目录名推断权属或许可证。

## 4. 许可与依赖放行门

1. 为每个选定源文件确认权利人、适用许可证与 NOTICE；Cargo `license` 字段和根许可证冲突时必须先解决，不能择一。
2. 对所选 crate 的 locked、feature-resolved 直接/传递依赖以及最终发行物生成 SPDX SBOM；逐项核对二进制、安装包、guest image 和 NOTICE/源码提供义务。当前仅记录 manifests 与 lockfile 哈希，**未**完成这些工作。
3. 对 AGPL 代码先确认 Star 商业交付和远程服务是否可以提供要求的 Corresponding Source。AGPL 不等于禁止商用；不能满足其条件时不复制或部署。
4. 如评估 RustGameServer Outbox，仅做最小适配原型：Star 自有 owner-local PostgreSQL 事务写 Outbox/Inbox，adapter 层隔离可替换 transport；禁止 NATS subject/ACK/cursor 泄漏到业务/BI 事实。由 Star 的 Run authorization、budget、idempotency、receipt、取消与性能验收重新定标。
5. 任何候选都不能证明 Multipass VM 生命周期、K3s cluster identity/generation、Host/cluster 双层 reservation、RunInfrastructureBinding 或独立 Agent sandbox；这些必须由 Star 的 Rust Host Infrastructure Manager 和 Multipass adapter 自行提供并验证。

官方许可参考（用于定位条款；不替代权利链及完整依赖闭包审查）：[GNU 对 GPL 商业销售的 FAQ](https://www.gnu.org/licenses/gpl-faq.en.html#GPLCommercially)、[GNU 对 AGPLv3 网络交互要求的说明](https://www.gnu.org/licenses/license-list.en.html#AGPL)、[Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0.html) 与 [Apache 发行许可政策](https://www.apache.org/legal/release-policy.html)。

## 5. 哈希与审计说明

以上文件 SHA-256 都是只读取得的当前工作文件内容哈希。Git HEAD 与 dirty/untracked 状态经 RTK 包装的只读 Git 命令核对；IM1.0 的 dirty 协议文件另记录 HEAD blob ID。未提交来源文件没有被视为可移植代码。没有执行 Cargo build/test/benchmark，也没有写入任何源仓库。
