# PHASE-W16-EVENT-PUSH-IMPL-20261027

> 状态：后端与前端事件唤醒切片已合入本地 `dev@95732e7c280a95560add99ab76609955fc172f08`；W16 整体保持 **OPEN / PARTIAL**。

## §0 目的

记录 Canvas Worktree 事件从固定轮询迁移为 owner Outbox 驱动唤醒的实现边界、集成证据和未完成验收。此阶段只交付变更提示与授权 catch-up，不把通知传输当作持久事实或协作冲突系统。

## §1 改动矩阵

| 项目 | 实现与集成状态 | 证据/边界 |
|---|---|---|
| PostgreSQL wake producer | 已实现事务触发器；Outbox INSERT 在提交时发仅含 Worktree UUID 的通知 | `db/migrations/2026-10-28-canvas-outbox-wakeup.sql`；通知不是 durable queue |
| Rust SSE adapter | 每个 API 进程一个 `PgListener`，固定广播队列和 stream 并发上限；wake 时重新核验读取权限 | `crates/star-api-rest/src/group_api/canvas_event_stream.rs`；业务正文仍通过授权 Outbox REST 获取 |
| 前端连接 | Canvas Worktree 一个复用 SSE 连接；wake/reconnect/resync 后 catch-up | 前端测试与 `pnpm typecheck` 在 rebase 后通过；单次/每 Worktree 连接，不以固定轮询维持更新 |
| 可恢复提示 | 丢失、lag、listener reconnect 产生 resync 提示 | 由 Outbox catch-up 恢复；不代表本切片已实现 draft/conflict 恢复 |
| 集成 | brief、前端与后端提交已快进到本地 dev | 本地当前 SHA `95732e7c280a95560add99ab76609955fc172f08`；未 fetch/push |

## §2 验证摘要

| 检查 | 结果 |
|---|---|
| `rtk proxy cargo check -p star-api-rest --all-targets -j 4`（在 Orca dev checkout） | 通过；存在仓库已有 warnings |
| `rtk proxy cargo test -p star-api-rest --lib -j 4 canvas_event_stream::tests` | 3 passed，0 failed |
| `rtk proxy cargo test -p star-api-rest --test canvas_event_stream_pg -j 4 -- --nocapture` | 2 passed，1 ignored；ignored 用例需要隔离、已迁移 PostgreSQL 和 fixture Worktree |
| 前端 `worktreeGroupApi` / `groupProjection` 测试（rebase 后） | 42 passed |
| `pnpm typecheck`（rebase 后） | 通过 |
| PostgreSQL trigger / RLS / 双实例运行验收 | NOT RUN；没有可用的 Neon 或 Multipass 测试数据库与 fixture |
| PT-09 p99、RSS/CPU、背压与长时恢复 | NOT RUN |

Rust `PgListener::try_recv()` 返回 `None` 的断连语义已根据 [SQLx 官方 API 文档](https://docs.rs/sqlx/latest/sqlx/postgres/struct.PgListener.html)核对；这与 listener 的显式 resync/reconnect 路径一致。CodeRabbit CLI 在当前执行环境返回 binary permission denied，未运行；本报告只记录协调者对变更的本地源码审阅。

## §3 已知缺口

- 没有 owner-local 持久草稿 revision、draft retention/RLS、冲突事实、双方持久未读提醒、conflict inbox 或有界 base/current/mine/theirs diff API。
- 没有显式 Resolve 预览/确认/CAS、取消无副作用、stale-CAS 保留双方草稿的实现闭环。
- PostgreSQL trigger 与多实例 fan-out 尚未在目标 DB 运行；ignored 测试不是通过证据。没有验证真实 Vercel gateway 或 Multipass K3s 连接。
- 没有 E2E-COLLAB-09/10/11、RLS 并发/resource-fence、撤权恢复或 PT-09 性能实测。100 subscriptions、64 wake events 等是有界代码参数候选，不是经目标负载验证的商业容量承诺。
- W16/WLD-7/PT-09 均不因本切片关闭；协作写入仍须由 owner 持久化、Hook/授权与独立验证门控制。

## §4 子代理失败接手清单

本报告 lane 未新派子代理，因此没有本 lane 的子代理失败待接手项。前端与后端来源均按提交和本报告列出的本地验证证据审查，未以代理状态代替验证结果。

## §5 守门规则

1. 只有 Outbox 是持久事件事实；`NOTIFY` 与 SSE 仅作 wake hint，不能承担正文、授权或恢复游标。
2. SSE 连接建立及每个匹配 Worktree wake 都重新验证 Project/Run/Worktree 读取 scope；撤权时关闭该连接。
3. queue 和 active stream 有界；达到上限应 fail closed/resync，不允许无限缓冲。
4. 没有目标 PostgreSQL/迁移/RLS、多实例、双边草稿冲突与 E2E 证据前，不标记 W16 完成。
5. 只集成本 lane 到本地 `dev`；没有更新远端、`main`，也未修改 W8 来源与 `D:\Star` 用户内容。
6. 不把 ≤1000 ms 的第二份 proposal 测试刺激解释成冲突窗口或性能承诺；PT-09 p99 从 conflict commit 起测。

## §6 签字栏

| 角色 | 结论 |
|---|---|
| 架构 / 本报告审阅 | 架构师（Mavis 接手审核）；通过本切片的范围与证据审查，不代表 W16 整体验收 |
| SRE Lead | 未单独参与本切片审阅 |
| 平台 | 未单独参与本切片审阅 |
| 评审主持 | 未单独参与本切片审阅 |
| PM | 未单独参与本切片审阅 |
| 修订人 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 |

## §7 修订历史

| 版本 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 首次记录 W16 Rust/前端 wake slice、本地 dev 集成及严格的 OPEN/PARTIAL 验收边界 | 后端事件流合入本地 dev 并完成集成后检查 |
