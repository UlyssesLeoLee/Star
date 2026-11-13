# Brief: WBS-W16-OUTBOX-OWNER-CONTRACT-20261107

**Agent**: design-engineer
**Phase**: WBS-W16-OUTBOX-OWNER-CONTRACT-20261107
**Created**: 2026-11-07 02:17:47

---

## 目标
按已完成的 W16 owner-contract review，修订 W16 的需求、基本设计、详细设计与测试设计，冻结设备撤销事实的 owner-local Outbox、Collaboration owner-local Inbox 与幂等应用、回放/cursor/gap recovery 和授权失效语义。仅做设计及文档追踪，不改产品代码/迁移，不声称实现或目标环境验收已完成。

## 当前基线与结论
目标本地 dev 基线：996ae3ae7d6e4c9c075715f3799f5186734d32f0。已知 review：docs/reports/PHASE-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261106.md。Identity/Auth owner 是设备 generation/status/revoke/audit 与该状态转移 Outbox 的唯一事实 owner；Collaboration owner 通过认证消息接收，以自己的 Inbox/事务更新失效投影，并可产生自己的 metadata-only Outbox。两者不共享事务，不跨 owner 写表。事件只带版本化 opaque 身份、generation、稳定 event identity、owner-local source sequence、correlation/idempotency 与时间信息；不带 token、私钥/证明、正文、明文 DEK 或敏感请求体。至少一次 delivery，接收端事务提交后确认；same identity/same digest 是 exact replay，same identity/different digest 隔离告警；cursor 与 provider/profile 绑定，不跨 provider 比较。Gap、过期 cursor、保留窗口缺口或 owner 不可用时，对应授权投影 fail closed，并有界重放/状态校验；事件只能加速失效，不能替代每次业务操作的当前授权复验。

## 范围
只编辑：docs/requirements/SRS-W16-PAYLOAD-SECURITY-001.md、docs/design/BD-W16-PAYLOAD-SECURITY-001.md、docs/design/DD-W16-PAYLOAD-SECURITY-001.md、docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md，并在 docs/reports/PHASE-WBS-W16-OUTBOX-OWNER-CONTRACT-20261107.md 写阶段/差异/缺口，在 docs/reviews/WBS-W16-OUTBOX-OWNER-CONTRACT-REVIEW-20261107.md 独立核对同一改动。保留历史修订和已运行测试记录；仅把未运行/候选 acceptance 标为设计或 NOT_RUN。不要修改共享 WORKTREE-GROUP-NEXT-WBS 文件：其与现存 Rust lane 有 manifest 重叠，协调者稍后另行同步。Brief 与 dispatcher audit 由已登记路径记录。

## 验收
1. SRS/BD/DD/TD 对 producer owner、consumer owner、event identity/schema、至少一次、Inbox-before-ack、重放、游标、gap recovery、当前授权复验、安全载荷与资源上界完全一致，并能追溯需求→设计→测试。
2. 不把 pg_notify/SSE 当 durable transport，不宣称 exactly-once，不让 Collaboration 写 Identity 表或 Identity 共享协作表。
3. 测试设计覆盖同摘要重复、同身份异摘要、并发重复、接收事务 rollback/ACK 顺序、重启恢复、cursor 过期/gap、目标 owner 不可用、tenant 隔离、撤权后既有 SSE 关闭、Neon 与 Multipass/K3s 两实例路径的区别；未运行项明确 NOT_RUN。
4. 对照 2026-11-06 review 与当前规范，列出差异。只做文档静态检查、链接检查、diff check，不运行 Cargo、数据库、云或 VM 测试。
5. 只提交 manifest 中的文件；不 rebase、merge、push、tag/release、清理任何 branch/worktree。协调者独立复审后再决定集成。

## 保留边界
测试阶段 Multipass 最多 2 个 inventory 实例；不使用 Docker Desktop 作为前置条件。Vercel 使用 Neon；本地/CI K3s 由 Multipass 管理。用户终端文件与其他工作树均不可触碰。

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
