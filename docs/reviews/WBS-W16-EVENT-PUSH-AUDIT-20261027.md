# WBS W16 低消耗实时协作现状审计

**审计日期**：2026-10-27
**基线**：`dev@25c3a2d4515b5287e52567e9ae768b12d9d2dd8f`
**结论**：W16 保持 **OPEN**。当前存在可读的 Canvas Outbox API、浏览器游标提示及 BFF WebSocket 骨架，但没有证据表明它们已组成满足 W16 验收条件的端到端实时协作闭环。

## §0 目的与范围

本报告对照《Worktree Group Next WBS》W16，审计事件唤醒与推送、游标恢复、双方草稿及冲突闭环、跨实例投递和资源消耗设计。检查范围为本 lane 基线中的 WBS、`DD-WORKTREE-GROUP-001.md` §8.27、Group 前端投影及 Canvas Outbox 读取代码、Canvas API 读取处理器和 BFF Collaboration WebSocket 实现。

这是限定范围的源码与设计文档审计。没有运行测试、构建、数据库、浏览器、运行时或性能测量；因此本报告不声称任何运行验收通过，也不把未检查范围的情况推断为全仓库不存在。

## §1 证据矩阵

| W16 能力 | 观察到的实现证据 | 评估 |
|---|---|---|
| Canvas Outbox 事件读取与授权 | `list_worktree_canvas_events` 通过 `canvas:read` 与 Group scope 授权，按 `stream_offset > after_offset` 排序分页读取，并将结果和 `next_offset` 返回；API page limit 上限为 100。[`canvas.rs:471–533`](../../crates/star-api-rest/src/group_api/canvas.rs) | **部分实现**：存在有界读取及请求时授权，不等于服务端唤醒、push 或断点完整性验证。 |
| 浏览器正常路径 | 选中并订阅 Canvas 的 Group projection 创建 `CanvasOutboxPoller` 并启动 `run()`。[`groupProjection.tsx:199–215`](../../frontend/src/lib/group/groupProjection.tsx) 默认间隔为 1,000 ms、页长 100；每轮请求 Outbox 页面，刷新授权投影后才保存游标；成功的小页等待固定间隔，失败退避至 30 秒。[`worktreeGroupApi.ts:1298–1395`](../../frontend/src/lib/group/worktreeGroupApi.ts) | **与目标不符**：当前浏览器正常路径依赖固定间隔轮询；尚非由 owner commit 唤醒的 push。 |
| 客户端游标 | `BrowserCanvasEventCursorStore` 把游标写到 `localStorage`，代码注释明确它是“不可信恢复提示”，每次 poll 仍做授权；存储不可用时只在当前会话保留内存游标。[`worktreeGroupApi.ts:1269–1296`](../../frontend/src/lib/group/worktreeGroupApi.ts) | **部分实现**：有客户端 offset 提示及刷新后推进游标；没有观察到 owner 持久投递游标、明确 gap 检测/修复或一致快照加游标恢复实现。 |
| BFF 实时传输 | `CollaborationWssHub` 是容量 1,024 的 Tokio 进程内 broadcast，由 `Arc` 状态持有；超出容量的慢接收端会收到 `lagged` 提示，事件已可能丢失。[`wss_hub.rs:41–83, 207–283`](../../bff/src/collaboration/wss_hub.rs) | **仅有传输骨架**：进程内易失 fan-out 不是业务事实或持久恢复机制。 |
| BFF 事件发布接线 | 在本次有界源码/调用点检查中，`broadcast()` 的可见调用为 hub 定义及同文件单测；未观察到生产发布调用。 | **未证实接线**：不能据此作全仓库不存在的断言，但当前证据不足以证明 Outbox owner commit 已驱动该 hub。 |
| WebSocket 连接及范围检查 | 四个 Canvas WebSocket 路由分别要求 `View` 权限及非空 tenant 后升级连接。[`wss_hub.rs:92–175`](../../bff/src/collaboration/wss_hub.rs) | **部分实现**：有连接时检查；设计要求的单个多路复用连接、按资源订阅授权、活跃订阅撤权关闭及重放再授权未由这些处理器证明。 |
| WebSocket tenant 过滤 | `event_tenant()` 对各事件变体实际返回 `canvas_id`，而 socket handler 将该值与 `state.tenant_id` 比较，不匹配时跳过事件。[`wss_hub.rs:209–237, 285–295`](../../bff/src/collaboration/wss_hub.rs) | **已观察到身份字段不匹配/过滤缺陷**：常见的 `canvas_id != tenant_id` 情形会导致合法 Canvas 事件被过滤掉。此证据说明过滤身份使用不匹配，不据此声称发生跨租户泄漏。 |
| 双方草稿与冲突事实 | 在限定搜索范围 `frontend/src`、`bff/src`、`crates/star-api-rest`、`db/migrations` 中，针对 `edit_draft_head`、`edit_conflict`、`conflict_id`、draft checkpoint、participant/unread conflict 等词未发现持久草稿/冲突 API 或 schema 命中。详细设计把对应记录明确列为 target schema，并说明尚非已部署 inventory。[`DD-WORKTREE-GROUP-001.md:1344–1358`](../design/DD-WORKTREE-GROUP-001.md) | **未见实现证据**：这是有界检索结果，不代表其余未检查模块必定不存在；W16 不能据此关闭。 |
| 跨实例唤醒与恢复 | 现有 BFF hub 状态为单进程 `Arc`；WBS 把跨实例 fan-out 和 cursor recovery 列为验收。详细设计规定 owner Outbox、重启后从 durable facts 对账和重放。[`DD-WORKTREE-GROUP-001.md:1301–1305, 1338–1344`](../design/DD-WORKTREE-GROUP-001.md) | **未见生产接线或验收证据**：进程内广播不能证明实例间传播、重启恢复或重复投递幂等。 |
| 冲突提醒和时延 | WBS 要求按 base revision 与重叠范围判冲突，持久保留双方 exact draft，并向双方给出相同 unread `conflict_id`；在线目标为从冲突事务提交至双方可见提醒和首个有界 diff 页 p99 ≤ 1 秒。[WBS §2.1 W16](../implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md) | **未实现/未测量**：本次没有发现冲突闭环的实现证据，也没有时延、并发、资源或故障恢复测量。1 秒是端到端验收目标，不是轮询周期或冲突检测窗口。 |

