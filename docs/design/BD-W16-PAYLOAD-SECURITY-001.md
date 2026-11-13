# BD-W16-PAYLOAD-SECURITY-001 — 协作载荷安全基本设计

> 状态：Draft v0.9；补充 BD-CANVAS-AGENT-001 v0.2 的 W16 安全边界，不取代其 Run/owner 结构。此次补充冻结设备 generation/source sequence、版本化 snapshot/replay、服务授权、lease、Collaboration 连续 checkpoint 及各 owner 的物理记录分类，属于目标设计，尚无对应实现或完整验收。W16-A 的部分设备身份与 PostgreSQL owner/RLS 切片已验收，OAuth repository 本地 PG18.6 slice 3/3 通过；Canvas 活跃 SSE 流撤销关闭单用例已在候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac 上通过（run 49b512fbfe334f0e98e46fe7eae7ea39），生产协作链仍未闭合。W16-C 只有 provider-neutral envelope core。

## 1. 目标与边界

沿用 Project → Engineering Run → Worktree → 本地 Branch。Identity/Auth owner 唯一负责 actor/tenant/设备凭据、generation/status、revoke、refresh-family 变化及相应审计；event contract READY 后授权 generation 状态变化与 Identity/Auth 自己的 append-only Outbox 在同一事务提交。READY 前的变化只构成 cutover baseline，不补造历史事件。Collaboration owner 负责协作正文/版本、资源授权与 retention，并在自己的事务中认证接收撤销事件、写 durable Inbox/幂等回执及仅用于协作授权失效的本地投影；它可为自己产生的协作事实写 metadata-only Outbox。两个 owner 不共享事务、不跨 owner 写表；Outbox/Inbox 的传输可替换，业务契约不依赖 broker、通知或 SSE 的确认语义。每次业务请求仍实时复验当前身份和权限，事件只加速失效。Agent、插件和任务卡 CLI 不接触 KMS 凭证或宿主管理接口。

本文件是 `SRS-W16-PAYLOAD-SECURITY-001` 的基本设计；Provider 候选及其来源证据见 W16 provider research。尚未批准任何生产 provider。

## 2. 身份与撤销设计

- Enrollment 由已认证 actor 发起；服务端为设备分配稳定 opaque `device_id`，并验证设备凭据的持有证明后才激活绑定。client body/header 只提供 challenge/proof，不提供可信 actor、tenant、scope 或设备作者身份。
- 令牌由 issuer 绑定 `actor_id / tenant_id / device_id / device_generation / client_id / scope`。服务主体、Agent Runtime 与用户设备采用不同 principal kind，不以 nil UUID 或个人设备 claim 代替服务身份。
- 每个受保护读写与订阅均核验当前设备 generation/status。event contract READY 后，每次授权相关 generation 推进都必须由 Identity/Auth owner 在同一事务写入恰好一个相同 `source_sequence` 的 metadata-only Outbox 事件；READY 前状态推进纳入 baseline。当前 device stream 使用 `stream_id=device_id` 与 `source_sequence=device_generation`；READY 后新 binding activation event 为 generation 1，重复撤销不推进。普通 refresh-token 轮换而设备 generation 不变时不产生设备授权事件。撤销产生 append-only audit 与失效事件，清理该设备 refresh-session；现有 SSE 被关闭。事件暂时丢失时只允许有界授权租约，未知状态不可视为有效。
- OAuth authorization-code 和 refresh-token 单次消费采用持久化原子条件更新并返回唯一胜者；消费事实与令牌签发/轮换处于其所属 auth owner 的事务边界。

### 2.1 Owner-local 撤销事件与恢复

