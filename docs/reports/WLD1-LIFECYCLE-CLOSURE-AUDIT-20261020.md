# WLD-1 Worktree Lifecycle Closure Audit

**日期：** 2026-10-20  
**审计基线：** `dev` 派生审计 worktree，HEAD `35a52c972a58b339cb790ad30ed07ee4c56a797a`  
**范围：** 只读核对 Project Worktree lifecycle provider、Run binding 边界、archive/restore/cleanup 持久化路径、相关 migration 与测试证据。未修改产品源码、SQL、UI 或 WBS。

## §0 目的

确认 WLD-1 中 create/import、归属绑定、archive/restore、transfer 与 cleanup 哪些具有可用的 owner 命令和持久化闭环，哪些仍须 fail closed。源代码、迁移文件、单元/集成测试与部署运行证据分层记录；接口存在不等于 provider 已装配，migration 存在不等于目标数据库已应用。

本 worktree 没有 `.codegraph/`；CodeGraph 工具确认此 checkout 未索引，且当前没有 `check_index_coverage` 可调用。因此本报告使用限定范围的 `rtk proxy rg` 和带行号 PowerShell 直接读取。搜索结果只用于描述检索过的源码路径，不据此宣称仓库外或目标环境不存在某组件。

## §1 生命周期核对矩阵

