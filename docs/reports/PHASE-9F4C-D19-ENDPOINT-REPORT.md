# PHASE-9F4C-D19-ENDPOINT-REPORT

## §0 目的

本报告记录 D19 Transport Profile 的部署可信 endpoint identity 解析、current Profile composition 与 native gRPC client binding follow-up。该工作属于 TLS/Profile production wiring，不代表后续 D21 Multipass/K3s 阶段已启动，也不启用 native transport traffic 或 Schedule/Run execution。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `crates/platform-messaging-runtime/src/transport_endpoint.rs` | 新增有界 deployment trust registry、profile-bound HTTPS/TLS endpoint resolver、脱敏 Debug 和 fail-closed 单测 | 已实现；Rust 1.98.1 定向测试与 feature 编译通过，未达到 1.99.0 基线 |
| `crates/platform-messaging-runtime/src/lib.rs` | 导出 resolver 模块并同步 Cypher 关系头 | 已实现；Guardian 通过 |
| `crates/platform-messaging-runtime/src/trusted_profile_endpoint.rs` | 每次通过 PostgreSQL current Profile resolver 与部署可信 endpoint registry 组合；错误向外保持有界 | 已实现；不缓存 Profile revision，不接受请求提供的 endpoint |
| `crates/platform-messaging-runtime/src/native_grpc.rs` | send/receive config 改为必须绑定 `ResolvedTransportEndpoint`；建连前核对 provider、Profile digest、endpoint identity | 已实现；typed HTTPS/TLS destination，拒绝裸 URI 配置 |
| `crates/run-owner-runtime/src/native_schedule_admission_consumer.rs` | 将 Result-route 连接前拒绝测试 fixture 迁移到 trusted Profile/endpoint resolver | 已实现；focused test 1/1 通过 |
| `docs/reports/PHASE-9F4C-D19-ENDPOINT-REPORT.md` | 记录本 lane 实施与验证状态 | 已完成 |

解析器绑定 deployment UUID，只按 Profile revision 中的精确 `endpoint_identity` 选择部署配置，并将 profile ID、revision 和 digest 一并固定在返回类型中。URI 与 TLS server name 必须匹配；URI 验证拒绝非 HTTPS、userinfo、query、fragment、非法端口/路径、过长值及不受支持的 IP/wildcard TLS 名称。Registry 最多 64 项，并拒绝重复 identity 和重复规范化目的地。profile Disabled/Revoked、未知 identity 与 deployment mismatch 均 fail closed。凭证只透传 `OpaqueSecretReference`，从不读取 secret 内容。

## §2 验证摘要

