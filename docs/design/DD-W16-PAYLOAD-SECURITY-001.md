# DD-W16-PAYLOAD-SECURITY-001 — 设备身份与协作载荷 envelope 详细设计

> 状态：Draft v0.5；补充 DD-CANVAS-AGENT-001 v0.2 §8.7。W16-A 部分 device/auth owner 与 PG/RLS 切片已在本地 PostgreSQL 验证，OAuth repository slice 3/3 通过；Canvas 单个已连接 SSE 流的设备撤销关闭用例已在当前 dev 基线上通过（run 6d23b8ceb1f0499091885c42271c2fb0），生产 owner/KMS 闭环仍未完成；W16-C provider-neutral envelope core 尚未接入生产 owner。

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

### 2.4 当前 OAuth / Canvas 验收切片边界

- OAuth repository implementation 在本地 disposable PostgreSQL 18.6 上通过三项 ignored tests：auth-code 竞争最多一个胜者且消费/令牌对持久化；refresh 竞争最多一个胜者、唯一 successor 与旧 token 撤销；stale token snapshot 不能消费新状态。run `1b4673269a04435daa1fab5c93333133` 的测试进程以 disposable cluster superuser 连接，并依赖测试专用 role/audit/SCD2 migration prerequisites。该结果只验 repository transaction slice，不替代生产 grants/RLS、认证 HTTP flow、目标数据库或完整 E2E-COLLAB-14。
- 当前 dev 基线上的 crates/star-api-rest/tests/canvas_device_auth_pg.rs::active_canvas_sse_closes_after_device_binding_revocation 已通过 Rust 1.99.0 focused build 与 PostgreSQL 18.6 disposable loopback acceptance（run 6d23b8ceb1f0499091885c42271c2fb0）：41/41 runner steps exit 0，exact test 1 passed/0 failed/0 ignored/3 filtered，0.16s。仅该活动 SSE stream 撤销关闭切片为 PASS；runner 使用 disposable postgres superuser、test-only no-op audit/SCD2 prerequisites，build attestation unsigned，且未应用 replay-offset migration。不得把此结果扩大为生产权限、生产审计、完整 W16-A 或部署验收。

## 3. Envelope 数据与密码学契约

### 3.1 保护上下文

加密与解密使用同一版本的 canonical binary encoding 构造 AAD。W16-C 将 optional Worktree 作为明确的设计增量加入旧版上下文；旧版 DD 字段中不包含 Worktree。编码固定如下：

```text
domain_separator("STAR-W16") |
aad_schema_version:u16_be |
tenant_id:uuid16 | project_id:uuid16 | run_id:uuid16 |
worktree_tag:u8 (0=absent, 1=present) | [worktree_id:uuid16 when tag=1] |
resource_kind_length:u16_be | resource_kind:1..64_ascii_bytes |
resource_id:uuid16 | actor_id:uuid16 | base_revision:u64_be |
payload_id:uuid16 | checkpoint_revision:u64_be |
retention_policy_revision:u64_be | envelope_format_version:u32_be
```

所有 ID 来自经验证的 owner/context，UUID 是固定 16-byte 表示；所有整数使用 big-endian。`resource_kind` 只允许小写 ASCII 字母、数字及 `._-`，首尾必须是字母或数字。`base_revision` 可为 0；checkpoint 与 retention revision 必须非零。不存在的 Worktree 只编码单字节 tag `0`，存在时编码 tag `1` 后跟 UUID；未知 tag 不属于当前 schema。最大 AAD 长度可由字段宽度推导：无 Worktree 为 201 bytes，有 Worktree 且 kind 为 64 bytes 时为 217 bytes。不得使用客户端 JSON 原串或字符串拼接冒充 canonical AAD。跨 tenant/Project/Run/Worktree/resource/revision 替换记录必须认证失败。

### 3.2 Envelope 逻辑字段

`ProtectedTranscriptBody` 的版本化编码至少包含：