| 能力 | 当前证据 | 审计结论 |
|---|---|---|
| Project repository 列表、create、import API | REST 定义 provider trait、DTO 和四条路由；create/import handler 做请求校验、Project writer 授权，并向 provider 传递 idempotency key、fingerprint、actor、permission snapshot 和 correlation ID（`crates/star-api-rest/src/group_api/worktree_lifecycle.rs:108-201,228-245,248-280,319-337,388-474`）。`GroupApiState` 将 provider 设为 `None`，主程序只安装 Group App registry，没有安装 Worktree lifecycle provider（`crates/star-api-rest/src/group_api.rs:313-343,386-392`; `crates/star-api-rest/src/main.rs:79-89`）。对 `crates/` 中 `ProjectWorktreeLifecycleProvider for` 做定向搜索未找到实现。 | 生产 REST composition 中 provider 缺失时，repository list/import candidates/create/import 均返回 `worktree_lifecycle_unavailable`。这是 fail-closed 的不可用状态；create/import 目前没有已证实的生产成功路径。trait 要求 provider 在 Git 副作用前重新验证当前 membership 与 Project/Repository binding，并在返回前持久化 operation、Worktree/Project binding、Audit、Outbox（`worktree_lifecycle.rs:175-201`），但这些是实现契约，不是已实现行为。 |
| Create/import 身份与幂等 | DTO 不接收本机路径或 URL；import 使用 opaque candidate ID（`worktree_lifecycle.rs:67-89,108-137,204-226`）。handler 生成/校验 request fingerprint 和 receipt，并返回 202（`worktree_lifecycle.rs:388-430,433-474,487-572`）。 | API 侧输入约束存在，但 handler 不持久化、重放或比较 idempotency key；这些责任在未安装的 provider。当前 receipt 的状态只有 `Provisioning` 和 `Ready`，未形成可检查的持久失败/重试/恢复状态机。通过 receipt 字段校验不能证明 checkout 已创建或 operation 已 durable commit。 |
| Project → Worktree 与 Run → Worktree 归属 | Phase 2B migration 定义 `multica.worktree_project_binding` SCD2、活动唯一索引及 FORCE RLS（`db/migrations/2026-09-29-worktree-group-phase-2b.sql:67-97`）。Run binding 有独立 PUT/DELETE 路由和版本化绑定写入；当前 API 校验 Project/Run、Worktree 所属、版本、唯一当前绑定及幂等键，并写 Run owner Outbox（`crates/star-api-rest/src/group_api/run_worktree_bindings.rs:225-233,286-359,613-690,759-840,886-892`）。 | Project→Worktree 的 SCD2 migration 是数据模型证据；Run→Worktree binding 是独立 Run Directory owner 的源码切片，不能代替 Worktree owner 的 Project/Repository checkout lifecycle provider。两者的目标 DB/grants/RLS、真实认证写入与部署 reconciliation 在本审计中未验证。 |
| Archive/restore 与 owner transfer | 管理计划仅允许 `assign_owner`、`set_archived`，带 requester/key/hash/version/短确认期（`db/migrations/2026-09-29-worktree-group-phase-2d.sql:29-58`）。创建计划检查授权并实现同 key replay / 不同请求冲突（`crates/star-api-rest/src/group_api/worktrees.rs:381-429`）。确认 archive 时要求 Host readiness 与 Git lock 观察、drain、freshness、fence 余量和 Hook Allow；未知状态失败关闭（`worktrees.rs:942-1081`）。成功确认仅更新 `archived`、`expires_at`、`version` 并写 append-only management audit（`worktrees.rs:719-833`）。 | archive/restore 是索引可见性与保留期的逻辑状态转换；UI 也明确 archive 不删除 Git checkout（`frontend/src/app/worktree/page.tsx:1013-1022`）。没有由该确认路径执行 Git removal/checkout restore。owner assignment 不等同 checkout 在 Project/Repository 间 transfer；未见相应 lifecycle provider command。Run reassign 也限于当前 Project/repository binding 规则。 |
| Audit、Outbox 与幂等持久化 | Phase 2D 有 management plan、owner assignment、append-only management audit 及 FORCE RLS（`2026-09-29-worktree-group-phase-2d.sql:9-29,29-95,97-129`）；archive Hook outcome 和管理结果在管理事务写入 Hook ledger/Audit（`worktrees.rs:603-665,735-743,798-832,1083-1139`）。 | owner/archive 管理操作有 plan replay 与 audit；未发现 create/import lifecycle operation ledger 或其 owner-local Outbox。Run binding Outbox 是另一事实 owner。所读 Phase 2B/2D migrations 显示 FORCE RLS/policies，但不含该 lifecycle provider 的 app grants；目标 role grants 与应用身份下的 RLS 行为未实测。 |
| Cleanup 与保留 | legacy `PgWorktreeService::cleanup_batch` 以“早于 cutoff、Merged、未锁（或 force）”筛选，再调用 `delete_by_id` 删除数据库行并发 in-process event（`crates/worktree-service/src/pg_worktree_service.rs:408-435`; SQL 为 `DELETE FROM worktree_canvas_worktree`：`crates/worktree-service/src/pg_worktree_repository.rs:402-409`）。同一 service 的 archive 只置 `archived=true`、upsert 并发 in-process event（`pg_worktree_service.rs:438-453`）。 | 此旧服务路径不调用 Git 删除 checkout，也未显示 Run/Agent lease、file claim、PTY/子进程 drain、fresh retention-lock observation、Hook decision 或 owner audit/outbox 检查。它不是 WLD-1 安全物理清理命令。安全 cleanup 仍须经 fresh facts、最终重授权/CAS、durable intent/receipt 与 reconciliation。 |
| 较旧 create/import service | `PgWorktreeService::create` 生成 `/worktrees/{branch}` 合成路径并 upsert（`pg_worktree_service.rs:191-241`）；外部导入方法接收调用方 `repo_path` 并扫描、upsert，`force=true` 可覆盖 branch 冲突（`pg_worktree_service.rs:456-559`）。 | 这些 generic service primitive 不实现 Project writer 复验、受信 repository-root 解析、Run binding、provider receipt 或同事务 Audit/Outbox，不能据此声称 Group lifecycle 功能可用。当前 Project 对应的本机 repository-root registry 和 creator/owner 规则仍待确定/验证。 |

## §2 验证摘要

