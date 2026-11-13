# PHASE-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261106

> 状态：设计审查完成；推荐采用按事实 owner 分别持有 Outbox、按接收 owner 持有 Inbox 的边界。本报告不修改规范文件或 WBS，不证明代码、部署或运行验收；W16 仍为 **OPEN / PARTIAL**。

## §0 目的与范围

审查 W16 设备撤销事件的事实归属、持久发布和协作侧接收边界，解决详细设计中 Outbox owner 表述不一致的问题。仅审阅当前 SRS/BD/DD、已存在的 W16 阶段证据及 WBS §23.14；未编辑产品代码、规范文件或 WBS。

推荐结论：Identity/Auth owner 是设备撤销事实与撤销事件的生产者；Collaboration owner 是协作域的持久接收者，以自己的 Inbox 和事务应用失效状态。Identity 与 Collaboration 各自在自己的事务中写本 owner 的事实，不跨 owner 写表，也不共享数据库事务。该结论是待写回规范的设计建议，不代表 owner 实现已经存在或被目标环境验收。

## §1 证据与审查矩阵

| 依据 | 当前文档事实 | 审查结果 |
|---|---|---|
| [SRS-W16 §0、§1、§2](../requirements/SRS-W16-PAYLOAD-SECURITY-001.md#0-目的与范围)（§0、SRS-W16-PAYLOAD-01..08；lines 5–7, 11–30） | 作者身份必须来自可信认证；撤销后读、写、订阅、resolve 拒绝，事件流关闭或在有界租约到期前重验；Outbox/NOTIFY/SSE 不得含正文或明文 DEK；恢复、取消与资源边界仍需验证。 | 权威身份与 fail-closed 行为已有要求，但没有完整定义 Identity→Collaboration 的 durable Inbox/cursor 合约。 |
| [BD-W16 §1–4](../design/BD-W16-PAYLOAD-SECURITY-001.md#1-目标与边界)（lines 5–37） | §1 把身份/设备撤销交给身份 owner，把 draft/conflict/revision/RLS/审计/Outbox 交给协作 owner，同时要求每个 owner 只写自己的事实；§2 定义撤销；§3 把 Outbox 列为 owner 事务中的 Transaction；§4 将正文排除在 Outbox/wake payload 外。 | owner-local 与无跨 owner 事务已是基本设计约束；Outbox 归属应限定为“该 owner 自己产生的业务事实”。 |
| [DD-W16 §1、§2.3、§3.3](../design/DD-W16-PAYLOAD-SECURITY-001.md#1-模块边界)（lines 5–12, 29–31, 75–79） | §1 的表把 Outbox 列入 Collaboration owner；§2.3 又要求 Identity owner 在设备撤销事务写 Outbox；§3.3 要求协作 owner 在自己的事务中写 metadata-only Outbox。 | 存在 WBS 所述矛盾。应按事件源事实拆分：Identity 持有撤销 Outbox；Collaboration 持有协作事实 Outbox。禁止把 Identity 撤销事实移交给 Collaboration 写入。 |
| [WBS §23.14](../implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md#2314-2026-10-30-w8w16-就绪度审计与下一步顺序)（lines 756–768） | §23.14 明确标记 Outbox owner、版本化事件、replay/cursor、授权和 gap recovery 为阻塞；也明确 `pg_notify` 不是 durable delivery。 | 本审查解决推荐的 owner 分工；具体规范与 WBS 状态同步仍需独立登记。 |
| [PHASE-W16-EVENT-PUSH-IMPL §1–3、§5](PHASE-W16-EVENT-PUSH-IMPL-20261027.md)（lines 9–39, 52–59） | 已记录 PostgreSQL wake、SSE adapter、授权 catch-up 与有限测试；双实例/目标 DB/RLS/性能未运行，draft/conflict durable 闭环未实现。 | 证明的是通知与 catch-up 切片的历史实现边界，不证明 Identity Outbox→Collaboration Inbox 的生产持久链。 |

### 推荐 owner 合约

1. **生产者与原子事实**：Identity/Auth owner 在单个本地事务内推进 `device_generation`、标记 revoked、撤销关联 refresh family、追加审计，并插入同一状态转移的 Identity Outbox 事件。若该事务回滚，状态、审计和事件全部回滚；若提交，事件可在进程重启后继续发布。协作 owner 不写 Identity 表或其 Outbox。
2. **协作接收者**：Collaboration owner 通过认证的 owner-to-owner 消息接收撤销事件，在自己的本地事务中写 Inbox/幂等回执，更新仅供协作授权失效与订阅关闭使用的本地投影，并按需写自己的 metadata-only Outbox 唤醒 SSE/API 适配器。它不能把 Inbox 投影当作 Identity 的权威撤销事实，也不能写回 Identity 表。
3. **事件候选契约**：建议稳定 `event_id`、`event_type`、`schema_version`、`tenant_id`、opaque `actor_id/device_id`、新的 `device_generation`、`stream_id`、owner 内单调 `source_sequence`、`occurred_at` 与 correlation/idempotency identity。任何 token、设备私钥/证明、正文、明文 DEK 或敏感请求体都不得进入事件。字段名、保留期和兼容规则仍待规范 owner 冻结。
4. **至少一次与幂等**：relay 按至少一次交付设计；接收端以 producer + stream + sequence 以及稳定 event ID 建唯一约束。同一身份、同一摘要的重复事件返回既有结果；相同身份不同摘要必须隔离并报警，不能覆盖。Inbox 本地事务提交后才确认/推进消费游标；事务失败或取消则不确认并可重放。不得宣称跨服务 exactly-once。
5. **游标与补偿**：Identity Outbox 为其 stream 维护可重放的 owner-local 顺序；消费端只在 Inbox 与本地失效投影提交后持久推进 checkpoint。cursor 必须是不透明且绑定 transport/profile 的恢复句柄；不可跨 provider 比较或复用。检测 sequence gap、过期 cursor 或 retention 缺口时，暂停该 consumer 的授权投影并向 Identity owner 拉取有界当前状态/重放；状态无法核验时拒绝共享操作并关闭或到期关闭 stream，不能把“无事件”当成有效授权。
6. **授权与事件信任**：每个 API 读写/订阅/resolve 仍依据可信身份与 Identity owner 的当前 generation/status 复验；接收事件只负责加速失效和恢复协作投影，不能扩权或取代授权查询。生产者身份、租户 scope、event type 和 schema 需在 receiver 验证；任何身份/目录/数据库不可用、未知版本或授权不完整均 fail closed。
7. **重试、取消与 drain**：Outbox relay 对暂时故障有界退避并保持待发送事实；接收端持久化失败时不确认。撤销事实已提交后，取消 HTTP 请求或 worker 不得撤销/丢弃 Outbox 记录；worker 只在事务边界取消，重启后从 durable cursor 恢复。永久坏事件进入隔离/告警并触发 fail-closed reconciliation，不能静默死信后继续授权。切换 transport/profile 前 drain 旧 profile 的待处理记录或执行显式迁移；不得用新 provider 的游标解释旧 provider 状态。具体退避、租约、队列、保留期与 drain SLA 需 profile 冻结和目标负载测量。

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| 规范一致性阅读 | 已逐段核对 SRS-W16 §0–2、BD-W16 §1–4、DD-W16 §1/§2.3/§3.3，并比对 WBS §23.14。 |
| 既有实现/运行证据 | 仅引用已有 PHASE-W16-EVENT-PUSH-IMPL 与 WBS 所记录证据；本 lane 未重新运行程序、数据库或服务。 |
| 新增内容 | 本 phase report 与配套独立审查记录；不改规范、不改 WBS。 |
| 文档检查 | 待提交前执行新文件内部链接目标检查、`git diff --check` 和变更清单核对；不运行 Cargo、应用测试、数据库或性能门。 |

已有证据边界：WBS §23.14 记录的设备/绑定、OAuth repository 与单个 Canvas SSE 关闭测试只证明相应局部切片；W16-A authenticated HTTP revoke + audit、生产 grants/RLS、跨实例 durable fan-out、cursor gap recovery 和完整 E2E 仍未验收。事件推送阶段报告还将双实例目标 DB/RLS、RSS/CPU 与 p99 标为 NOT RUN。以上是文档中记录的历史证据，本 lane 不将其重新表述为当前环境复测结果。

## §3 已知缺口与待决项

- **规范尚未收敛**：DD-W16 §1 的 Collaboration Outbox 字段与 §2.3 Identity revoke Outbox 仍同时存在。本 lane 无权修改 SRS/BD/DD；需独立规范 lane 写明“每个 owner 仅负责自己事实的 Outbox”，并补齐 Identity event→Collaboration Inbox 契约。
- **投递拓扑未冻结**：建议 Collaboration owner 是协作域的持久接收 owner；Identity 事件如何经 Inbox/本地 Outbox 触达承载 SSE 的各 API 节点，以及是否另有直接授权失效订阅，尚未决定。
- **事件 schema/顺序未冻结**：event type/version、source sequence 的分区键、generation 与 sequence 的关系、schema evolution、未知版本策略、事件保留和快照恢复 API 需 owner 决策。
- **传输与运行策略未冻结**：消息 provider/profile、认证方式、跨 owner database grants、退避上限、并发/队列/重试预算、租约上限、drain SLA 与 poison-event 运维流程未在本审查中定值。
- **HTTP 重放契约仍未决**：重复 revoke 请求返回既有结果或特定冲突响应的 API 语义需先冻结；WBS §23.14 将该项列为 W16-A acceptance 前置决策。
- **运行证据仍不足**：未验证受限应用角色下撤销状态 + refresh family + 恰一条 audit + Outbox 的同事务 commit/rollback；未验证 receiver Inbox 幂等、乱序/gap、取消重试、跨实例关闭与恢复。不得据本设计建议打开 W16-D 草稿/冲突写入。

## §4 子代理失败接手清单

本 lane 未派发子代理，也没有子代理产出或失败需要接手。协调者后续若分派规范同步或实现 lane，应分别登记文件清单、保留本报告的设计/实现证据边界，并在合入前重新审阅 owner 事务与接收幂等闭包。

## §5 守门规则

1. Identity/Auth 是设备撤销权威事实的唯一 owner；撤销状态、generation、refresh-family 撤销、append-only audit 与 Identity Outbox 必须处于同一 owner 事务。
2. Collaboration owner 通过自己的 Inbox 接收和投影失效事实；任何 owner 不直接写另一 owner 的表，不跨 owner 共用事务。
3. 对外 Outbox 事件只传最少必要 metadata；无正文、token、proof、private key 或明文 DEK；未知 event/schema 版本 fail closed。
4. 采用至少一次投递 + owner-local 幂等 Inbox；Inbox/本地投影未提交前不得 ACK 或推进 durable cursor；同事件身份摘要不一致必须隔离与告警。
5. API 操作始终复验权威 Identity 状态；事件和本地投影只加速撤权传播，不能作为授权真相或补发权限。
6. 丢失、断线、乱序、cursor gap、retention gap、consumer 重启和 transport 切换都必须有可审计恢复路径；未知授权时拒绝共享操作。
7. 撤销提交后 worker/客户端取消不得丢弃待投递事实；切换 profile 前必须 drain 或显式迁移旧 profile，禁止跨 provider 解释 cursor。
8. 事件队列、并发订阅、缓存和日志均须有界；WBS 的 p99 目标、具体负载、RSS/CPU 与队列预算必须在目标 Neon 与 Multipass/K3s 路径分别实测后才能称为通过。
9. 本设计审查不关闭 SRS 验收映射、W16-A/B/C、WLD-7 或 PT-09；只在生产角色、目标数据库、多实例恢复、双端协作与独立 E2E 门通过后重新评估阶段状态。

## §6 签字栏

| 角色 | 状态与范围 |
|---|---|
| 架构 / 本报告审阅 | 架构师（Mavis 接手 agent per DEC-008）；仅审查本报告提出的 owner 边界与文档证据范围，不代表规范合并或生产验收。 |
| SRE Lead | 本 lane 未执行部署、恢复或运行验收；不在本报告签署范围。 |
| 平台 | 本 lane 未实施传输、身份或数据库权限；不在本报告签署范围。 |
| 评审主持 | 本报告提供可审查的冲突、建议与未决项；需协调者集成前复核。 |
| PM | 用户/产品 owner 最终确认未决投递拓扑、配置预算与 API 重放语义。 |
| 修订人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。 |

## §7 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 W16 durable Outbox owner 合约审查；给出按事实 owner 分开 Outbox、receiver owner-local Inbox 的推荐，并明确未验证项。 | WBS §23.14 将 Outbox owner 与 durable replay 作为 W16 阻塞项。 |
