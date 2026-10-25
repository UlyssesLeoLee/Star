# WLD-7 多用户实时协作目标数据设计

> 状态：目标逻辑数据设计 v0.4；尚未生成迁移、部署或目标数据库验证。
> 适用：Task Card / WorkItem 与 Canvas 等支持协作编辑的 Run 资源。
> 上游：requirements v7.08 §50.8AW / AC-COLLAB-01–08、basic design v7.12 §16.78、Group DD §8.27、domain collaboration spec v1.2 §13、test design v1.7 E2E-COLLAB-09。
> 此文件定义 owner-local 数据边界，不代表实时协作已实现。

## 0. 所有权和事实边界

WorkItem owner 保存任务字段的 revision、draft、conflict、participant notice、view receipt 和 resolution；Canvas owner 保存画布文档/元素的对应事实。每个 owner 只在自己的数据库事务内改自己的表，并在同一事务写 Audit 与 owner-local Outbox。Collaboration runtime 只负责连接、资源授权订阅、Presence 和事件路由；它不得复制 Task/Canvas 正文或直接写入两个业务 owner 的表。

以下记录是在 WorkItem 和 Canvas 两个 owner schema 中分别采用相同逻辑契约的目标结构。资源 ID 必须由 owner 校验；可空 worktree_id 允许尚无 checkout focus 的 Run 资源。应用前须按目标物理 schema、现有 canonical revision/outbox 实现完成迁移映射，禁止把这些目标表计入已部署表清单。

## 1. 目标记录与 W/T/M 分类

| Owner-local 目标记录（每个 owner 各一份） | W/T/M | 主要字段与唯一约束 | 事实与保留 |
|---|---|---|---|
| edit_draft_head | W | tenant_id、project_id、engineering_run_id、可空 worktree_id、resource_id、actor_id、device_id、draft_id、draft_revision、base_revision、changed_path_keys、encrypted_payload_ref、payload_digest、last_operation_id、last_request_digest、updated_at、expires_at；每 actor/device/draft 仅一条当前 head | 增量以 upsert/coalesce 更新，不为每次击键追加一行；按 Project retention 到期清理。 |
| edit_draft_checkpoint | W | checkpoint_id、资源 scope、actor/device/draft identity、draft_revision、base_revision、changed_path_keys、payload_ref、digest、byte_length、operation_id、sealed_at、retention_until；proposal identity 唯一 | 普通增量只合并更新 draft head；不为每次键击或每个 250ms delta 追加 checkpoint。只有显式持久化检查点、安全 flush 或冲突事务需要固定提案时才 seal 不可变快照；冲突引用时由 FK/引用保护禁止 TTL 删除。未引用旧 checkpoint 按策略清理。 |
| edit_conflict | T | conflict_id、资源 scope、base_revision、current_revision、按稳定规则排序的两侧 checkpoint_id、overlap_path_keys、overlap_detector_version、created_at、pair_key | append-only。canonical resource + 有序 proposal identity 唯一，确保重试/并发 detector 得到同一 conflict_id；正文留在被 pin 的 checkpoint/blob 中。 |
| edit_conflict_participant | T | conflict_id、actor_id、device_id、created_at；(conflict_id, actor_id, device_id) 唯一 | 在 conflict transaction 中为两个参与者各写一条；提供离线时仍存在的持久未读提醒事实，不因 socket ACK 变化。 |
| edit_conflict_view_receipt | T | conflict_id、actor_id、device_id、operation_id、viewed_at；按 participant 幂等 | 用户实际打开冲突后 append-only 写入；UI unread 由 participant notice 且无有效 view receipt 推导，传输 ACK 不产生该事实。 |
| edit_conflict_resolution | T | resolution_id、conflict_id、resolver_actor_id、expected_revision、result_revision、resolution_digest、resolution_method、suggestion_algorithm_version(nullable)、user_confirmed_at、operation_id、created_at | append-only 的显式解决事实；旧 conflict 不更新覆盖。过期 CAS 产生 successor conflict 并保留原双方草稿。 |
| owner transactional Outbox event | T | event_id、stream_id、sequence、schema_version、resource/revision、conflict_id、participant IDs、correlation/idempotency IDs | 沿用事实 owner 的 Outbox 与事务，不另建 Collaboration 业务 outbox；正文 patch 不进入广播事件。 |

`created_at`/`occurred_at` 由 owner 服务端在事务中生成；冲突排序以 owner revision、stream sequence、resource fence 与提案身份为准，不以客户端时间戳排序。两份完整提案在服务端 1000 ms 内到达只是并发验收条件，不对应数据库字段、冲突时间窗或延迟任务。

