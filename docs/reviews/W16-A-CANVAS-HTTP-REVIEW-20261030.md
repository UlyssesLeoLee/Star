# W16-A Canvas SSE revocation source review

> **Status:** completed bounded source review; PostgreSQL runtime acceptance was not run by this review.
> **Reviewed candidate:** `5f437c06933f9f84ac26de6f13f1b7cf916cfb23`
> **Integration baseline:** `dev@9e9f48ae74fe03655cba054f41a4f8330df71e7a`

## §0 目的与范围

对 Canvas SSE 设备撤销候选测试及其当前 API、授权和 migration 路径作独立静态复核。复核判断测试实际触发和断言了什么、依赖哪些已安装数据库对象，以及哪些端到端门仍未覆盖。本报告是源代码审查，不是 PostgreSQL、RLS、HTTP 认证或生产运行验收。

复核范围：候选 `crates/star-api-rest/tests/canvas_device_auth_pg.rs`；dev 中的 `canvas_event_stream.rs`、`canvas.rs`、W16 device-auth 与 Canvas wake migrations，以及 `TEST-DESIGN-WORKTREE-CANVAS-001.md` 的 W16 验收映射。正在变动的 PG runner 不在本报告审查范围。

## §1 发现与证据矩阵

| ID / 严重度 | 结论 | 证据与影响 | 处置 |
|---|---|---|---|
| R1 / P2 范围边界 | 候选测试通过直接 SQL 将 device binding 设为 `revoked`，没有调用用户可用的设备撤销 HTTP/API，也不核验 `oauth_device_binding_audit` 中的撤销事实。 | 候选 `canvas_device_auth_pg.rs:279-394`；`2026-10-28-w16-device-auth.sql:30-37`。这能验证已提交撤销状态触发 wake 并关闭当前流，但不能证明撤销端点的身份、权限、owner 事务及审计完整链路。 | 保留为窄 PG acceptance slice；E2E-COLLAB-13/19 与完整 W16-A 继续 OPEN。后续另验撤销 API 与 append-only audit。 |
| R2 / P2 范围边界 | 测试启动一个进程内 `CanvasEventWakeHub`，不覆盖双 gateway、通知丢失、listener 重连、跨实例 fan-out 或 60 秒授权租约兜底。 | 候选 `canvas_device_auth_pg.rs:306-394` 只创建一个 hub/router；`canvas_event_stream.rs:74-75,121-128,294-339` 显示每进程 listener、进程内广播和 60 秒重验证租约。测试只验证当前进程收到 device-change wake 后重新授权并结束流。 | 不将此用例写成跨实例或恢复通过；这些条件继续由 W16/E2E/PT 门验收。 |
| R3 / P2 Harness 前置条件 | Rust 测试自身只判断 PostgreSQL 地址为 loopback 或 Unix socket，不证明目标数据库为本次专属临时库；fixture 提交持久行且测试不逐项清理。 | 候选 `canvas_device_auth_pg.rs:288-306,412-579`；本测试通过 `STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL` 连接并写入带随机 ID 的 Project/Run/Worktree/device fixture。候选测试还执行可替换 trigger 定义的 Canvas wake migration。 | 运行证据只能来自经独立验证的 disposable harness；harness 必须证明其自有实例/数据目录、迁移顺序、受控停止和成功后清理策略。失败时保留数据和日志。该 review 未检查 runner，故不判定 runner 已满足此门。 |
| R4 / P2 性能边界 | 5 秒流关闭 deadline 是功能性上限，不是撤权传播的 p99 或低延迟证明。 | 候选 `canvas_device_auth_pg.rs:376-382` 只断言一次事件后 `body.next()` 在 5 秒内返回 EOF；测试设计 `TEST-DESIGN-WORKTREE-CANVAS-001.md:283,291` 将其归入持久身份/撤权验收，另行要求 PT-09。 | 不据此关闭 PT-09、W14 或 W16 性能门。 |
| R5 / 无代码缺陷 | 候选的被测路径与窄目标一致：先建立带设备 generation 的 SSE 会话并证明 stream 可收取一条 Worktree wake，再提交 device status 撤销，验证活动流在有界时间内关闭。 | 候选 `canvas_device_auth_pg.rs:313-394`；`canvas_event_stream.rs:154-179,205-240`；`canvas.rs:1767-1835`。每个 Canvas wake/device-change wake 都调用当前 Run/Worktree/Project 与 device generation 授权查询；撤权后该查询拒绝 generation/status，stream 返回 EOF。 | 源码断言路径看起来与本用例窄目标相符；仍需运行其隔离 PG acceptance 才能记为 PASS。 |
| R6 / 无 migration 缺项 | Hub 启动显式检查 Canvas Outbox trigger 和 device-binding trigger；候选会安装 Canvas wake migration，并要求完整迁移库已提供 W16 trigger。 | `canvas_event_stream.rs:96-129`；候选 `canvas_device_auth_pg.rs:300-307`；`2026-10-28-w16-device-auth.sql:34-37` 与 `2026-10-28-canvas-outbox-wakeup.sql:18-33`。缺少 device migration 时 hub startup fail closed。 | Runner 应先安装 Canvas schema/Phase 2D 与 W16 device-auth migrations，再执行候选 test；不得把单独执行的 Outbox migration 当作完整数据库准备。 |

