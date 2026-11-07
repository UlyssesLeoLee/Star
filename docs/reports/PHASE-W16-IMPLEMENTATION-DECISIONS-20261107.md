# PHASE-W16-IMPLEMENTATION-DECISIONS-20261107

## §0 目的

在 `dev@89f2f5bd2f34a5977fdd843bbd9f2abc93c8ce66` 基线上，冻结 Identity/Auth 设备撤权 Outbox → Collaboration Inbox 实现前需要的接口、顺序、授权和数据所有权决定。本阶段只更新登记的设计/测试/留档文档；不新增 Rust 实现、migration、运行配置或验收结果。

## §1 改动矩阵

| 路径 | 修改 | 当前结论 |
|---|---|---|
| `docs/requirements/SRS-W16-PAYLOAD-SECURITY-001.md` | v0.9；generation/source sequence、持久 replay-floor、snapshot/replay API、service capability、lease、checkpoint、旧事件 receipt 与数据留存要求 | 目标需求；全新验收仍 NOT_RUN |
| `docs/design/BD-W16-PAYLOAD-SECURITY-001.md` | v0.9；owner-local 流程、writer readiness gate、oauth_refresh_tokens Work 与物理记录 W/T/M 分类 | 目标基本设计；未创建 DDL |
| `docs/design/DD-W16-PAYLOAD-SECURITY-001.md` | v0.9；versioned DTO、待验证的服务身份 adapter/capability、legacy writer drain/readiness gate、按行 replay-floor 初始化、同点 snapshot 水位、连续 checkpoint、延迟旧事件 receipt、lease、relations/indexes/RLS/grants | 目标详细设计；未实现 |
| `docs/test-design/TEST-DESIGN-WORKTREE-CANVAS-001.md` | v2.8；新增 UT/IT/E2E 对应断言、writer cutover/readiness race、READY 前 baseline 与 READY 后 replay-floor/事件边界、旧事件防回退，UT 10 项、E2E 24 项 | 全部新增用例 NOT_RUN |
| `docs/briefs/W16-IMPLEMENTATION-DECISIONS-20261107.md` | 目标、清单、限制与 Read COMMENTS 留档 | 已生成；明确说明首次只读 review 在 brief 之前发生 |
| `docs/reports/WBS-DISPATCH-W16-IMPLEMENTATION-DECISIONS-20261107.jsonl` | brief 与 review dispatch audit | 含实际发生的 dispatcher 记录 |
| `docs/reviews/W16-IMPLEMENTATION-DECISIONS-REVIEW-20261107.md` | 对需求/设计/测试的审查结论与开放事项 | 当前 v0.9/v2.8 设计一致性复审 APPROVE；没有运行时验收 |

设备授权流选择 `stream_id=device_id` 且 `source_sequence=device_generation`。只有 event contract 为 READY 后，每个授权相关 generation 推进才与一个同序号事件同事务提交；READY 前允许状态变化，但不发事件，既有状态归入首次 post-READY baseline，绝不补造历史事件。READY 前创建的 binding（包括 enrollment）保持 NULL floor，READY 后新 binding 才以 floor 0 + generation 1 activation event 同事务启动。版本化 readiness gate 只有在不兼容 writer/session drain 且旧凭据撤销后才可 READY；READY 前 API/consumer/floor 初始化关闭。存量 binding 由 READY 后首次 snapshot、首次授权变更或可选 backfill 在绑定行锁下惰性初始化为当前 generation G，mutation/snapshot/backfill 竞争不会产生无事件窗口；READY 后所有写入器经同一 gate 读锁及绑定行锁执行。既有设备首次 bootstrap 必须取得权威 snapshot；激活后出现无法解释的 gap 是完整性异常，不能被 snapshot 静默掩盖。snapshot 水位后的合法旧事件持久记录为 `superseded_by_snapshot`，不能回退投影或水位；普通 refresh-token 轮换不发设备授权事件。业务 API 仍每次实时核验；lease 只限 SSE/cache 新鲜度。

## §2 验证摘要

| 检查 | 结果 | 范围 |
|---|---|---|
| `git diff --check` | PASS | 已跟踪文档差异无空白错误或冲突标记 |
| 目标 manifest 文件路径 | PASS | 工作树仅含 MAGOS 登记的 8 个路径；没有 WBS/代码/migration 改动 |
| Markdown 相对链接 | PASS | 7 份 Markdown 文档检查 16 个链接，0 个缺失 |
| 注册文件静态格式 | PASS | 8 个登记文件 trailing whitespace=0；dispatch JSONL 解析错误=0 |
| Cargo / PostgreSQL / HTTP / Neon / Multipass / K3s | NOT_RUN | 文档阶段没有实现变更或新验收 |

历史局部 PG18/18.6 与单流 SSE 结果保持原样，不外推为 owner-event 闭环。新增 UT/IT/E2E 均是设计态，未执行；本阶段静态检查结果只说明登记文档自洽，不构成运行验收。

## §3 已知缺口

