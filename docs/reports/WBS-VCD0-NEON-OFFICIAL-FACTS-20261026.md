# WBS-VCD0 Neon 官方事实只读核验报告

> 日期：2026-10-26
> 状态：准备性平台事实研究；不构成设计批准、账户验证或部署验收。
> 范围：仅为 Vercel 部署提案 SRS GAP-06 收集官方来源；canonical SRS/ADR/BD/DD/TD 未修改。

## 核验范围

按 brief 阅读 Vercel SRS v0.4、ADR、BD、DD、TD、VCD-0 合同缺口审计及 Worktree Group WBS §22.3–§22.4。Neon 产品事实只引用 Neon 官方文档；Vercel Postgres 生命周期由 Vercel 官方文档核验。下列网页于 2026-10-26 查阅。Neon 文档会更新，任何目标套餐/账户结论仍须在具体部署配置下复核。

## 官方事实矩阵

| Claim | 官方来源页面与直接 URL | 文档支持的事实 | 影响的 Star 文档/章节 | 设计后果（推论） | 仍需验证 |
|---|---|---|---|---|---|
| Vercel Postgres 生命周期与新接入路径 | [Postgres on Vercel](https://vercel.com/docs/postgres)；[Leveling Up our Partnership with Vercel](https://neon.com/blog/leveling-up-our-partnership-with-vercel) | Vercel 当前文档称 Vercel Postgres 已不再提供，既有库于 2024 年 12 月自动迁至 Neon；新项目通过 Marketplace Postgres 集成。Neon 2024-08 公告记录了迁移与 Marketplace 过渡计划。Vercel 页面最后更新日为 2026-01-13。 | SRS §0、§3.1、§8 GAP-06、FR-VC-009；BD §6.3；DD §5.2 | 后续 canonical 同步可将 SRS 第三方来源替换为 Vercel 官方当前说明，并区分 Neon 是已选择的托管 PG provider 与 Star 实际 Owner/访问路径。此事实不授权 Vercel Function 绕过 Star Owner 直连。 | 目标 Vercel 账户/套餐下 Marketplace 集成的可用性、目标 Neon project/branch/region 与实际连接授权均 UNKNOWN。 |
| pooled 与 direct endpoint 的事务语义 | [Connection pooling](https://neon.com/docs/connect/connection-pooling) | Neon PgBouncer 使用 transaction mode，事务结束即将后端连接还给池。官方列出 pooled 下不支持 `SET`/`RESET` session variables、`LISTEN`/`NOTIFY`、SQL `PREPARE`/`DEALLOCATE`、部分临时表、session advisory locks 等。官方将 schema migration（工具可能不支持 transaction pooling）、`pg_dump`/`pg_restore`、logical replication 和需要 session feature 的管理操作列为 direct 用例。 | SRS FR-VC-010/011、§8 GAP-04；BD §6.3；DD §5.2；TD §4.5 | Star 仍须由既有 Owner 在同一事务建立 RLS 上下文并执行数据操作；需要 session state 的迁移作业应评估 direct endpoint 与独立、受控身份。文档没有证明 Star 的 `SET LOCAL` + RLS 在 Neon pooler 上满足隔离要求。 | 精确的 `SET LOCAL`、事务封装、连接复用和双租户互不可见行为保持 UNKNOWN，须以批准的目标配置做隔离并发验证；Star migration job/role 到 direct endpoint 的授权也 UNKNOWN。 |
| branch、role、database 与 pooled URI 选择 | [Neon CLI connection-string](https://neon.com/docs/cli/connection-string)；[Retrieve connection URI API](https://neon.com/docs/reference/api/projects/get-connection-uri)；[Manage roles](https://neon.com/docs/manage/roles) | CLI 可用 branch 名称或 ID 选择分支，并支持 database、role、endpoint 类型及 pooled 选项。API 可指定 branch、endpoint、database、role；`pooled=true` 返回带 `-pooler` 的 URI。Neon role 属于 branch；创建 child branch 时通常复制父 branch role，但从历史时间点创建时不复制该时间点以后创建的 role。 | SRS FR-VC-009/012、§8 GAP-06；BD §6.3；DD §5.2；WBS §22.3 | Neon 的 branch/role/endpoint 组合可作为连接配置事实固定，但它不是 Star Project、Engineering Run 或当前授权身份。每个环境的 branch、database、role、endpoint 与 URI 来源仍应由 Star Owner/配置边界明确。 | 目标环境的 branch 身份、role 生命周期/密码轮换、endpoint 类型、凭据注入和实际授权 UNKNOWN。 |
| Neon role 的权限语义 | [Manage roles](https://neon.com/docs/manage/roles) | Neon 文档称经 Console/API/CLI 创建的 Postgres roles 会成为 `neon_superuser` 成员；该角色含 `BYPASSRLS`（文档注明此属性适用于 2023-08-15 之后创建的项目）。SQL 创建的角色只获得基础 public schema 权限，需逐对象授权。每个 branch 最多 500 个 roles。 | SRS FR-VC-010、§8 GAP-04；DD §5.2；TD §4.5 | 不可从“Neon role 已创建”推断 Star RLS 已生效；运行时与迁移身份必须按最小权限设计，并在目标项目核对实际继承、`BYPASSRLS`、对象 grants 与 FORCE RLS。 | 目标项目的创建时间、实际 role 属性/成员、对象授权、RLS/FORCE RLS 与 Owner 事务行为 UNKNOWN。 |
| Pooler 与 PostgreSQL 连接上限 | [Connection pooling](https://neon.com/docs/connect/connection-pooling)；[Manage computes](https://neon.com/docs/manage/computes) | Neon 文档列出 PgBouncer `max_client_conn=10,000`；每 user/database 的 `default_pool_size` 为 `max_connections` 的 90%，等待池连接最多 120 秒；这些 PgBouncer 参数不可由用户配置。`max_connections` 依 compute 而异，Neon compute 文档给出按 CU 的设置且说明具体能力受套餐影响。10,000 是 pooler 客户端连接上限，不是 Star 的活跃事务吞吐量；90% 是每 user/database pool，不是全产品总容量。 | SRS FR-VC-012、§8 GAP-04；BD §6.3；DD §5.2；TD §4.4–§4.5 | Star 的 connection budget 仍须按 Owner/worker/branch、实例并发与目标 compute 逐层核算；不可将 Neon 的客户端连接上限当作应用预算或验收结果。 | 实际套餐、compute 配置、同时活跃连接、池等待/超时、Vercel 文件描述符预算与 Star workload 压测均 UNKNOWN。 |
| Compute scaling、暂停与恢复 | [Manage computes](https://neon.com/docs/manage/computes) | Neon 支持固定 compute 和 min/max autoscaling；compute 大小影响处理资源和 `max_connections`。Scale to Zero 文档说明默认在 5 分钟无活动后转 idle，重新激活会有恢复延迟；compute 调整/重启可能断开连接，自动 autoscaling resize 则无需重启。套餐决定可用 compute 规格。 | SRS NFR-VC-06、FR-VC-012；BD §6.3；DD §7；TD §4.4 | 部署设计应考虑目标 compute 的恢复延迟、自动重连和并发/连接预算，但官方描述不是 Star 的冷启动 SLO 或可用性证明。 | 具体套餐资格、min/max、暂停设置、真实恢复时延、重连恢复行为及目标 workload 结果 UNKNOWN。 |

## 平台事实不能替代的 Star 合同

Neon 的 Project/branch/role/endpoint 不是 Star Project 或 Engineering Run 身份，也不提供 Star 当前授权判定。Neon 文档未定义 Star-owned RLS owner context，不能证明 `SET LOCAL` 的 Star 租户边界、owner transaction 或 owner-local Outbox 原子性。Neon Postgres 连接/池化资料不定义 Star Gateway 或 chat SSE 的 route、session、Origin、授权、撤权、生命周期或恢复合同。公开文档和 Marketplace 页面也不证明任何目标 Vercel/Neon 账户、数据库、角色 grants、实际部署或生产接受结果；上述 Star 映射与验收一律 UNKNOWN。

## 状态与未决项

- VCD-0 仍为 **OPEN**；VCD-1..VCD-4 不因本报告启动或关闭。
- SRS GAP-04 与 GAP-06 的 canonical 状态保持原值；GAP-06 的平台事实已有官方来源输入，但须经后续 canonical 文档同步与审查后再更新状态。
- 既有 TD 用例状态保持 **NOT_RUN**；本 lane 未执行代码测试、账户/数据库连接、云资源创建、Vercel 部署或运行时验收。
- 执行顺序不变：先完成 WLD-7 Canvas 设计同步，再进行 Vercel canonical 设计同步。本文只为后续同步提供事实依据，不更改优先级或关闭 WLD-7/VCD-0。
- 流程偏差：本 lane 曾在 automation/dispatcher.py brief 落地前启动一次只读来源核对，违反仓库子代理派发门。该子代理未写文件，随后已停止；其输出未作为本报告证据。本报告事实仅依据本 lane 对 Context7 返回材料及上述官方页面的独立核验。
