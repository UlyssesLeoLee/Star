# W16-A 设备撤销 API 与 owner 事务审查报告

**审查基线**：`dev@be3dd7b42881cc0ebd229a622f87bc5f407e9f69`
**范围**：已认证的设备撤销 HTTP handler、认证中间件、PostgreSQL owner 事务、迁移权限与相关测试。
**结论**：静态源码中未发现可确认的身份越权或事务原子性缺陷。发现两项需要在关闭 W16 验收门前处理的 P2：设备失效通知没有实现详细设计要求的事务 Outbox；设备撤销 HTTP 路由及其审计结果缺少直接验收测试。重复请求的错误语义另列为待冻结的 API 契约。

## §0 目的

检查 `POST /oauth/device/{device_id}/revoke` 是否使用可信 actor/tenant、在 owner 范围内撤销指定设备并清理 refresh token、写入审计，以及其测试是否覆盖 HTTP 与实时协作失效路径。

这是固定基线上的源码审查，不是实现或运行验收。只读范围包括：

- `crates/star-api-rest/src/auth/oauth/device_enrollment.rs`
- `crates/star-api-rest/src/auth/oauth/middleware.rs`
- `crates/star-pg-adapter/src/repository/device_bindings.rs`
- `db/migrations/2026-10-28-w16-device-auth.sql`
- 直接相关测试和 W16 需求、设计、测试设计

## §1 审查结果矩阵

| ID | 轴 / 级别 | 结果与证据 |
|---|---|---|
| F-01 | Spec / P2 | **失效通知与详细设计不一致。** 详细设计要求 Identity owner 在撤销事务中写入 Outbox 通知 API 节点（`DD-W16-PAYLOAD-SECURITY-001.md:31`）。当前迁移仅在设备 generation/status 更新后发送通用 `pg_notify('star_device_binding_wake','changed')`（`2026-10-28-w16-device-auth.sql:34-37`）；定向搜索未找到设备撤销 Outbox 写入或重放消费者。Canvas listener 收到通知后广播唤醒，SSE 再次授权；每 60 秒也会重新授权（`canvas_event_stream.rs:71-77,187-240,287-319`）。因此现有实现有瞬时唤醒和有界重新检查，但没有所要求的持久事件及断线重放证据。这是设计一致性缺口，**不是本次确认的越权漏洞**。应实现事务 Outbox，或正式修订设计并以验收证据明确批准“瞬时通知 + 60 秒租约复验”的边界。 |
| F-02 | Spec / P2 | **HTTP 撤销与审计缺少直接验收。** 撤销路由和 handler 存在（`device_enrollment.rs:83-88,240-258`）。已找到的 PostgreSQL repository 测试验证 generation 变为 2、旧 generation 失效、refresh family 被撤销（`device_auth_pg.rs:116-131`），但没有断言审计行，也没有经 HTTP handler 调用。对 `crates/star-api-rest/tests` 的定向搜索未找到该 revoke 路由测试；现有 `canvas_device_auth_pg.rs:150-234` 覆盖的是过期 generation 的 Canvas 写入拒绝。W16 测试设计要求的身份撤销与 SSE 过期场景仍标为设计未执行，Canvas SSE 候选也标记为 NOT_RUN（`TEST-DESIGN-WORKTREE-CANVAS-001.md:283,287-291`）。因此 handler、错误映射、审计恰好一条及其与 SSE 的端到端效果尚未被验收。 |
| N-01 | Spec / Info | **重试语义需冻结。** 目标不存在或已经撤销时，repository 返回 `None`，handler 返回 `invalid_grant`；错误映射为 HTTP 400（`device_bindings.rs:368-373`、`device_enrollment.rs:252-255`、`handlers.rs:721-726`）。并发重复请求由 active 状态条件更新保证至多一次撤销审计，但若首次提交成功而客户端未收到 204，重试会看到 400。当前所查规范没有规定这类重试应返回成功还是错误；需在 API 契约中决定并测试，现阶段不判定为实现缺陷。 |
| S-01 | Standards | 独立静态标准审查未发现已证实的代码或安全标准违规。handler 只使用已验证 claims 中的 tenant/actor；owner 事务设置对应 tenant/actor scope，并把目标更新、refresh 撤销、审计插入放在同一事务中。 |

### F-01/F-02 的规范依据

- SRS 要求以签名 claims 的 actor、tenant、device 与当前 generation 为准，撤销后阻止受保护操作并对未知状态 fail closed：`SRS-W16-PAYLOAD-SECURITY-001.md:13-14`。
- 基本设计要求撤销写 append-only audit 与版本化失效事件、撤销 refresh session 并关闭现有 SSE：`BD-W16-PAYLOAD-SECURITY-001.md:15`。
- 详细设计明确要求事务 Outbox 通知失效；失效事件丢失时由有界授权租约兜底：`DD-W16-PAYLOAD-SECURITY-001.md:31`。

## §2 验证摘要