1. 尚无 owner-event Outbox/Inbox 代码、可靠投递、snapshot/replay handler、service capability gate、Collaboration checkpoint/projection、quarantine 或 migration。
2. 当前设备 repository 只证明 enrollment/revoke 的局部实现与数据库切片；撤销路径仍没有 Outbox append，尚未证明每个授权 generation 转换都发且只发一个事件。
3. 服务身份 profile、`identity.device_authorization.read` capability/audience resolver、versioned writer-readiness relation/rollout preflight 与面向目标部署的认证路由均未实现；transport 机制和现有 adapter 可复用性尚待核验。
4. W14 尚未批准具体 lease/page/queue/in-flight/retry/retention/RSS/CPU 数值；没有获批 profile 时不得启用 consumer。
5. 当前 device binding migration 仍原位修改 generation/status，未实现 Master SCD Type 2。新增 owner tables、readiness control relation、FK/index、非 superuser grants、FORCE RLS 与目标数据库验收都未实现。
6. T 事实不自动 TTL/删除；分区、备份和冷层归档的长期存储成本及可读恢复路径需要容量审查。任何归档都必须保留审计和 replay 能力。
7. Vercel/Neon 与 Multipass 管理的 K3s 需分别验证；本地测试最多两个 Multipass inventory 实例，不能据此声称高可用。

## §4 子代理失败接手清单

- 一次只读设计审查在 brief 文件落地前已完成；该次顺序不符合 dispatcher 先 brief 的治理要求，brief 已事后如实记录，没有回写成“事前授权”。
- 最终只读复审代理核对 SRS/BD/DD v0.9、Test Design v2.8 和阶段报告，给出 APPROVE；明确 pre-READY baseline、NULL floor、READY 后 floor=0/generation-1 event 与行锁竞态边界一致。代理未改文件。
- 协调者完成精确八路径 manifest、Markdown 相对链接、dispatch JSONL、diff whitespace 检查；这些是静态文档检查，不代表 Rust/数据库/API/部署验收。

## §5 守门规则

1. Identity/Auth 与 Collaboration 各写各自 owner 数据；不共享事务、不跨 owner 直接写表。
2. Outbox/Inbox 是至少一次传递加 owner-local 幂等，不宣称跨服务 exactly-once。
3. 两个 tenant-scoped replay identity keys 必须同一 Inbox row 且 digest 相同；单键冲突/分叉/异摘要进入 durable quarantine。
4. 正常事务或 quarantine intent 提交后才 ACK；持久化失败不 ACK，按 profile 有界重试/暂停。
5. device `source_sequence` 等于 generation；READY 前无事件并纳入 baseline；READY 后每次授权相关 generation 推进恰好产生一个同序号事件，重复撤销不增加序号。
6. Legacy consumer 必须通过 snapshot 建立 `replay_floor`；激活后的序号 gap 触发审计、停流和 fail-closed。
7. Snapshot 的 state 与 watermark 来自同一一致性读；Replay 按序并受 row/byte profile 限制。
8. 内部 API 只允许所选认证 adapter 验证的 Star service identity 且有 `identity.device_authorization.read` capability；tenant scope 来源于认证上下文。transport 机制须经实现阶段审查。
9. Agent、Plugin、任务 CLI 与普通用户不直接取得 Identity 内部读取 capability。
10. Opaque cursor 固定 provider/version/profile digest/endpoint/log generation；重试不换 profile，不跨 provider 比较 cursor。
11. Checkpoint 保存连续已应用水位；每条 stream 单飞，缺口不得以 MAX 跳过。
12. Identity/Auth 给撤权 freshness lease 最大值，Collaboration 只可缩短；lease 不替代业务请求实时授权。
13. Transaction 事实 append-only、不自动 TTL；Work lease/delivery/checkpoint/projection 需显式 owner、RLS 与 TTL。
14. 所有 tenant-bearing 关系 FORCE RLS；应用角色无 BYPASSRLS；无跨 owner 表级 grants。
15. Profile/预算缺失或目标数据库未验证时消费者 fail closed；通知仅作 durable log wake-up。
16. 不将单一 SSE/PG slice、文档合同或 unsigned run 证据扩成生产/多实例验收。
17. 本 lane 不修改 candidate、根工作区用户数据、main、远端，也不 push。

## §6 签字栏

| 角色 | 责任人 | 签批状态 | 说明 |
|---|---|---|---|
| 架构 | Mavis 接手终审 | 等待当前版本独立复审后签批 | 本文档不能替代生产实现审批 |
| SRE Lead | 尚未到位 | 未签 | 需确认留存、恢复和容量预算 |
| Platform Lead | 尚未到位 | 未签 | 需确认 transport/service identity 与 owner roles |
| 评审主持 | Mavis 接手终审 | 等待复审 | 核对 W/T/M、RLS 和跨 owner 边界 |
| PM | Mavis 接手终审 | 等待复审 | 范围为 W16 owner-event 实现决策 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 冻结 W16 owner-event API、顺序、授权、lease、checkpoint 与物理数据决策；同步 SRS/BD/DD/Test Design；新验收全部保留 NOT_RUN。 | W16 Outbox owner-contract review 指出实现前决策缺口。 |
| v0.2 | 2026-11-07 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 收紧 READY 前的状态事件边界：旧状态进入首个 baseline、不补造事件；READY 后新 binding 从 generation 1/floor 0 启动，pre-READY binding 保持 NULL；同步详细设计与 UT 验收刺激，未执行任何新测试。 | 最终一致性复核发现概括措辞可能误读为 READY 前也发布事件。 |
