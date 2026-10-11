# PHASE-9F4C-D14-ADMISSION-RECEIVER-REPORT

> v0.4 · 2026-10-12 · Ulysses — Mavis 接手审核
> 状态：broker-neutral quarantine port 与失败路径 fixture 已写；Rust 1.98.1 诊断检查下聚焦测试和包级 all-targets 检查通过，但尚未在要求的 Rust 1.99.0 工具链验收。持久化 quarantine、原生替换和任务执行闭环未关闭。

## §0 目的

接续 D13，将 Schedule request → Run preparation result → Schedule resolution → Run demand 的接收路径接入 broker-neutral port。保持 owner-local 事务、current authorization、幂等回放、资源预算和不可执行状态，支持以后替换为自有 Rust server。

## §1 改动矩阵

| 范围 | 实际变更 |
|---|---|
| Shared receive policy | `admission_receive.rs` 提供 transport/delivery、authorization、owner handler；单条串行、16KiB typed wire、10秒 end-to-end deadline、取消/控制通道关闭、提交后复验授权再 ACK |
| JetStream adapter | `admission_receive_jetstream.rs` 固定精确路由、stream/domain/consumer/message ID；每次刷新 durable explicit-ACK 配置；一条 in-flight、有限 bytes/waiting/retries；保存原 wire，无 payload clone |
| Poison disposition seam | 新增 provider-neutral `AdmissionReceiveQuarantine`、source-only verifier 与 terminal disposition；来源身份含可信配置的 `log_generation`，按 provider/generation/log/sequence 幂等；quarantine 写入后复验 source，失败/撤权/确认失败保持未确认；增加 5 个失败路径 fixture test，尚未接入 durable provider |
| Run owner | request 接 begin/prepare_and_commit；resolution 接 confirm/compensate；只在已提交后返回来信 ID |
| Schedule owner | result 接 resolve_admission_result，已提交后返回来信 ID |
| Dependencies | 沿用 async-nats 0.50；显式 server_2_10 feature 检查多过滤器；沿用 workspace futures-util；Star PG 新增 platform path dependency。未新增第三方版本，provider dependency isolation 仍开放 |
| Integration | 已有 positive handshake 加入真实 receiver；结果事务故障检查 broker pending=1，成功检查 ACK floor/cursor 与 pending=0；fixture 授权与生产身份明确区分 |
| Runner | `scripts/automation/phase9f4d_run_owner_consumer.py` 新增 `--admission-receiver-tests` 别名，沿用受控隔离服务与进程树回收 |
| Docs | 需求/基本/详细设计、Schedule SRS/BD/DD 与阶段计划同步；沿用自有模块复用选择，无跨仓源码搬迁 |

新增 DB 表 W/T/M=0/0/0；业务事实、audit 与 Outbox/Inbox 使用既有 owner schema。

## §2 验证摘要

之前已完成的基线 runner `target/phase9f4d-run-owner-consumer/a77d490f-04b0-4290-82e8-7aacbb2c7c05/audit.jsonl` 为 66/66 步骤通过；真实 PostgreSQL 测试 `schedule-admission-postgres-sqlx-integration` 1/1 通过，隔离 NATS 与 PostgreSQL 均干净停止，Multipass inventory 未修改。该证据验证 broker request → Run owner事务 → accepted result → Schedule owner事务 → confirm resolution → Run demand；数据库故障保持未 ACK，成功提交后 double-ACK。身份是显式测试 fixture，不是生产 service identity，也没有验证 executor 启动。