- Identity/Auth owner 在同一事务中推进设备 generation/status、撤销 refresh family、写适用审计事实并追加设备授权 Outbox event；READY 前的 generation 变化不发事件，作为 cutover baseline。普通 refresh-token 轮换不产生设备授权事件。READY 后新 binding 的 generation 1 activation event 与 floor 0 同事务提交；READY 前创建的 binding（包括 enrollment）保持 NULL floor，post-READY 首次 snapshot/授权变更/backfill 在行锁下将当前 generation G 作为 baseline，不补造 G 之前事件。READY 后所有授权相关 generation 转换一对一对应同序号事件。存量绑定初始化只允许在无既存事件历史时进行。READY 前禁止任何 floor 初始化/backfill；事件 API 与 consumer 保持关闭。`identity_device_authorization_event_contract` 以受 RLS 保护的 Master readiness 记录保存单调状态；切至 READY 前须部署兼容 writer、drain 所有旧 writer 及活动 DB session、撤销旧凭据并验证其不能重连。READY 后每个 generation 写路径都读取并锁住同一 readiness 记录，再按统一顺序锁 binding 行；未知/null floor、旧 writer、锁/Outbox 失败一律 fail closed。floor 未初始化但已有设备事件属于完整性异常，不得以 snapshot 覆盖。snapshot 初始化、生产者变更和 backfill 必须序列化于同一 binding 行；Collaboration 不读取/写入 Identity 表，而由授权消息接口接收事件，在自己的事务内写 Inbox、幂等回执和撤权投影；如需通知下游，只能追加自己的 metadata-only Outbox。
- 事件至少一次投递。Inbox 以 `(tenant_id, producer_id, stream_id, source_sequence)` 业务键和 `(tenant_id, event_id)` tenant-scoped event 键分别唯一；两键必须解析到同一已存事件，且 canonical digest 相同，才可作为 exact replay 只返回既有回执。若业务键已存在但 event_id 不同、event_id 已存在但业务键不同、双键映射到不同记录，或同一双键记录的 digest 不同，均须进入 durable quarantine 并告警，禁止投影和正常业务 Outbox 副作用。quarantine 记录连同待发送的告警意图先在 Collaboration owner 本地事务提交，之后才 ACK poison message；隔离路径不写授权投影或正常业务 Outbox。quarantine 存储失败时不 ACK，按版本化 profile 的有界 retry/backoff 重试；预算耗尽后暂停该 consumer 分区/流并施加背压，告警运维，不推进 cursor。正常接收仍须先提交 Inbox、回执、投影和自有 metadata-only Outbox 再 ACK/checkpoint。不得宣称跨 owner exactly-once。
- 并发插入由上述两个 tenant-scoped unique constraint 仲裁。遇到唯一冲突时，使用事务安全的冲突处理/新鲜一致性读取，在已认证的 tenant RLS scope 中分别按业务键和 event 键读取，并对两条结果执行同记录+同 digest 的分类；不同记录映射、单键命中或 digest 不同均 quarantine。tenant 身份必须来自认证的 producer/transport context；请求载荷 tenant 不匹配时在任何去重查询前统一拒绝。所有查询受 tenant RLS 限制，外部确认/错误不区分“其他租户已有记录”与其他不可授权状态，避免 tenant 存在性侧信道。
- 规范事件只包含 `event_id`、`event_type`、`schema_version`、opaque tenant/actor/device identity、producer/stream/source sequence、新 generation/status、occurred time、correlation/idempotency identity 与 canonical metadata digest。不得包含 token、私钥/证明、协作正文、明文 DEK 或敏感请求 body；未知 schema 不应用。
- source sequence 是业务事件顺序；传输 cursor 是 adapter 不透明恢复位置。首次 claim 将 cursor 固定到 provider/version/profile digest/endpoint identity，重试沿用该 profile；provider 迁移须 drain 旧 profile 后显式完成，不跨 provider 比较 cursor 或复用去重窗口。
- Identity/Auth 提供版本化 `DeviceAuthorizationSnapshotV1` 与 `DeviceAuthorizationReplayPageV1` 应用接口；snapshot 返回当前 status、generation、`through_source_sequence` 与 replay floor。若 binding floor 尚未初始化，snapshot 在 owner 事务中锁定绑定行，将 floor 设为当前 generation，再从该一致性点读取并返回状态、水位和 floor；已初始化时也须保证三者来自同一一致性点。Replay 使用 `after_source_sequence` 按升序返回受 max-row/max-byte profile 限制的页面，并带 `next_after_source_sequence`、当前 high-water mark 与 `reset_required`。请求的 tenant scope 由可信服务身份上下文提供，payload/path 中的 tenant 仅用于一致性断言，不控制授权或 RLS。
- API caller 必须由选定且经审查的认证 adapter 验证为 Star 内部服务身份，并持有 `identity.device_authorization.read` capability/audience；服务身份映射与 capability 由版本化服务身份 profile 给出。mTLS 可作为 transport 方案，但当前不假设已有实现或可复用代码。Agent、Plugin、任务 CLI 和普通用户 principal 不可直接调用。跨 tenant/missing capability 在读取设备或 Outbox 前统一拒绝；对外不区分无资源与不可授权状态。
- 发现 `after_source_sequence` 低于 bootstrap/replay floor 时返回 `reset_required`，consumer 先取 snapshot，再从其水位之后 replay。激活后出现任何无法解释的 sequence gap 表示 owner 事务/Outbox 完整性故障：不得 ACK 或更新投影；创建受审计告警并暂停该 stream，直到权威修复。未知 schema、cursor/profile mismatch 或授权源不可用同样 fail closed。任何业务 API 仍重新查验当前 issuer 身份、设备 generation 和 Project/Run capability。
- Collaboration 持久化每个 `(tenant_id, producer_id, stream_id)` 的连续 `applied_source_sequence`；按 stream 单飞处理，跨 stream 以 profile 的固定并发度并行。收到 N+1 而 N 未提交时，先从权威 replay 补齐 N；不能在有界恢复预算内补齐则暂停该 stream，不推进水位。不得仅记录最大序号后跳过缺口。正常事件或 durable quarantine 在 Collaboration owner 事务提交后才 ACK。
- Identity/Auth 的版本化撤权策略定义 `max_revocation_freshness_lease`；Collaboration runtime profile 只能设置更短值，有效 lease=`min(identity_max, collaboration_max)`。lease 只约束缓存和长连接的撤权新鲜度，不构成业务授权；续租须通过权威身份状态核验，lease 到期且核验不可用时关闭 SSE 并 fail closed。事件实时唤醒保持主路径，lease 复核只用于安全上界，不代替协作内容轮询。具体时间阈值、页面大小、并发和重试值由 W14/profile 批准，批准前不启用消费者。

