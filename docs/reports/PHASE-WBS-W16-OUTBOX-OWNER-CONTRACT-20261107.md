# PHASE-WBS-W16-OUTBOX-OWNER-CONTRACT-20261107 — 阶段报告

> **状态**：文档设计修订完成，等待协调者重新独立审查；不是实现完成或生产验收。
> **Lane**：`codex/wbs-w16-outbox-owner-contract-20261107`
> **Worktree**：`D:\orcaWork\Star\wbs-w16-outbox-owner-contract-20261107`
> **基线**：`dev@996ae3ae7d6e4c9c075715f3799f5186734d32f0`
> **范围**：已登记的 W16 SRS/BD/DD/测试设计、brief、dispatch、phase report 与 review；不含共享 WBS、代码、migration、数据库或部署变更。

## §0 目的与范围

依据 [`PHASE-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261106.md`](PHASE-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261106.md) 将撤销事件的业务 owner、持久传递、幂等应用、恢复与授权投影写入 W16 需求、基本设计、详细设计及测试设计。协调者独立审查随后指出双键映射一致性、quarantine 持久化与 ACK 顺序需要明确；本次 follow-up 已在同一登记 lane 中补齐规范和设计态测试。上述均为目标规范，不描述为已实现事实。

本 lane 通过只读 MAGOS `list --json` 核验了当前登记：branch 与 worktree 匹配；manifest 恰为本报告 §1 列出的 8 个文件。共享 `WORKTREE-GROUP-NEXT-WBS-2026-10-18.md` 受另一 lane 冲突保护，未编辑。仅本地 lane 文件在范围内；未改 `dev`、`main`、远端、其他分支/worktree、产品代码或数据库迁移。

## §1 改动矩阵与追踪

| 文档 | 版本/变更 | 需求与验证追踪 | 状态 |
|---|---|---|---|
| [`SRS-W16-PAYLOAD-SECURITY-001.md`](../requirements/SRS-W16-PAYLOAD-SECURITY-001.md) | v0.8；冻结 owner-local 事务与事件身份；exact replay 要求业务键与 tenant-scoped event_id 同时命中同一记录且 digest 一致；任一键错配/异摘要隔离、不应用投影，隔离记录提交后 ACK，存储失败有界重试/暂停。 | 新要求指向 UT-W16-OUTBOX-01/INBOX-01/02、IT-W16-INBOX-IDENTITY-01/QUARANTINE-ACK-01 等、E2E-COLLAB-20..23、PT-11/12。 | 设计已写；实现与新测试 NOT_RUN。 |
| [`BD-W16-PAYLOAD-SECURITY-001.md`](../design/BD-W16-PAYLOAD-SECURITY-001.md) | v0.8；Identity/Auth 唯一拥有撤销事实、审计和本地 Outbox；Collaboration 只写自己的 Inbox、回执、失效投影和 metadata-only Outbox；定义双键冲突归类、durable quarantine、tenant-RLS 约束。 | 需求到 DD、owner/RLS、恢复与 PT gate 的映射已补齐。 | 设计已写；未批准 DDL/provider。 |
| [`DD-W16-PAYLOAD-SECURITY-001.md`](../design/DD-W16-PAYLOAD-SECURITY-001.md) | v0.8；定义事务性 owner-local source sequence、双唯一键精确重放、两种错配/异摘要 quarantine、并发唯一冲突分类、告警意图 durable commit、隔离 ACK 顺序和存储失败时的 bounded retry/pause。 | 每次业务操作仍实时核验当前身份/generation/Project/Run capability；tenant RLS 查询不泄漏存在性；NOTIFY/SSE 仅 wake-up。 | 目标详细设计；无代码/migration 证明。 |
| [`TEST-DESIGN-WORKTREE-CANVAS-001.md`](../test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md) | v2.6；owner-event 增为 3 UT、7 IT；E2E-COLLAB-21 补齐双身份键错配、映射分叉、quarantine ACK/失败与不泄漏断言；E2E-COLLAB-20..23、PT-11/12、W16 owner-event AC 映射及已知缺口同步。 | 覆盖 rollback、并发唯一冲突分类、ACK 顺序、告警持久意图、quarantine commit failure、有界重试/暂停、gap/cursor/retention 恢复、tenant isolation 与撤权后 SSE；Neon/Vercel 与 Multipass/K3s 分开验收，inventory ≤2。 | 新增用例均设计态、NOT_RUN；原 PG18/18.6 证据范围未扩大。 |
| `docs/briefs/WBS-W16-OUTBOX-OWNER-CONTRACT-20261107.md` | dispatcher brief 已登记，保留本阶段目标、边界与验收要求。 | 明确 WBS 不在此 lane。 | 原样随 lane 记录。 |
| `docs/reports/WBS-DISPATCH-W16-OUTBOX-OWNER-CONTRACT-20261107.jsonl` | brief dispatch receipt 已登记。 | 记录 phase/task/brief 路径。 | 原样随 lane 记录。 |
| 本报告 | 新增 §0–§7 阶段证据。 | 记录差异、静态检查、缺口和签批状态。 | 随本 lane 同次提交；待协调者审查。 |
| [`WBS-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261107.md`](../reviews/WBS-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261107.md) | 新增预合并文档自审矩阵。 | 标明需协调者独立 review；不代替 review/集成批准。 | 随本 lane 同次提交；独立 review pending。 |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| MAGOS `list --json` | 只读核验成功。当前登记 branch=`codex/wbs-w16-outbox-owner-contract-20261107`，worktree 路径匹配，登记路径共 8 项，与本阶段提交清单一致。 |
| 相对 Markdown 链接检查 | PowerShell 对 7 个 lane Markdown 文档检查相对链接：**PASS，22 个相对链接均可解析，0 个缺失**。 |
| `rtk proxy git diff --check` / `rtk proxy git diff --cached --check` | 本 follow-up 两项均 **PASS，退出码 0**；当前 6 个已登记文档的工作树及 staged diff 无 whitespace error。Brief 与 dispatch manifest 文件保持未修改。 |
| UT/IT/E2E/PT、Cargo、PostgreSQL、Neon、Multipass/K3s | 按 lane 范围不运行；新增测试均为 `NOT_RUN`。 |