已有 collaboration.presence 与 collaboration.realtime_subscription 仍是 W 类短 TTL 观察/连接数据；它们不是 revision、草稿或冲突真相。普通 revision/history 如果尚无 owner-local durable 表，应先纳入该 owner 的 Transaction data design，再由 owner 决定映射。

## 2. 共同列、索引与一致性约束

所有 owner-local 行都带 tenant_id 与 canonical Project/Engineering Run/resource identity；适用时保存 worktree_id 作为被校验的上下文，不把它当授权 token。actor_id/device_id 必须从认证上下文取得；客户端传入值不能决定 RLS、参与者或收件人。所有唯一键同时含 tenant 与 owner resource identity，禁止跨租户碰撞。

- draft_head 使用唯一键 (tenant_id, resource_id, actor_id, device_id, draft_id)，只保存最新 coalesced head；draft_revision 单调增加，base_revision 保持该 draft 的起始 owner revision。
- draft_checkpoint 使用唯一键 (tenant_id, resource_id, actor_id, device_id, draft_id, draft_revision)。sealed checkpoint 内容和 digest 不可变。edit_conflict 对两侧 proposal identity 排序后建立唯一约束；保存 checkpoint 引用，删除保护必须拒绝删除 unresolved conflict 所引用的草稿。
- 所有变更命令必须使用 owner-local 幂等回执机制：优先复用 canonical owner 现有账本；若当前 owner 尚无对应能力，则在该 owner 边界内补齐。事务内固定 `(tenant, actor, operation, idempotency_key, request_digest, result_ref)`；同键同 digest 返回原回执，同键不同 digest 拒绝。不得新增跨 owner 的 Collaboration 幂等事实库。
- Overlap 查找先按 tenant_id + resource_id 限定，再用 changed_path_keys 的 GIN 或等效 owner 索引筛选；不得扫描全租户草稿。活跃 draft/编辑器预算由版本化 Collaboration Profile 限制，达到上限时保留本地稿并显式返回容量状态，不降低冲突检测完整性。
- Conflict detector 不按一秒桶聚合，不依据 client timestamp 延迟判定。第二份完整且 digest-verified 的重叠 head 在 resource fence 下提交冲突；任何更长间隔仍按有效 revision/draft 语义检测。1 秒仅用于 E2E 的同秒到达刺激及 PT-09 的 commit-to-first-diff p99 测量，不落为一个 holdback 或冲突 deadline。
- 语义重叠由独立的 overlap_detector_version 记录并强制生成持久 conflict；是否存在可执行 OT transform 不改变冲突判定。resolution_method 只有在显式 resolve 命令中由用户选择后才可记录；suggestion_algorithm_version 仅记录预览算法，生成预览不得修改 canonical revision、sealed checkpoints、participant unread 或 conflict 状态，最终写入仍由 owner 的当前 revision CAS 决定。
- Draft checkpoint/seal、revision CAS 和 conflict/resolve 共享同一短时 resource fence；网络收块、上传与差异读取均在 fence 外。不同 resource 使用独立 fence 可并行。
- 每个资源最多生成一个有序 proposal pair 的 conflict；冲突摘要只保存必要重叠 path/key。resolution、view receipt 与 audit 均为追加事实，不原位改写历史。

## 3. 内容存储、分块与限额

patch 以有界 delta 传输；单帧不超过 16 KiB，序号、总帧数和 digest 必须校验。未完成上传只保存在作者隔离的短期 receiving state，不能生成对外 conflict。常态 delta 只 coalesce 到 draft head；当两个完整 draft head 出现重叠，owner 在同一个 resource-fence transaction 中再次核验帧连续性与 digest，并一次性固定两侧精确 revision 为 immutable checkpoint，再提交 conflict/participant/Outbox facts。用户无需按最终 Save 才能被告知冲突；未完整、未验证或仍在接收的 head 不能被 pin 或公开。冲突读取通过 owner API 获取有界差异页，不能把正文或每个 participant 的副本塞入实时 fan-out。

应用层必须从版本化 profile 固定总 draft bytes、同时活动 draft 数、frame 数、差异页大小、设备缓存总量与 retention 上限。没有这些预算、目标对象存储能力或加密密钥时 fail closed 为 local_only / sync_pending，不得声称已共享保存。Blob 以 tenant/resource 作用域加密，DB 只保存引用、digest 与经授权查询所需元数据；键轮换、孤儿 blob 回收与备份恢复须和 owner retention 一致。

## 4. RLS、应用角色与查询路径