## 3. W/T/M 目标物理记录分类

| Owner / 目标关系名 | 主分类 | 生命周期与约束 |
|---|---|---|
| `oauth_device_bindings` / Identity/Auth | Master | 目标为 SCD Type 2；撤销不物理删除；tenant+actor RLS；凭据私钥不上传。READY 后新 binding 在 generation 1 activation event 前设 `authorization_event_replay_floor=0`；READY 前创建的 binding 保持 NULL，ready 后仅在无历史事件时由首次 snapshot/授权变更或可选 backfill 锁定当前 generation，此后不可降低/静默移动。当前 W16-A migration 只保留当前 binding 行并原位推进 generation/status，尚无历史版本表或有效期列，SCD Type 2 仍未实现。 |
| `identity_device_authorization_event_contract` / Identity/Auth | Master | 单例、版本化 readiness/contract record；状态仅从 NOT_READY 转为 READY；保存兼容 writer contract/release digest 与 cutover 审计引用。仅在旧 writer/session drain 且旧 DB credential 撤销后可 READY。非 tenant-bearing，但仍 FORCE RLS，策略只授予 Identity/Auth control-plane service identity；无 TTL。 |
| `oauth_refresh_tokens` / Identity/Auth | Work | 短期、可轮换/撤销的凭据记录；tenant+actor RLS，具有明确 expires-at 与获批清理 TTL；清理不得删除独立 append-only 撤销审计事实。普通 refresh-token 轮换若设备授权 generation 不变，不产生设备授权事件。 |
| `oauth_device_binding_audit` / Identity/Auth | Transaction | append-only、禁止 UPDATE/DELETE/TRUNCATE、审计必携、tenant RLS；包含 enrolled/revoked 事实。 |
| `oauth_device_authorization_outbox` / Identity/Auth | Transaction | append-only metadata-only 事件，不设自动 TTL；PK `(tenant_id,event_id)`、UNIQUE `(tenant_id,producer_id,stream_id,source_sequence)`，按 tenant/stream/sequence 有序读取；与 binding、refresh-family、audit 同事务。 |
| `oauth_device_authorization_delivery_work` / Identity/Auth relay | Work | mutable claim/lease/retry/route pin；部分索引待投递状态与 next-attempt time；失败次数、队列与并发受 profile 限制；终态 receipt 持久后按批准 TTL 清理。 |
| `oauth_device_authorization_delivery_receipt` / Identity/Auth relay | Transaction | terminal delivery fact append-only；记录固定 provider/version/profile digest/endpoint/log generation、opaque durable cursor、canonical wire digest 与结果，不保存 wire payload；tenant RLS。 |
| `collaboration_device_authorization_inbox` / Collaboration | Transaction | append-only receipt/dedupe；双 tenant-scoped UNIQUE 键 `(tenant,producer,stream,source_sequence)` 与 `(tenant,event_id)`；两键必须同一行+同 digest 才 exact replay；无自动 TTL。 |
| `collaboration_device_authorization_quarantine` / Collaboration | Transaction | append-only、安全 metadata 与 durable alert intent；tenant RLS，禁止正文/token；无自动 TTL。 |
| `collaboration_device_authorization_projection` / Collaboration | Work | 可重建授权失效投影；PK `(tenant_id,device_id)`，保存 status/generation/last-confirmed/lease expiry；到期即视为未知并 fail closed，过期后由受控 cleanup 按批准 TTL 清理。 |
| `collaboration_device_authorization_checkpoint` / Collaboration | Work | 每 tenant/producer/stream 保存 contiguous applied sequence 与首次 claim 固定的 provider/profile/endpoint/log generation/opaque cursor；只由 Collaboration owner 更新；明示 recovery TTL，过期后强制 snapshot bootstrap。 |
| Device enrollment challenge / Identity/Auth | Work | 有界 TTL、有限尝试次数、过期删除；不记录 proof secret。 |
| Protected draft checkpoint metadata / Collaboration | Work | 有明示 retention；正文密文以受控 blob/payload reference 保存；不可在主表放明文。 |
| Collaboration Outbox / Collaboration | Transaction | 仅保存该 owner 产生的 opaque resource/version/fact identity 与 digest；append-only，owner-local 同事务。 |
| Conflict / canonical revision / collaboration idempotency receipt / Collaboration | Transaction | append-only 事实或受控状态版本；只在 Collaboration owner 事务内原子写入、幂等、RLS。 |
| KMS profile 与可用 key identity 映射 | Master | 版本化、SCD Type 2、无 secret；密钥生命周期由 provider 管理。 |