## §2 审查摘要

现有前端确实会周期读取 Canvas Outbox：默认每秒一次，最多每页 100 条；提交后的游标提示保存在浏览器本地，失败时退避。Canvas API 在每次读取时做 scope 授权，并按 offset 分页。这些属于可复用的读取与基础恢复材料，但与 W16 要求的“无固定轮询、owner commit 唤醒、持久跨实例 fan-out、gap 检测与恢复”仍有明确差距。

BFF 有四个分开的 WebSocket 路由和一个有界的进程内广播通道。当前观察到的慢消费者处理会报告丢失事件后继续接收，并不提供从 owner durable Outbox 恢复。当前范围内还未观察到生产者把 owner 事务性 Outbox 接到 hub。连接权限检查、事件资源范围过滤和未来的多路复用订阅仍需按 Run/Worktree/资源授权契约核对。

《DD-WORKTREE-GROUP-001.md》§8.27 已表达目标 owner 边界和验收：Canvas/WorkItem owner 持有 revision、CAS、草稿、冲突与参与者未读事实，并在同一 owner 事务写 Audit/Outbox；协作网关只持有受限连接、订阅和 fan-out 状态。文档同时标明该设计仍是 design-only。故本次结论是 **部分基础能力存在，端到端目标没有验收证据，W16 未关闭**。

## §3 已知缺口与风险

1. **固定轮询仍处于正常路径**：`CanvasOutboxPoller.run()` 成功空页后等待 interval；该路径需要由 owner commit 唤醒加有界 push/replay 取代，不能通过调大 interval 宣称满足低消耗实时协作。
2. **传输与业务事实尚未连通**：进程内 broadcast 的容量上限和 `lagged` 通知不构成 durable delivery；跨进程/实例、Outbox claim、重试、幂等以及重连追赶都没有本次验收证据。
3. **恢复游标边界不足**：localStorage offset 是可丢失、可篡改的客户端提示，不是授权凭据或 owner 的持久 delivery receipt。现有摘录未展示连续性 gap 检测、快照与 cursor 原子对齐、过期 cursor 恢复等完整流程。
4. **草稿/冲突闭环没有证据**：未证明双方草稿 exact revision、冲突 pin/retention lock、共同 `conflict_id`、参与者 unread receipt、base/current/mine/theirs 有界读取或显式 CAS resolve 已落地。详细设计所列 schema 仍是计划。
5. **WebSocket tenant 过滤存在身份字段不匹配**：BFF 的 `event_tenant()` 返回 `canvas_id`，handler 却把它与 `tenant_id` 比较；当两者不相等时，合法事件会被过滤。应让事件明确携带 tenant 身份，并在 tenant 验证后按授权资源范围过滤，同时增加“tenant 与 canvas ID 不同仍可收到本资源事件”及跨 tenant/非参与者拒绝测试。当前证据指向事件被误过滤，不证明跨租户泄漏。
6. **资源预算和 p99 未验证**：1,024 broadcast capacity 是固定进程内消息数，不是按连接总字节数、订阅数或 RSS 设定的容量证明。WBS 中 `100 subscriptions`、`64 events / 128 KiB` 是待测候选值；本次未测 RSS、CPU、队列背压、取消/drain、p99 或两实例恢复。

