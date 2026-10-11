# PHASE-9F4C-D14-ADMISSION-RECEIVER-REPORT

> v0.1 · 2026-10-11 · Ulysses — Mavis 接手审核
> 状态：实现及验证进行中；完整 D14/原生替换/任务执行闭环未关闭。

## §0 目的

接续 D13，将 Schedule request → Run preparation result → Schedule resolution → Run demand 的接收路径接入 broker-neutral port。保持 owner-local 事务、current authorization、幂等回放、资源预算和不可执行状态，支持以后替换为自有 Rust server。

## §1 改动矩阵

| 范围 | 实际变更 |
|---|---|
| Shared receive policy | `admission_receive.rs` 提供 transport/delivery、authorization、owner handler；单条串行、16KiB typed wire、10秒 end-to-end deadline、取消/控制通道关闭、提交后复验授权再 ACK |
| JetStream adapter | `admission_receive_jetstream.rs` 固定精确路由、stream/domain/consumer/message ID；每次刷新 durable explicit-ACK 配置；一条 in-flight、有限 bytes/waiting/retries；保存原 wire，无 payload clone |
| Run owner | request 接 begin/prepare_and_commit；resolution 接 confirm/compensate；只在已提交后返回来信 ID |
| Schedule owner | result 接 resolve_admission_result，已提交后返回来信 ID |
| Dependencies | 沿用 async-nats 0.50；显式 server_2_10 feature 检查多过滤器；沿用 workspace futures-util；Star PG 新增 platform path dependency。未新增第三方版本，provider dependency isolation 仍开放 |
| Integration | 已有 positive handshake 加入真实 receiver；结果事务故障检查 broker pending=1，成功检查 ACK floor/cursor 与 pending=0；fixture 授权与生产身份明确区分 |
| Runner | `scripts/automation/phase9f4d_run_owner_consumer.py` 新增 `--admission-receiver-tests` 别名，沿用受控隔离服务与进程树回收 |
| Docs | 需求/基本/详细设计、Schedule SRS/BD/DD 与阶段计划同步；沿用自有模块复用选择，无跨仓源码搬迁 |

新增 DB 表 W/T/M=0/0/0；业务事实、audit 与 Outbox/Inbox 使用既有 owner schema。

## §2 验证摘要

最终 runner `target/phase9f4d-run-owner-consumer/a77d490f-04b0-4290-82e8-7aacbb2c7c05/audit.jsonl`：66/66步骤通过；真实 PostgreSQL 测试 `schedule-admission-postgres-sqlx-integration` 1/1 通过，隔离 NATS 与 PostgreSQL 均干净停止，Multipass inventory 未修改。此测试验证 broker 上新 request → Run owner事务 → accepted result → Schedule owner事务 → confirm resolution → Run demand；一个实际插入 resolution Outbox 前数据库故障使消息保持未 ACK（num_ack_pending=1），成功事务提交后 double-ACK 归还 pending=0 且 ACK floor 与发布 cursor 匹配。租户 RLS 与既有owner接收结果同时覆盖。此项为显式测试身份 fixture，不是生产 service identity 证据，也没有验证 executor 启动。

三个 Rust 包全目标 check 成功：`cargo check --offline --locked -p platform-messaging-runtime -p run-owner-runtime -p star-pg-adapter --all-targets -j 4`。platform-messaging-runtime 15/15、run-owner-runtime 15/15、star-pg-adapter 35/35 lib unit tests 通过。`cargo clippy --offline --locked -p platform-messaging-runtime -p run-owner-runtime --all-targets --no-deps -j 4 -- -D warnings` 成功（domain-kms 仍有3个既有 deprecation warnings）。包括 Star PG 在内的 strict Clippy 全包运行失败：Star PG 仓库有约150个既有 deny-warnings lint，定位于未修改的 OAuth/ops 等模块；没有全包 strict-Clippy 通过声明。首轮链接曾出现 LNK1104，未删除缓存/降低系统安全策略；后续测试完成，具体链接器根因未确定。图头检查 `cgg_guard.py --changed --stamp` 通过9个源文件；当前局部 Git diff check 通过。CodeRabbit wrapper 因 `/root/.local/bin/coderabbit: Permission denied` 不可用，未获独立 review。未运行 workspace/release/doc/bench 全仓验收。

## §3 已知缺口

生产 service identity/verifier、recipient provisioning 与 main composition 未提供，本实现没有默认允许授权。malformed/poison 保持不 ACK，durable quarantine/终结与真正接收 ACK 丢失恢复/重启仍需实现验收；max_deliver 耗尽不是已审计终结。当前 fixture stream 在每项断言后删除，不证明长期 replay/recovery。Compensate handler 有代码，但本轮 live receiver positive case只证明 Confirm，既有 compensation 事务测试不替代该 receiver 路径证明。公平多租户调度/吞吐/内存性能、native provider、Cargo provider 隔离与完整分发 SBOM 仍开放。Host/cluster reservations、Multipass provider、native Hook、Runtime sandbox、CLI 独立验证与 BI/Benchmark 完整闭环仍开放。单 receiver 的可变借用串行不证明全部署最多一个 worker；部署侧仍须限制实例数与租户预算。授权检查与异步事务存在时间窗口，生产 verifier 的版本/fence 与 owner 事务校验须一并接入，不宣称强一致撤权已实现。

## §4 子代理失败接手清单

本阶段未派发新子代理。Root 在已存在的隔离 lane 实施；源项目只读，用户根工作区及 dev CodeGraph 保留。

## §5 守门规则

1. Project/Branch/Run/Worktree canonical 边界保持。
2. 每个 owner 使用自己的业务事务。
3. 禁止跨 owner 直接写表。
4. 原始 wire 与 typed identity 一致。
5. verifier 必须显式提供，没有默认允许实现。
6. 注入 UUID 不代表认证成功。
7. 事务前和 ACK 前检查授权。
8. 已提交且来信 ID 匹配才确认。
9. DB 错误/未知 commit 不确认。
10. 控制通道关闭按取消处理。
11. 整个操作共享十秒 deadline。
12. 单 receiver 一条 in-flight；配置每次刷新。
13. 16KiB wire 与有界 transport metadata。
14. poison 无持久化 quarantine 时不终结。
15. fixture/VM/transport readiness 不开启 executor。
16. 聚焦测试不冒充 workspace/生产验收。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis：owner 与 transport 边界按实际代码审核；phase 未关闭 |
| SRE Lead | Mavis：deadline/取消与实际失败证据保留；恢复/性能未验收 |
| 平台 | Mavis：显式 verifier 与 fail closed；生产身份缺口保留 |
| 评审主持 | Mavis：聚焦验证进行中；独立评审状态待记录 |
| PM | Mavis：全部产品 phases 目标继续，未缩小为本切片 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 / 触发 |
|---|---|---|---|
| v0.1 | 2026-10-11 | Ulysses — Mavis 接手审核 | D14 receiver/owner/adapter 实施与诚实缺口；继续完整 phase 目标 |