以上是目标表/索引/授予草案，不是已创建 DDL。每个 tenant-bearing 表都必须 `ENABLE ROW LEVEL SECURITY` 与 `FORCE ROW LEVEL SECURITY`；global event-contract readiness 关系也强制 RLS，由已认证 `app.service_id` control-plane policy 限定。trusted adapter 从已认证调用上下文设置事务局部 `app.tenant_id` / `app.actor_id` / `app.service_id`，请求 payload 不可直接设置这些 scope。应用角色不得 `BYPASSRLS`。Identity owner 可追加自己的事件/审计但不可改删；relay 只按授权 tenant 读事件、更新 delivery Work；Collaboration consumer 只写读自身 Inbox/投影/checkpoint/quarantine，不能查询或写 Identity 表。T 表由 owner append-only grant；Work 表仅 owner runtime/有审计的 cleanup role 按 TTL 更新或删除；无跨 owner 数据库 grant。Outbox/Inbox/receipt/quarantine 的数据只留安全 metadata，T 记录不物理删除；通过分区和索引管理查询成本，不得以丢弃分区冒充归档。实际表名、索引、保留/冷归档、role/grant 与现有 SCD Type 2 差异须经 DDD/RLS review 后再写 migration。

## 4. 保护与持久化边界

1. owner 先从认证上下文取得 actor/device/tenant，再校验 Project/Run/Worktree 资源授权、expected revision 与当前 retention policy。
2. Rust `TranscriptBodyProtector` 对已 coalesce 的 checkpoint 生成/取得短期 DEK，用 versioned AEAD 加密正文，再由配置的 `KmsClient` wrap DEK。Project/Run/optional Worktree/resource/revision/schema 的 canonical identity 作为 AAD；只用经 owner 校验的值构造上下文。
3. owner 保存密文、wrapped DEK 与必要 envelope metadata；正文不进入 audit、metric、trace、Outbox、NOTIFY 或 SSE。保护和解密都要求相同的稳定 AAD version。
4. KMS 调用成功但 owner 事务失败时，不能产生业务成功回执；孤儿 envelope 可按幂等/引用对账审计并清理。客户端收到失败时只显示 `local_only / sync_pending`。
5. 解密前重新授权当前 actor/device/Run/resource 并检查 retention。撤权、密文篡改、provider 不可用、未识别 envelope 或旧 key 不可用均拒绝，不泄露明文错误详情。