## §4 后续 owner 与交接清单

| 顺序 | Owner | 工作与完成证据 |
|---|---|---|
| 1 | Canvas / WorkItem 业务 owner | 实现 owner-local revision fence、CAS、草稿 head/checkpoint、冲突/解决事实、参与者未读状态及同事务 Audit/Outbox；保存双方精确草稿并定义 retention lock。提供迁移、权限/RLS 和幂等契约。 |
| 2 | Collaboration relay / Rust 服务 owner | 由 owner commit 触发有界 wake-up；对 owner Outbox 做持久 claim、重试与幂等转发。保证 relay/进程重启可从事实重建，传输 ACK 不改变 owner 冲突/未读事实。 |
| 3 | Gateway / API owner | 收敛为每窗口单个授权多路复用连接；按资源 scope 独立授权/撤权，限制订阅数、队列事件数和总字节数；慢消费者发送 `resync_required` 后从 durable cursor 追赶，不复制完整事件给每个订阅者。修复当前 `canvas_id` 与 `tenant_id` 比较造成的误过滤，明确事件身份字段契约，并覆盖 tenant 和资源隔离测试。 |
| 4 | Group 前端 owner | 正常状态停止固定间隔读取；仅冷启动、检测到 gap、重连或 `resync_required` 时按需 catch-up。展示双方持久 unread 提醒和同一 `conflict_id`，显示首个有界差异页，并支持恢复草稿、显式解决及离线后回到 conflict inbox。 |
| 5 | 验收 owner / DB 与 SRE | 在目标数据库及至少两个 gateway 实例验证 CAS/草稿 seal 竞态、RLS/非参与者拒绝、Outbox 丢唤醒/重复/乱序、重启/断线/过期 cursor/撤权恢复、慢消费者背压；按冻结的 workload 记录 p99、RSS、CPU、队列/帧字节和恢复结果。通过后才可更新 WBS 关闭状态。 |

## §5 守门规则

- owner 事务是 revision、draft、conflict、参与者 unread 和 Outbox 的业务真相；WebSocket、通知和 ACK 只是可丢失/可重放的传输状态。
- 游标仅作为不可信、非授权的恢复提示；每次订阅、写入、提交、重放、读取差异和 resolve 都按当前 Project/Run/Worktree/resource capability 复验。
- 常态不能用固定间隔轮询；wake-up 只触发有界 Outbox claim，允许合并唤醒但不能丢失已提交事实。重连及发现 gap 后必须按 owner durable facts 对账。
- 同一冲突的双方 exact draft 和 revision 必须持久保留。查看通知或收到 transport ACK 不得清除 unread；只能由明确的参与者读取/解决事实变更其状态。
- 内存边界以连接总量、事件数和字节数共同限制，并规定每草稿 in-flight、coalescing、取消/drain 和慢客户端行为；不可把固定的广播槽数当作 RSS 或容量验收。
- 1 秒是定义清楚的在线负载下从冲突事务 commit 到双方可读首个有界差异页的 p99 目标；它不是检测窗口、上传期限或轮询周期。没有目标环境测量时不得标记通过。

## §6 审阅与签署状态

| 角色 | 状态 | 结论/边界 |
|---|---|---|
| 架构 | 本次审计结论 | 保持 W16 OPEN；本报告不批准功能验收或关闭。 |
| SRE Lead | 未执行运行验收 | 两实例、重启、故障恢复、资源与时延均未测量。 |
| 平台 | 未执行数据层验收 | 未运行目标数据库迁移、RLS/grants、回滚或恢复验证。 |
| 评审主持 | 审计范围已复核 | 仅基于本报告列出的文件、调用点和定向搜索；负面结论均限定于检查范围。 |
| PM | 等待实现与验收证据 | 需按 §4 完成后再审阅 WBS 关闭条件。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-27 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 W16 有界现状审计；记录已观察实现、设计目标、缺口、owner 边界及未执行的验收项。 | WBS-W16-EVENT-PUSH-AUDIT-20261027 lane brief |
