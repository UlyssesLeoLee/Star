# PHASE-9F4J-3 — 独立代码复审

> **状态：REVIEW BLOCKED / NOT APPROVED FOR INTEGRATION** · 修订 v0.1 · 2026-11-07
> 复审分支：`codex/9f4j-3-independent-review-20261107` · 目标基线：当前本地 `dev@8c2d78cbe77aeb81ed28d2396c319a1bbec24ed9`
> 被审来源：`codex/9f4j-3-native-composition-20261107`，HEAD `e22d92bcbefaf009e18d043ab2c8a762c150fc01`，有未提交源码/锁文件改动。

## §0 目的

独立审查 9F4J.3 Task Event 应用组合层，检查 BI/Platform owner 边界、profile pin/授权复验、Inbox commit-before-ACK、重复与冲突、启动与关闭、资源界限、readiness 及来源阶段报告/WBS 一致性。本 lane 只写本报告和 `docs/reviews/9F4J-3-INDEPENDENT-CODE-REVIEW-20261107.md`；来源实现 worktree 全程只读。

复审结果为**不批准集成**：有一项 P1 写身份隔离缺口、两项 P2 运行恢复/健康探针缺口，以及来源报告与实际快照不一致。该结论是静态审查，不是构建或部署结论。

## §1 复审矩阵

| 范围 | 静态结论 | 证据/限制 |
|---|---|---|
| Owner 与依赖边界 | 部分符合 | adapter 由 `star-api-rest` 组合；`bi-owner-runtime` 不依赖 Platform transport。BI write identity 未与 app owner identity/其他 owner role 隔离证明，P1。 |
| Profile、Event 与 route pin | 有限度符合 | startup 固定 provider/version/profile digest/endpoint/identity/destination；Native server 对 verify/ACK 复验 active Profile。Profile 漂移后无 reload/restart contract，P2。 |
| Inbox exact replay 与 ACK 顺序 | 源码顺序符合 | `task_event_receiver.rs:447-505` 表现出 commit 后复验再 ACK；编译、SQL 与真实 transport 行为未验证。 |
| Duplicate/conflict/quarantine | 源码有路径 | exact replay 接收 receipt；identity conflict 转 metadata quarantine；目标 PostgreSQL、并发与 redelivery 证据未运行。 |
| Startup/readiness/shutdown | 部分符合 | opt-in、配置 fail closed、有限 receive 和 drain 存在；worker panic 可使 readiness 保留旧 Ready，P2。 |
| 资源边界 | 静态有界 | 单 consumer/单 permit、有限 payload、有限请求 deadline、有限 shutdown；RSS/延迟/公平性未测。 |
| 文档/基线 | 不一致 | 来源 phase report 的基线、Cargo.lock 登记状态、先前“无发现”结论均已过期，详见代码复审报告。 |

## §2 验证摘要

| 验证 | 结果 | 证据与限制 |
|---|---|---|
| 指定源代码、WBS、DD 与 phase report 静态读取 | 已完成 | 仅读 `D:\orcaWork\Star\9f4j-3-native-composition-20261107`；未修改来源文件。 |
| CodeGraph | 不可用 | 当前工具没有匹配来源 worktree 的索引/coverage；按协调者指示直接读取目标文件，不索引或修改 `.codegraph`。 |
| `cargo check` / `cargo test` / Clippy | **NOT RUN** | 子任务禁止 Cargo；不能据此声称编译或测试通过。 |
| PostgreSQL、迁移、grants、RLS | **NOT RUN** | 没有执行目标数据库或 disposable DB 探针。 |
| gRPC/mTLS 投递、commit-before-ACK 网络 latch、恢复 | **NOT RUN** | 没有 transport/deployment 环境运行验收。 |
| deployment、生产 runtime、RSS/延迟/公平性 | **NOT RUN** | 无部署或性能测量。 |
| 来源报告中 rustfmt/静态别名对照/Cypher/diff-check | 未独立确认 | 这些 PASS 只记为来源 phase report 的自述，不是本复审执行结果。 |

## §3 已知缺口

