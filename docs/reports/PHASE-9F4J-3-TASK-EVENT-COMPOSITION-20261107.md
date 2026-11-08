# PHASE-9F4J-3 — BI Task Event Application Composition

> **状态：PARTIAL / INTEGRATION PENDING** · 修订 v0.8 · 2026-11-07
> 当前 lane：`codex/9f4j-3-native-composition-20261107` · 分支基点：`e22d92bc` · 当前计划集成目标：本地 `dev@90505991da53df9131df20d1dac8494c6f32abdb`

## §0 目的

记录 star-api-rest 中原生 BI Task Event 接收组合层的实现与实际验证边界。该切片将 transport adapter 与 transport-neutral BI owner 接收器组合，并保持 opt-in、受限写入身份、有限消费、独立 readiness 与优雅关闭。它不证明生产部署或目标数据库验收，9F4J.3 尚未关闭。

## §1 改动矩阵

| 路径 | 改动 | 状态 |
|---|---|---|
| `crates/star-api-rest/Cargo.toml`、`Cargo.lock` | composition host 增加 native gRPC adapter 依赖并同步锁文件 | 已修改并纳入登记清单 |
| `crates/star-api-rest/src/main.rs` | 在服务资源初始化前绑定 listener；接收 OS signal 后停止 accept、执行 bounded BI drain；`Serve` future 的错误会传给 main | 已实现并通过 all-targets 类型检查 |
| `crates/star-api-rest/src/bi_task_event_ingestion.rs` | 增加 opt-in runtime、可信 profile/endpoint 解析、route pin、TLS identity 检查、写入身份隔离、周期性 profile pin 复核、panic/drain 就绪保护；实现 signal/Serve future race | 已实现；补齐 Star role 与 transport EXECUTE 边界、shutdown 失败传播及有界服务退出语义 |
| `crates/star-api-rest/tests/bi_task_event_ingestion.rs` | 覆盖 readiness、role capability、shutdown failure、Serve future cancellation/server error 与 listener-bind startup order | 已编写并运行；状态见 §2 |
| 本报告、brief、dispatch JSONL | 记录范围、门禁和阶段证据 | 本报告已新增 |

首轮独立复审发现 writer identity 未与应用池比较、owner membership 检查不足、profile pin 不复核及 worker panic readiness stale。后续候选修复了应用池/BI 查询池双身份比较、owner membership/SET 限制、每 30 秒 profile pin 复核、panic readiness 保护。第二轮复审曾指出 writer 可能继承其他 `star_*` 运行角色并拥有 transport EXECUTE；source query 已要求 `star_bi_runtime` membership、拒绝其他 Star role MEMBER/SET（排除 principal 自身避免把身份误判成 membership），并检查七个精确 `platform_transport` 函数签名。spec reviewer 已确认 source self-exclusion 正确并撤回该 source P1；但独立审查发现分支外的 `crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs:573-584` 没有对每个 principal 排除其自身角色，可能产生 false positive。该 harness 不在 manifest，本 lane 未修改它，真实 PostgreSQL acceptance 仍为 NOT_RUN。

第二轮复审另确认两个关停问题：P2 在 listener bind 失败时可能绕过已启动的 worker；P2 Axum graceful shutdown 会等待活动连接，无限 SSE 可令进程退出无界等待。本轮把 listener bind 前移到 JWT/DB/Canvas/BI 初始化之前；接收 signal 后与 Axum `Serve` future 竞争，signal 分支立即丢弃 Serve future，然后调用 bounded BI shutdown。helper 保留并返回传入 `Serve` future 产生的 `Err`；但 Axum 0.8.9 会在内部记录并重试 TCP accept 错误，这些 accept 错误不会作为 `Err` 传播到 main。Axum 将 connection handlers 作为 Tokio tasks 执行，因此丢弃 Serve future 只停止 accept/结束该 await，并不单独关闭已运行的 HTTP/SSE handlers；BI drain 结束后 main 返回，Tokio runtime teardown 才会取消残留请求。增加针对 Serve future drop/error 与 bind-before-startup 顺序的测试。Spec review 确认没有 P0/P1，关停 findings 已关闭；profile drift 后仍停止拉取并要求先 drain/migrate 再滚动重启；显式权限拒绝也停止 worker，不进入无限重试。真实 PostgreSQL 角色矩阵仍未运行。

