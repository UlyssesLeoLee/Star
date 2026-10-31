# SRS-W16-PAYLOAD-SECURITY-001 — 协作载荷身份与持久加密要求

> 状态：Draft v0.6；这是 SRS-CANVAS-AGENT-001 v1.4 的 W16 补充需求，待后续无冲突时并回主 SRS。W16-A 的设备身份源码、隔离 PostgreSQL owner/RLS 与 OAuth repository 切片已部分验收；Canvas 活跃 SSE 流撤销关闭用例对应的 PostgreSQL 18.6 acceptance run 49b512fbfe334f0e98e46fe7eae7ea39 在候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac 上通过；该 HEAD 已包含于当前 dev@0e977306a1aafb031557d28d7fd19795297dd027。此结果只关闭该单用例，生产协作闭环仍未验收；W16-C 只有有界 provider-neutral envelope core。

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

## 2. 验收映射与状态

| Requirement | 目标证据 | 当前状态 |
|---|---|---|
| 01–02 | E2E-COLLAB-12/13；身份绑定、共享读写、订阅与 SSE 撤销失效 | Canvas 活跃 SSE 流撤销关闭的单项 PostgreSQL 18.6 acceptance 已在候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac 上通过（run 49b512fbfe334f0e98e46fe7eae7ea39）；该 HEAD 已包含于当前 dev@0e977306a1aafb031557d28d7fd19795297dd027；设备身份完整链、伪造客户端身份拒绝、Canvas 共享读写、订阅/resolve、生产 grants/RLS 与恢复仍未验收，需求整体 PARTIAL / OPEN |
| 03 | E2E-COLLAB-14；数据库并发单次消费测试 | OAuth repository 的本地 disposable PG18.6 slice 通过 3/3 named tests；使用临时 superuser 和 test-only prerequisites。完整 E2E-COLLAB-14、生产角色/RLS 与 HTTP auth flow 仍 PARTIAL / OPEN |
| 04–06 | W16-C envelope-core focused unit tests；E2E-COLLAB-15/16/17、真实 provider 故障与 DB/RLS 测试 | domain-kms lib tests 21/21 PASS（lane 专属 Cargo target）；未接 owner/REST/DB，生产 provider、密文持久化与恢复仍 OPEN / NOT_RUN |
| 07 | E2E-COLLAB-18/19；恢复与轮换测试 | 未实现 / NOT_RUN |
| 08 | PT-10 与 PT-09-GATE-01；固定 workload/profile 的内存、CPU、队列和 KMS 调用证据 | 未实测 / NOT_RUN |

W16-A 隔离本地 PostgreSQL 18 runner 于 2026-10-29 完成 70/70 步骤（run 6af04e5a765446c8b3baaffff126bd52），其中设备 repository PG 测试 2/2 通过，并验证迁移、owner/RLS、设备 generation FOR SHARE/拒写、refresh-token tenant+actor 可见性与约束升级/回滚。OAuth repository runner（run 1b4673269a04435daa1fab5c93333133）在 disposable PostgreSQL 18.6 上通过 3/3 named tests，但其 superuser/test-only prerequisites 不证明生产 grants、RLS runtime、audit/SCD2 或 HTTP authorization flow。候选 HEAD d1269a7928c9323f94403f3f1bb7888048c1dbac 上的 Canvas SSE 撤销 acceptance（run 49b512fbfe334f0e98e46fe7eae7ea39；该 HEAD 已包含于当前 dev@0e977306a1aafb031557d28d7fd19795297dd027）通过 41/41 runner steps；Rust 1.99.0 focused build exit 0，精确测试 active_canvas_sse_closes_after_device_binding_revocation 为 1 passed/0 failed/0 ignored/3 filtered，0.17s。该用例使用全新 PostgreSQL 18.6 IPv4 loopback disposable cluster、postgres superuser 与 test-only no-op audit/SCD2 prerequisites；build attestation unsigned，无签名/CI proof，未应用 replay-offset migration。它只证明单个活动 SSE 流在已提交设备撤销后关闭，不证明生产 grants/RLS、HTTP revoke/audit、跨实例恢复或完整 E2E。此前的内存门中断是历史尝试，不是本次验收状态。以上局部结果不关闭 E2E-COLLAB-12..14 或 W16；其余要求仍须在目标 PostgreSQL/RLS 与实际 provider profile 下验收。测试环境使用 Neon（Vercel 路径）或 Multipass 管理的 K3s（本地路径），Multipass 实例总数最多 2；不要求 Docker Desktop。

## 3. 已知缺口

- 现有源码已加入认证设备上下文、enrollment/proof/revoke 端点与 PostgreSQL DeviceBindingRepository，并在 Run/Worktree 写事务复验当前设备 generation；设备 repository、选定 owner/RLS、OAuth repository 并发消费已有局部 PostgreSQL acceptance。Canvas 单个已连接 SSE 流在已提交设备撤销后的关闭用例已于当前 dev 基线通过（run 49b512fbfe334f0e98e46fe7eae7ea39）；这不覆盖共享读写/resolve、生产身份与 grants/RLS、HTTP revoke/audit、多实例恢复或完整 E2E。详见既有设备报告、Canvas SSE acceptance 报告与 PHASE-W16-A-OAUTH-PG-ACCEPTANCE-20261029.md。
- W16-C lane 加入 `TranscriptEnvelopeProtector` 与单元测试，但尚未接入 `TranscriptBodyProtector`、业务 owner 或生产请求路径；`LocalMockKms` 不能跨重启恢复，生产 KMS、RLS、持久化、跨重启解密仍未实现/NOT_RUN。详见 `W16-KMS-SOURCE-AUDIT-20261028.md` 与 W16-C 阶段报告。
- Provider 研究仅形成候选建议；完整依赖/镜像/部署闭包、密钥恢复演练、运行预算和生产配置尚未批准。详见 `W16-KMS-PROVIDER-RESEARCH-20261028.md`。
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
