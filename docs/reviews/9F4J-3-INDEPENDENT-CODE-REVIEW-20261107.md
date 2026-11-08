# 9F4J.3 独立代码复审

> 日期：2026-11-07
> 来源：`D:\orcaWork\Star\9f4j-3-native-composition-20261107`，分支 `codex/9f4j-3-native-composition-20261107`，审阅时 HEAD `e22d92bcbefaf009e18d043ab2c8a762c150fc01`。
> 范围：仅静态只读审阅新组合模块、接线、测试和相关 owner/transport 契约；未改来源 worktree，未运行 Cargo。

## 结论

**当前不建议集成。**发现一个需先修复的 P1 权限隔离缺口，以及两个 P2 运行恢复/健康状态缺口。`TaskEventReceiver` 的静态调用顺序体现了 Inbox 持久化后再做第二次来源校验并 ACK；这只是代码路径审阅，不构成编译、数据库或网络验收。

本复审没有发现 `bi-owner-runtime` 直接依赖 `platform-messaging-runtime`：新增 adapter 和连接生命周期均放在 `star-api-rest` 组合层。该边界检查来自工作区当前源码，CodeGraph 没有覆盖此 worktree；按协调者指示改为直接读取明确的源文件，未索引或改动 `.codegraph`。

## Findings

### [P1] BI 写入连接未证明与 Project/Run 应用身份隔离

**证据**：`crates/star-api-rest/src/bi_task_event_ingestion.rs:813-890` 只从 `query_pool` 读取 `query_role`，再读取 `write_pool` 的 `write_role`，最后仅拒绝空角色、与 BI query 同角色或不满足 `WriteDatabaseCapabilities::is_restricted()` 的身份。该谓词位于 `:209-229`，只检查 BI runtime/BI owner membership、指定 BI 表权限、若干角色属性和两项函数调用能力；`app_pool` 不参与身份比较，也未检查其他 owner-role membership/SET ROLE 或目标数据库身份。

**影响**：如果部署把 `BI_TASK_EVENT_INGESTION_DATABASE_URL` 配成 Project/Run 应用登录，且该身份同时拥有 `star_bi_runtime` membership 与 Project/Run owner 权限，当前守门可能只因它不是 BI query 角色、没有 BI 表直连权限而放行。此时专用写池仍携带其他 owner 的能力，违背 WBS 9F4J.3 的“隔离 BI write pool”和多 owner 隔离要求；在同一数据库拥有其他 schema/function 权限时会扩大事件进程的可写范围。当前没有目标数据库角色事实，不能证明生产配置已发生此误配；问题是实现没有 fail closed 地排除此配置。

**修复/验收**：从 `app_pool` 与 BI query pool 读取并比较 `session_user/current_user` 和数据库身份；要求 BI writer 是显式受限 BI runtime 身份，拒绝与 app owner 共用的登录/当前角色，并检查可能授予其他 owner 权限的 membership 与 `SET ROLE` 路径。添加可运行的角色矩阵测试：BI-only 身份通过；复用 app 身份、可 `SET ROLE` 到 Project/Run owner、或连接到错误数据库时配置保持 Blocked/503。目标 PG grants/RLS 仍需另行验收。

### [P2] Active Profile 漂移后 consumer 只会用旧快照循环重试

**证据**：组合只在 `bi_task_event_ingestion.rs:711-724` 调用 `TrustedProfileEndpointResolver::resolve`，并在 `:745-760` 创建一次 profile-pinned native client；route pin 随后固定在 `:762-775`。后台循环 `:895-928` 收到任一错误后只将 readiness 设为 Blocked、等待一秒，再对同一 receiver 重试，不重新 resolve Profile 或重建 client。native client 契约也明确指出 Profile/route 在实例生命周期内固定，调用方必须重新解析后替换 client（`crates/platform-messaging-runtime/src/native_grpc.rs:1043-1048,1199-1206`）。

**影响**：服务器侧对每次 origin verify/ACK 复验当前 Profile，因而旧 Profile 失效时此路径会 fail closed；但配置中的 Profile revision/digest/endpoint 一旦轮换，旧 client 会持续被拒绝，readiness 永久 Blocked，直到进程外部重启。当前没有轮换后的 drain/re-resolve/替换协议或验收，不能把它描述为可自愈的 consumer。