## §2 验证摘要

| 检查 | 结果 | 证据与限制 |
|---|---|---|
| `rustc --version` | PASS | `rustc 1.99.1 (58cf8f9c3 2026-10-13)` |
| 本 lane Rust 文件 `rustfmt --edition 2024 --check` | PASS | `bi_task_event_ingestion.rs`、`main.rs`、`tests/bi_task_event_ingestion.rs` 定向格式检查；未格式化其他包 |
| `cgg_guard.py --changed --stamp` | PASS | 2 个本次修改的 Rust source/header 关系戳已检查 |
| `git diff --check` | PASS | 当前候选差异无空白错误 |
| `cargo check -p star-api-rest --all-targets -j 4` | PASS | exit 0；0 errors，28 warnings；警告来自既有 crate/test fixture |
| `cargo test -p star-api-rest --test bi_task_event_ingestion -j 4` | PASS | 16 tests passed；新增 Serve future drop/error 与 bind-before-startup 语义测试，不证明 PostgreSQL、真实连接取消或部署闭环 |
| DB capability alias/field 静态核对 | PASS | SQL boolean alias、Rust field、row read 与 predicate 保持一致；覆盖 `current_user`/`session_user`、允许的 `star_bi_runtime` membership、非 BI Star role MEMBER/SET 和七个精确 transport function signatures；未运行真实角色矩阵 |
| PostgreSQL / grants / RLS / receiver E2E | NOT RUN | 无目标测试数据库与可验证的部署身份；本阶段不宣称数据库或生产接收闭环通过 |

## §3 已知缺口

1. 本地 all-targets 类型检查通过，但 workspace formatter 有大量其他包既有差异；本 lane 定向格式检查才是本 lane 的格式证据。
2. 多身份比较、owner/非 BI Star role membership 拒绝、七个 transport EXECUTE 拒绝、profile pin 漂移与 panic readiness 当前只有本地逻辑测试/静态检查；尚无真实 PostgreSQL role/membership/grant 集合验证。
3. 聚焦测试覆盖 disabled/incomplete readiness、Serve future drop/error、bind-before-startup 顺序和 worker shutdown seam；尚无真实 TCP listener 绑定冲突接入、OS signal、活动 SSE 请求被 runtime teardown 取消的端到端测试。enabled composition、profile 轮换、route mismatch、owner DB rejection、commit-before-ACK 也仍无端到端证据。
4. Profile 变化不会被本 worker 自动应用；发现 revision/digest/endpoint 变化后停止拉取并显示不可就绪，必须先排空旧 profile、执行显式迁移并滚动重启。授权拒绝同样 fail closed。
5. 未部署目标 PostgreSQL、Rust 消息服务、Vercel/Neon 或 Multipass/K3s；跨实例恢复、生产授权撤销和资源性能预算未验证。
6. 9F4J.3 只完成组合层；完整生产接收闭环仍须等待数据库、profile、TLS、route、部署与独立验证门通过。Spec review 未发现 P0/P1，关停 findings 已关闭；dev 集成与 PostgreSQL/部署门仍未完成。
7. 外部 acceptance harness `crates/bi-owner-runtime/tests/task_event_inbox_runtime_grants_pg.rs:573-584` 在扫描 `star_*` membership 时未排除 principal 自身角色，可能对当前角色产生 false positive。文件在本 lane manifest 之外，未修改；PostgreSQL acceptance 继续为 NOT_RUN。

## §4 子代理复审与失败接手

