# SRS-W16-PAYLOAD-SECURITY-001 — 协作载荷身份与持久加密要求

> 状态：Draft v0.9；这是 SRS-CANVAS-AGENT-001 v1.4 的 W16 补充需求，待后续无冲突时并回主 SRS。此次补充冻结设备 generation/source sequence、权威 snapshot/replay API、服务授权、租约、Collaboration 连续 checkpoint 与物理数据/RLS 分类；这些均为目标设计，尚非实现或部署验收。W16-A 的设备身份源码、隔离 PostgreSQL owner/RLS 与 OAuth repository 切片已部分验收；Canvas 活跃 SSE 流撤销关闭用例对应的 PostgreSQL 18.6 acceptance run 49b512fbfe334f0e98e46fe7eae7ea39 在候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac 上通过。该结果只关闭该单用例，生产协作闭环仍未验收；W16-C 只有有界 provider-neutral envelope core。

## 0. 目的与范围

本文件为多人协作草稿、冲突内容和会话载荷补齐可信设备身份、持久密钥、密文完整性、恢复与资源边界。它继承 AC-COLLAB-02/04/06/09 的 owner-local、Run 授权、双边草稿和显式 Resolve/CAS 约束；不改变各业务 owner 的数据所有权，也不允许客户端自报身份成为授权事实。

## 1. 规范性要求

