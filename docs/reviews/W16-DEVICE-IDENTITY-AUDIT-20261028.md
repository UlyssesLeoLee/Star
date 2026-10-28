# W16 认证设备身份审计

- **基线：** `dev@3b873d3e452f3340bb2e92f1d155fc5e54730a3e`
- **结论：BLOCKED。** 当前 REST OAuth/JWT 链路不能产生或核验可信的设备身份；WLD-7 远端协作须保持 `local_only / sync_pending`，直到身份与撤销链通过实现验收。
- **范围：** OAuth/JWT、REST 请求身份、Device 领域、PostgreSQL adapter/迁移。直接读取源文件并按标识定向搜索；未运行测试、构建或数据库操作。

## 1. 阻断发现

**B1 — 认证上下文没有设备身份。** `Claims`、`AuthUser` 没有 device_id，`issue_token` 不接收 device_id；REST 的 `AuthenticatedUser` 只验证签名及用户/租户字段。故 REST 请求无法证明当前是哪台已注册设备。不得以 body/header 中的 device_id 或 actor_id 补齐；那会让客户端定义参与者身份。

**B2 — Device 仅有领域/内存实现。** Device 模型和 register/revoke ports 存在，但定向搜索没有找到可供 REST/OAuth 使用的 PostgreSQL identity adapter、设备路由或设备 DDL。MCP identity handler 自己建立 `InMemoryIdentityService`，不能作生产认证来源。当前注册命令接受调用方给出的 fingerprint 字符串，没有可见的持钥挑战证明；project_ids 非空检查也不等于项目或 Run 授权。

**B3 — OAuth 一次性兑换存在并发重放窗口。** 授权码先读后标记消费；refresh token 先读后撤销。两个 PostgreSQL 更新都返回 `Result<(), _>`，忽略受影响行数。两个并发请求可先都读到有效记录，第二个更新 0 行仍被视为成功并继续签发令牌。兑换必须由事务内原子消费/领取决定，只有唯一成功者能签发；败者应返回 `invalid_grant`。

**B4 — 设备撤销不能即时撤销现有 JWT。** `verify_token` 只验证签名、issuer、audience 和 expiry；REST extractor 不查 Access Token 或设备状态。因此，新增 device claim 后，撤销设备本身仍不会令未过期 JWT 失效。敏感写必须查当前设备状态/代次，或定义有可靠失效通知且有界的缓存 SLA；校验不可用时 fail closed。

**B5 — 服务凭证不能冒充用户设备。** client_credentials 在 `issue_tokens` 中传入 None 用户，helper 以 nil UUID 和 `user` role 生成 token；`AuthenticatedUser` 又拒绝 nil subject。此路径没有形成经本审查确认的服务主体契约。应为 ServiceClient 建独立身份/授权上下文，不得伪造 user/device。

## 2. 当前身份链

| 环节 | 源证据与实际行为 |
|---|---|
| Claims 与 JWT | `crates/star-api-rest/src/auth/mod.rs:54-77, 134-176`：claim 有 sub/iss/aud/时间/jti/tenant/user/roles/scope，无 device；签发和验证均不绑定设备，验证只解码/验签。 |
| REST extractor | `crates/star-api-rest/src/auth/oauth/middleware.rs:121-167`：提取 Bearer、验签、检查 sub=user 且非 nil、tenant 非 nil，然后构造 AuthUser；未查设备/Access Token 状态。 |
| Authorization Code + PKCE | `crates/star-api-rest/src/auth/oauth/handlers.rs:195-253`：查 code，验 PKCE，标记 consumed，再用记录中的 user/tenant 签发。code 与请求模型没有 device 字段；PKCE 证明 verifier 持有，不证明设备密钥持有。 |
| Refresh / token rows | `handlers.rs:302-435`：从 refresh row 取 user/tenant/scope，撤旧并发新；Access/Refresh row 不含 device。client_credentials 无 user 且不发 refresh。 |
| PostgreSQL 消费操作 | `crates/star-pg-adapter/src/repository/oauth_authorization_codes.rs:109-149`、`oauth_refresh_tokens.rs:105-146`：条件 UPDATE 防止重复状态变更，但没有把 rows_affected 暴露给 handler。 |
| OAuth DDL | `db/migrations/2026-09-09-oauth2-server.sql:82-107, 156-179, 218-241`：auth code、access、refresh 表无 device_id。定向搜索 device 表/fingerprint 列未发现设备 DDL。 |
| Device domain | `crates/domain-identity/src/lib.rs:162-185, 376-388, 448-536`：定义 Device、注册命令、ports；`552-568, 619-655, 693-728, 841-867` 为 InMemory service/repository。注册会生成 DeviceId，但仅核用户/租户、project_ids 非空；未证明设备密钥持有或对 project/Run 的授权。 |
| MCP 示例 | `crates/star-mcp/src/handlers/identity.rs:29-47, 57-93`：handler 持有 InMemory service，并从 resource URI tenant/user 值构造 ActorContext；不属于 REST trusted-auth 链。 |
| Group API | `crates/star-api-rest/src/group_api/canvas.rs:557-567`：写处理器接收 AuthenticatedUser(actor) 并按 actor 授权，但 AuthUser 未提供 device。 |

`star-context::ActorContext` 有 `device_id: Option<Uuid>`（`crates/star-context/src/actor.rs:61`），但出现此字段不代表 OAuth → AuthUser → REST owner handler 已映射可信设备身份。

## 3. 生产实现与参考实现

