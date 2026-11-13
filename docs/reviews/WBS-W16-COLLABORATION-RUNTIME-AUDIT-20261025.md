# WBS-W16 多用户低消耗协作运行闭环审计

**日期：** 2026-10-25
**审计基线：** dev `3c00634d1317acb78ac48043c0cfb9339ec8b709`
**范围：** 只读审计 Group/Canvas owner API、事件 Outbox、BFF 推送、前端 cursor 与 WBS；本次没有修改源码、schema、WBS 或设计基线。

## 1. 目标与检查边界

目标设计要求：资源级授权与 revision/CAS；同字段冲突时保留双方精确 draft 并用稳定 `conflict_id` 通知双方；双方可读取有界差异；事件由 owner commit 唤醒 durable Outbox relay；跨实例有界 fan-out；cursor 用于 bootstrap、gap、reconnect 与恢复；撤权、RLS、retention 和 p99 均须有可运行证据。连接期间不能依靠固定周期轮询，也不能用进程内广播替代 durable delivery。

CodeGraph MCP 返回 `Transport closed`，本仓当前没有可用的 CodeGraph 索引；本审计使用限定路径的源码检索和行号核对。WBS 与设计文档中的目标用于验收标准，不作为实现证据。本次没有运行编译、测试、迁移、PostgreSQL、gateway、多实例或性能测试。

## 2. 证据矩阵

| 验收面 | 状态 | 证据与判断 |
|---|---|---|
| Canvas resource authorization 与 revision CAS | **部分实现** | REST 读路径校验 Worktree/Canvas scope：`crates/star-api-rest/src/group_api/canvas.rs:471-480`。文档与元素写入携带 `expected_version`，不匹配时拒绝：`:842-845,1145-1174,1301-1371`。这是单资源的当前版本写保护，不能证明两份并发完整 draft 已进入语义冲突处理事务。 |
| 双方 draft 的 seal/pin、stable `conflict_id` | **缺失** | Canvas migration 有当前实体、audit 与 event outbox，但审计范围内没有双边 proposal/conflict/unread 表：`db/migrations/2026-09-29-worktree-group-phase-3-canvas.sql:20-165`。WBS 仍把双边 draft/conflict 保存列为 OPEN：`docs/implementation-plans/WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:80,369,385`。 |
| 有界 base/current/mine/theirs diff | **缺失** | 当前 event page 是有界 `after_offset` 元数据分页：`canvas.rs:471-533`；没有冲突双方 diff 的读取 API。WBS 要求第一批冲突通知携带可读的有界差异：`WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:80,337,344`。 |
| 双方持久未读提醒 | **缺失** | 本次限定的 owner/API/schema 路径未发现参与者冲突 Inbox；WBS 仍将离线 conflict inbox 与双边持久提醒列为未完成：`WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:80,385`。 |
| owner-local transactional Outbox | **部分实现** | Canvas `append_event` 使用调用方事务，将 Audit 与 Outbox 事件一起写入，并按 Worktree stream 串行化 offset：`canvas.rs:1796-1841`。`stream_offset` 另有持久列与索引：`db/migrations/2026-10-05-worktree-group-canvas-stream-offset.sql:13-20`。这证明源代码设计，不证明迁移已部署或 PostgreSQL 原子提交已实测。 |
| owner commit wake-up / durable relay | **缺失** | 限定代码范围内没有找到 owner commit 后的 wake-up latch、持久 relay consumer 或从 Canvas Outbox 到 BFF hub 的生产发布调用。当前 Group 投影会启动定时 poller：`frontend/src/lib/group/groupProjection.tsx:189-216`；WBS 也将 connected polling 与缺少 relay 标为 OPEN：`WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:80,337,343`。 |
| 跨实例 relay 与 fan-out | **缺失** | BFF `CollaborationWssHub` 持有单进程 `Arc` 中的 Tokio broadcast sender，容量 1024：`bff/src/collaboration/wss_hub.rs:41-82`。限定源码搜索只在 hub 测试中找到 `.broadcast(...)` 调用，未发现 owner Outbox relay 发布者。WBS 明确要求 WLD-6 durable cross-instance delivery：`WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:337,343,379-386`。 |
| 订阅队列与慢消费者背压 | **部分实现** | Tokio broadcast 缓冲容量有限；消费者落后时会收到 `Lagged` 提示，但被跳过的帧不保存在 durable subscription queue 中：`wss_hub.rs:41-45,249-257`。没有 measured Collaboration Profile，也没有跨实例慢消费者恢复证据。 |
| Cursor bootstrap、gap 与恢复 | **部分实现；连接期仍固定轮询** | REST cursor 接口支持 `after_offset` 与 1–100 条有界页：`canvas.rs:471-533`。浏览器 cursor store 使用 localStorage；poller 默认 1 秒间隔、page size 100，持续调用事件 API 并刷新投影：`frontend/src/lib/group/worktreeGroupApi.ts:1262-1394`，由 `groupProjection.tsx:199-216` 启动。WSS Lagged 后 durable catch-up、过期/gap 响应与 cursor recovery 尚无证据；30 秒 Worktree 页面定时调用是插件目录刷新，见 `frontend/src/app/worktree/[id]/group/page.tsx:952-964`，不能与 Canvas poller 混为一谈。 |
| 撤权、RLS 与 retention | **部分实现 / DB 验收未知** | REST 事件读取重新检查资源授权：`canvas.rs:471-480`；Canvas migration 声明 ENABLE/FORCE RLS：`phase-3-canvas.sql:167-288`。BFF WebSocket 使用进程内角色/权限快照并在连接时检查：`bff/src/collaboration/permission.rs:79-97`；未见每条资源事件发出前重新授权的证据。本 migration 范围内没有 Outbox retention policy。目标 PostgreSQL 的 grants/RLS、撤权传播与 retention 均未运行验收。 |
| 1 秒 p99 与真实 workload/Profile | **缺失 / NOT_RUN** | WBS 要求固定 Collaboration Profile、真实 workload 与目标运行环境，并把 PT-09 作为独立 p99 测量：`WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:80,337,344,386`。当前没有测量数据或通过结果。E2E-COLLAB-09 的 ≤1000 ms 是第二份完整 proposal 到达冲突 owner 的接收/入账门；PT-09 的 p99 ≤1 s 是冲突 commit 后双方收到提醒并首读有界 diff 的另一项门，两者不得混用。 |