### 4.1 W16-C 当前代码切片与界限

- `domain-kms::TranscriptEnvelopeProtector` 是 provider-neutral core；它只接受固定的 `KmsProfileIdentity` 与 `KmsProviderClient`，每次保护调用一次 `generate_data_key`，读回调用一次 `unwrap_data_key`，不在逐 envelope 路径触发 capability discovery。
- 当前切片使用 AES-256-GCM、96-bit nonce、默认 128-bit tag。nonce 由 rand 0.10 `rand::rng()` 提供的 `ThreadRng` 填充；该 RNG 通过 `SysRng/getrandom` seed 并支持自动 reseed，不能描述成每字节直接由 OS RNG 提供。
- 单对象上限为明文 256 KiB、wrapped DEK 64 KiB、JSON envelope 1 MiB。该固定上限限制单次编码/加密对象，但不能证明并发 RSS、队列、取消、进程内密钥清零或生产 workload 达标。
- `SecretDataKey` 当前仅有 best-effort 清除行为；没有启用经审计的 zeroize 依赖/feature。AES 实现内部临时副本也没有清零保证。生产启用保持阻断。
- 该类型尚未接入 `TranscriptBodyProtector`、REST、owner transaction、RLS、持久 KMS provider、profile resolver 或跨重启恢复；不得把 branch 级实现标为共享协作可用。

### 4.2 W16-A 已验证切片与边界

- 隔离本地 PostgreSQL 18 acceptance run `6af04e5a765446c8b3baaffff126bd52` 共 70 步、失败 0；覆盖 W16 migration 顺序、owner role/ACL、设备 generation `FOR SHARE` 与拒写、tenant+actor refresh-token RLS、NULL generation 约束及旧弱同名 CHECK 升级。
- Rust `device_auth_pg` 两项 PostgreSQL 测试通过，覆盖 proof/revoke repository 行为与并发 enrollment challenge 上限；独立 Run/Worktree binding HTTP 场景验证其自身的绑定、并发、回滚与 detach 路径。后者不证明 Canvas 设备身份端到端。
- OAuth repository 的隔离 PostgreSQL 18.6 runner `1b4673269a04435daa1fab5c93333133` 通过三个 named tests，覆盖 auth-code 并发单赢家和消费/令牌持久化，以及 refresh 并发单赢家、唯一 successor、旧 refresh 撤销和 stale-snapshot 拒绝。runner 使用 disposable superuser 与迁移所需 test-only placeholders，不证明生产应用角色 grants、RLS runtime、audit/SCD2 或 OAuth HTTP 流程；完整 E2E-COLLAB-14 仍 PARTIAL/OPEN。
- Canvas SSE 撤销用例 active_canvas_sse_closes_after_device_binding_revocation 在候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac 使用生产 group GET SSE router/hub、已提交 device binding 撤销与 PostgreSQL Outbox wake，经本地 PostgreSQL 18.6 disposable runner（run 49b512fbfe334f0e98e46fe7eae7ea39）通过：41/41 runner steps exit 0，exact test 1 passed/0 failed/0 ignored/3 filtered，0.17s。测试使用 postgres superuser 与 no-op audit/SCD2 prerequisites、unsigned build attestation；只证明单个活动流在设备撤销后关闭。
- 此切片未证明目录锁与并发 revoke 的阻塞关系、全体共享读写/订阅/resolve、多实例 SSE wake-up 与恢复、生产 audit/SCD2、Neon/Multipass/K3s 部署、KMS 持久化恢复或 PT-09/10。OAuth repository 并发单赢家仅有独立 disposable PG18.6 slice；生产 runtime grants/RLS 与 HTTP auth flow 未验收。Canvas SSE 单用例 PASS 不关闭上述门；相关 W16 门继续 OPEN/PARTIAL。

