# 9F4J.3 独立代码复审

> 日期：2026-11-07（第二轮只读复审）
> Source：`D:\orcaWork\Star\9f4j-3-native-composition-20261107`，分支 `codex/9f4j-3-native-composition-20261107`，HEAD `e22d92bcbefaf009e18d043ab2c8a762c150fc01`（相关改动未提交）。
> 目标：本地 `dev@8c2d78cbe77aeb81ed28d2396c319a1bbec24ed9`。
> 范围：只读复核 writer 身份/权限、Profile pin、worker readiness/shutdown 与证据报告；未修改 source lane，未运行 Cargo 或 PostgreSQL。

## 结论

**NOT APPROVED：暂不集成。**首轮指出的 app/query/write 身份对比缺失、Profile 不复验和 panic 后 readiness 可能陈旧的问题已得到针对性修复。第二轮仍发现 **P1**：BI writer 仅排除名称以 `_owner` 结尾的角色，并未限制所有可加入的 runtime 角色，也未排除非 BI 函数权限，因此不能证明 writer 是 BI-only。另有 **P2**：drain timeout 会将 readiness 设为 Blocked，但 shutdown 不返回错误，main 仍可能以成功状态退出。

## 已修复并确认的正向发现

- `bi_task_event_ingestion.rs:974-1101` 读取 app、BI query、writer 三个连接池的 `current_user/session_user`；writer 两个身份均须非空，且不得与 app/query 的四个身份相同。
- `bi_task_event_ingestion.rs:615-670,1125-1156` 每 30 秒通过 resolver 复验固定 Profile，并比较 revision、digest、endpoint 等 pin。漂移时停止接收并设为 Blocked；解析不可用时保持 Blocked、延迟重试，不热切换 client。
- `bi_task_event_ingestion.rs:1103-1113,1207-1239` 的退出 guard 区分 panic/停止状态；readiness 仅在 worker 运行且 phase 为 Ready 时返回 200。panic 或意外停止会被映射为 Blocked。
- `bi_task_event_ingestion.rs:438-484` 的 drain timeout/JoinError 会设为 Blocked，未发现仍报告 Ready 的路径。

## Findings

### [P1] BI writer 没有 runtime 角色与非 BI 函数权限 allowlist

**证据：**`bi_task_event_ingestion.rs:185-197,220-224,261-282` 要求 writer 属于 `star_bi_runtime`，拒绝当前角色可加入的 `*_owner` 和登录身份可 SET ROLE 到的 `*_owner`，但没有拒绝其他 `star_*` runtime 角色，也没有盘点或拒绝 BI 以外的 EXECUTE 权限。`star_platform_transport_runtime` 在 `db/migrations/2026-10-12-platform-native-transport-log.sql:45-46,627-630` 被创建并获授 transport API EXECUTE；认证接口授权见 `db/migrations/2026-10-12-platform-transport-z-native-server-auth.sql:542-547`。writer 若可加入/切换到该角色，此能力不因其名称不以 `_owner` 结尾而被当前谓词拒绝。

**影响：**专用写连接可能同时拥有 BI 事件写入和跨域 transport 操作能力，破坏 BI writer 最小权限边界。此处未运行 PostgreSQL 角色矩阵，不能断言部署已存在该组合；但静态准入条件没有 fail closed 地排除它。

**修复/验收：**将 writer 有效角色和可 SET ROLE 目标限定为显式、版本化的 BI allowlist；检查直接/继承/可切换角色，并拒绝 BI 之外 schema/function 的 EXECUTE 与其他运行时能力。真实 PostgreSQL role matrix 应验证 BI-only 身份通过，而同时具有 `star_platform_transport_runtime`、其他 `star_*` runtime 能力或非 BI EXECUTE 的身份一律 Blocked/503。

### [P2] drain timeout 未作为进程关闭失败传递

**证据：**`bi_task_event_ingestion.rs:438-484` 在 drain timeout 或 JoinError 时设为 Blocked，但 `shutdown()` 返回 `()`。`crates/star-api-rest/src/main.rs:127-133` 调用 shutdown 后仅执行 `server_result?` 并返回 `Ok(())`。server 正常结束但 worker drain 超时时，进程仍可能以成功退出码结束。

**影响：**readiness 在退出前不会谎报 Ready，但 supervisor/部署系统无法通过退出码区分正常 drain 与强制 abort，可能把未完成关闭记录为成功。

**修复/验收：**shutdown 返回可区分 DrainTimeout/JoinError 的 `Result`，main 将其传播为失败退出；测试 worker 超时、panic/JoinError 与正常 drain 的 readiness 和进程结果。

## 报告及验证证据

源阶段报告 v0.5 将 source 基点记为 `e22d92bc`、集成目标记为 `dev@8c2d78cbe77aeb81ed28d2396c319a1bbec24ed9`，与本轮 source HEAD/目标一致；Cargo.lock 已记为纳入登记。源报告自述 `cargo check -p star-api-rest --all-targets -j 4` 通过、定向测试 10 项通过，但本复审未运行 Cargo，这些不算本轮独立验证。源报告也明确 PostgreSQL/grants/RLS、真实 receiver、transport/deployment/runtime 尚未验收。

CodeGraph 未覆盖 source worktree；按协调者指示直接读取指定文件，未索引或改动 `.codegraph`。

## 尚未关闭的集成门

- P1 runtime-role/function allowlist 修复与真实 PostgreSQL role matrix。
- P2 drain 失败传播到进程退出结果。
- PostgreSQL grants/RLS、enabled receiver、mTLS/transport 投递与恢复、部署及资源性能仍 NOT RUN。

因此维持 **NOT APPROVED FOR INTEGRATION**；不得据此声明 production-ready。