1. **P1：BI 写身份未证明与 Project/Run app identity 及其他 owner 权限隔离。** 来源 `bi_task_event_ingestion.rs:813-890` 只将 BI writer 与 BI query role 比较；`is_restricted` 的边界为 `:209-229`。修复应核对 app/query/write 的实际数据库及登录/当前角色，并拒绝共享 owner 身份、额外 owner membership 与危险 `SET ROLE` 路径；补角色矩阵和目标 grants/RLS 验收。
2. **P2：Profile drift 后旧 client 无法重建。** 配置解析和 client 构造各执行一次（`:711-760`）；运行循环错误后一直用原 receiver 每秒重试（`:895-928`）。需定义受控 drain/re-resolve/swap 或明确终态 Blocked + 运维重启/迁移流程，并加入 Profile 轮换测试。
3. **P2：worker panic 后 readiness 可能假阳性。** task 的 JoinHandle 未被 supervisor 监视（`:312-323`、`:364-388`），readiness 按最后快照返回（`:943-959`）。需把意外退出变为 Blocked/503 或终止进程，覆盖 panic/提前退出测试。
4. 来源报告在 `PHASE-9F4J-3-TASK-EVENT-COMPOSITION-20261107.md:4,14,31,36,45` 的基线、Cargo.lock 和“最新复审无发现”叙述已经过期；源 lane 尚需对当前 `dev` 复核/rebase，并同步报告/WBS。审查结论不能代替实现修复。
5. Inbox/ACK、quarantine 与角色边界仍缺 Cargo 与真实 PostgreSQL、mTLS transport、故障恢复、运行资源及部署证据；在这些门关闭前保持 ingestion/下游 accepted settlement 不可宣称生产可用。

## §4 子代理失败接手清单

- 本复审没有派生子代理；没有子代理代码或验证产物待接手。
- Source lane 由协调者指定只读，且禁止运行 Cargo；未尝试修改或清理来源 worktree。
- 本报告提交后，协调者需将 P1/P2 逐项交回实现 owner，并以新的源代码快照重新审查；不要把此静态报告当作修复或验收记录。

## §5 守门规则

1. BI owner crate 保持与具体 transport crate 解耦；transport wiring 仅由应用组合层持有。
2. Event 使用独立 message kind/scope，不复用 Admission Request/Result/Resolution。
3. 明确区分 Project/Run app owner、BI query、BI writer 与 Platform transport owner 身份。
4. BI writer 必须为显式最小权限身份，并核对 session/current role、数据库身份、owner membership 与 `SET ROLE` 路径。
5. 任何授权事实、Profile revision、endpoint 或 destination generation 不可验证时都 fail closed。
6. 每条消息的来源、租约、message ID 与 exact-wire digest 必须经过可信 adapter 校验。
7. Inbox 事务提交或 exact replay receipt 返回前禁止 ACK；未知 DB/ACK 结果不伪报成功。
8. identity/content conflict 仅可在 metadata-only quarantine receipt 持久提交且来源复验后 terminal disposition。
9. 单次 pull/in-flight、有界 payload、请求 deadline、队列和重试应受限；禁止无界 prefetch。
10. Profile 轮换须定义旧 route drain、待处理 lease、profile swap 和恢复；禁止跨 provider 比较 cursor/fallback。
11. readiness 必须反映 worker 实际存活、授权接收与恢复状态；panic/意外退出不得保留 Ready。
12. shutdown 停止新 pull 并有限 drain；超时必须标为失败/未确认，不丢弃持久化事实。
13. 服务启动 opt-in 默认关闭；配置错误、身份不一致、连接失败或权限不可见时不可启动消费。
14. 不把静态源码顺序、fixture、编译通过或内存状态冒充 PostgreSQL/mTLS/生产证明。
15. 本复审范围仅含两个 review 输出；source、Cargo.lock、W8/Dependabot、D:\Star、main 与 remote 不在写入范围。
16. 集成前更新来源报告/WBS 的当前基线、manifest 与验证状态；按当前 dev 重审所有差异。
17. 未取得 Cargo、PostgreSQL grants/RLS、网络 commit-before-ACK、失败恢复和部署证据前，不关闭 9F4J exit gate、不声称生产 ready。

## §6 签字栏

| 角色 | 审核责任 | 状态 |
|---|---|---|
| 架构 | Mavis 接手审核 owner 边界、transport-neutral composition 与 Profile pin | 不通过：P1 身份隔离缺口待修 |
| SRE Lead | Mavis 接手审核 worker、readiness、drain 与恢复 | 部分：Profile drift 与 panic readiness 待处理；runtime 未测 |
| 平台 | Mavis 接手审核 mTLS native Event adapter 与授权契约 | 静态顺序可接受；网络、PG/RLS 与 grants 未运行 |
| 评审主持 | Mavis 接手审核范围、证据和差异 | 静态复审完成；未批准集成 |
| PM | Mavis 接手审核阶段状态/WBS | 9F4J.3 保持 OPEN/PARTIAL，exit gate 未关闭 |

修订人：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。没有人工部署或生产验收声明。

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次记录 9F4J.3 静态独立复审：一个 P1 owner identity 隔离缺口、两个 P2 运维/健康缺口、来源报告快照不一致，以及所有未运行验收门 | 9F4J.3 独立 review lane |
