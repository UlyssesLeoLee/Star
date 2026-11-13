# WBS-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261106

**审查结论：推荐解决 owner 归属矛盾，不代表规范已变更或运行能力已完成。**

本记录独立于产品实现 lane，依据本地当前规范、已有 W16 报告与 WBS §23.14 做静态设计审查。未读或修改被保留的 Rust baseline lane；未检查产品代码、没有运行程序或数据库，也未修改 SRS、BD、DD 或 WBS。

## 1. 审查问题与判定

| 问题 | 判定 | 依据与理由 |
|---|---|---|
| 谁拥有设备撤销事实？ | **Identity/Auth owner。** | DD-W16 §1（Identity/Auth 行）和 §2.3（lines 5–10, 29–31）将设备 credential、generation/status、撤销及审计归 Identity/Auth；BD-W16 §1–2（lines 5–17）同向。撤销不应因共享协作而改由 Collaboration owner 决定。 |
| 谁写撤销事件的 Outbox？ | **Identity/Auth owner，在其撤销事务中写。** | DD-W16 §2.3 已要求这点，但 §1（lines 5–10）的 Collaboration owner 表格把 Outbox 无限定归给 Collaboration。§3.3（lines 75–79）又将协作 owner 事务中的 Outbox 用于 draft/checkpoint 元数据。按 owner-local 原则，应收窄为各 owner 仅给自己负责的事实写 Outbox。这样才能保证撤销与失效事件原子提交，避免跨 owner 写表。 |
| 谁接收撤销事件并管理协作影响？ | **Collaboration owner，以自己的 durable Inbox 接收。** | 协作 owner 管理其 Run/resource scope、草稿/冲突/修订及本地失效投影；它在自己的 Inbox 事务处理撤销通知，不成为身份真相 owner。其 API/事件适配层据本地失效信号关闭相关 SSE，并仍向身份 owner 复核有效授权。 |
| Collaboration 自己的 Outbox 是否取消？ | **不取消。** | Collaboration 仍需为它自己的 revision/draft/conflict 等业务事实，在本地事务提交 metadata-only Outbox。身份撤销事件不能借用它的表或 owner 来写入。 |
| 是否共享事务或跨 owner 直写？ | **否。** | Identity source transaction 与 Collaboration inbox transaction 独立；relay 至少一次交付，Inbox 幂等，提交后确认。BD-W16 §1 禁止跨 owner 共写；DD-W16 §1 亦将跨 owner 写事务列为职责之外。 |
| `pg_notify`/SSE 是否为持久事实？ | **否；只作唤醒/传输。** | WBS §23.14 明确 NOTIFY 不等于 durable delivery；PHASE-W16-EVENT-PUSH-IMPL §1、§5 也将通知/SSE 定义为 wake hint，并依赖获授权 Outbox catch-up。 |

## 2. 建议的事件和持久化流程

```text
设备撤销命令
  → Identity/Auth 本地事务：generation/status + refresh family + append-only audit + Identity Outbox
  → 可替换 relay（至少一次，重试沿用固定 profile）
  → Collaboration 本地事务：Inbox 幂等收件 + 失效投影 + Collaboration metadata-only Outbox
  → API/SSE 适配器收到 wake，核验授权并关闭受影响订阅
```

流程里的箭头是消息交付，不是跨服务 SQL 写入，也不共用提交事务。Adapter 的 ACK 只能在 receiver Inbox 与本地投影提交后发生；发件人 Outbox relay 以持久 checkpoint 管理自身进度。应用请求对当前身份仍须重新授权，避免失效事件延迟时将 stale token 当有效凭证。

候选事件最小字段：稳定 `event_id`、`event_type`、`schema_version`、tenant 与 opaque actor/device identity、撤销后的 `device_generation`、owner 内 `stream_id/source_sequence`、`occurred_at`、correlation identity。Schema 与 sequence 分区键尚待规范冻结。事件不含正文、token、设备私钥/证明、明文 DEK 或敏感请求体（SRS-W16-PAYLOAD-04）。

建议唯一性与冲突处理：以 producer + stream + source sequence 和稳定 event ID 去重；同 ID 同摘要视为重放，同 ID 不同摘要隔离并告警；旧 generation 不能覆盖新 generation；发现序号缺口时拉取 Identity owner 的 durable replay 或当前状态快照，未完成恢复前对共享 API fail closed。ACK/checkpoint 落后时允许重投，不承诺 exactly-once。

取消与迁移：撤销事实提交之后，客户端断开或 worker 取消不撤回 Outbox。消费者只能在数据库事务边界响应取消，未提交不 ACK；重启从 durable cursor 恢复。transport/profile 改变前先 drain 旧 profile 或执行有审计的迁移；cursor 不跨 provider 解释。具体 deadline、退避、保留期、队列限制和资源预算不能从现有文档推导，应冻结为版本化 profile 并测量。