| ID | 要求 |
|---|---|
| SRS-W16-PAYLOAD-01 | 共享写入必须绑定由认证签发方确认且当前有效的 `actor_id`、`tenant_id`、`device_id` 与设备凭据代次。客户端 body/header 中的 `device_id` 只能被拒绝或忽略，不能决定作者、RLS scope、冲突参与者或通知接收者。缺少可信设备身份时保持 `local_only / sync_pending`。 |
| SRS-W16-PAYLOAD-02 | 设备撤销后，后续读取、写入、订阅和 resolve 必须拒绝；已连接的事件流须收到撤销失效并关闭，或在有界授权租约到期前重新核验。撤销状态未知、授权目录不可用或代次不匹配时 fail closed。 |
| SRS-W16-PAYLOAD-03 | 授权码和 refresh token 的单次兑换必须原子化。并发重放时最多一个调用成功签发新令牌；其余返回认证失败，不得仅依赖“先读、后更新”的两步逻辑。身份、设备、tenant、client、scope 绑定必须在刷新轮换后保持。 |
| SRS-W16-PAYLOAD-04 | 任何持久化协作正文须由 Rust owner 服务通过版本化 envelope-encryption 保护；密文完整性和 canonical tenant/Project/Run/optional Worktree/resource/revision 上下文须被认证。数据库、备份、审计、日志、指标、Outbox、PostgreSQL NOTIFY 与 SSE wake payload 不得包含正文或明文 DEK。 |
| SRS-W16-PAYLOAD-05 | 每份密文必须固定可恢复的 provider/profile identity、版本/digest、opaque key identity、envelope format/AAD schema 版本、wrapped DEK、AEAD suite 与 nonce。只保留足够的密文元数据；不得持久化明文 DEK、长期 KMS 凭证或未保护的正文摘要。当前 envelope core 使用 AES-256-GCM、96-bit nonce、128-bit tag。 |
| SRS-W16-PAYLOAD-06 | KMS/provider 不可用、密钥版本未知/已禁用、身份撤销、AAD 不匹配、密文/tag/digest 篡改或解密失败时，不得回退至 `LocalMockKms`、明文、进程内密钥或内存队列。正文只能留在受控本地草稿并标记未同步，不能呈现为已共享。 |
| SRS-W16-PAYLOAD-07 | 轮换、重启和备份恢复必须保留仍在 retention 与备份窗口内的合法读取能力；旧密钥只能在所有引用与保留备份迁移/到期并完成审计后销毁。跨 provider/profile 切换须显式迁移，不得将旧 provider 的 opaque wrapped bytes 交给新 provider 解包。 |
| SRS-W16-PAYLOAD-08 | 保护过程必须有界且可取消。客户端编辑不逐键调用 KMS；owner 仅对已 coalesce 的 checkpoint 执行密钥操作。Envelope core 当前单条正文上限为 256 KiB、wrapped DEK 上限为 64 KiB、JSON envelope 上限为 1 MiB；这些单对象界限不证明进程 RSS、并发、队列或取消预算已通过。指标只记录脱敏的操作/耗时/字节范围。 |
| SRS-W16-PAYLOAD-09 | Identity/Auth owner 是设备 generation/status/revoke、refresh-family 变化与撤销审计的唯一事实 owner；event-contract readiness 为 READY 后，每个授权相关 generation 转换必须与恰好一个该 owner 的 append-only Outbox 事件在同一 owner 事务中提交或回滚。READY 前的状态推进属于事件流 cutover baseline，不伪造历史事件；Collaboration event consumer 保持关闭。Collaboration owner 只能在自己的事务中持久化 Inbox、幂等回执和用于协作授权失效的本地投影；不得跨 owner 写表或共享事务。 |
| SRS-W16-PAYLOAD-10 | 撤销事件必须包含稳定 `event_id`、`event_type`、`schema_version`、opaque `tenant_id/actor_id/device_id`、`producer_id`、`stream_id`、owner-local `source_sequence`、新 generation/status、时间、correlation/idempotency identity 与 canonical metadata digest。READY 后设备授权 stream 固定 `stream_id=device_id` 且 `source_sequence=device_generation`：READY 后新设备 activation event 从 generation 1 开始；撤销及任何授权相关 generation 变更各发且只发一个对应事件；幂等重复撤销不生成新序号。READY 前已存在的状态由首次 floor snapshot 纳入 baseline，不补造历史 Outbox。普通 refresh-token 轮换若设备授权 generation 未变化，不生成设备授权事件。事件和相邻日志不得包含访问/刷新 token、私钥/证明、协作正文、明文 DEK 或敏感请求体；未知 schema fail closed。 |
| SRS-W16-PAYLOAD-11 | 跨 owner 传递语义为至少一次。规范重放身份同时由 `(tenant_id, producer_id, stream_id, source_sequence)` 与 `(tenant_id, event_id)` 构成；两键必须解析到同一已存事件且 canonical digest 相同，才是 exact replay。任一键指向不同事件、同一业务键携带不同 event_id、同一 tenant-scoped event_id 携带不同业务键，或双键匹配但 digest 不同，均须持久隔离并告警，不得应用授权投影或产生正常业务 Outbox 副作用。正常 Inbox/投影/回执/自有 Outbox 事务提交后才 ACK；隔离消息须先提交 quarantine 记录（包含可重试的告警意图）再 ACK。quarantine 持久化失败时不得 ACK；按有界策略重试，耗尽后暂停/背压并告警。并发唯一约束冲突须在认证 tenant scope 内重新读取两键并分类，不得泄漏其他 tenant 的存在或数据。不得宣称跨服务 exactly-once。 |
| SRS-W16-PAYLOAD-12 | 领域 `source_sequence` 与传输 cursor 必须分开；opaque cursor 固定绑定首次 claim 时的 provider/version/profile digest/endpoint identity，所有重试沿用该身份。Identity/Auth 必须提供版本化权威 snapshot 与有界 replay；snapshot 的状态和 `through_source_sequence` 必须来自同一一致性点。检测序列缺口、cursor 过期、恢复水位不可用或授权源不可用时，停止使用可能过期的授权投影并 fail closed；事件只加速撤权失效，每次业务请求仍须实时复验当前身份、generation 与权限。 |
| SRS-W16-PAYLOAD-13 | Outbox/Inbox、重试、批量、在途消息、保留期限、事件尺寸与恢复工作必须受版本化资源 profile 限制，并具备背压、取消、幂等恢复与安全日志约束。审计事件、Inbox 去重事实与 quarantine 为 append-only、无自动 TTL；可变投递/lease、checkpoint 与撤权投影须有显式 Work retention。profile/预算缺失或边界未知时不得启动消费者或宣称可用。具体资源阈值由后续 profile/W14 批准，不在本需求中臆造。 |
| SRS-W16-PAYLOAD-14 | 权威 API 以版本化 `DeviceAuthorizationSnapshotV1` 和 `DeviceAuthorizationReplayPageV1` 暴露最小撤权 metadata。调用者必须是通过受信服务身份认证且具备 `identity.device_authorization.read` capability 的 Star 服务；tenant scope 来自认证上下文，不能由 Agent、插件、任务 CLI 或请求 payload 决定。版本化 Identity/Auth event-contract readiness 只有在不兼容 writer 与其 DB session 已 drain 且旧凭据已撤销后才可 READY；READY 前 API/consumer 与 floor 初始化关闭。READY 后新绑定的 `replay_floor=0`；READY 前创建的 binding（包括 enrollment）保持 NULL，由首个 post-READY snapshot、授权 generation 变更或可选 backfill 在绑定行锁下以当时 generation G 初始化，且仅在无既存设备事件历史时允许。floor 缺失但发现已有事件事实属于完整性异常，必须 fail closed，不得用 snapshot 掩盖。snapshot 同时返回状态、水位和该 floor。分页按 `after_source_sequence` 有界排序；低于 floor 返回 `reset_required`，不得猜测或静默越过历史缺口；未知事件类型、租户不匹配与越权请求不得泄漏资源是否存在。 |
| SRS-W16-PAYLOAD-15 | Collaboration owner 独占其 Inbox、授权投影和 consumer checkpoint。`applied_source_sequence` 必须是每个 tenant/device stream 的连续已应用水位；消费按 stream 串行，缺口先执行有界 replay，不得以 `MAX(sequence)` 跳过缺失事件。snapshot 水位推进后收到合法且 `replay_floor < source_sequence <= watermark` 的延迟事件时，必须以 exact replay 或提交 `superseded_by_snapshot` Inbox receipt 后确认，不改变投影或降低水位；键冲突/异摘要仍 quarantine。Inbox/投影/checkpoint 在 owner 事务中提交后才 ACK；恢复期间不以旧投影授权。撤权 freshness lease 的最长值由 Identity/Auth 的版本化策略给出，Collaboration profile 只能缩短，不能延长；lease 过期且无法重新核验时关闭 SSE 并 fail closed。 |