- 独立 review lane 首轮报告提交 `08e59cf3`，结论 NOT APPROVED：P1 写入身份与 app/其他 owner 身份隔离不足；P2 profile drift 对旧 client 无限重试；P2 worker panic 后 readiness 可能停留 Ready。复审同时指出本报告基线和 Cargo.lock 状态已过期。
- 源 lane 已修复：比较 writer/app/query 两种身份；拒绝任一 `_owner` membership/SET；每 30 秒复核 profile pin，漂移时阻断并要求 drain/migrate/重启；worker panic、异常退出和 drain timeout 均不可继续显示 Ready。
- 第二轮只读复审的 source P1（其他 `star_*` membership/transport EXECUTE）经 spec reviewer 确认 current-principal self-exclusion 后已撤回；source 边界仍保留。另有两个 P2：listener bind 失败绕过 worker shutdown、Axum graceful wait 可能被无限 SSE 卡住。本 lane 已前移 bind，并让 signal 立即取消 accept future 后执行 bounded BI drain。helper 保留传入 `Serve` future 的任何 `Err`；Axum 0.8.9 会在内部记录并重试 TCP accept 错误，这些错误不会作为 `Err` 传给 main。Axum 已 spawn 的连接 handlers 直到 main 返回后的 Tokio runtime teardown 才取消。Spec review 确认没有 P0/P1 且关停 findings 已关闭；dev 集成仍待协调者基于最新 baseline rebase/复验，范围外 harness 与受保护的 W8/Dependabot 来源保持不动。
- 外部 PostgreSQL acceptance harness 自身的 principal self-role false-positive 风险位于未登记文件，已记录为已知限制但未触碰；目标 PostgreSQL/roles/grants/RLS/transport 仍 NOT_RUN。
- 子代理未执行 PostgreSQL、native transport 或部署验收；W8/Dependabot 分支、工作树及 `D:\Star` 用户文件保持未触碰。

## §5 守门规则

1. BI owner crate 保持 transport-neutral，传输适配仅在 star-api-rest 组合层接入。
2. 仅使用版本化 native Event adapter，不重复实现接收协议。
3. BI 查询池与写入池分离；写入池 `current_user` 与 `session_user` 必须分别不同于应用池和 BI 查询池的两种身份。
4. BI writer 的 `session_user` 与 `current_user` 均须满足最小权限边界。
5. 拒绝 superuser、BYPASSRLS、CREATEROLE、CREATEDB、REPLICATION、任一 `_owner` role membership/SET ROLE、非 BI `star_*` role membership/SET ROLE 和 schema CREATE 能力；`current_user` 与 `session_user` 必须属于 `star_bi_runtime`。
6. 受保护表拒绝 SELECT/INSERT/UPDATE/DELETE/TRUNCATE 直接授权；写入与认证身份均不得拥有七个精确 `platform_transport` function signatures 的有效 EXECUTE。
7. 只授予所需的 commit/quarantine 安全函数执行权；能力无法核验即 fail closed。
8. 仅在显式 opt-in 且配置完整后启动 consumer。
9. 固定并核对当前 profile digest、endpoint、service identity 与 destination generation。
10. TLS CA、client certificate、private key 必须是受限大小的有效文件；缺失或错误时不启动。
11. 同时只允许一个有界 in-flight delivery；deadline、重试和空闲等待有上限。
12. 只有 owner 事务提交后才允许消息确认；不得将 transport 写入成功当作持久化确认。
13. 未就绪、启动失败、授权拒绝或 profile 漂移均需在 readiness 明确暴露并保持不可执行。
14. 启动时固定 profile revision/digest/endpoint；每 30 秒复核，profile 不可用时暂停拉取，漂移时停止 worker。
15. 不原地替换 transport/profile；变化必须显式 drain/迁移后滚动重启，禁止跨 profile 续用旧 lease。
16. worker panic 或意外退出必须覆盖陈旧 Ready 状态并返回 blocked。
17. Unix 同时响应 SIGINT 与 SIGTERM；Windows 响应 Ctrl+C。信号 handler 无法安装时触发有界关闭。
18. 关闭时停止拉取并 drain；超过期限或 JoinError 后 readiness 保持失败，且 shutdown 错误必须传播到进程结果，不伪报成功。
19. fixture、静态检查、纯逻辑测试或编译通过均不等于目标 PostgreSQL/RLS/transport 或部署验收。
20. 只修改 MAGOS 登记路径；保留 D:\Star 用户数据、main、远端及所有 W8/Dependabot 工作树和分支。
21. HTTP listener 必须在服务资源/worker 初始化前成功 bind；收到 OS signal 后停止 accept 并丢弃 Axum Serve future，不等待无限 SSE 连接；然后执行 bounded BI drain。Axum spawned HTTP/SSE handler 在 main 返回后的 Tokio runtime teardown 才会被取消。`serve_until_shutdown` 必须传播它收到的 Serve future `Err`；但 Axum 0.8.9 内部会记录并重试 TCP accept 错误，不会把这些 accept 错误返回给 main。