### 验证证据分层

- 源码存在：上述 CAS、Outbox 插入、分页 cursor 与进程内 hub 源码存在。
- 测试资产：Canvas cursor 校验和前端 poller 有测试源码；WSS hub 有单元测试源码。该事实不等同于本轮测试通过。
- 本轮运行：没有运行测试或编译；没有迁移目标 PostgreSQL、RLS/grants、双实例 gateway 或断线恢复验证。
- 生产闭环：不能根据 API/迁移/测试源码存在而宣称 durable conflict、跨实例 fan-out 或 1 秒 p99 已可用。WBS 将 WLD-7 标为 OPEN/NOT_RUN：`WORKTREE-GROUP-NEXT-WBS-2026-10-18.md:80-82,348`。

## 3. 下一批实现 lane 与验收顺序

1. **WLD-2 + WLD-6 前置 lane：owner event contract 与 durable delivery。** 先冻结资源授权/fence、事件身份和 owner-local Inbox/Outbox 事务；实现可恢复、幂等的 relay、commit wake-up 与多实例 fan-out。验收：owner commit 后 relay 可重启恢复；接收端事务提交前不确认；重复事件幂等；两 gateway 收到相同授权事件；慢消费者有界并能 cursor catch-up。不要先用前端轮询或单进程广播替代。
2. **WLD7-C1：双边 proposal/conflict owner。** 依赖 WLD-2 owner API 与 WLD-6 durable delivery。增加版本化完整 proposal、双边精确 draft seal/pin、稳定 `conflict_id`、双方持久未读提醒、参与者授权的 bounded diff 与 expected-revision CAS resolve。验收：同时到达的重叠 proposal 都被接受进同一个 conflict；任何一边失败时两份草稿仍可读；未经参与者授权读不到对方内容；重放不会覆盖草稿或重复创建提醒。
3. **WLD7-C2：低消耗推送与恢复验收。** 依赖 C1 和已测 Collaboration Profile。实现连接期由 commit wake-up 驱动的资源级推送、有界 coalescing/queue、撤权、WSS lag cursor recovery 与断线重连。验收：E2E-COLLAB-09 证明第二份完整 proposal 在 ≤1000 ms 到达 owner 时不被 holdback 且进入冲突事务；E2E-COLLAB-10/11 验证双方提示与授权 diff；双 gateway/restart/gap/revoke 可恢复；PT-09 单独测量 Profile 下冲突 commit 至两方提醒及首个可读 diff 的 p99 ≤1 秒，并记录 RSS/CPU/队列水位。数据库与性能未验收前保持 OPEN。

## 4. 已知限制

- 负向结论限制在本次限定的 Group Canvas、BFF collaboration、migration、测试与 WBS 范围；无法据此证明仓库所有其他 owner 都不存在可复用能力。
- 本次没有证明 schema 已应用于目标数据库；源码事务边界不等于实际 PostgreSQL transaction/RLS 成功。
- 当前前端 CanvasOutboxPoller 是连接期间周期轮询实现；验收设计要求改成 commit-triggered push，将 cursor catch-up 限于 bootstrap、reconnect、gap 和恢复。
- WLD-7 与总体 WBS 仍 OPEN；未关闭任何运行时或产品验收门。