## 2. 验收映射与状态

| Requirement | 目标证据 | 当前状态 |
|---|---|---|
| 01–02 | E2E-COLLAB-12/13；身份绑定、共享读写、订阅与 SSE 撤销失效 | Canvas 活跃 SSE 流撤销关闭的单项 PostgreSQL 18.6 acceptance 已在候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac 上通过（run 49b512fbfe334f0e98e46fe7eae7ea39）；该 HEAD 已包含于当前 dev@0e977306a1aafb031557d28d7fd19795297dd027；设备身份完整链、伪造客户端身份拒绝、Canvas 共享读写、订阅/resolve、生产 grants/RLS 与恢复仍未验收，需求整体 PARTIAL / OPEN |
| 03 | E2E-COLLAB-14；数据库并发单次消费测试 | OAuth repository 的本地 disposable PG18.6 slice 通过 3/3 named tests；使用临时 superuser 和 test-only prerequisites。完整 E2E-COLLAB-14、生产角色/RLS 与 HTTP auth flow 仍 PARTIAL / OPEN |
| 04–06 | W16-C envelope-core focused unit tests；E2E-COLLAB-15/16/17、真实 provider 故障与 DB/RLS 测试 | domain-kms lib tests 21/21 PASS（lane 专属 Cargo target）；未接 owner/REST/DB，生产 provider、密文持久化与恢复仍 OPEN / NOT_RUN |
| 07 | E2E-COLLAB-18/19；恢复与轮换测试 | 未实现 / NOT_RUN |
| 08 | PT-10 与 PT-09-GATE-01；固定 workload/profile 的内存、CPU、队列和 KMS 调用证据 | 未实测 / NOT_RUN |
| 09–11 | UT-W16-OUTBOX-01/INBOX-01/INBOX-02；IT-W16-OWNER-OUTBOX-01、IT-W16-INBOX-DELIVERY-01、IT-W16-INBOX-ISOLATION-01、IT-W16-INBOX-IDENTITY-01、IT-W16-QUARANTINE-ACK-01；E2E-COLLAB-20/21 | 仅设计；Owner 原子提交/回滚、双身份键一致的精确重放、两种键错配/异摘要隔离、隔离记录先于 ACK、并发归类与跨租户不泄漏均 OPEN / NOT_RUN |
| 12 | IT-W16-RECOVERY-01、IT-W16-AUTH-PROJECTION-01；E2E-COLLAB-22/23；PT-11 | 仅设计；gap/cursor/retention 恢复、跨 provider 身份隔离、当前授权复验与多实例恢复 OPEN / NOT_RUN |
| 13 | PT-12 与 PT-12-GATE-01；固定 owner/profile/workload 的队列、大小、延迟、RSS、CPU、取消和保留窗口证据 | 仅设计；预算未批准，未实现 / NOT_RUN |
| 14 | UT-W16-RECOVERY-API-01、IT-W16-SNAPSHOT-REPLAY-01/SERVICE-AUTH-01/CUTOVER-01；E2E-COLLAB-24 | 仅设计；认证服务身份、capability、tenant scope、legacy-writer preflight、snapshot 原子水位、无历史事件前提下的绑定行锁 replay-floor 初始化与有限 replay OPEN / NOT_RUN |
| 15 | UT-W16-STREAM-01/CHECKPOINT-01/CHECKPOINT-02、IT-W16-CHECKPOINT-ORDER-01/CUTOVER-01/WTM-RLS-01；E2E-COLLAB-24 | 仅设计；generation/source sequence 一对一、连续消费水位、旧事件 durable receipt 且不回退、回填竞争无丢事件、owner grants/RLS 与 Work TTL OPEN / NOT_RUN |