本轮新增的 5 个 Rust fixture 测试覆盖 quarantine 后才 terminal disposition、quarantine/source verification 失败不终结、source 撤权不终结及终止确认失败保留 quarantine；来源代次字段新增 nil-generation 拒绝断言。主机找不到 RTK wrapper；已安装 Rust 1.98.1，而 workspace crates 要求 Rust 1.99.0。未加覆盖参数的命令被 Cargo 版本门拒绝；随后用 `--ignore-rust-version` 作诊断检查：`cargo test --offline --locked --ignore-rust-version -p platform-messaging-runtime --lib -j 4` = 20/20 pass；默认 feature `cargo check --offline --locked --ignore-rust-version -p platform-messaging-runtime --all-targets -j 4` pass；`--no-default-features --all-targets` pass；`cargo check --offline --locked --ignore-rust-version -p run-owner-runtime --test schedule_admission_pg --all-targets -j 4` pass；`cargo fmt -p platform-messaging-runtime -- --check` pass。扩大到 run-owner-runtime 的 formatter check 报告未修改 `src/lib.rs` 中模块声明顺序的既有格式差异；未格式化该文件。`cgg_guard.py --changed --stamp` = 4 个源码文件 PASS。由于 RTK wrapper 不可用，上述 Cargo 命令使用已安装可执行文件的绝对路径直接运行；这些结果不能替代 Rust 1.99.0 验收。未运行实际 NATS/PostgreSQL integration target、Clippy、workspace/release/doc/bench。独立 CodeRabbit 命令因 `/bin/bash: /root/.local/bin/coderabbit: Permission denied` 退出码 1，未获独立审查通过。

## §3 已知缺口

生产 service identity/verifier、recipient provisioning 与 main composition 未提供。新增 `AdmissionReceiveQuarantine` 只是 port，未接到现有 JetStream `PlatformPoisonQuarantine` 或其他 durable writer；没有持久化实现就不能 quarantine/终结。`log_generation` 由 adapter 调用方固定，生产 verifier 仍须绑定已注册的实际 stream generation；不能用随意配置 UUID 代替授权来源。新增 5 个 fixture 测试已在 Rust 1.98.1 诊断模式运行通过，Run owner integration test target 编译通过，但实际 NATS/PostgreSQL integration 未运行；尚未以要求的 Rust 1.99.0 工具链验收。真正 ACK 丢失恢复/重启仍需实现验收；max_deliver 耗尽不是已审计终结。既有 fixture stream 在每项断言后删除，不证明长期 replay/recovery。Compensate handler 有代码，但 live receiver positive case只证明 Confirm。公平多租户调度/吞吐/内存性能、native provider、Cargo provider 隔离与完整分发 SBOM 仍开放。Host/cluster reservations、Multipass provider、native Hook、Runtime sandbox、CLI 独立验证与 BI/Benchmark 完整闭环仍开放。单 receiver 的串行不证明部署最多一个 worker；部署侧仍须限制实例数与租户预算。授权检查与异步事务存在时间窗口，生产 verifier 版本/fence 与 owner 事务校验须一并接入，不宣称强一致撤权已实现。

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
14. poison 仅在 source 身份前后复验、durable quarantine 幂等提交及 terminal receipt 成功后终结；缺 provider/任一失败均不确认。
15. fixture/VM/transport readiness 不开启 executor。
16. 聚焦测试不冒充 workspace/生产验收。

## §6 签字栏

| 角色 | 审核 |
|---|---|
| 架构 | Mavis：owner 与 transport 边界按实际代码审核；诊断验证受工具链版本限制，phase 未关闭 |
| SRE Lead | Mavis：deadline/取消与失败确认边界已写；durable recovery/性能未验收 |
| 平台 | Mavis：source verifier/quarantine 缺失默认拒绝；生产身份和写入 provider 缺口保留 |
| 评审主持 | Mavis：静态复核与聚焦验证完成；扩大范围门禁未运行，CodeRabbit 调用因权限拒绝未形成独立审查 |
| PM | Mavis：全部产品 phases 目标继续，未缩小为本切片 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 / 触发 |
|---|---|---|---|
| v0.1 | 2026-10-11 | Ulysses — Mavis 接手审核 | D14 receiver/owner/adapter 实施与诚实缺口；继续完整 phase 目标 |
| v0.2 | 2026-10-12 | Ulysses — Mavis 接手审核 | Add provider-neutral poison quarantine port and fail-closed source/terminal ordering; mark tests and durable provider wiring incomplete |
| v0.3 | 2026-10-12 | Ulysses — Mavis 接手审核 | 对齐本轮 20/20 聚焦测试及 all-targets 诊断检查状态，清除“测试未运行”的过期标记，并保留 Rust 1.99.0 与持久化 provider 未验收缺口 |
| v0.4 | 2026-10-12 | Ulysses — Mavis 接手审核 | Fix same-name log recreation ambiguity with trusted log generation in the neutral source identity and quarantine key; verify package tests and the Run owner test target compilation |