- 各 owner 表启用并 FORCE RLS；owner runtime role 只能写本 owner schema。Collaboration gateway 无权直接读写 draft/conflict 内容。
- ReadConflict 先由 canonical owner 复验当前 Project/Run/Worktree binding、参与者/显式 resolver 与资源 capability，再按 conflict_id 读取两侧 pinned checkpoints；越权用户不能区分 conflict ID 是否存在。
- draft owner 可读取自己的 draft；两名冲突参与者可读 conflict 所 pin 的两侧 proposal。其他 Run 成员不能因此看到未冲突私有草稿。
- 冲突事件与 WebSocket payload 仅带稳定 ID、revision、sequence 和脱敏摘要。BI、Plugin、普通 Audit/telemetry 只接收批准的 metadata/evidence reference，不接收 patch text。
- 目标库迁移需验证应用角色 grants、FORCE RLS、跨 tenant/Project/Run 负向读取、事务回滚、幂等 replay 和 backup restore；仅本地内存 hub 或测试角色成功不放行生产。

## 5. 低资源写放大与恢复

每个窗口复用一个有界 multiplexed connection；每份 draft 同时最多一个在途网络写入，未发送输入合并为单个受 Project/profile 预算约束的 pending delta；超过预算时保留本地稿并显示 `sync_pending`，不扩展为无界请求队列。正常 edit 仅合并写 owner draft head；只在显式 flush/安全检查点或冲突 seal 时固定 immutable checkpoint；commit Outbox 由 transactional wake-up 合并唤醒，不逐订阅者写业务副本或运行周期扫表。普通 edit event 含 delta metadata，不含全文档；冲突参与者 notice 只对两名参与者各记一条事实。presence/cursor 可 coalesce 和 TTL 删除。

Outbox 仍是持久真相，wake-up 只提示 relay 立即 claim。Relay 重启先从 opaque durable cursor/backlog catch up；慢消费者达到既定 count/byte caps 后断开并用 cursor 补齐，不能静默跳过事件。对 conflict 的任何跨实例送达失败都保留 participant unread fact，直到客户端读取/显式处理；cursor、ACK 和 socket 关闭不删除 draft/conflict。

## 6. 迁移前验收与实施状态

WLD-7 实施时必须把以上目标实体逐表对齐 Data Inventory、W/T/M、数据库迁移、owner grants、RLS、retention、blob 生命周期和 schema fingerprint。需要并发证明 draft seal 与 revision CAS 同 fence；E2E-COLLAB-09 用两 actor 完整重叠提案在 ≤1000 ms 到达的刺激验证第二份提案不会等待一秒时间窗，并确认双方草稿精确 pin、同一 conflict_id 与双方首个有界差异可读；两个 gateway 与丢失 wake-up 仍恢复同一 conflict_id；participant view receipt 幂等、未授权第三人读失败；以及容量/RSS/CPU/commit-to-first-diff p99 基准（p99 由 PT-09 单独测量）。每个表的 physical inventory 行和 IPA table detail 只有在具体 DDL 确定后追加；本文件不声称当前已部署这些表。

修订 v0.1 · 2026-10-24 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。依据：用户要求事件触发推送、双边提示、保留双方草稿、可查看对方改动且降低轮询与全量广播开销；单 draft 单在途写入与一个有界合并待发增量限制慢网络下的内存/请求增长。所有结构均为 WLD-7 目标设计，实施/目标库验收开放。

修订 v0.2 · 2026-10-24 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。触发：将 Basic Design 上游引用从重复编号 §16.56 更正为实时协作章节唯一编号 §16.78；旧 §16.56 仍指 Schedule D5。数据模型和未实施状态不变。

修订 v0.3 · 2026-10-25 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。触发：为 owner-local 协作记录规定服务端时间与 revision/sequence 排序权威；明确 ≤1000 ms 只作为同秒 E2E 刺激，不形成数据库冲突窗口；要求完整重叠提案立即在 resource fence 内建立双方 pin 与冲突事实，并单独通过 PT-09 测量 commit-to-first-diff p99。同步 requirements v7.07、basic v7.11、collaboration spec v1.1、test design v1.6。逻辑目标未迁移或实施。
修订 v0.4 · 2026-10-25 · Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核。新增独立 overlap_detector_version，明确 OT 可变换不豁免语义冲突；显式解决时可记录无副作用 OT suggestion algorithm 与用户选择，canonical 写入必须通过 owner 当前 revision CAS。同步 requirements v7.08、basic v7.12、collaboration spec v1.2、test design v1.7。目标逻辑 schema 尚未迁移、部署或验证。