W16-A 隔离本地 PostgreSQL 18 runner 于 2026-10-29 完成 70/70 步骤（run 6af04e5a765446c8b3baaffff126bd52），其中设备 repository PG 测试 2/2 通过，并验证迁移、owner/RLS、设备 generation FOR SHARE/拒写、refresh-token tenant+actor 可见性与约束升级/回滚。OAuth repository runner（run 1b4673269a04435daa1fab5c93333133）在 disposable PostgreSQL 18.6 上通过 3/3 named tests，但其 superuser/test-only prerequisites 不证明生产 grants、RLS runtime、audit/SCD2 或 HTTP authorization flow。候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac 上的 Canvas SSE 撤销 acceptance（run 49b512fbfe334f0e98e46fe7eae7ea39；该 HEAD 已包含于当前 dev@0e977306a1aafb031557d28d7fd19795297dd027）通过 41/41 runner steps；Rust 1.99.0 focused build exit 0，精确测试 active_canvas_sse_closes_after_device_binding_revocation 为 1 passed/0 failed/0 ignored/3 filtered，0.17s。该用例使用全新 PostgreSQL 18.6 IPv4 loopback disposable cluster、postgres superuser 与 test-only no-op audit/SCD2 prerequisites；build attestation unsigned，无签名/CI proof，未应用 replay-offset migration。它只证明单个活动 SSE 流在已提交设备撤销后关闭，不证明生产 grants/RLS、HTTP revoke/audit、跨实例恢复或完整 E2E。此前的内存门中断是历史尝试，不是本次验收状态。以上局部结果不关闭 E2E-COLLAB-12..14 或 W16；其余要求仍须在目标 PostgreSQL/RLS 与实际 provider profile 下验收。测试环境使用 Neon（Vercel 路径）或 Multipass 管理的 K3s（本地路径），Multipass 实例总数最多 2；不要求 Docker Desktop。

## 3. 已知缺口

- 现有源码已加入认证设备上下文、enrollment/proof/revoke 端点与 PostgreSQL DeviceBindingRepository，并在 Run/Worktree 写事务复验当前设备 generation；设备 repository、选定 owner/RLS、OAuth repository 并发消费已有局部 PostgreSQL acceptance。Canvas 单个已连接 SSE 流在已提交设备撤销后的关闭用例已于当前 dev 基线通过（run 49b512fbfe334f0e98e46fe7eae7ea39）；这不覆盖共享读写/resolve、生产身份与 grants/RLS、HTTP revoke/audit、多实例恢复或完整 E2E。详见既有设备报告、Canvas SSE acceptance 报告与 PHASE-W16-A-OAUTH-PG-ACCEPTANCE-20261029.md。
- W16-C lane 加入 `TranscriptEnvelopeProtector` 与单元测试，但尚未接入 `TranscriptBodyProtector`、业务 owner 或生产请求路径；`LocalMockKms` 不能跨重启恢复，生产 KMS、RLS、持久化、跨重启解密仍未实现/NOT_RUN。详见 `W16-KMS-SOURCE-AUDIT-20261028.md` 与 W16-C 阶段报告。
- Provider 研究仅形成候选建议；完整依赖/镜像/部署闭包、密钥恢复演练、运行预算和生产配置尚未批准。详见 `W16-KMS-PROVIDER-RESEARCH-20261028.md`。
- Outbox/Inbox owner-local 撤销传播、版本化 snapshot/replay API、服务身份/capability 接入、连续 consumer checkpoint、持久去重/隔离、ACK 顺序、跨实例恢复、provider-bound cursor 与资源 profile 目前只有规范设计；未新增实现、迁移或目标环境验收。append-only Transaction 记录无 TTL，在线/归档容量与可恢复索引方案仍需在 DDD/运行预算中验证。Neon/Vercel 与 Multipass/K3s（测试 inventory 总数最多 2）都须各自完成真实部署验证，后者不以两个 VM 证明 HA。
- 本补充文件尚未合并进主 SRS；在主文件冲突解除之前，它与对应 BD/DD/TD 作为一个独立且规范性的 W16 增量维护。

