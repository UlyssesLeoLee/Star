# WLD-0 Project / Run / Worktree 绑定契约审查

- 审查日期：2026-10-20
- 基线：dev 944b29d5
- 范围：本审计仅核对需求、基本设计、Group 详细设计、实施计划、WBS、Run/Worktree API 与迁移源码；未修改这些基线。
- 结论：**导航与实体语义已基本稳定；绑定 owner 和版本词汇尚未统一，因此当前不应冻结完整契约。** 下文给出建议的规范词汇和冻结条件。

## §0 目的

将 Project → Engineering Run → Worktree → Branch 的身份与授权边界，和现有 Run-to-Worktree 写入切片对照。审计分别记录设计意图、当前源码证据、此前报告中的验证结果和目标数据库门禁；不能把源码迁移或先前测试结果当成目标库验收。

本地证据采用定向文档/源码读取。当前 worktree 没有可用 CodeGraph 索引，且没有 coverage-check 工具；因此此报告是所列文件与行区间内的有界审查，不宣称仓库级穷尽。未运行测试，也未连接或修改数据库。

## §1 契约审查矩阵

| 契约面 | 规范建议 / 当前证据 | 结论 |
|---|---|---|
| 产品树与实体 | Project 是顶层归属；Engineering Run 是 Project 下可独立于宿主 Branch 的协作和分析范围；Worktree 是本地 checkout；Branch 是 Worktree 下本地 checkout 状态叶。远端 repository/Branch 是可选 SCM 元数据。需求、基本设计及 Group DD 对此一致（docs/requirements.md:2312, 2326; docs/basic-design.md:4226, 4251-4253; docs/design/DD-WORKTREE-GROUP-001.md:135-153）。 | 词义稳定。Branch 的本地观察仍需来源、时间和新鲜度字段。 |
| 关系和可选性 | 建议将 EngineeringRunWorktreeBinding 定义为 Project/Run 与本地 Worktree 的当前归属事实：Run 可有多个 Worktree；每个 Worktree 最多一个当前 Run 绑定，也可以无绑定。hostless Run 可创建、列出和打开；Worktree focus 可选。选中 focus 时必须验证同一 Project 与当前 Run binding。DB 部分唯一索引表达每个 Worktree 至多一条有效 Run 关系（docs/design/DD-WORKTREE-GROUP-001.md:203; db/migrations/2026-10-01-engineering-run-directory.sql:191-220）。 | 可选关系与层级一致；历史、陈旧和重复当前行的部署后 reconciliation 未完成。 |
| 关系 owner | DD 明确称 Worktree Domain 持有 checkout lifecycle/binding（docs/design/DD-WORKTREE-GROUP-001.md:140）。当前绑定端点却在 run_worktree_bindings.rs 中按 Project/Run 授权，关闭/追加 SCD2 事实并写 Run Outbox（crates/star-api-rest/src/group_api/run_worktree_bindings.rs:129-131, 285-359, 801-902）。 | **未统一，阻止冻结。** 建议区分两种事实：Worktree Domain 拥有 checkout lifecycle 与 Project-Repository 归属；Engineering Run Directory 拥有 EngineeringRunWorktreeBinding、Run 授权命令与 Run Outbox。若要由 Worktree Domain 拥有 Run 绑定，则必须同步规定 owner command、审计/Outbox 聚合和事务责任，不能只靠 API 文件位置推定。 |
| 本地 Branch 与远端 Branch | WorktreeIndexItem 当前把 branch 暴露为无类型字符串（docs/design/DD-WORKTREE-GROUP-001.md:149-152）；Run 远端 repository/branch 可为空，而 binding 必须保存本地 Worktree repository identity，branch_id 仅在对应远端 SCM 上下文存在时使用（docs/basic-design.md:4253; db/migrations/2026-10-15-run-host-independent-directory.sql:23-39, 56-60）。 | 不能把 local branch/ref 与远端 branch_id 合并为一个字段。建议使用 local_branch_ref + observation_source/observed_at/freshness，以及独立 optional remote_branch_id。未观察或过期时为 unknown。 |
| Run 与 focus 授权 | Run 列表、普通 Run App 与 hostless Run 依赖 Project membership 和 Run grant；Worktree focus 只为 checkout/Git/CLI 提供独立上下文，不是打开 Run App 的授权凭证。无 focus 的 RunContext 合法，传入 focus 时另行解析并校验（docs/design/DD-WORKTREE-GROUP-001.md:132-135; crates/star-api-rest/src/group_api/engineering_runs.rs:824-845; docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md:2205-2207）。 | 设计清楚。不得因为没有 Worktree/远端 Branch 而隐藏 Run；不得自动挑选 Worktree。远端 Branch grant 仅用于对应 SCM 能力。 |
| bind / reassign / detach | 使用 typed bind/reassign 与 detach 命令；写前复验 Project writer/manage scope、Project membership、当前 Worktree 所属及目标旧/新 Run grant。请求同时带 expected binding 状态、expected Project binding version、Idempotency-Key 与 correlation_id（docs/design/DD-WORKTREE-GROUP-001.md:214-216; crates/star-api-rest/src/group_api/run_worktree_bindings.rs:156-168, 285-359）。 | API 源码具备该切片；名称应在 API、DD、WBS 中统一为 attach/bind、reassign、detach，并定义 archive/restore 不隐式 detach。 |
| CAS、陈旧和歧义 | 源码 expected_binding 是 Unassigned 或 Bound{binding_id, version}；另有 expected_project_binding_version。当前绑定查询依赖唯一活动行约束；迁移定义 partial unique index，但 handler 的 fetch_optional 不显式报告多条当前行（crates/star-api-rest/src/group_api/run_worktree_bindings.rs:156-168, 755-817; db/migrations/2026-10-01-engineering-run-directory.sql:217-220）。 | 建议固定词汇为 binding_id + binding_version；Project Repository binding 版本单称 project_binding_version；RunContext focus revision 单称 focus_version。WBS 的 stale/ambiguous generation 应改为明确的 binding 状态与版本术语。缺失、陈旧、跨 Project/Run 或重复活动行必须 fail closed，之后由 reconciliation 处理，不得任取一行。 |
| 持久化写路径 | 源码有 per-Worktree advisory lock、SCD2 close/append、审计上下文、持久幂等回执与同一 owner transaction 的 Outbox（crates/star-api-rest/src/group_api/run_worktree_bindings.rs:663-704, 801-902）。迁移有关系约束、RLS policy/trigger 与 FORCE RLS DDL（2026-10-01-engineering-run-directory.sql:191-220, 444-455）。 | 仅源码证据。目标角色权限、授权请求写入、真实数据库锁竞争、回滚与幂等重放未通过目标库验收。 |
| hostless Task / CLI 连续性 | Run-owned Task Card 可不带 repository、remote Branch 或 Worktree focus 进行 list/create（docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md:2205-2227）。但 legacy authorize_run_task_scope 仍要求 repository_id 与 branch_id，否则返回冲突（crates/star-api-rest/src/group_api/engineering_runs.rs:386-406）。 | 目录和 Task Card 允许 hostless，不等于 CLI/执行支持 hostless。任务执行须保持 fail closed，直到 CLI owner/runtime 使用同一 Run/可选 Worktree binding 契约并通过独立 sandbox、授权和验证门。 |
| 证据时效 | 2026-10-19 API 实施报告记录了历史 source-level checks/tests，并明确目标 DB、grants/RLS、并发和历史 reconciliation 未验证（docs/reports/PHASE-WLD0-BINDING-API-IMPL-REPORT-20261019.md:17-34）。2026-10-18 审计中“无绑定 API”的结论早于新 API 切片，不能作为当前状态（docs/reports/WLD-0-RUN-WORKTREE-BINDING-AUDIT-20261018.md:13-15）。 | 本次未重跑历史测试，也未把旧审计结论当作当前 API 状态。 |

