# DD-W16-PAYLOAD-SECURITY-001 — 设备身份与协作载荷 envelope 详细设计

> 状态：Draft v0.1；补充 `DD-CANVAS-AGENT-001` v0.2 §8.7。接口、schema 与算法仍须实现 review；当前无生产实现。

## 1. 模块边界

| 模块 | 权威事实 | 不负责 |
|---|---|---|
| Identity/Auth owner | actor、tenant、设备凭据、设备 generation/status、OAuth 一次性消费、设备撤销与审计 | Canvas 内容、draft/conflict、密文正文 |
| Collaboration owner | 当前 Run/resource scope 授权、retention、canonical revision、draft/conflict、idempotency、RLS、audit、Outbox | 设备凭据私钥、KMS master key、跨 owner 写事务 |
| Rust KMS adapter | profile 解析、受限认证、DEK wrap/unwrap、provider/key version 与健康状态 | Project/Run 授权、业务 retention、正文 owner、用户通知 |
| `TranscriptBodyProtector` | canonical AAD、AEAD、envelope 编解码、受控明文/DEK 生命周期 | 决定作者、recipient、RLS policy 或业务提交成功 |

## 2. 认证链与防重放

### 2.1 设备绑定

1. 用户以现有可信身份开始 enrollment；身份 owner 返回短期单次 challenge。设备在平台安全存储中生成或读取非导出私钥，签署 challenge 与协议域分离标记。
2. owner 验签并检查 user/tenant、challenge TTL、一次性状态及并发限制，再分配 `device_id` 和 `device_generation`。仅持久化公钥/凭据引用与状态；私钥从不发送到服务器。
3. Auth issuer 将已验证的 `device_id`、generation 和 principal kind 放入签名令牌。旧无 device claim 的 session 可继续非协作功能，但不能提交共享草稿。
4. REST extractor 从经签名验证的 claims 构造 `AuthenticatedDeviceContext`；请求字段中的 actor、tenant、device 与 scope 不覆盖它。可选诊断 mismatch 只返回通用 401/403。

规范化上下文至少包含 `actor_id`、`tenant_id`、`device_id`、`device_generation`、`principal_kind`、`client_id`、`scope` 与 token `jti`；不包含设备私钥或 token 原文。

### 2.2 原子授权码/refresh 消费

授权码采用单条事务性 `UPDATE … WHERE code_hash = ? AND consumed_at IS NULL AND expires_at > now() RETURNING …`。refresh 使用等价的条件消费并原子插入唯一 successor。只有 `RETURNING` 取得一行的调用才能签发令牌；零行/冲突返回 `invalid_grant`。数据库 unavailable、事务取消和重复 key 不能退化为继续签发。并发测试必须同步启动多个消费者，不以顺序单测证明单次消费。

### 2.3 撤销与活跃 stream

撤销先由 Identity owner 原子推进 device generation、标记 revoked、撤销 refresh family 并追加审计；事务 Outbox 通知 auth/API 节点失效。每个 API 请求在 owner 授权前确认当前 generation/status；SSE 订阅收到失效事件即关闭，连接重建必重新认证/授权。缓存为容量有界、事件失效、短 TTL；事件丢失后的授权租约上限必须写入 profile。撤销检查无法完成时拒绝读写，不能假定 JWT 未过期即仍有效。

## 3. Envelope 数据与密码学契约

### 3.1 保护上下文

加密与解密使用同一版本的 canonical binary encoding 构造 AAD：

```text
aad_schema_version | tenant_id | project_id | run_id | resource_kind |
resource_id | actor_id | base_revision | payload_id | checkpoint_revision |
retention_policy_revision | envelope_format_version
```

所有 id 来自经验证的 owner/context，字段顺序、UUID 编码、空值表示及规范化版本固定；不得用客户端 JSON 原串直接作为 AAD。跨 tenant/Project/Run/resource/revision 替换记录必须认证失败。

### 3.2 Envelope 逻辑字段

`ProtectedTranscriptBody` 的版本化编码至少包含：

| 字段 | 规则 |
|---|---|
| `format_version` / `aad_schema_version` | 不可省略；未知版本 fail closed。 |
| `provider_profile_id` / `profile_revision` / `profile_digest` | 在第一次持久化时固定，不随运行配置变化。 |
| `key_ref` / `key_version` | opaque、可恢复、可审计；不得含凭证。 |
| `wrapped_dek` | 仅 provider 可 unwrap 的密钥封装；禁止 DEK 明文持久化。 |
| `aead_suite` / `nonce` / `ciphertext_and_tag` | suite 明确版本；nonce 对同一 DEK 不复用；tag 校验失败拒绝。 |
| `ciphertext_digest` | 仅用于传输/对象完整性，不产生明文摘要或可识别内容标记。 |
| `resource/payload reference` | 从 owner 的授权记录解析；客户端不能通过替换 reference 越权读回。 |