| 字段 | 规则 |
|---|---|
| `format_version` / `aad_schema_version` | 不可省略；未知版本 fail closed。当前 core 只接受 envelope format 1 / AAD schema 1。 |
| `provider_profile_id` / `profile_revision` / `profile_digest` | 在第一次持久化时固定，不随运行配置变化。当前代码由 `wrapped_dek.profile` 携带该固定 profile identity。 |
| `key_ref` / `key_version` | opaque、可恢复、可审计；不得含凭证。 |
| `wrapped_dek` | 仅 provider 可 unwrap 的密钥封装；禁止 DEK 明文持久化。 |
| `aead_suite` / `nonce` / `ciphertext_and_tag` | suite 明确版本；当前 core 为 AES-256-GCM、96-bit nonce、128-bit tag；tag 校验失败拒绝。 |
| `ciphertext_digest` | 目标持久化 envelope 的可选传输/对象完整性字段；W16-C core 尚未实现该字段，不产生明文摘要或可识别内容标记。 |
| `resource/payload reference` | 从 owner 的授权记录解析；客户端不能通过替换 reference 越权读回。 |

逻辑结构不冻结物理 SQL/对象存储表。若拆 envelope manifest 或 blob table，须依 basic design §3 分类并落实 per-table RLS/retention/audit。

### 3.3 保护及读回序列

1. Auth extractor 验证 JWT 和 active device generation；owner 从可信 context 重新解析 Project/Run/Worktree scope、actor 权限、expected revision 与 retention policy revision。
2. 前端只在有限 pending delta 中 coalesce；达到 profile 的 checkpoint 条件后提交单个幂等 operation id。禁止逐字符写出 immutable snapshot 或逐键调用 KMS。
3. Rust protector 取固定 `KmsProfile`，通过短期最小权限身份请求 DEK wrap/generate，使用本地 AEAD 对正文加密；DEK/明文缓冲进入有限生命周期区，正常/错误/取消路径均尽最大努力清零。W16-C branch 的 `TranscriptEnvelopeProtector` 已实现 core API：`protect(context, aad, plaintext)` 调用固定 `KmsProviderClient::generate_data_key`，`decrypt(context, aad, envelope)` 调用同 profile 的 `unwrap_data_key` 后再做认证解密。该切片不构成 owner/REST/DB 接入或生产 `TranscriptBodyProtector`。
4. 协作 owner 在自己的事务中写密文元数据、draft head/checkpoint、审计和幂等 receipt；只有完整密文 digest、tag 及 retention 通过验证后才可参与冲突判定。owner Outbox 只带 resource/revision/event/conflict id 等 metadata。
5. 读回先重验 actor/device/Run/resource 权限与 retention，再用记录固定的 provider/profile/key version、同一 AAD unwrap 和 AEAD 解密。撤权检查先于 KMS call；所有保护结果只在内存中短暂供授权响应使用。

### 3.4 当前 core 的有界编码与未实现保证

- 单条明文最大 256 KiB；密文含 16-byte GCM tag，最大 256 KiB + 16 bytes。wrapped DEK 最大 64 KiB；解码前拒绝超过 1 MiB 的 JSON envelope，并拒绝 envelope 外层未知字段、版本、suite、nonce 长度及超限密文。
- binary AAD 最大 217 bytes。JSON 序列化/解码和 AEAD 目前是完整对象操作而非 streaming；上述上限约束单对象及单次缓冲规模，不证明并发峰值 RSS、系统队列、延迟或可取消性已达标。
- nonce 使用 rand 0.10 `rand::rng()` 提供的 `ThreadRng`；它由 `SysRng/getrandom` 自动 seed，且支持周期 reseed。该实现不是逐次直接调用 OS RNG。目标 workload 的熵源可用性与安全配置仍须部署验证。
- provider 错误仅转换为 `KmsProviderErrorKind`，对外错误不携带 provider 原始文字。`TranscriptEnvelope` 的 Debug 实现不输出 nonce/ciphertext；envelope 只序列化 wrapped key 元数据与密文。
- `SecretDataKey` 当前仅 best-effort 清零；本 lane 未启用 audited `zeroize` feature，且不保证 AES 内部副本被清除。不得宣称可靠零化。无生产 provider、profile resolver、持久化、REST/RLS、跨重启恢复、key rotation/backup restore 或 `TranscriptBodyProtector` adapter；这些仍 OPEN/NOT_RUN。

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

