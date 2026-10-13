# PHASE-9F4C-D19-MTLS-E2E-REPORT

**版本**：v0.1
**日期**：2026-10-14
**工作分支**：`UlyssesLeoLee/d21-mtls-e2e`
**基线**：`d7cf5d3b`（dispatcher brief）
**范围校正**：派发 brief 保留原名 `D21-mtls-e2e` 作为任务记录；协调端已确认本实现是 D19 TLS E2E 后续，不代表 D21 Multipass/K3s 阶段启动。

## §0 目的

以真实 Tonic mTLS 握手和 PostgreSQL-backed `NativeGrpcTransportServer` 补足 D19 验收缺口：可信客户端可以提交一次真实 RPC；缺少客户端证书、错误服务器 CA、服务器名称不匹配均被 TLS 层拒绝；数据库撤销已呈现的客户端证书后，同一已建立 TLS 客户端连接的下一次 RPC 被授权数据库函数拒绝。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs` | 新增 opt-in ignored 集成测试；使用生产 TLS config helper、真实 gRPC 服务、真实 PostgreSQL profile/certificate/log/binding 事实；写 fixture 前读取 `current_database()` 并严格要求 `star_d19_test`；生成临时测试证书并覆盖四项拒绝/成功路径 | E2E 实跑通过；新增 DB 名称守门已编译 |
| `crates/platform-messaging-runtime/Cargo.toml` | 添加 `rcgen`、`tempfile`、`tokio-stream` 到 `[dev-dependencies]`；测试同时要求 `native-grpc-adapter` 与 `durable-quarantine` feature | 已实现；生产依赖与默认 feature 未变 |
| `Cargo.lock` | 为新增 test-only 闭包锁定 14 个 crate；既有 `bit-vec`、`pem`、`synstructure` 条目只作多版本消歧，未升级既有版本 | 已审阅 |
| `docs/reports/PHASE-9F4C-D19-MTLS-E2E-REPORT.md` | 记录本 lane 的改动、验收、环境审计及限制 | 本报告 |

依赖审阅：上游 `rcgen 0.14.10` 于 2026-08-28 发布，许可为 `MIT OR Apache-2.0`；新增闭包的 Cargo metadata 许可均为 MIT、Apache-2.0 或二者择一。直接测试依赖 `tempfile 3.27.0`（MIT OR Apache-2.0）和 `tokio-stream 0.1.19`（MIT）也只在 dev-dependencies 中。它们不进入产品运行依赖闭包或分发内容。Cargo.lock 的差异仅来自这一测试依赖闭包及多版本引用消歧。

## §2 验证摘要

| 验证 | 结果 |
|---|---|
| Rust toolchain | 使用已安装 Rust/Cargo `1.98.1` 做诊断验证；此版本低于项目要求 `1.99.0`，不能替代正式版本门 |
| `rustfmt --edition 2021 --check crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs` | 通过 |
| `cargo test --locked --ignore-rust-version -j 4 --no-default-features --features native-grpc-adapter,durable-quarantine -p platform-messaging-runtime --test native_grpc_tls_e2e --no-run` | 编译通过；仅代表该测试 target 的诊断编译 |
| `cargo test --ignore-rust-version -j 4 --no-default-features --features native-grpc-adapter,durable-quarantine -p platform-messaging-runtime --test native_grpc_tls_e2e -- --ignored --nocapture` | **1 passed，0 failed**；实际连接独立 PostgreSQL 并经过真实 gRPC/TLS RPC |
| CypherGraph Guardian `cgg_guard.py --changed --stamp` | 通过，检查 1 个新增 Rust 源文件并写入当前 `@cypher` source hash |
| `git diff --check` | 通过 |

后续审查补充：集群清理后增加了数据库身份 fail-closed 检查。该修订执行 target `--no-run` 编译、rustfmt、Guardian 和 diff check；按协调端要求没有重新启动 PostgreSQL。先前真实 E2E 已在名称为 `star_d19_test` 的 disposable DB 上实跑通过；本次新增 `current_database()` 守门本身只有编译验证，没有在已清理的实例上再运行。

E2E 实证路径：可信客户端 Append 写入 PostgreSQL-backed 服务并返回持久化回执；缺少 client identity、错误 CA、错误 server name 的客户端 RPC 均失败；owner-scoped SQL 更新将该客户端证书设为 revoked 后，复用原 `DurableTransportClient` 发起下一次 Append，返回 `PermissionDenied`。业务连接池在每条连接上执行 `SET ROLE star_platform_transport_runtime`，fixture 创建和撤销使用对应 owner scope 与事务。没有替代业务服务、mock 或合成撤销结果。

数据库审计摘要：仅为本 lane 新建 WSL PostgreSQL 18.6 集群，PGDATA 为 `/tmp/star-d19-mtls-e2e-20261014-01`，数据库为 `star_d19_test`，仅监听 `127.0.0.1:55432`；未连接或写入既有 `5432` 实例。按依赖顺序应用且只应用以下迁移：

1. `2026-10-12-platform-transport-profile-master.sql`
2. `2026-10-12-platform-native-transport-log.sql`
3. `2026-10-12-platform-transport-z-native-server-auth.sql`
4. `2026-10-12-platform-native-origin-verification.sql`

核验了必需 profile/certificate/log 表、Append 函数及临时登录角色的 profile-owner、transport-owner、transport-runtime membership；测试后仅对该 lane 集群执行 fast shutdown，确认 `55432` 不再响应、`5432` 仍接受连接，再删除已解析确认的 lane PGDATA。最终 Multipass inventory 与开始时一致：仅 `informed-impala`，状态 `Running`，IPv4 `172.27.240.143`，无 inventory 变更。

工具记录：`rtk` 不在 PATH（`Get-Command rtk` 无结果），因此使用 Rust 工具链绝对路径直接运行命令；没有因工具缺失宣称正式 Rust 1.99 门通过。

## §3 已知缺口

- Rust 1.99.0 尚未安装；本报告中的编译与实跑结果是 Rust 1.98.1 诊断门，须在正式 1.99.0 环境重跑。
- 这个集成测试默认 ignored；执行者必须提供 `STAR_PLATFORM_PROFILE_TEST_DATABASE_URL` 指向 disposable PostgreSQL。测试代码在任何 fixture 写入前使用 SQL `current_database()` 精确要求 `star_d19_test`，并 fail closed 拒绝非 loopback 地址或默认 5432 端口。
- 本 lane 只关闭 D19 的真实 gRPC/TLS/证书撤销验证缺口；没有启动或验收 D21 Multipass/K3s provider、集群绑定、容量预留或 Agent sandbox。
- 未运行 workspace 全量 test、clippy 或 release/doc/bench gates；不能据此声称全 workspace 质量门通过。

## §4 子代理失败接手清单

- 本 lane 未派生子代理，没有子代理提交待接手。
- 首次迁移顺序把 server-auth 放在 transport-log 之前，PostgreSQL 返回 `schema "platform_transport" does not exist`。该迁移在自身 `BEGIN` 事务内失败回滚；随后按 §2 的正确顺序在 lane-only 数据库重新应用并通过 schema/role/function 审计。没有触碰 5432。
- 没有遗留后台 Cargo、测试服务或 PostgreSQL 进程；lane 临时证书目录由测试结束时清理。

## §5 守门规则

| # | 本 lane 的守门条件 | 证据/结果 |
|---|---|---|
| 1 | 默认不执行；必须显式运行 ignored 集成测试 | 成功测试命令含 `-- --ignored` |
| 2 | DB URL 必须解析成功且 host 为 loopback | 测试入口检查通过 |
| 3 | 拒绝默认 PostgreSQL 5432 | 测试入口检查通过；本次端口为 55432 |
| 4 | 任何 fixture 写入前必须核对数据库身份 | `SELECT current_database()` 必须精确返回 `star_d19_test`；修订后 target `--no-run` 编译通过 |
| 5 | PostgreSQL 只监听 loopback | 临时 `pg_ctl` 启动参数固定为 `127.0.0.1` |
| 6 | 只应用四个直接所需迁移，按依赖顺序执行 | 四个迁移均成功提交 |
| 7 | 测试 RPC 使用 runtime role，不用 superuser 执行业务请求 | pool `after_connect` 设置 runtime role，Append 通过 |
| 8 | fixture 写入/撤销必须带 owner scope 和事务 | 创建及撤销助手均在本 lane owner 事务内 |
| 9 | 证书与私钥只能测试生成且短期保存 | rcgen 生成并放在 `TempDir`，结束后释放 |
| 10 | 服务端必须使用 production mandatory-mTLS TLS helper | `server_mtls_config` 实际用于 Tonic server |
| 11 | RPC 必须使用真实 PostgreSQL-backed server | `NativeGrpcTransportServer` 连接 SQLx pool |
| 12 | 可信 client 必须到达实际 Append RPC | E2E 回执、log id、sequence、payload digest 断言通过 |
| 13 | 缺 client cert 必须被拒绝 | 真实 TLS RPC 返回错误 |
| 14 | 错误 CA 与错误 server name 必须分别失败 | 两条独立 TLS client 负例通过 |
| 15 | 撤销必须是真实 DB 身份事实，且同一客户端下一 RPC 拒绝 | SQL 设置 revoked；复用已连接 client 后得到 `PermissionDenied` |
| 16 | native gRPC feature 保持默认关闭，依赖仅限测试构建 | `native-grpc-adapter` 不在默认 feature，新增 crate 位于 dev-dependencies |
| 17 | Rust 1.98.1 结果不可冒充 1.99.0 正式门 | 报告单独列出正式 toolchain 缺口 |

## §6 签字栏

| 角色 | 审核范围 | 记录 |
|---|---|---|
| 架构 | D19 mTLS 与真实服务边界 | 架构师（Mavis 接手 agent per DEC-008；限本 lane） |
| SRE Lead | 独立 PostgreSQL 生命周期、端口与清理证据 | Mavis 接手 agent per DEC-008（限本 lane） |
| 平台 | Tonic TLS、runtime role、撤销 RPC | Mavis 接手 agent per DEC-008（限本 lane） |
| 评审主持 | 变更与验证证据 | Mavis 接手 agent per DEC-008（限本 lane） |
| PM | 阶段标签及已知缺口 | Mavis 接手 agent per DEC-008（限本 lane） |

修订人：`Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核`。本报告批准范围只覆盖此 D19 lane；rebase 与合入 dev 由协调端串行 review，本 lane 未执行 push、rebase 或 merge。

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-14 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增真实 gRPC mTLS/PostgreSQL E2E、测试依赖闭包与验证报告；按协调端要求修正阶段为 D19 follow-up | D19 TLS live-handshake 验收缺口 |