逻辑结构不冻结物理 SQL/对象存储表。若拆 envelope manifest 或 blob table，须依 basic design §3 分类并落实 per-table RLS/retention/audit。

### 3.3 保护及读回序列

1. Auth extractor 验证 JWT 和 active device generation；owner 从可信 context 重新解析 Project/Run/Worktree scope、actor 权限、expected revision 与 retention policy revision。
2. 前端只在有限 pending delta 中 coalesce；达到 profile 的 checkpoint 条件后提交单个幂等 operation id。禁止逐字符写出 immutable snapshot 或逐键调用 KMS。
3. Rust protector 取固定 `KmsProfile`，通过短期最小权限身份请求 DEK wrap/generate，使用本地 AEAD 对正文加密；DEK/明文缓冲进入有限生命周期区，正常/错误/取消路径均尽最大努力清零。
4. 协作 owner 在自己的事务中写密文元数据、draft head/checkpoint、审计和幂等 receipt；只有完整密文 digest、tag 及 retention 通过验证后才可参与冲突判定。owner Outbox 只带 resource/revision/event/conflict id 等 metadata。
5. 读回先重验 actor/device/Run/resource 权限与 retention，再用记录固定的 provider/profile/key version、同一 AAD unwrap 和 AEAD 解密。撤权检查先于 KMS call；所有保护结果只在内存中短暂供授权响应使用。

## 4. 轮换、恢复与删除

- 新数据使用 active key version，旧 envelope 保留其 provider/profile/key identity；profile update 不隐式改写历史 envelope。
- 对仅 wrapping key 轮换，provider 可 rewrap wrapped DEK；若 AAD、suite、DEK 或 provider 改变，则按幂等迁移生成新 envelope，保留旧版本引用直到迁移审计与备份 retention 完成。
- 每次 rewrap/reencrypt 保存 append-only key lifecycle audit 与迁移操作 id；中断后可安全续跑，不允许部分成功对用户表现为已完成。
- 销毁 key 前先在受信 owner 索引中核验 live payload、immutable revisions、retention、备份与恢复点的引用；引用未知时禁止销毁。provider 不可用时读取保持 unavailable，不以旧进程缓存解密。
- 恢复演练从 database/blob backup 和 provider 快照/可恢复 key identity 建立干净进程，再读出授权测试资源；恢复后对撤销 actor 的读仍须失败。

## 5. 数据存储、安全日志与错误映射

| 故障 | API 行为 | 业务状态 |
|---|---|---|
| `device_unbound` / `device_revoked` / generation mismatch | 401/403，通用错误 | 不写 owner facts；local-only |
| `credential_replayed` | `invalid_grant`；重复消费不签发 successor | 事务回滚或唯一败者记录 |
| KMS timeout/denied/unknown profile/key version | 503/不可用 | 不提交正文或成功 receipt；保留本地草稿 |
| AAD/context mismatch、tag/ciphertext tamper | 受控 integrity error，安全审计 | 不回传明文，不更新 revision/conflict |
| queue/buffer/resource limit | backpressure 或 retryable status | canonical revision 不变，pending delta 有界 |

应用和 provider 日志只能记录 operation ID、profile/key opaque id、结果码、耗时分桶和字节数分桶。禁止记录正文、DEK、access/refresh token、enrollment proof、provider credentials、未脱敏 headers 或完整 KMS request/response。

## 6. 验证映射与实施门

| Gate | 必须证明 |
|---|---|
| Device/Auth | forged body/header identity 被拒绝；已撤销设备对新 API 与活跃 stream 均失效；并发 auth-code/refresh 消费恰好一个胜者。 |
| Envelope | 重启/第二进程可在合法授权下解密；跨 tenant/resource/revision AAD 替换、ciphertext/tag 篡改均失败。 |
| KMS lifecycle | unavailable/permission revoked/unknown profile fail closed；key rotation 前后历史读取；database/blob/provider backup restore；有引用时 key deletion 被阻止。 |
| Owner/RLS | Neon 或真实 PostgreSQL 的 RLS、owner transaction、idempotency、retention 与只含 metadata 的 Outbox；两 actor 互读 scope 被拒绝。 |
| Bounded runtime | 固定 Collaboration Profile 与 workload 下 KMS 请求频次、缓冲/队列上限、RSS/CPU、取消、慢消费者与 PT-09 p99 结果。 |

测试环境最多两个 Multipass inventory 实例；这只支持可重建验证，不证明两节点 HA。Vercel 路径使用 Neon 并单独核验云身份。没有真实 provider、数据库/RLS、撤销、恢复和 profile 性能证据时，W16、WLD-7、PT-09/10 保持 OPEN/NOT_RUN。

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义认证设备链、原子 OAuth 凭据消费、canonical AAD、envelope 字段、撤销、轮换/恢复、错误语义及资源有界验证门；未批准物理 DDL/provider。 | W16 device 与 KMS 审计。 |
