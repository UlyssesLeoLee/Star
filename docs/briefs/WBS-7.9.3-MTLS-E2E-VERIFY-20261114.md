# WBS 7.9.3 mTLS 与服务身份供应、commit-before-ACK 恢复矩阵 — E2E 验证报告

- **Brief**: docs/briefs/D21-mtls-e2e.md
- **日期**: 2026-11-14
- **分支**: agent/wbs-mtls-20261114（基于 dev，只 commit 不 push）
- **验证目标**: native gRPC TLS 端到端验证 + 证书撤销拒绝

## 完成项

1. 重构 `crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs`，把原先全部塞在一个 ignored
   PG 用例里的四个场景拆分为「可执行 TLS 层用例」+「PG-backed opt-in 用例」：
   - `trusted_client_completes_the_mtls_handshake_and_reaches_the_service`（可执行，无 PG）：
     真实生产 `client_tls_config`/`server_mtls_config` + 真实 `NativeGrpcTransportServer`，可信客户端完成
     mTLS 握手且请求抵达真实服务（以服务端独有的数据库错误映射消息为证据，非 TLS 层失败）。
   - `client_without_a_certificate_is_rejected`（可执行）：无证书客户端被强制 mTLS 握手拒绝。
   - `client_trusting_an_unrelated_ca_is_rejected`（可执行）：错误 CA 客户端拒绝服务器证书。
   - `client_pinning_a_mismatched_server_name_is_rejected`（可执行）：错误 server name 客户端拒绝。
   - `trusted_append_succeeds_and_sql_revocation_fails_the_followup`（ignored，opt-in）：
     可信证书 → bounded append 成功（log_id/generation/sequence>0/wire_sha256 全部核对）；
     SQL 层撤销该 peer 证书后，同一已建立 TLS 连接上的下一次 authenticated 操作被拒绝
     （`PermissionDenied`）。不 fake business server、不合成撤销决策。
2. TLS 仅握手用例使用「从不连接的 lazy PgPool」构造真实 `NativeGrpcTransportServer`，因此
   不需要 PostgreSQL 即可执行——这是真实服务，不是替代业务服务。
3. PG 用例保留精确环境要求（见下），本机无 live PG 时如实报告 unexecuted，不把 ignored 当 pass。

## 验证输出（实测命令+结果）

```
$ cargo fmt --all --check
（无输出，exit 0）

$ cargo check -p platform-messaging-runtime --all-targets | grep -cE "^error"
0

$ cargo check -p platform-messaging-runtime --test native_grpc_tls_e2e --features native-grpc-adapter | grep -cE "^error"
0

$ cargo test -p platform-messaging-runtime --test native_grpc_tls_e2e --features native-grpc-adapter
running 5 tests
test trusted_append_succeeds_and_sql_revocation_fails_the_followup ... ignored, requires STAR_PLATFORM_PROFILE_TEST_DATABASE_URL pointing at a loopback disposable PostgreSQL database named star_d19_test on a non-default port, with the four D20 migrations applied and login memberships in star_platform_profile_owner, star_platform_transport_owner, and star_platform_transport_runtime
test client_without_a_certificate_is_rejected ... ok
test client_trusting_an_unrelated_ca_is_rejected ... ok
test client_pinning_a_mismatched_server_name_is_rejected ... ok
test trusted_client_completes_the_mtls_handshake_and_reaches_the_service ... ok

test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 2.05s

$ cargo test -p platform-messaging-runtime --test native_grpc_tls_e2e
test result: ok. 0 passed; 0 failed; 0 ignored   （默认 features 下 native-grpc-adapter 关闭，目标按 cfg 编译为空）

$ cargo test -p platform-messaging-runtime
lib 36 passed；native_origin_verification_pg 2 passed / 2 ignored；transport_profile_pg 1 ignored —— 全部 green

$ python .agents/.../cgg_guard.py --changed --stamp  → CypherGraph Guardian: PASS (1 source file(s) checked)
```

环境事实：本机无 `pg_isready`、无 5432 监听、`STAR_PLATFORM_PROFILE_TEST_DATABASE_URL` 未设置 →
PG-backed 用例按 opt-in 设计保持 ignored。

## 文件清单

- 修改 `crates/platform-messaging-runtime/tests/native_grpc_tls_e2e.rs`（@cypher 头已更新并 stamp）
- 新增 `docs/briefs/WBS-7.9.3-MTLS-E2E-VERIFY-20261114.md`（本报告）

## 未完成 + 原因（含 ignored 用例的环境要求）

- PG-backed 用例 `trusted_append_succeeds_and_sql_revocation_fails_the_followup` 未执行：
  - 原因：本机无 live PostgreSQL（无 5432 监听、env 未设置）。
  - 环境要求：`STAR_PLATFORM_PROFILE_TEST_DATABASE_URL` 指向 loopback 上非 5432 端口的
    一次性数据库 `star_d19_test`，已应用 4 个 D20 migration，且登录账号具有
    `star_platform_profile_owner` / `star_platform_transport_owner` / `star_platform_transport_runtime`
    三个角色成员资格。满足后运行：
    `cargo test -p platform-messaging-runtime --test native_grpc_tls_e2e --features native-grpc-adapter -- --ignored`
  - 该用例被明确标记为 ignored，不视为通过；其代码路径（除 DB 事实外）与已执行用例共用同一生产
    TLS 配置与服务构造。