| 守门 | 命令/证据 | 结果 |
|---|---|---|
| CypherGraph Guardian | `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --root . --changed --stamp` | PASS；检查 2 个 Rust 源文件 |
| CodeGraph | 查询 current resolver/client symbols | MCP 未返回本阶段目标符号，结果落到无关 worktree shared-dir resolver；实现审查以合并 diff、精确 source commit 和定向 Cargo 验证为准 |
| RTK | `rtk --version` | BLOCKED；当前 PowerShell 环境无法解析 `rtk`，Rust 命令改为直接调用已安装工具链路径 |
| Rust 1.99.0 基线 | Rustup toolchain inventory | BLOCKED；本机没有 1.99.0。可用诊断工具链为 `C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin`，以下 Rust/Cargo 结果均低于项目基线 |
| 修改文件格式 | Rust 1.98.1 `rustfmt --check --edition 2021 --config skip_children=true crates/platform-messaging-runtime/src/transport_endpoint.rs crates/platform-messaging-runtime/src/lib.rs` | PASS；仅检查本 lane 两个 Rust 文件，避免递归格式化未修改的子模块 |
| 整 crate 格式 | Rust 1.98.1 `cargo fmt --check -p platform-messaging-runtime` | FAIL；还报告多个未修改文件的格式差异（`admission_delivery.rs`、`admission_receive.rs`、`native_grpc*.rs`、`transport_profile_pg.rs`）；未格式化这些无关文件 |
| 单元测试 | Rust 1.98.1 `cargo test --offline --locked --ignore-rust-version -p platform-messaging-runtime --lib -j 4` | PASS；35/35；包含本 lane 8 个 resolver 单测及 Debug path/secret 脱敏断言。此结果低于 workspace 1.99.0 基线 |
| 无默认 features 库检查 | Rust 1.98.1 `cargo check --offline --locked --ignore-rust-version -p platform-messaging-runtime --no-default-features --lib -j 4` | PASS；只出现现有 `transport_profile.rs` dead-code warning；此结果低于 workspace 1.99.0 基线 |
| native gRPC feature 全 targets | Rust 1.98.1 `cargo check --offline --locked --ignore-rust-version -p platform-messaging-runtime --all-targets --features native-grpc-adapter -j 4` | PASS；存在依赖 crate 既有 deprecation warning；此结果低于 workspace 1.99.0 基线 |
| 合并后联合 feature all-targets check | Rust 1.98.1 `cargo check --offline --locked --ignore-rust-version -p platform-messaging-runtime -p run-owner-runtime --all-targets --features platform-messaging-runtime/native-grpc-adapter,run-owner-runtime/native-grpc-adapter -j 4` | PASS；`domain-kms` 有 3 条未修改的 `aes-gcm` deprecation warnings；此结果低于 workspace 1.99.0 基线 |
| 合并后 library tests | Rust 1.98.1 `cargo test --offline --locked --ignore-rust-version -p platform-messaging-runtime -p run-owner-runtime --lib --features platform-messaging-runtime/native-grpc-adapter,run-owner-runtime/native-grpc-adapter -j 4` | PASS；platform messaging 53/53，Run-owner 22/22，含新增 profile/client guards 与 pre-connect route test；此结果低于 workspace 1.99.0 基线 |
| 合并源代码格式与图头 | Rust 1.98.1 targeted rustfmt + CypherGraph Guardian | PASS；`lib.rs`、resolver、caller fixture 的 rustfmt 检查通过；Guardian 对 4 个变更 Rust 源文件 PASS。完整 `native_grpc.rs` rustfmt check 只报告 `tonic` import 排序，原样存在于 base `30d48377`，本次未格式化该无关行 |
| 合并代码 whitespace | `git diff 30d48377..HEAD --check` | PASS |
| 无默认 features 全 targets | Rust 1.98.1 `cargo check --offline --locked --ignore-rust-version -p platform-messaging-runtime --no-default-features --all-targets -j 4` | FAIL；现有 `transport_profile_pg` 与 `native_origin_verification_pg` integration tests 在 feature 关闭时仍无条件导入被 gate 的模块；同配置下 `--lib` 检查已通过 |
| 独立 CodeRabbit review | `coderabbit --version` | BLOCKED；Windows wrapper 最终调用的 `/root/.local/bin/coderabbit` 返回 `Permission denied`，exit 126；不视作 review pass |

## §3 已知缺口

1. Profile resolver 与 native client API 已代码级组合，但 application production startup/route 尚未实例化 resolver、提供当前认证 session 并调用 client；native traffic 维持关闭。
2. D19 既有真实 PostgreSQL-backed Tonic mTLS/peer-identity/revocation E2E 为 1/1；本 follow-on 未重跑该网络场景。生产证书签发/轮换/吊销控制面仍未验收；本解析器只验证 DNS TLS server name 与固定 HTTPS host 相符。
3. 本环境没有 Rust 1.99.0；新增单测和 feature 编译使用 Rust 1.98.1 加 `--ignore-rust-version` 做诊断验证，需在 1.99.0 环境复跑。
4. 当前输出不给调用方任意 URI 或 redirect target，也不提供 redirect override。具体 transport adapter 的 no-follow 行为仍需在集成时验证。
5. `TrustedProfileEndpointResolver` 每次从 PostgreSQL 读取当前 Profile revision，再用固定 deployment registry 解析，不缓存 revision。调用方仍须在每次建立/重建 channel 前调用它；现有 channel 不会由此自动更新或关闭。
6. `TrustedEndpointConfig`、resolver 和 resolved endpoint 的 Debug 输出只暴露 endpoint identity、TLS host 与 profile pin，不输出 HTTPS URI path 或 opaque secret reference；新增单测覆盖三类 Debug 输出。
7. 本报告只描述 D19 follow-up；D21 Multipass/K3s 阶段未开始。