### 建议的冻结词表

- **Project**：顶层授权与归属上下文。
- **Engineering Run**：Project-owned、可 hostless 的协作/分析范围；不等同于 TaskExecutionRun。
- **Worktree**：本地 checkout 身份，拥有本地 repository identity；Run binding 对它是可选归属。
- **Branch**：Worktree 下由可信 checkout observation 提供的本地 ref/state 投影。远端 Branch 是独立 optional SCM identity。
- **EngineeringRunWorktreeBinding**：有版本、可关闭并追加 successor 的归属事实；绑定记录的关系版本不与 Worktree generation、远端 Branch revision 或 focus_version 混用。
- **WorktreeFocus**：当前 Run 请求中的可选 checkout 上下文；不授予 Project/Run 权限。
- **Project binding version**：Worktree 当前 Project/Repository 归属版本；CAS 时与 Run-to-Worktree binding version 独立核验。

## §2 验证摘要

本次只做了静态文档与源码审查：

- 需求/基本设计：docs/requirements.md:2312, 2322-2326；docs/basic-design.md:4226, 4251-4253, 4263。
- 详细设计与 Task Card：docs/design/DD-WORKTREE-GROUP-001.md:132-153, 140, 203-216；docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md:2205-2227。
- 写路径与 hostless CLI：crates/star-api-rest/src/group_api/run_worktree_bindings.rs:129-168, 285-359, 663-704, 755-902；crates/star-api-rest/src/group_api/engineering_runs.rs:360-406, 824-845。
- DDL 与门禁：db/migrations/2026-10-01-engineering-run-directory.sql:191-220, 444-455；db/migrations/2026-10-15-run-host-independent-directory.sql:23-60；docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md:25, 72。
- 此前测试数据仅来自已提交实施报告，不代表本次执行（docs/reports/PHASE-WLD0-BINDING-API-IMPL-REPORT-20261019.md:19-26）。