## 5. Provider 与部署策略

- 共同 provider-neutral 模型为 Star 生成 DEK、Star Rust 侧完成 AEAD，provider 负责保护/恢复 DEK；业务 owner 不依赖 provider subject、API 路径、token 格式或 cursor。
- 每份持久 envelope 固定 provider/profile ID、版本与 digest、opaque key identity、key version、envelope format 与 AAD schema。切换 profile 前须 drain 待处理操作并执行显式迁移。
- W16 研究将 OpenBao Transit 作为首个自托管候选，将 AWS KMS 作为可选云端适配器；两者均未批准生产。OpenBao MPL-2.0 不限制商业用途，但发布镜像、chart 和依赖许可闭包仍须核验；AWS KMS 是托管服务，不是自托管 OSS 方案。详见 provider research 及其来源。
- 测试阶段最多使用 2 个 Multipass inventory 实例；两节点 Raft 不能容忍任一节点故障，也不代表 HA。Neon/Vercel 与本地 Multipass/K3s 是两个独立验证拓扑；不以 Docker Desktop 为前提。

## 6. 资源与错误状态

每 draft 至多一个在途 checkpoint 写入，一个合并后的 bounded pending delta；KMS 并发、超时、队列、暂存明文、DEK、密文帧与 cache 均来自固定版本的 Collaboration Profile。KMS 不逐键调用，UI 不因 KMS 慢操作阻塞输入。取消、超时、OOM/队列超限与 provider 失效统一保持 canonical revision 不变，并报告可恢复的 local-only 状态。

验收必须覆盖 `device_unbound`、`device_revoked`、`credential_replayed`、`kms_unavailable`、`key_version_unknown`、`aad_mismatch`、`payload_integrity_failed`、`profile_mismatch`；对外错误不含 key、正文、token 或内部 provider secret。

## 7. 上位与下位追踪

| 需求 | 下位实现/验证 |
|---|---|
| SRS-W16-PAYLOAD-01..03 | DD §2；E2E-COLLAB-12/13/14 |
| SRS-W16-PAYLOAD-04..07 | DD §3–§5；E2E-COLLAB-15..19 |
| SRS-W16-PAYLOAD-08 | DD §6；PT-10 + PT-10-GATE-01 |
| SRS-W16-PAYLOAD-09..11 | DD §1、§2.3、§2.5；UT-W16-OUTBOX-01/INBOX-01/INBOX-02；IT-W16-OWNER-OUTBOX-01/INBOX-DELIVERY-01/INBOX-ISOLATION-01/INBOX-IDENTITY-01/QUARANTINE-ACK-01；E2E-COLLAB-20/21 |
| SRS-W16-PAYLOAD-12 | DD §2.5、§6；IT-W16-RECOVERY-01/AUTH-PROJECTION-01；E2E-COLLAB-22/23；PT-11 |
| SRS-W16-PAYLOAD-13 | DD §2.5、§6；PT-12 + PT-12-GATE-01 |
| SRS-W16-PAYLOAD-14 | DD §2.5.1–2.5.2；UT-W16-RECOVERY-API-01；IT-W16-SNAPSHOT-REPLAY-01/SERVICE-AUTH-01/CUTOVER-01；E2E-COLLAB-24 |
| SRS-W16-PAYLOAD-15 | DD §2.5.2–2.5.3；UT-W16-STREAM-01/CHECKPOINT-01/CHECKPOINT-02；IT-W16-CHECKPOINT-ORDER-01/CUTOVER-01/WTM-RLS-01；E2E-COLLAB-24 |