## §6 签字栏

| 角色 | 审核责任 | 状态 |
|---|---|---|
| 架构 | Mavis 接手审核组合层边界与 fail-closed 设计 | Spec review 未发现 P0/P1；关停 findings 已关闭 |
| SRE Lead | Mavis 接手审核 readiness、背压、drain 与运行限制 | panic/drain/profile 状态已补；目标环境未验证 |
| 平台 | Mavis 接手审核消息 adapter 与 owner 隔离 | writer/app/query 隔离已加；真实 PG role 检查未运行 |
| 评审主持 | Mavis 接手审核变更范围和证据 | Spec review 无 P0/P1；协调者尚未完成基于最新 dev baseline 的集成复验 |
| PM | Mavis 接手审核阶段声明与 WBS 状态 | 16 项 focused tests passed；dev 集成与 DB/部署验收仍待完成，DB/部署保持 NOT_RUN |

修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。没有人工审批或部署验收的声明。

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次记录组合层范围、权限复审修正、已运行静态检查和 Cargo.lock/数据库验证门禁 | 9F4J.3 lane 实施与独立复审 |
| v0.2 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 增加 SIGTERM drain；补拒绝 CREATEDB/REPLICATION；同步 17 项守门规则与未运行验证状态 | Linux/K3s 生命周期与权限边界独立复审 |
| v0.3 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录最新静态独立复审通过及信号路径缺少专门测试的限制；保持构建、测试和集成门关闭 | 最终源码差异复审 |
| v0.4 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Rust 1.99.1、rustfmt check 与 19 项 capability 字段映射核对；Cargo 构建/测试仍未运行 | 锁文件门禁与静态验证复核 |
| v0.5 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修正分支基点与 Cargo.lock 状态；补齐 app/query 双身份隔离、任一 `_owner` 角色约束、profile pin 复核与 panic/drain readiness；记录实际构建与测试状态 | 独立 review 的 P1/P2 发现和本轮聚焦验证 |
| v0.6 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 修复非 BI `star_*` MEMBER/SET 与七个 transport EXECUTE 权限检查；shutdown JoinError/timeout 传播到进程错误结果并增加 focused tests；同步 13 项测试、all-targets、rustfmt、CGG、diff-check 及 PostgreSQL NOT_RUN 状态；当时的 dev 目标快照为 `eccdd6f5` | 第二轮独立复审 P1/P2 与源 lane 聚焦验证 |
| v0.7 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | listener bind 前移到服务资源启动之前；signal 到达后丢弃 Axum Serve future，再执行 bounded BI drain；记录 spawned HTTP/SSE handlers 由 Tokio runtime teardown 取消；增加 bind-order、Serve drop/error 测试；记录外部 PG harness self-role false-positive 且未触碰 | 第二轮复审的新 P2 发现、spec reviewer 对 source self-exclusion 的确认与本轮 16 项 focused tests |
| v0.8 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 更新当前计划 dev baseline 为 `90505991da53df9131df20d1dac8494c6f32abdb`，保留 `eccdd6f5` 为历史快照；精确说明 helper 保留 Serve future `Err`，Axum 0.8.9 TCP accept 错误由内部记录/重试、不传播给 main；记录 spec review 无 P0/P1、关停 findings 已关闭，PostgreSQL acceptance 仍 NOT_RUN | 协调者提供的新 dev baseline 与 Axum 0.8.9 错误语义复核 |