## §4 子代理失败接手清单

Profile resolver 与 native client 两项由两个子代理在独立 Orca worktree 并行实现。协调端发现 client API 变更导致 Run-owner 测试仍构造裸 URI，遂补齐 trusted endpoint fixture 与 Cypher 关系头，并复跑 focused test。随后先 rebase/fast-forward Profile lane，再把 native client lane rebase 到新 dev 并 fast-forward；没有 cherry-pick 或合入其他分支的大型改动。CodeRabbit wrapper 最终调用 `/root/.local/bin/coderabbit` 时返回 permission denied；独立 CodeRabbit review 未完成，不记作通过。

## §5 守门规则

- Endpoint 选择只接受不可变 Profile revision 中的 exact identity，并限定在一个 deployment trust registry 内。
- 解析器配置来源必须是部署可信配置；请求不能提供或覆盖 URI、host、TLS name 或 redirect target。
- profile 非 Active、deployment 不匹配、identity 未登记或 registry 有歧义时 fail closed。
- TLS name 只接受无 wildcard 的 ASCII DNS 名称；URI 必须为 HTTPS，host 与 TLS name 一致。
- registry、identity、URI 和 path 均有显式长度/数量上限；secret 内容不进入 resolver，仅保留 opaque reference。
- endpoint resolution 本身不创建连接、不改变默认 feature、不打开 native traffic 或任务执行。
- CypherGraph Guardian、定向编译与单测均通过；Rust 1.99.0 基线、profile freshness 集成和真实 TLS/adapter 行为仍未验证，不能据此宣称生产闭环完成。

## §6 签字栏

| 角色 | 记录 |
|---|---|
| 架构 | 协调端对照 D19 API composition 与 fail-closed 边界完成手工自审；生产 startup wiring 仍开放 |
| SRE Lead | 本 follow-on 未运行部署/集群验证；D21/production operations 门禁仍开放 |
| 平台 | Rust 1.98.1 联合 all-targets check 与两 crate library tests 通过；此前 D19 TLS E2E 1/1，本 follow-on 不重跑网络场景；Rust 1.99.0 正式门仍未完成 |
| 评审主持 | 对三项合入提交与文档同步完成协调端 review；CodeRabbit wrapper 权限失败，未获得独立 CodeRabbit review |
| PM | 本 follow-on 只完成 resolver/client API composition；不代表 D19/D21 或 Run execution 总体完成 |

## §7 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | endpoint-resolver worker | 新增部署可信 endpoint identity 到 HTTPS URI/TLS server-name 的有界解析器和本报告；明确阶段标签为 D19 follow-up | 协调端确认实施计划 §6.134 对应 D19 Profile/TLS production wiring，D21 保留给 Multipass/K3s |
| v0.2 | endpoint-resolver worker | 补充 1.98.1 诊断测试/feature 检查结果、整 crate formatter 的既有差异、独立 review 限制及 Rust 1.99.0 缺口 | 目标 crate 单测与 feature checks 完成 |
| v0.3 | endpoint-resolver worker | 更正定向编译/测试与 rustfmt 状态，记录不可变 profile snapshot 的 freshness 要求，并补充 URI path/secret reference 的 Debug 脱敏 | 最终静态自审与守门复跑 |
| v0.4 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 合入 current PostgreSQL Profile→trusted endpoint resolver 与 native client typed endpoint guard，修复 Run-owner fixture，并同步 merged dev 的 53/53、22/22 library tests、联合 all-targets check、Guardian 与残留生产门 | 用户要求子代理/worktree 并行处理后 rebase/merge 到 dev；合并验证完成 |