**修复/验收**：明确 profile 轮换策略。若支持进程内更新，必须先停止旧 route 的新 pull、等待/释放旧 lease，按版本化规则 drain 旧 provider，再重新解析并原子替换 transport/auth/route pin；保持旧消息 redelivery 与 provider 不可比较的 cursor 规则。若只支持重启，明确将 Profile drift 作为终态 Blocked/告警并规定安全重启/旧 profile drain。测试 Profile revision、endpoint、log generation 变更；旧 pin 不得 ACK，新配置恢复后才能继续，且不得跨 provider 回退。

### [P2] Consumer panic 后 readiness 可能仍报告 Ready

**证据**：worker 在 `bi_task_event_ingestion.rs:312-323` 通过 `tokio::spawn` 创建，`JoinHandle` 仅保存供关闭时等待（`:364-388`）；没有监视任务退出/`JoinError` 的 supervisor。readiness 只由 `run_consumer` 的正常结果和循环退出路径更新（`:895-940`）；若任务在已设为 Ready 后 panic，`readiness_endpoint` 仅读取最后快照并在 `Ready` 时返回 HTTP 200（`:943-959`）。

**影响**：worker 不再消费，但探针可以继续返回 200，调度器不会重启或隔离该实例，导致事件积压而健康状态失真。现有集成测试只构造 disabled/incomplete 状态（`crates/star-api-rest/tests/bi_task_event_ingestion.rs:30-75`），没有异常退出用例。

**修复/验收**：监督 `JoinHandle`，将 panic/意外退出原子转换为 Blocked，并按明确策略有限重启或退出进程；readiness 仅在 worker 存活且最近一次授权接收检查有效时返回 200。注入 panic/提前退出并断言 503 或进程退出，不允许保留过期 Ready 状态。

## 文档和基线不一致

来源阶段报告 `docs/reports/PHASE-9F4J-3-TASK-EVENT-COMPOSITION-20261107.md` 当前写着基线 `dev@e2393618...`（第 4 行），而来源工作树 HEAD 是 `e22d92bc...`，本复审工作树当前 `dev` 是 `8c2d78cb...`；来源分支与当前 `dev` 的 merge-base 为 `e22d92bc...`，因此该 lane 还需以当前 dev 复核/rebase。来源报告还在第 14、31、36 行称 Cargo.lock 未同步/不在 manifest，然而本次只读状态显示源工作树的 `Cargo.lock` 已修改，协调者已确认该文件已进入独立 MAGOS manifest。第 45 行声称“最新静态独立复审…无发现”，本报告取代该结论。

**建议**：在来源 lane 同步 WBS/阶段报告，更新基线和 Cargo.lock 登记状态；保留 Cargo 检查、测试、PG/grants/RLS 与网络验收为 NOT RUN，直到实际运行并记录证据。本复审未修改这些来源文档。

## 静态核对的正向证据

- `main.rs` 将 BI query port 与 pool 分开传递，并将 ingestion readiness 和关闭生命周期放在应用组合层；Cargo 依赖由 `star-api-rest` 引入 native adapter。
- `TaskEventReceiver::process_one` 的源码顺序为来源校验 → Inbox exact-wire commit → 第二次来源校验 → ACK（`crates/bi-owner-runtime/src/task_event_receiver.rs:447-505`）；`IdentityConflict` 进入 quarantine，而不是直接 ACK。
- 组合层只运行一个 receiver；adapter 采用单个 outstanding/in-flight permit，且存在 wire/frame 限制与请求 deadline。此处仅确认相关源代码约束，不确认运行时行为。

## 未执行的验证

本复审只读取源码、WBS/设计和阶段报告。按任务边界未运行 Cargo、数据库迁移或 SQL 探针，也未运行真实 gRPC/mTLS transport、部署或运行验收。CodeGraph 未匹配该源 worktree；已按协调者指示直接读取指定源码文件，未索引代码图。来源阶段报告中 rustfmt、静态 alias 对照、Cypher Guardian 与 `git diff --check` 的 PASS 是来源报告所述证据，本复审没有重新运行或独立确认。

因此**不得声称 9F4J.3 已通过集成门或可用于生产**。在 P1 修复、P2 决策/修复和报告基线同步前，不批准本代码 lane 集成。