| 检查项 | 静态证据 | 结果 |
|---|---|---|
| 路由与身份输入 | `device_enrollment.rs:83-88,240-250` 从 `AuthenticatedUser` 提取 actor，路径只提供目标 device UUID；handler 不从 body/header 接受 actor 或 tenant。 |
| JWT 与设备上下文 | `middleware.rs:246-279,283-294` 校验 subject/user 一致、tenant 非空；设备上下文来自签名 claims；除 bootstrap 路由外，POST 变更要求当前有效设备并调用 repository 检查 generation。 |
| owner transaction | `device_bindings.rs:341-390,394-412` 在单个事务中设置 owner role 与 tenant/actor 局部 scope；UPDATE 同时限定 tenant、actor、device、active 状态，generation 递增。无匹配行返回 `None`。 |
| refresh 与审计 | `device_bindings.rs:375-389,414-435` 在同一事务撤销该 actor/device 的未撤销 refresh token 并插入 `revoked` 审计行，随后提交。静态代码支持原子提交判断；本轮未运行数据库验证。 |
| RLS 与 append-only | `2026-10-28-w16-device-auth.sql:30-50,55-69,85-94` 定义审计变更保护触发器、FORCE RLS、owner scope policy、受限列权限及 refresh-token actor/tenant scope。实际部署登录的 owner-role membership 需由运行环境验收。 |
| 测试覆盖 | repository 测试验证部分持久化状态；没有找到 HTTP revoke 或审计计数测试。测试设计将完整身份/SSE 流程和 Canvas SSE 候选标为未执行。 |

本次只进行精确文件读取与限定字符串搜索。工作树没有可用 CodeGraph 索引，且本会话无索引覆盖工具，因此没有图谱完整性结论。按 brief 禁止运行 Cargo、PostgreSQL、浏览器或迁移；未声称这些验证通过。

## §3 已知缺口

1. 没有经实际 HTTP revoke handler 的成功、失败、重复和并发验收。
2. repository 测试没有核对 `oauth_device_binding_audit` 的行数、event kind、generation 或回滚行为。
3. 设备失效使用通用、非持久 `NOTIFY`；跨节点 listener 断连后的 durable replay 未实现或未发现。当前 SSE 的 60 秒授权租约是有界复验，不等价于详细设计所要求的事务 Outbox。
4. 真实部署登录能否按部署配置加入 `star_oauth_credential_owner`、以及应用角色下的实际 RLS/ACL 行为，本轮没有连接目标数据库验证。
5. 当前分支测试设计记录 Canvas SSE revoke candidate 为 NOT_RUN / RESOURCE_INTERRUPTED；不能将 endpoint 到活跃 SSE 的闭环标为已验收。
6. 本报告只基于指定 dev 快照及有限源码范围；没有对其他分支、工作树或数据库状态作结论。

## §4 子代理失败接手清单

- 标准与安全轴的独立只读复核已完成：未报告具体源码缺陷；指出 HTTP revoke route 测试缺口，与 F-02 一致。
- 规格轴由本报告完成；未改动源码、迁移、WBS 或测试设计。
- 无子代理失败或需要接手的未完成改动。

## §5 后续守门与最小验收集

在将 W16-A 设备撤销 HTTP/SSE 链路标为通过前，至少完成：

1. 使用有效、当前设备上下文调用 HTTP revoke，验证 204；确认目标设备 generation/status、该设备 refresh family 与恰好一条 `revoked` audit 同事务持久化。
2. 验证无效/缺失 bearer、无设备上下文、过期或已撤销 caller device、跨 actor/tenant 的目标、malformed/missing/already-revoked 目标均按固定契约拒绝，且不产生额外状态或审计写入。
3. 并发提交相同目标撤销，验证至多一个状态迁移和一条审计；另固定重复重试返回语义。
4. 让活跃 Canvas SSE 通过真实 HTTP revoke 失效，验证收到通知时关闭、不再发送 payload；再覆盖 listener 中断或通知丢失，验证不超过配置租约即 fail closed。
5. 以实际应用登录和目标 PostgreSQL 验证 owner-role membership、RLS/ACL、事务回滚及多 gateway 行为。
6. 按详细设计决定并验证失效传播：实现可重放的事务 Outbox；或更新 SRS/BD/DD 与验收标准，明确瞬时 PostgreSQL NOTIFY 的边界及 60 秒复验保证。

## §6 签字栏

| 角色 | 审核者 | 结论 |
|---|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008） | 静态审查完成；F-01/F-02 保持开放，不将局部源码正确性等同端到端验收。 |
| SRE Lead | Mavis 接手审核 | 运行期数据库、owner grants 与 SSE 故障恢复未验证。 |
| 平台 | Mavis 接手审核 | 未检查或修改部署与集群配置。 |
| 评审主持 | Mavis 接手审核 | 范围和证据限制已记录。 |
| PM | Mavis 接手审核 | W16-A 设备撤销 HTTP/SSE 验收在上述门关闭前保持开放。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 内容 |
|---|---|---|---|
| v0.1 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 基于固定 dev HEAD 完成设备撤销 handler、owner transaction、RLS/审计与测试证据审查；记录 F-01、F-02 与重试契约观察。 |
