# Phase 9F4I — Native event transport authorization

v0.2 · 2026-10-18 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核

## §0 目的

解除自有 Rust transport 对 Task event 的类别阻断；保持独立授权与 owner 事务边界。全部 Worktree-first phases 仍未完成。

## §1 改动矩阵

| 范围 | 实际改动 |
|---|---|
| Rust server | 原生 RPC 与 delivery 类别校验加入 event；未知/大小写不同类别拒绝 |
| 新迁移 | 绑定表类别 CHECK 与 authorize_profile_route 参数白名单扩展；不修改旧迁移，不改变旧 Profile |
| 授权边界 | certificate/tenant/provider/endpoint、current Profile、服务方向、独立 kind、log binding/generation、锁和生命周期条件保留；helper 不向 runtime/PUBLIC 开放 |
| 数据分类 | 没有新增表，Master/Work/Transaction 分类、既有 retention、RLS 与 WORM 保留 |
| 自动化 | 新增专用唯一目录/loopback disposable PostgreSQL runner，按历史顺序验证来源证明升级 |
| 文档 | Requirements/basic/Task SRS/BD/DD、Data Design、Rust reuse design、实施计划及 automation registry 同步 |

## §2 验证摘要

- PostgreSQL：`python scripts/automation/phase9f4i_native_event.py --pg-bin D:\PostgreSQL\18\bin`，15/15 步通过。6 个迁移各应用两次，server-auth pre-attestation fixture、origin verification 与 event contract 各一次。每份契约包含多条 assertions；15 是步骤数，不是断言总数。
- Event 契约覆盖实际 authenticated DB Append → replay → Pull → VerifyOrigin → ACK，changed-wire identity conflict、错误 generation、未知 peer/kind、event-only Profile 拒绝 request、旧 request-only Profile 拒绝 event；继承既有 grants/RLS/WORM 与来源证明检查。
- 首次运行因先执行 origin migration 再写旧 fixture 触发 `pre-attestation message must remain explicitly unverified`。修复 runner 顺序，保留失败诊断，之后三次运行通过；没有修改 origin 验收要求。
- Rust 1.99.0：`cargo test -p platform-messaging-runtime --no-default-features --features native-grpc-adapter --lib --locked --offline -j 4`，44/44，通过。此结果只覆盖 native-only library，非全 workspace 或真实网络测试。
- Rust 1.99.0：`cargo check -p platform-messaging-runtime --features native-grpc-adapter --all-targets --locked --offline -j4`，通过；这是该 crate 的 native-gRPC feature all-targets 检查，不是 workspace 全量检查。
- 变更文件 `native_grpc_server.rs` 通过定向 `rustfmt --check --edition 2024`。`cargo fmt -p platform-messaging-runtime -- --check` 仍报告其他未改文件的既存格式差异（包括 `admission_delivery.rs` 与 `native_origin_verification_pg.rs`）；包级 fmt 门未通过，因此没有对整个 crate 做格式化改写。
- AssetsLake `event_publisher_service.rs` 只读复核，SHA-256 仍为 `3A2114863827CB8CE893AD5A24090D827F89B7752C6FF23546DBDB4DC3B8A4A5`；其 dry-run 标发布与未链接 Kafka 逻辑不满足 Star durable transport 要求，本片没有复制源代码。
- Guardian：Rust/Python 2 个 source files PASS；SQL 扩展未被脚本计入，已人工核对 SQL headers 和关系。CodeGraph sync 完成 2 个 changed source files；不是覆盖率证明。`git diff --check` 通过。
- 子代理只读精确比较新旧授权函数，唯一差异为 event 白名单；未发现本片安全/正确性阻断。另显式补充 runtime 不得调用 authorize helper 的 SQL assertion 并重跑通过。CodeRabbit CLI 再次返回 `/root/.local/bin/coderabbit: Permission denied`，没有外部 review。未执行目标数据库部署、Multipass 创建、远端 push 或 Agent execution。

## §3 已知缺口

BI 专用原生客户端、route-pin verifier、可信组合入口、Task sender dispatcher、BI owner 持久化提交后 ACK 的真实网络联测仍未实现。身份/证书/Profile 控制面、目标 DB grants、fan-out/accepted settlement、crash/recovery、消费完整性 checkpoint、BI aggregate/Benchmark 和性能验收仍开放。现有 admission client 的 typed request/result/resolution 枚举没有被放宽，不可用它冒充 Task event adapter。生产 ingress 与 execution 保持 fail closed。

## §4 子代理失败接手清单

`native_event_review` 根据已指定源码只读定位并复核两个 DB 阻断与 server 类别校验；主代理实现、实际执行验证和串行集成。无子代理写入或提交需要接管。CodeGraph 命中不充分后对指定范围直接读取，coverage MCP 不可用，不声明全仓穷尽。

## §5 守门规则

1. Project → Run → Worktree → Branch。
2. Event 独立授权。
3. 不放宽既有 Profile。
4. 证书 registry 派生身份。
5. tenant/provider/endpoint 必须匹配。
6. current Profile 与 immutable digest/revision 匹配。
7. 固定 log generation。
8. 不改原 wire 或来源证明。
9. 不跨 owner 事务。
10. durable transport acceptance 不等于业务 Inbox acceptance。
11. runtime 无表权限或裸日志 helper 权限。
12. RLS/FORCE RLS 与 WORM 保留。
13. 16 KiB wire、RPC/DB 并发与 deadline 不变。
14. test fixtures 不作生产证明。
15. PostgreSQL 停止后仅清理 runner 自有目录。
16. owner/HEAD/clean 复核后才快进 dev。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | 架构师 (Mavis 接手 agent per DEC-008) |
| SRE Lead | Mavis 接手审核：生产部署门仍开放 |
| 平台 | Mavis 接手审核：仅原生 event 基础授权 |
| 评审主持 | Mavis 接手审核：定向验证，非全量验收 |
| PM | Mavis 接手审核：后续 BI adapter 与完整 phases 未完成 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-17 | Ulysses — Mavis 接手审核 | 原生 event 类别授权、15 步 PostgreSQL 与 44 个 Rust tests | 9F4H 后继续完善真实 BI 消费链路 |
| v0.2 | 2026-10-18 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补充 native-gRPC all-targets 编译证据，修正迁移文件日期和上游文档版本引用；仍保持生产/BI 消费门禁开放 | 9F4I 独立复核和跨文档同步 |