## 3. 未决事项（不能由静态审查代替产品决策）

1. **接收/扇出拓扑**：是否固定 `Identity → Collaboration Inbox → Collaboration Outbox → API/SSE nodes`，以及 API 节点是否还直接订阅 Identity 失效事件。无论选哪种，持久业务接收方仍为 Collaboration owner，Identity 不接受跨库直写。
2. **事件合同**：正式 event type/version、字段、source sequence 分区键、事件顺序与 `device_generation` 关系、保留/压缩策略、snapshot/replay endpoint 和未知版本处置。
3. **服务与权限部署**：Identity、Collaboration 是否各有独立 owner service/DB；实际 producer/consumer service identity、签名或 mTLS、最小 grants/RLS 与 tenant 绑定方式。不得从当前 PG superuser 测试推导生产权限。
4. **投递 profile**：transport adapter、固定 profile identity/version/digest、重试与退避参数、退避上限、租约、queue/backpressure、poison event 告警、drain 与恢复 SLA。用户已要求 Rust 服务可替换当前 broker，因此 domain contract 不应暴露 broker-specific ACK/subject/cursor。
5. **HTTP revoke 幂等响应**：重复撤销请求是回放既有成功结果还是返回专门冲突/状态；WBS §23.14 要求在 W16-A acceptance 前冻结此 API 契约。
6. **目标验收**：明确每个部署路径的数据库角色与两实例资源清单，并验证跨实例事件关闭、掉线重放、gap/snapshot、撤权期间请求拒绝、SSE deadline、p99、RSS/CPU、队列上限和长时恢复。已记录的单个 SSE 关闭测试不能代替这些门。

## 4. 证据状态与验收建议

**设计建议**：由事实 owner 写本地 Outbox，由 consumer owner 写本地 Inbox；至少一次 + 幂等；授权持续回到权威 Identity owner；通知仅唤醒，不作为持久事实。此建议能消除 DD-W16 §1 与 §2.3 的 owner 语义冲突，并保持 BD-W16 §1 的“owner 只写自己的事实”。

**已有实现/运行记录**（非本次复测）：PHASE-W16-EVENT-PUSH-IMPL §1–2 记录过 Canvas Outbox wake、Rust SSE 和 catch-up 测试；该阶段 §2 将目标 PostgreSQL/RLS、双实例、p99 与资源测试标为 NOT RUN。其 §3、§5 明确没有 durable drafts/conflicts 闭环，`NOTIFY`/SSE 只是 wake hint。WBS §23.14 仍将 authenticated HTTP revoke/audit、生产 RLS/grants、跨实例 durable fan-out 和完整 E2E 标为 OPEN / PARTIAL。故本报告没有把历史 Canvas 测试扩大为 Identity Outbox 或 Collaboration Inbox 的生产证据。

**下一步建议**：先由独立规范 lane 收敛 SRS/BD/DD 的 owner 行、event/Inbox/replay contract 和状态；再登记实现 lane，先验 Identity revoke + audit + Outbox 的原子性与受限角色，再验 Collaboration Inbox 的重复/乱序/故障/恢复，最后才跑跨实例 SSE 和性能/E2E。规范未更新或 receiver gap recovery 未验收前，不打开 W16-D durable drafts/conflicts。

## 5. 来源索引

- [SRS-W16-PAYLOAD-SECURITY-001](../requirements/SRS-W16-PAYLOAD-SECURITY-001.md)：§0、SRS-W16-PAYLOAD-01..08，尤其 01–02、04、06–08（lines 5–30）。
- [BD-W16-PAYLOAD-SECURITY-001](../design/BD-W16-PAYLOAD-SECURITY-001.md)：§1–4，owner boundary、撤销、W/T/M transaction 与持久化约束（lines 5–37）。
- [DD-W16-PAYLOAD-SECURITY-001](../design/DD-W16-PAYLOAD-SECURITY-001.md)：§1、§2.3、§3.3，owner 表、撤销事务 Outbox、owner-local metadata Outbox（lines 5–12, 29–31, 75–79）。
- [WBS §23.14](../implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md#2314-2026-10-30-w8w16-就绪度审计与下一步顺序)：矛盾、当前证据限制与工作顺序（lines 756–768）。
- [PHASE-W16-EVENT-PUSH-IMPL-20261027](../reports/PHASE-W16-EVENT-PUSH-IMPL-20261027.md)：§1–3、§5，现有 wake/catch-up 证据与未完成门（lines 9–39, 52–59）。

审查范围不包括产品代码、受保留 Rust baseline lane、数据库实际 schema/roles、生产 broker 或当前部署状态；引用行号以本 lane 起始基线文件内容为准，规范行号在独立同步时可能变化。
