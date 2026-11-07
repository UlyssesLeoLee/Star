# WBS-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261107 — 预合并设计自审 v0.2

> **Review type**：lane 作者的预合并一致性自查，不是独立批准。
> **结论**：可交协调者独立审查；未达到可集成状态。
> **Lane / base**：`codex/wbs-w16-outbox-owner-contract-20261107` / `dev@996ae3ae7d6e4c9c075715f3799f5186734d32f0`
> **依据**：[`PHASE-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261106.md`](../reports/PHASE-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261106.md) 与 W16 companion SRS/BD/DD/测试设计当前文本。

## 1. 一致性检查

| 检查项 | 结果 | 证据 / 备注 |
|---|---|---|
| Owner 权责与事务边界 | 已修正 | SRS/BD/DD 一致规定 Identity/Auth owner 负责设备撤销事实、generation/status、refresh-family、审计和自己的 Outbox；Collaboration 仅写自己的 Inbox、幂等回执、失效投影及自有 Outbox。禁止跨 owner 写表和共享事务。 |
| 事件身份与 schema | 已补齐 | SRS-W16-PAYLOAD-10、DD §2.5 与 UT/IT/E2E 映射一致，包含 tenant、producer、stream、稳定 event ID、schema、owner-local source sequence、correlation/idempotency 与 canonical digest。 |
| 安全载荷 | 已补齐 | 事件不携带 token、私钥/证明、正文、明文 DEK 或敏感 request body；未知 schema 不应用；日志仅保留有限 metadata 与结果。 |
| 重试与回执 | 已修正并待复审 | at-least-once；exact replay 仅限业务键与 `(tenant_id,event_id)` 同时解析到同一 Inbox row 且 digest 匹配；同业务键异 event_id、同 tenant-scoped event_id 异业务键、双键映射分叉或摘要不同均 durable quarantine/告警，无投影或正常业务 Outbox 副作用。quarantine commit 后才 ACK；写失败不 ACK，有限 retry 后暂停/背压。未宣称跨 owner exactly-once。 |
| Cursor 与恢复 | 已补齐 | 领域 source sequence 与 opaque transport cursor 分离；cursor 固定 provider/version/profile digest/endpoint/log generation；gap、过期、retention 缺口及 source unavailable 触发有界恢复与 fail-closed。 |
| 实时授权 | 已补齐 | 撤销事件只加速缓存失效/SSE 关闭；每次业务操作仍实时验证当前身份、设备 generation 与 Project/Run capability。 |
| 资源与部署路径 | 已补齐 | profile 约束事件大小、队列、batch、in-flight、retry、retention、recovery 和租户预算；Vercel/Neon、Multipass/K3s 分开验收，测试 inventory ≤2。阈值尚未批准，保持 NOT_RUN。 |
| 既有验收证据 | 边界保留 | PG18/18.6 历史 slices 仍各自仅关闭原有局部断言；没有将单 SSE 撤权用例扩大为 owner-event、生产授权或部署通过。 |
| Manifest 与文件范围 | 已核对 | MAGOS list 记录该 lane 的 8 个登记路径；共享 WBS、代码、migration 与其他 lane 均未修改。 |

## 2. 待独立审查与阻断项

1. 已处理协调者独立审查 blocker：补充双键一致性、两种单键错配/映射分叉/异摘要 quarantine、ACK-after-quarantine-commit、失败不 ACK 并有界重试/暂停，以及 tenant-RLS 不泄漏语义；新增 UT-W16-INBOX-02、IT-W16-INBOX-IDENTITY-01、IT-W16-QUARANTINE-ACK-01，并扩展 E2E-COLLAB-21。该 finding 现等待协调者重新独立复审，不视为独立批准。
2. 实现期需决定权威 snapshot/replay API 的版本化接口、授权方式、lease 的 profile 来源和 consumer checkpoint 的持久 owner；本 lane 只冻结语义，不新增 API/DDL。
3. 需要后续 DDD/RLS review 确认 Inbox 双去重键、quarantine/告警意图、投影以及 source-sequence counter 的具体表/索引/retention 分类与跨租户策略。
4. 需有批准的 event profile/W14 workload/阈值后，才能实施 PT-11/12 并验证有界内存、背压、公平性、取消和 recovery。
5. Neon/Vercel 和 Multipass/K3s 真实路径、生产 grants/RLS、跨实例传递、恢复、撤权后的活动 SSE 与独立 gateway 尚未运行。
6. 当前文档测试设计新增 owner-event 用例全部 `NOT_RUN`；本 lane 未运行 Cargo、PostgreSQL、云服务、Multipass 或 K3s 验收。

## 3. Review verdict

**已修正审查 blocker，可重新进入协调者独立审查；暂不集成。** 任何上项被指出不一致时，先在该隔离 lane 修正文档并重跑静态链接/diff 检查。只有独立审查通过后，协调者才决定是否在目标分支集成。

## 4. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.2 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录协调者发现并要求修复的双键 replay identity/quarantine ACK blocker，核对补充 UT/IT/E2E 覆盖和 NOT_RUN 状态；结论为可重新交独立复审，尚未集成批准。 | 协调者独立审查 follow-up。 |
