# PHASE-9F4C-D19-ENDPOINT-REPORT

## §0 目的

本报告记录 D19 Transport Profile 的部署可信 endpoint identity 解析 follow-up。该工作属于 TLS/Profile production wiring，不代表后续 D21 Multipass/K3s 阶段已启动，也不启用 native transport traffic 或 Schedule/Run execution。

## §1 改动矩阵

| 文件 | 改动 | 状态 |
|---|---|---|
| `crates/platform-messaging-runtime/src/transport_endpoint.rs` | 新增有界 deployment trust registry、profile-bound HTTPS/TLS endpoint resolver、脱敏 Debug 和 fail-closed 单测 | 已实现；Rust 1.98.1 定向测试与 feature 编译通过，未达到 1.99.0 基线 |
| `crates/platform-messaging-runtime/src/lib.rs` | 导出 resolver 模块并同步 Cypher 关系头 | 已实现；Guardian 通过 |
| `docs/reports/PHASE-9F4C-D19-ENDPOINT-REPORT.md` | 记录本 lane 实施与验证状态 | 已完成 |

解析器绑定 deployment UUID，只按 Profile revision 中的精确 `endpoint_identity` 选择部署配置，并将 profile ID、revision 和 digest 一并固定在返回类型中。URI 与 TLS server name 必须匹配；URI 验证拒绝非 HTTPS、userinfo、query、fragment、非法端口/路径、过长值及不受支持的 IP/wildcard TLS 名称。Registry 最多 64 项，并拒绝重复 identity 和重复规范化目的地。profile Disabled/Revoked、未知 identity 与 deployment mismatch 均 fail closed。凭证只透传 `OpaqueSecretReference`，从不读取 secret 内容。

## §2 验证摘要

| 守门 | 命令/证据 | 结果 |
|---|---|---|
| CypherGraph Guardian | `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --root . --changed --stamp` | PASS；检查 2 个 Rust 源文件 |
| CodeGraph | 对当前 worktree 查询 | 当前 worktree 无 `.codegraph/`，MCP 返回未索引；未执行索引操作 |
| RTK | `rtk --version` | BLOCKED；当前 PowerShell 环境无法解析 `rtk`，Rust 命令改为直接调用已安装工具链路径 |
| Rust 1.99.0 基线 | Rustup toolchain inventory | BLOCKED；本机没有 1.99.0。可用诊断工具链为 `C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin`，以下 Rust/Cargo 结果均低于项目基线 |
| 修改文件格式 | Rust 1.98.1 `rustfmt --check --edition 2021 --config skip_children=true crates/platform-messaging-runtime/src/transport_endpoint.rs crates/platform-messaging-runtime/src/lib.rs` | PASS；仅检查本 lane 两个 Rust 文件，避免递归格式化未修改的子模块 |
| 整 crate 格式 | Rust 1.98.1 `cargo fmt --check -p platform-messaging-runtime` | FAIL；还报告多个未修改文件的格式差异（`admission_delivery.rs`、`admission_receive.rs`、`native_grpc*.rs`、`transport_profile_pg.rs`）；未格式化这些无关文件 |
| 单元测试 | Rust 1.98.1 `cargo test --offline --locked --ignore-rust-version -p platform-messaging-runtime --lib -j 4` | PASS；35/35；包含本 lane 8 个 resolver 单测及 Debug path/secret 脱敏断言。此结果低于 workspace 1.99.0 基线 |
| 无默认 features 库检查 | Rust 1.98.1 `cargo check --offline --locked --ignore-rust-version -p platform-messaging-runtime --no-default-features --lib -j 4` | PASS；只出现现有 `transport_profile.rs` dead-code warning；此结果低于 workspace 1.99.0 基线 |
| native gRPC feature 全 targets | Rust 1.98.1 `cargo check --offline --locked --ignore-rust-version -p platform-messaging-runtime --all-targets --features native-grpc-adapter -j 4` | PASS；存在依赖 crate 既有 deprecation warning；此结果低于 workspace 1.99.0 基线 |
| 无默认 features 全 targets | Rust 1.98.1 `cargo check --offline --locked --ignore-rust-version -p platform-messaging-runtime --no-default-features --all-targets -j 4` | FAIL；现有 `transport_profile_pg` 与 `native_origin_verification_pg` integration tests 在 feature 关闭时仍无条件导入被 gate 的模块；同配置下 `--lib` 检查已通过 |
| 独立 CodeRabbit review | `coderabbit --version` | BLOCKED；Windows wrapper 最终调用的 `/root/.local/bin/coderabbit` 返回 `Permission denied`，exit 126；不视作 review pass |

## §3 已知缺口

1. 此 lane 没有接入平台启动配置加载器，也没有将 resolver 串接到 `PgTransportProfileResolver` 与 native gRPC client；native traffic 维持关闭。
2. 证书吊销/轮换、mTLS handshake 与 peer identity 端到端验证属于独立 TLS lane；本解析器只验证 DNS TLS server name 与固定 HTTPS host 相符。
3. 本环境没有 Rust 1.99.0；新增单测和 feature 编译使用 Rust 1.98.1 加 `--ignore-rust-version` 做诊断验证，需在 1.99.0 环境复跑。
4. 当前输出不给调用方任意 URI 或 redirect target，也不提供 redirect override。具体 transport adapter 的 no-follow 行为仍需在集成时验证。
5. Resolver 输入必须是 trusted profile resolver 刚读取的当前 revision；它持有不可变快照，无法自行观察后续 disable/revocation。调用方必须在每次使用前刷新并校验当前 revision。
6. `TrustedEndpointConfig`、resolver 和 resolved endpoint 的 Debug 输出只暴露 endpoint identity、TLS host 与 profile pin，不输出 HTTPS URI path 或 opaque secret reference；新增单测覆盖三类 Debug 输出。
7. 本报告只描述 D19 follow-up；D21 Multipass/K3s 阶段未开始。

## §4 子代理失败接手清单

本 lane 未派生子代理；无失败子代理产出需要接手。协调端 review、串行 rebase 和合并由 parent 完成，本 worker 不执行 rebase、合并或 push。

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
| 架构 | 本 worker 完成本 lane 自检；D19 总体架构审核由协调端在集成时完成 |
| SRE Lead | 本 lane 未运行部署/集群验证；协调端集成门禁需复核 |
| 平台 | 本 lane 使用 Rust 1.98.1 完成 crate 单测与 feature 编译；未达到 Rust 1.99.0 基线，且未运行真实 TLS handshake；协调端集成门禁需复核 |
| 评审主持 | 本 worker 已检查本 lane 文件边界与 Cypher 头；协调端 review 尚未执行 |
| PM | 本 lane scope 与 D19 follow-up 一致；不代表 D19/D21 总体完成 |

## §7 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | endpoint-resolver worker | 新增部署可信 endpoint identity 到 HTTPS URI/TLS server-name 的有界解析器和本报告；明确阶段标签为 D19 follow-up | 协调端确认实施计划 §6.134 对应 D19 Profile/TLS production wiring，D21 保留给 Multipass/K3s |
| v0.2 | endpoint-resolver worker | 补充 1.98.1 诊断测试/feature 检查结果、整 crate formatter 的既有差异、独立 review 限制及 Rust 1.99.0 缺口 | 目标 crate 单测与 feature checks 完成 |
| v0.3 | endpoint-resolver worker | 更正定向编译/测试与 rustfmt 状态，记录不可变 profile snapshot 的 freshness 要求，并补充 URI path/secret reference 的 Debug 脱敏 | 最终静态自审与守门复跑 |