**当前实现状态（2026-10-30）**：W16-A 隔离 PostgreSQL 18 acceptance run 6af04e5a765446c8b3baaffff126bd52 完成 70/70 步骤，包含 device repository PG tests 2/2、迁移与 owner/ACL/RLS、设备 generation FOR SHARE 与拒绝 UPDATE、refresh-token tenant+actor 隔离及配对 CHECK 弱约束升级/非法值回滚。OAuth repository 的独立 disposable PostgreSQL 18.6 run 1b4673269a04435daa1fab5c93333133 通过 3/3 named tests，但以 disposable superuser 和 test-only migration prerequisites 运行，不证明 production grants/RLS、audit/SCD2 或 OAuth HTTP flow。Run/Worktree binding HTTP 测试也通过，但不代表 Canvas device-auth 请求面验收。当前 dev 基线 622b099f5e56e0edf2d4a4637bf0bd4f3d9401ba 上的 Canvas SSE 活动流撤销用例 run 6d23b8ceb1f0499091885c42271c2fb0 通过：41/41 runner steps exit 0；Rust 1.99.0 focused build exit 0；exact test 1 passed/0 failed/0 ignored/3 filtered，0.16s。它使用 disposable PostgreSQL 18.6 IPv4 loopback cluster 的 postgres superuser、test-only no-op audit/SCD2 prerequisites 和 unsigned operator build attestation，且未应用 replay-offset migration；因此只证明本地单流撤销关闭。尚未覆盖目录 row lock 与并发撤销争用、Canvas 共享 read/write/resolve、多实例失效/恢复、生产 audit/SCD2、Neon/Multipass/K3s、KMS 持久恢复或性能。

## 6. 验证映射与实施门

| Gate | 必须证明 |
|---|---|
| Device/Auth | forged body/header identity 被拒绝；已撤销设备对新 API 与活跃 stream 均失效；并发 auth-code/refresh 消费恰好一个胜者。 |
| Envelope | W16-C `domain-kms --lib` unit tests 21/21 PASS，覆盖同 profile 往返、canonical AAD、AAD/ciphertext/tag 篡改、版本/长度上限、profile mismatch 与安全 provider error；重启/第二进程解密、生产 provider、持久化、RLS、key rotation 与 backup restore 仍 OPEN/NOT_RUN。 |
| KMS lifecycle | unavailable/permission revoked/unknown profile fail closed；key rotation 前后历史读取；database/blob/provider backup restore；有引用时 key deletion 被阻止。 |
| Owner/RLS | Neon 或真实 PostgreSQL 的 RLS、owner transaction、idempotency、retention 与只含 metadata 的 Outbox；两 actor 互读 scope 被拒绝。 |
| Bounded runtime | 固定 Collaboration Profile 与 workload 下 KMS 请求频次、缓冲/队列上限、RSS/CPU、取消、慢消费者与 PT-09 p99 结果。 |

测试环境最多两个 Multipass inventory 实例；这只支持可重建验证，不证明两节点 HA。Vercel 路径使用 Neon 并单独核验云身份。Canvas 单个 SSE 撤销关闭用例已有本地 PG18.6 证据，但生产 provider、Canvas 共享读写/resolve、多实例恢复、并发竞争、持久恢复和 profile 性能证据仍缺；W16、WLD-7、PT-09/10 继续保持 OPEN/PARTIAL/NOT_RUN。

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 定义认证设备链、原子 OAuth 凭据消费、canonical AAD、envelope 字段、撤销、轮换/恢复、错误语义及资源有界验证门；未批准物理 DDL/provider。 | W16 device 与 KMS 审计。 |
| v0.2 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 在旧 AAD 字段基础上显式加入 optional Worktree（位于 Run 之后），固定二进制编码/tag/端序与长度；记录 W16-C core API、边界和未验证的生产/清零/持久化能力。 | W16-C envelope core 实施与 DD 对照。 |
| v0.3 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 W16-A 已通过的本地 PG18 owner/RLS、设备 repository 和 Run/Worktree binding HTTP 切片，并列出未覆盖的并发撤销、OAuth 单赢家、Canvas 设备请求面、生产 SCD2/KMS、云/VM 部署与性能验收。 | W16-A PG18 acceptance 70/70 步骤完成。 |
| v0.4 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 repository OAuth PG18.6 3/3 slice 的准确范围与生产授权限制；标记 Canvas SSE candidate compile 为资源中断、未运行 DB/未入 dev；保持 E2E 与 W16-A 开放。 | W16-A OAuth acceptance 和 Canvas compile 状态复核。 |
| v0.5 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步当前 dev 基线 Canvas SSE 设备撤销单用例 PG18.6 acceptance（run 6d23b8ceb1f0499091885c42271c2fb0）与 Rust 1.99.0 focused build，保留其本地 superuser/test-only/unsigned 限制；生产授权、HTTP audit、恢复、多实例、性能和部署门继续 OPEN。 | 当前 dev focused acceptance 与 DD 实现状态复核。