- 直接读取了上表所列的 REST provider/handlers、Group state/main composition、Run binding owner、archive management、legacy PG service/repository、Phase 2B/2D 与 Run-binding migration，以及 WLD-1 WBS 行（`docs/implementation-plans/WBS-RUN-LIFECYCLE-EXECUTION-CLOSURE-20261019.md:23-32`）。WBS 当前仍将 WLD-1 标为依赖 WLD-0 的未关闭工作项，验收明确要求并发、幂等、provider 故障、active lease/claim/process、dirty checkout、retention lock、owner audit/outbox、target DB/RLS 与真实 provider。
- lifecycle 模块内测试覆盖 Git ref/candidate/receipt/DTO 校验和路由合并（`worktree_lifecycle.rs:591-665`），没有对真实 provider persistence 的测试。archive gate 测试覆盖 unknown/stale/fence 边界（`worktrees.rs:1499-1578`），不能证明 Host observer 已安装或 checkout 已清理。
- Run binding route integration test 使用 `connect_lazy`，仅检查未认证 PUT/DELETE 返回 401（`crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs:29-67`）。PG Worktree service 集成测试文件说明不连真实 PG，fixture 只做 lazy-pool/结构验证（`crates/worktree-service/tests/pg_worktree_service_integration.rs:1-18,99-106`）。这些不构成 lifecycle provider 或目标 DB acceptance。
- 本审计未运行 Cargo/前端测试、真实 PostgreSQL、Host Runtime、Git provider 或 cleanup operation。仅在报告完成后运行 `rtk proxy git diff --check`。目标 migration 应用状态、app role grants、生产 provider/readiness observer、恢复 worker 和真实 Host 行为均为未验证。

## §3 已知缺口与最小下一步

1. **P0 — authoritative Project/Repository 与本地 checkout 根目录。** 定义服务端受信 repository registry、Host/checkout identity 和 creator→owner 规则；不得从请求路径或不透明但过期的候选推断。
2. **P0 — lifecycle provider 持久化。** 实现并安装 PostgreSQL-backed Worktree owner provider；在事务内对同一 idempotency key/fingerprint 做 exact replay/conflict，并原子写 operation、Project binding、Worktree projection、Audit 与 Outbox。当前授权和 repository binding 必须在 Host effect 前复验。
3. **P0 — provider reconciliation。** 将 durable `Provisioning` intent 与 Host Git effect 分离；worker/恢复路径需以可重放 operation ID 对账 checkout path/ref/HEAD，只有独立验证后转 Ready。provider timeout、crash 与重复请求必须不产生重复 checkout。
4. **P0 — 安全 archive/restore/cleanup。** 将已有逻辑 archive gate 复用为 domain command 的最终门；分别实现 logical restore、物理清理、owner/Project transfer 语义。物理删除前重新授权、CAS、确认 Hook Allow、drain lease/claims/process/PTY、fresh Git retention-lock 和 fence；未知/过期事实 fail closed。Cleanup 需先写 durable intent，再幂等执行 Host removal 与结果回执。
5. **P0 — target DB 与完整验收。** 对应用数据库角色验证 grants/FORCE RLS；执行真实事务 rollback/replay、并发冲突、身份撤权、跨租户/project/repository、provider timeout/restart、脏 checkout 和运行中 Agent/CLI 场景；用 disposable checkout 证明实际 Git 行为，并保留可恢复数据。

**建议的下一最小切片：** 先只启用带当前 Project 授权的 repository-list provider，并为 Project→repository→本机受信根路径建立清晰 owner 来源；其余 import/create 保持 unavailable。随后实现 owner-local operation/receipt/audit/outbox schema 与 exact replay，再按顺序接 Host adapter 与 cleanup。不要把 `PgWorktreeService` 的合成路径、caller-supplied root 或内存事件流提升为生产入口。

## §4 子代理失败接手清单