## 8. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增身份 owner、协作 owner、provider adapter、W/T/M 逻辑分类、密文边界、失败策略和最多两个 Multipass 测试实例约束。未定义或声称已实施 DDL/provider。 | W16 认证与 KMS 前置审计及 provider 研究。 |
| v0.2 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 纳入 W16-C 单对象有界 envelope core 的实际状态、AES-GCM/RNG 语义与未通过的生产/清零/并发/恢复限制；optional Worktree 纳入 AAD 上下文。 | W16-C envelope core 实施与源文档对照。 |
| v0.3 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 W16-A 的部分 PostgreSQL 18 owner/RLS 验收，并明确 Device binding 尚未实现 SCD Type 2；其它端到端撤销、OAuth 并发、生产 KMS、恢复与性能门继续开放。 | W16-A PG18 acceptance 及 SCD2 实现差异复核。 |
| v0.4 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 OAuth repository PG18.6 slice 3/3 的范围和临时 superuser/test-only prerequisite 边界，并注明 Canvas SSE candidate compile 因资源门中断、DB 未运行；完整端到端与生产授权验收保持开放。 | W16-A OAuth PG runner 结果和 Canvas candidate 资源门状态同步。 |
| v0.5 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步当前 dev 基线上 Canvas SSE 撤销单用例 PG18.6 PASS（run 6d23b8ceb1f0499091885c42271c2fb0）与精确 Rust 测试结果；保留 superuser、test-only no-op audit/SCD2 和 unsigned attestation 限制，生产授权、审计与完整协作链继续 OPEN。 | 当前 dev focused acceptance 与设计状态复核。
| v0.6 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将当前 Canvas SSE 撤销状态同步到最新可核验 acceptance run 49b512fbfe334f0e98e46fe7eae7ea39（候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac，已包含于 dev@0e977306a1aafb031557d28d7fd19795297dd027）；保持生产授权、审计、多实例恢复和完整协作链 OPEN。 | W16-A 规范文档 rebase 到当前 dev 并复核最新 acceptance 报告。 |
| v0.7 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 Identity/Auth 与 Collaboration 各自拥有 Outbox/Inbox/投影及事务边界；定义至少一次、提交后 ACK、事件身份与异摘要隔离、provider-bound cursor、有限恢复、当前权限复验、安全字段与资源 profile；新增测试追踪，保持未实现/NOT_RUN。 | W16 Outbox owner-contract review 与 SRS/DD/TD 一致性核对。 |
| v0.8 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 收紧双唯一键 exact replay 判据及并发冲突归类；规定两种 identity 错配与 digest mismatch durable quarantine、隔离告警意图先提交再 ACK、quarantine 写失败时有界不确认重试/暂停，以及 tenant RLS 下不泄漏存在性。新增对应 UT/IT 追踪，仍是设计且 NOT_RUN。 | 协调者独立审查指出重放 identity 映射、隔离 ACK 顺序和 quarantine 存储失败路径未明确。 |
| v0.9 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 冻结设备 generation/source sequence 一对一、READY 前状态纳入 baseline 且不补造事件、legacy writer/session drain 后才 READY 的版本化 readiness control、绑定行锁下的 lazy replay-floor 初始化及并发竞争验收、snapshot/replay API 与待验证的服务身份 adapter contract、lease 取值方、连续 checkpoint/缺口处理，并按物理 owner 表细分 W/T/M、索引、RLS、grant 与 TTL；纳入 refresh-token Work 关系与不产生设备授权事件的轮换边界；未新增实现或 migration。 | 实现前 owner-event 接口与持久化决策。 |