- **已存在：** REST JWT 验证与 AuthUser 提取；OAuth code/access/refresh 有 PostgreSQL repositories 和 migration。
- **未形成生产设备链：** 对 `crates` 和 `db/migrations` 定向检索 Identity ports、`device_id`、`device_cert_fingerprint`、设备建表等标识，仅发现 domain 内存实现、MCP 对其引用及上下文/运行时类型；未发现 REST Device 路由、PG Device repository 或 device DDL。
- **测试边界：** `domain-identity/src/lib.rs:1050-1153` 的 register/revoke 测试使用内存 service 与普通字符串 fingerprint，没有经过 OAuth、PG、设备 proof、REST extractor；不构成生产闭环证据。
- 以上是列明目录和关键词范围内的审查，不宣称覆盖其他主机、运行时或部署服务。

## 4. 最小可信契约建议

1. 由已认证用户发起 enrollment；服务端发一次性 challenge，设备证明持有其公钥/凭据后才签发 DeviceId。tenant/user 从可信 ActorContext 推导；只给 fingerprint 或 DeviceId 不算证明。浏览器、桌面、Local Runtime 分别定义版本化的凭据和恢复方式。
2. Auth code/session 与服务端已核验的 device 固定绑定；PKCE 不替代设备证明。Refresh row 保存 device/session/generation，轮换后保持相同 user/tenant/device/client/scope，并以数据库原子操作实现单次兑换。
3. AuthContext 显式区分 `UserDevice`、`ServiceClient`、`AgentRuntime`。服务凭证无 device；任务 CLI 是绑定 Run/Worktree/Task 的 AgentRuntime，不冒充个人设备。
4. 撤销设备须推进凭据代次并撤销该设备 refresh sessions；敏感 API 每次验证设备当前状态/代次。若使用缓存，需有界容量、失效传播和撤销延迟 SLA；未知或不可达时拒绝远端协作。
5. owner API 从 AuthContext 派生 actor/device/tenant、事件作者与接收者；忽略/拒绝请求中身份字段。设备身份只确定客户端，不替代当前 Project/Run/Worktree scope 与授权校验。
6. 未注册、proof 错误、已撤销、tenant/user 不匹配、重放、授权/设备状态不可用及 client/service 身份不全均 fail closed；前端保留本地草稿并标为 `local_only / sync_pending`。

## 5. 实施次序与验收矩阵

| 次序与文件范围 | 需要完成的工作 | 验收重点 |
|---|---|---|
| 1. `domain-identity`；新 device migration；`star-pg-adapter` identity repository | 持久化 enrollment、凭据 proof、状态/代次、审计；授权身份来自 server context。 | 错 proof、重复凭据、跨 tenant/user、撤销设备、数据库恢复；RLS/审计有效。 |
| 2. OAuth handlers、authorization/refresh/access repos 与迁移 | Auth code/session/refresh 保存 device 绑定；code/refresh 原子消费，撤销联动会话/代次。 | 并发兑换 N 次恰好一个成功，其余 invalid_grant；refresh 保留身份；设备撤销阻断 refresh 与现有 JWT。 |
| 3. `auth/mod.rs`、OAuth middleware、AuthContext 转换 | 加入明确主体类型与验证，保持 service/device/agent 边界。 | 缺失、未知、撤销、跨租户及状态服务故障均拒绝；不得接受伪造 body/header 身份。 |
| 4. 需要身份的 Group owner handlers（含 `group_api/canvas.rs`） | 使用可信 actor/device；保留原 Run 权限、scope 与 RLS 检查。 | 两台设备参与者各自可追溯；同用户多设备独立撤销；收件人/作者不可由请求伪造。 |
| 5. 新增 REST/PG OAuth-device 集成测试 | 覆盖真实 DDL、并发、Extractor、owner 写入和失败路径。 | 覆盖上述拒绝/成功场景；不以 domain 内存测试替代。 |

## 6. 未决事项与证据限制

- 需确认 Web/Desktop/Multipass Local Runtime 的设备凭据及恢复策略、用户本人撤销权和管理员授权。
- 需确认 JWT 即时撤销目标与可接受 SLA；仅把 device_id 加入 JWT 不会实现撤销。
- 本 worktree 无 CodeGraph 覆盖率检查工具；使用定向源码检索。未运行测试、构建、迁移或性能验证。建议验收矩阵不是测试通过结果。
- 基线：`3b873d3e452f3340bb2e92f1d155fc5e54730a3e`。核心源文件 SHA-256：
  - `auth/mod.rs` `6cc1b4af55f8a17d446f89cf11ed58aca6ea5ad2029cd62b150ae10497b78ec4`
  - `oauth/handlers.rs` `29b9c5993020bf33666cedefd28672c69b93ffc4163393cf4ae73bef7f49890b`
  - `oauth/middleware.rs` `e9cab07d1595c1674f2f6301ef9c033ed371f14ea96d12341a0ec494fa34a18a`
  - `oauth_refresh_tokens.rs` `40c1f370dad03926eae8763a613e3f657bcc42173336ce2835263046b33efc77`
  - `oauth_authorization_codes.rs` `03f2e0a2cb7e7e8df75ef49e8d1bd4448c6e798cea941ad3b9b40afa5e73cfa7`
  - `domain-identity/src/lib.rs` `72239056e7c35e499f92ad1d8fbb8250962195bf0728fbaf088f8caf479e1073`
  - `2026-09-09-oauth2-server.sql` `b623dbb46fdf43d710589ea81d2bb4a9d9c9efdff940130e2a3531737629a021`
  - `star-mcp/handlers/identity.rs` `cfb74397cc42acac017962f595f2739af25c858662fee634ec56b23ab94ac88d`
  - `group_api/canvas.rs` `742f93f3c558a08528bba91f0b817c0b42e680c2a40802ff4630d78e8bef4d56`

审计人：Codex（指定范围）