- 本审计 worker 未派发子代理；没有失败或遗留子代理产出需要接手。
- CodeGraph 覆盖检查工具不可用；已按 bounded fallback 直接读取上述关键源码/SQL/测试行。此限制已反映在结论中，不把定向搜索写成全仓证明。
- 未触碰其他 worktree、WLD-2 报告或 WBS 状态。下一实施 worker 应先依据 §3 建立 brief，再由 coordinator 分配具体 owner 文件与验收门。

## §5 守门规则

| # | WLD-1 守门 | 本审计状态 |
|---:|---|---|
| 1 | Project→Run→Worktree→Branch 层级不由远端宿主决定；Run binding 与 checkout lifecycle owner 分开 | 已对齐 WBS/源码边界 |
| 2 | Project/Repository 身份与本机 root 只能由可信服务端 registry 解析 | 未实现/未验证 |
| 3 | create/import handler 与可信 provider 分层；无 provider 必须 unavailable | 当前已 fail closed |
| 4 | provider 在每次 Git effect 前重验 actor、membership、Project/Repository binding | trait 要求；无实现证据 |
| 5 | permission snapshot 是审计/版本 hint，不是当前 grant | 契约如此；运行时 provider 未验证 |
| 6 | import candidate 必须 opaque，并在副作用前重新解析与当前 binding 比对 | DTO 有约束；provider 未实现 |
| 7 | 客户端不得提供 host path、remote URL、任意 shell 或根目录选择器 | 当前 lifecycle DTO 满足；legacy service 路径不满足生产边界 |
| 8 | lifecycle operation key、fingerprint、receipt 需持久化 exact replay/conflict | API 只构造输入；provider ledger 未实现 |
| 9 | operation、binding、Audit、Outbox 在 owner-local transaction 原子提交 | create/import 未实现；管理计划仅覆盖 owner/archive 状态 |
| 10 | Run→Worktree bind/reassign/detach 只由 Run Directory owner 写，并固定版本与 Outbox | 有源码切片；部署/RLS仍未验证 |
| 11 | Archive 与 detach/transfer 分离，不隐式改写 Run binding | 现有管理命令单独修改 archive/owner；完整 transfer 语义待补 |
| 12 | Archive 不是物理 checkout deletion；Restore 语义须明确且可验证 | 当前只有逻辑 flag/Index restore |
| 13 | Cleanup 必须有新鲜授权、CAS、Hook Allow、drain 与 Git retention-lock 检查 | REST archive gate 有部分检查；物理 cleanup 未实现，legacy batch 不满足 |
| 14 | 任何 unknown/stale/provider error/authorization outage 都 fail closed | lifecycle provider 缺失与 archive facts 缺失均阻断 |
| 15 | target DB app grants、FORCE RLS 和事务故障/并发路径须真实验证 | migration 文件存在；target DB 未验证 |
| 16 | 测试夹具、lazy pool、源代码契约及 UI 不能冒充生产 provider/运行态证据 | 本报告按证据层级区分；未运行生产验收 |

## §6 签字栏

| 角色 | 审核状态 |
|---|---|
| 架构负责人 | Mavis 接手审核：确认本报告只声明源码可支持的生命周期边界；不代表目标架构实施完成 |
| SRE Lead | 本次未进行 Host/数据库运行态签核；相关门保持开放 |
| 平台负责人 | 本次未进行目标 DB grants/RLS 或生产 provider 签核 |
| 评审主持 | Mavis 接手审核：报告范围、缺口与证据限制已核对 |
| PM | 未进行产品验收；WLD-1 仍开放 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-20 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 新增 WLD-1 生命周期闭环审计：区分 Project lifecycle provider、Run binding owner、逻辑 archive 与物理 cleanup；记录 migration/test 与部署证据边界、fail-closed 缺口及最小实施切片 | `docs/briefs/WLD1-LIFECYCLE-CLOSURE-AUDIT-20261020.md` bounded read-only audit |