本审计**没有**运行 cargo、单元/API 测试、迁移、目标 PostgreSQL、角色 grants、FORCE RLS 运行验证、并发/replay、回滚或历史 reconciliation。报告只证明列出的设计与源码文本存在；不证明目标部署可写、授权正确或数据已完成修复。

## §3 已知缺口与冻结决定

**冻结决定：有条件暂缓。** Project → Engineering Run → Worktree → Branch、hostless Run、optional Worktree focus、local Branch 与 remote Branch 分离、Project/Run 授权和绑定 CAS 的核心语义可以保留；完整 binding vocabulary 在以下事项闭合前不能标记“冻结”：

1. 统一 binding owner：建议 Engineering Run Directory 持有 Run-to-Worktree 关系与 Run Outbox；Worktree Domain 持有 checkout 和 Project-Repository 归属。由设计负责人确认并把 docs/design/DD-WORKTREE-GROUP-001.md:140 的 owner 描述与 API/Outbox 事实对齐。
2. 将 WBS 与 API/DD 的版本字段统一为 binding_id + binding_version、project_binding_version、focus_version；定义 Worktree generation 和 K3s cluster generation 各自语义，禁止互换。
3. 明确 local_branch_ref 的可信观察来源、观测时间与 freshness；将其与 optional remote_branch_id 分开。
4. 明确重复当前绑定的 fail-closed 响应与部署前/部署后的历史 reconciliation 责任。
5. 在目标 PostgreSQL 实证迁移顺序、应用角色最小权限 grants、FORCE RLS、认证绑定写入、事务回滚、durable replay 和竞争写入；报告证据需包含目标环境身份与结果。
6. 对 hostless Task CLI 保持执行关闭，直到它不再把 remote repository/Branch 作为隐式必要条件，并通过 Runtime/sandbox、撤权、取消/恢复和独立验证门。

## §4 子代理失败接手清单

本审计未派发子代理；无失败子代理产出需要接手。定向代码/文档读取由本任务完成。CodeGraph 未索引与 coverage-check 工具缺失已作为审查边界记录，不据此作仓库级完整性断言。

## §5 守门规则

以下 17 项作为 WLD-0 冻结与完成门：

1. 导航顺序保持 Project → Engineering Run → Worktree → Branch，不能由宿主连接状态改变。
2. Engineering Run 以 Project/Run 授权，不要求远端 Branch 或 Worktree 才可见。
3. Worktree 到 Run 的关系是可选的；无绑定 Worktree 不得被猜测归属。
4. focus Worktree 必须属于同一 Project 和当前 Run 的有效 binding。
5. local Branch observation 与 remote Branch identity 必须分字段、分授权。
6. Branch 无观察或 observation 过期时显示 unknown，不从远端元数据推断本地 checkout。
7. Project membership、Run grant、Worktree capability 与 SCM Branch grant 各自验证，不相互替代。
8. attach/reassign/detach 使用 typed owner command；archive/restore 不隐式改写 Run binding。
9. 每个命令显式传 expected binding state 和 Project binding version。
10. API/UI/Task/CLI 共用 binding_id 与 binding_version 语义，不用含混 generation 代替。
11. 幂等回执须持久化并绑定 actor、scope 与请求摘要；内存结果不算 replay 证明。
12. per-Worktree 锁、CAS、SCD2 close/append、Audit 与 Outbox 必须处于定义清楚的 owner transaction。
13. 数据库唯一约束、RLS、审计与应用角色 grants 均须在目标环境验证；DDL 存在不等于已部署。
14. 多活动绑定、错误 Project/Run、过期版本、撤权和不完整授权均 fail closed。
15. 并发、精确 replay、冲突 replay、回滚与历史 reconciliation 必须有可复现证据。
16. Run/Task Card 可 hostless，不代表 Task CLI、Agent 或 Schedule 已可执行。
17. 任何 execution admission 仍需独立 Runtime sandbox、权限/预算复验、取消恢复与验证证据；本绑定切片不打开执行。

## §6 签字栏

| 角色 | 审阅记录 |
|---|---|
| 架构 | Mavis 接手审核：本报告建议暂缓完整词表冻结，先解决 owner 与版本名冲突。 |
| SRE Lead | Mavis 接手审核：目标 PostgreSQL、应用 grants/RLS、并发及 replay 尚无本轮证据。 |
| 平台 | Mavis 接手审核：绑定写路径源码存在；owner transaction 与目标部署边界仍需验收。 |
| 评审主持 | Mavis 接手审核：本结论限定于列出的文档与源码范围。 |
| PM | Mavis 接手审核：hostless Run / optional focus 意图明确；执行闭环不在本次审计的已完成范围。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 WLD-0 binding 词汇审查，区分设计、源码、既有测试报告和目标数据库证据；记录 owner、generation/version、local/remote Branch 以及 hostless CLI 缺口 | Brief: docs/briefs/WLD0-BINDING-CONTRACT-REVIEW-20261020.md |