**审查结论：**未发现会让上述窄断言失效的确定性源码错误。R1–R4 是验收范围/运行证据边界，不应将局部用例扩大解释为完整认证、跨实例或性能通过。R3 的 runner 安全门仍须由独立审查和真实 run 证据确认。

## §2 验证摘要

- 以 `git show 5f437c06933f9f84ac26de6f13f1b7cf916cfb23:crates/star-api-rest/tests/canvas_device_auth_pg.rs` 核对候选测试源码。
- 静态检查 dev 中 SSE route、授权复验、listener signal 处理、device wake trigger 和 Canvas Outbox migration。
- 对照测试设计中 `IT-W16-IDENTITY-01`、`IT-W16-CANVAS-SSE-REVOCATION-PG-01` 与 W16 requirement mapping。
- 本 review 未执行 Cargo、测试、PostgreSQL、RLS、migration 或 runner。候选测试的运行状态仍是 **NOT_RUN**，不能记录为通过。

## §3 已知缺口

1. 用户撤销 API 的身份/授权、owner 事务、audit 写入及 API 响应仍未被该用例覆盖。
2. 未验证撤销与并发读写的事务竞争、通知丢失后的租约失效、listener 重连、多个 API 实例与重新连接恢复。
3. 未使用生产等价的受限登录/owner role 验证 grants、RLS/FORCE RLS；该测试 fixture 的数据库角色是否受限取决于隔离 runner。
4. 没有 p99、吞吐量、RSS、CPU、队列背压或撤权到流关闭延迟分布证据。
5. 未检查 acceptance runner，因此 disposable 实例生命周期、migration 次序与失败证据保留状态仍待独立确认。

## §4 子代理失败接管清单

- 本 review 为单人只读审查，没有派发子代理；无失败产物需要接管。
- PG runner 的实现与安全审查归属于独立 worker/reviewer；本报告不替代该审查。

## §5 守门规则

1. 只有隔离 PostgreSQL run 的原始 summary、命令退出状态和日志可以将此测试状态从 NOT_RUN 更新；源码检查不算运行通过。
2. 运行前由 runner 确认目标是本轮拥有的 disposable 数据库，不接触用户数据库；未确认时 fail closed。
3. 安装顺序须包含 Canvas 基础 schema/授权表、W16 device-auth trigger 与 Outbox wake migration；启动检查通过后才执行测试。
4. 使用 superuser 或 test-only migration placeholders 的通过结果必须标注局限，不能表述为 production grants/RLS 证明。
5. 直接 SQL 撤权只证明持久状态变化后的 SSE 重新授权，不证明撤销 HTTP endpoint 或审计闭环。
6. 单进程测试不证明双 gateway、通知丢失、重连、跨实例可靠性或 PT-09 p99；W16/WLD-7 保持 OPEN，未关闭的功能继续 fail closed。
7. 运行失败时保留由该 run 产生的数据库目录、日志和 summary；不得仅为清理而删除其他进程或数据。

## §6 签字栏

| 角色 | 审查/审批者 | 状态与边界 |
|---|---|---|
| 架构师 | 架构师 (Mavis 接手 agent per DEC-008) | 已完成本报告限定的源码/验收边界复核；未批准关闭 W16-A。 |
| SRE Lead | 本报告未执行 SRE 运行验证 | 不推定运行审批。 |
| 平台 | 本报告未执行 target DB/RLS/grants 验收 | 不推定平台验收。 |
| 评审主持 | 本报告限定为源码复核 | 不推定跨域验收。 |
| PM | 本报告未执行产品验收 | 不推定产品审批。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 W16-A Canvas SSE 撤权候选的限定源码审查；标明 direct-SQL、单进程、临时数据库与性能边界；保持运行验收 NOT_RUN。 | 并行独立复核候选测试路径与 migration 前置条件。 |