本阶段未将已有记录的局部证据改写为完整验收：设备/RLS PG18 run `6af04e5a765446c8b3baaffff126bd52`、OAuth repository PG18.6 run `1b4673269a04435daa1fab5c93333133`、SSE 单用例 run `49b512fbfe334f0e98e46fe7eae7ea39` 仍只证明原有范围及其各自的 superuser/test-only/unsigned 限制。

## §3 已知缺口

- 本次只冻结设计契约。Identity/Auth 的原子撤销 Outbox、Collaboration 的持久 Inbox/投影/回执、双键并发分类、durable quarantine/告警投递、失败重试/暂停、持久 relay 与恢复尚未实现或验证。
- 生产 RLS/grants、身份撤权的端到端审计、跨实例 consumer ACK/cursor、source sequence gap、retention 丢失和 tenant isolation 尚未完成目标环境测试。
- Neon/Vercel 与 Multipass/K3s 两种路径均未由本 lane 运行；Multipass 测试 inventory 上限 2 个，且两实例不是 HA 证据。
- 事件尺寸、队列、in-flight、租户配额、重试/保留/恢复时间窗与 PT 阈值尚需版本化 profile/W14 批准；未知时保持 fail closed/NOT_RUN。
- 新用例尚无对应实现入口；此前局部 PG18/18.6 acceptance 不覆盖这些合同，不可据此关闭 W16。
- 主 WBS 未在当前 manifest 内，故本阶段没有变更其状态。

## §4 子代理失败接手清单

本 lane 未再派生子代理，也没有子代理失败或待接手产物。预合并 review 文件是本 lane 自查，不宣称为独立审批；协调者仍需完成独立审查。

## §5 守门规则

1. Identity/Auth owner 独占设备 generation/status/revoke/refresh-family 与相应审计事实。
2. Identity/Auth Outbox 与它拥有的撤销状态在同一 owner 事务提交或回滚。
3. Collaboration 不写 Identity/Auth 表；两 owner 不共享写事务。
4. Collaboration Inbox、回执、撤权投影与其自有 Outbox 在 Collaboration 本地事务处理。
5. 传递语义为 at-least-once；文档不声称跨服务 exactly-once。
6. 接收事务提交前不得 ACK 或推进 consumer checkpoint。
7. exact replay 必须满足业务键与 `(tenant_id, event_id)` 同时指向同一 Inbox 记录且 digest 相同。
8. 单键错配、双键映射分叉或 digest 不同必须 durable quarantine/告警，不更新投影或正常业务 Outbox。
9. 只有 quarantine 记录与告警意图提交后才 ACK poison；quarantine 写失败不 ACK，有限重试耗尽后暂停并背压。
10. 冲突分类只能在认证的 tenant RLS scope 内查询，不泄漏其他 tenant 的记录存在性。
11. Event identity/schema 与 tenant scope 稳定、版本化且可审计。
12. Token、私钥/证明、正文、明文 DEK 和敏感 request body 不进入事件、Outbox、日志或 wake payload。
13. source sequence 与 transport cursor 分离；cursor 固定 provider/version/profile digest/endpoint/log generation。
14. Gap、过期 cursor、retention 缺口或权威身份源不可用触发有界恢复；未确认时 fail closed。
15. 每次业务请求仍实时验证身份、设备 generation 与 Project/Run 授权；事件只加速失效。
16. queue、batch、in-flight、retry、dedupe retention、event size、tenant quota 和恢复窗口受版本化 profile 限制。
17. 无批准预算/上界时不得启用生产 consumer 或把内存队列称为 durable acceptance。
18. Neon/Vercel 与 Multipass/K3s 分别验收；测试 inventory 最多 2 个 Multipass 实例，不声明 HA。
19. 本报告只包含文档与静态检查；旧局部实测边界保持原样，未运行项保持 NOT_RUN。

## §6 签字栏

| 角色 | 责任人 | 状态 | 边界 |
|---|---|---|---|
| 架构师 | Mavis 接手审核 | 协调者独立审查 pending | 本报告只记录 lane 自审，不作最终集成批准。 |
| SRE Lead | — | pending | 双部署拓扑、owner 可用性及恢复预算未验收。 |
| 平台 Lead | — | pending | profile/provider 与持久 relay 实现未验收。 |
| 评审主持 | 协调者 | pending | 等待审查规范一致性及 manifest 范围。 |
| PM | Ulysses | 待确认 | WBS 状态须由主协调者另行同步。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新建阶段报告，记录 W16 owner-local Outbox/Inbox 设计差异、未运行验证、manifest 范围和剩余风险。 | WBS-W16-OUTBOX-OWNER-CONTRACT-20261107 文档 lane。 |
| v0.2 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 根据协调者独立审查 finding 修订双身份键 exact replay 条件、冲突/tenant-safe 分类、durable quarantine 与告警意图、ACK 顺序、quarantine commit failure 的 bounded retry/pause；增加 1 UT 与 2 IT 设计并更新 E2E 追踪。全部新增测试保持 NOT_RUN，等待重新独立审查；7 个 Markdown 文档的 22 个相对链接及当前修改的 worktree/staged diff 静态检查均通过。 | 独立审查发现原规范未明确双键映射一致性及 quarantine ACK/存储失败路径。 |