## 4. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增设备身份、撤销、一次性令牌、防持久正文泄露、KMS fail-closed、轮换/恢复和低消耗验收要求；状态保持 NOT_RUN。 | W16 前置审计发现认证、持久 KMS 与恢复链尚未闭合。 |
| v0.2 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 canonical AAD 包含可选 Worktree；记录 W16-C 单对象大小界限和仅有 envelope core 单元实现，所有生产 provider/owner/DB/恢复门仍 OPEN/NOT_RUN。 | W16-C envelope core 实施与字段顺序复核。 |
| v0.3 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 同步 W16-A 源码与隔离 PostgreSQL 18 owner/RLS 验收：记录设备 repository、generation 锁/拒写、refresh actor+tenant 过滤和约束迁移通过；Canvas 读写/订阅/SSE、OAuth 并发单赢家、生产 provider/恢复与性能保持 OPEN。 | W16-A 全量本地 PG18 acceptance 70/70 步骤完成。 |
| v0.4 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 OAuth repository disposable PG18.6 3/3 测试通过及其 superuser/test-prerequisite 限制；记录 Canvas SSE candidate compile 被资源门中断、未启动 PostgreSQL；完整 E2E、生产 RLS/身份与 W16 保持 OPEN。 | W16-A OAuth PostgreSQL acceptance 与 Canvas candidate 状态复核。 |
| v0.5 | 2026-10-30 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录当前 dev 基线上的 Canvas SSE 撤销单用例 PostgreSQL 18.6 acceptance PASS（run 6d23b8ceb1f0499091885c42271c2fb0）及 Rust 1.99.0 focused build；标明 superuser、test-only no-op audit/SCD2、unsigned attestation 边界；完整身份/E2E、生产 RLS/grants、HTTP revoke/audit、恢复与性能继续 OPEN。 | 当前 dev focused SSE acceptance 复跑与跨层文档核对。
| v0.6 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 将 Canvas SSE 当前状态更新为最新可核验 acceptance run 49b512fbfe334f0e98e46fe7eae7ea39（候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac，已包含于 dev@0e977306a1aafb031557d28d7fd19795297dd027）；保留 superuser、test-only、unsigned 与完整协作链未验收边界。 | W16-A 规范文档 rebase 到当前 dev 并复核最新 acceptance 报告。 |
| v0.7 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 冻结设备撤销事件的 owner-local Outbox/Inbox、至少一次投递、稳定事件身份/schema、精确重放与异摘要隔离、Inbox commit-before-ack、provider/profile-bound opaque cursor、有限恢复与每次请求的当前授权复验；追加未运行的测试映射并明确本次只更新设计。旧 PG18/18.6 证据和限制原样保留。 | W16 Outbox owner-contract review 与当前 dev 文档一致性核对。 |
| v0.8 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 明确 exact replay 必须同时匹配业务键、tenant-scoped event_id 并解析到同一记录及相同 digest；补充两种键错配/异摘要隔离、quarantine commit-before-ACK、持久化失败时有界重试/暂停及 tenant 不泄漏要求；增加设计态测试映射，所有新测试保持 NOT_RUN。 | 协调者独立审查指出仅写 identity+digest 未明确双键映射一致性、隔离 ACK 与失败重试语义。 |
| v0.9 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 冻结设备 generation/source sequence 一对一、legacy writer/session drain 后才 READY 的版本化 readiness gate、READY 前状态纳入 baseline 且不补造事件、绑定行锁下的持久 replay-floor 基线及并发初始化边界、存量设备 snapshot bootstrap、受可信服务身份/capability 保护的版本化 snapshot/replay、Identity/Auth lease 上限与 Collaboration 可缩短规则、连续 consumer checkpoint、snapshot 后延迟事件的 durable receipt 与不回退要求、owner-local 物理表/RLS/TTL 分类；明确普通 refresh-token 轮换不产生设备授权事件。所有新增实现验收保持 NOT_RUN。 | W16 实现前的 API、恢复、授权、资源与数据 owner 决策审查。 |
