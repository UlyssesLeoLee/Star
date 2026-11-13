# PHASE-WLD1 Repository Directory 来源门禁审计

- 版本：v0.1
- 日期：2026-10-23
- 阶段：WBS-NEXT-20261023 / WLD-1 identity/provider slice
- 基线：`e5c692772261a72fe07f4211e635d422344de07e`
- 分支：`codex/wld1-repository-directory-20261023`
- 结论：**只交付审计报告；不实现或接线 provider。当前接口保持 fail-closed。**

## §0 目的

复核 PostgreSQL 只读 repository-list provider 是否具备可证明的 canonical Project→Repository、Run 绑定、当前 creator/owner grants 与可信本地 repository-root registry。仅当四项都能由当前持久事实和运行代码证明时才实施。此 lane 的证据未满足 repository 主数据与受控根目录两项，因此本阶段不创建 provider、不修改启动接线，也不启用写侧 Worktree 操作。

## §1 核查与任务完成矩阵

| 核查项 | 当前证据 | 结论 |
|---|---|---|
| Project→Repository canonical source | `domain-scm::Repository` 含 tenant/project identity，但可见的 `ScmQueryPort::list_by_project` 实现是 `InMemoryScmService`（`crates/domain-scm/src/lib.rs:1016-1049, 1512-1543`）。当前 SQL migrations 定义 `scm.cloud_branch` 的 branch registration，记录 `project_id` 与 `repository_id`（`db/migrations/2026-10-01-engineering-run-directory.sql:70-104`），没有可用于列出全部 Project repositories 的 repository master table/PG adapter。分支记录不能证明无分支 repository 的完整性，也不提供 repository display name/default branch。 | **未满足**。不能从 branch rows 或内存实现冒充 PG repository directory。 |
| Run→Repository binding | Run 的 `repository_id` 与 `branch_id` 明确可空，Branch 是可选宿主元数据（`db/migrations/2026-10-15-run-host-independent-directory.sql:23-38, 256-263`）。列表路由调用 Run 授权后把 `scope.repository_id: Option<Uuid>` 传给 provider（`crates/star-api-rest/src/group_api/worktree_lifecycle.rs:430-453`）；写侧校验会拒绝未绑定或不匹配的 repository（同文件 517-525）。 | **部分满足**。已绑定 Run 可被验证；未绑定 Run 不能据此列出“已绑定”仓库。provider 必须对 `None` fail closed，或等正式绑定契约落地。 |
| Creator/owner grants | Run 创建事务写入当前 `owner_user_id` 和创建者的 `permission.engineering_run_role_binding`（`crates/star-api-rest/src/group_api/engineering_runs.rs:606-640`）。读取路径在事务内加载当前 Project binding 与未撤销的 Run grant，并以 `FOR SHARE` 锁定（同文件 376-399, 424-440）。owner role 对现有 Project/Run/Worktree 来源表有 SELECT grant（`db/migrations/2026-10-20-engineering-directory-owner-role.sql:86-96`）。 | **已证明 Project/Run 授权链**。repository master/root 尚无表，因此无相应受限 SELECT grants 可核验。权限 snapshot 不能替代 provider 内实时授权复验。 |
| Trusted local repository-root registry | `ExternalWorktreeImportRegistry` 是进程内 `HashMap<Uuid, PathBuf>`；公开 `register` 直接接收路径（`crates/worktree-shared-dir/src/external_worktree_import.rs:530-565`），不是持久化、Project-scoped、带授权的根目录事实。`worktree_canvas_worktree.path` 的 DDL 注释明确是 Worktree checkout path（`db/migrations/2026-09-16-worktree-canvas-worktree.sql:12-25`），不是 Repository root。`worktree_shared_dir.path` 是 per-repository shared-directory 配置（`db/migrations/2026-09-22-worktree-shared-dir.sql:22-36`），也不是受控 repository-root registry。 | **未满足，阻断实现**。不得用这些路径来源替代可信 host-owned registry。 |
| Provider 与启动状态 | 路由 trait 将 discovery 与 Git 生命周期放在同一个 `RunWorktreeLifecycleProvider` 中（`crates/star-api-rest/src/group_api/worktree_lifecycle.rs:327-358`）。`GroupApiState::new` 默认将 provider 设为 `None`（`crates/star-api-rest/src/group_api.rs:342-359`）；当前 `main.rs` 只安装 Group App registry 与可选 BI reader（`crates/star-api-rest/src/main.rs:79-88`）。列表 handler 经 `require_lifecycle_provider`；缺失时返回 feature-unavailable/503（`worktree_lifecycle.rs:528-532`）。 | **Fail-closed 已成立**。没有启动接线，也没有写入 provider 文件。 |

本阶段唯一改动为本报告。未创建 `worktree_lifecycle_provider.rs`，未改 `group_api.rs`、`main.rs` 或 route tests；未开放 create/import/archive，也未增加 seed/mock、用户路径输入、目录扫描或新依赖。

## §2 验证摘要

- 指定 lane 的初始 HEAD 与 branch 符合 brief：`e5c692772261a72fe07f4211e635d422344de07e`、`codex/wld1-repository-directory-20261023`；初始工作树干净。
- 对 `crates/star-api-rest`、`crates/domain-scm`、`crates/worktree-shared-dir` 与当前 `db/migrations/*.sql` 做了定向源码/DDL 检查。repository master/root 的否定结论限于该 checkout 的现有 SQL schema 与这些实现，不代表运行时 PostgreSQL 部署状态。
- CodeGraph MCP 对指定 lane 返回未索引；未初始化索引，按 brief 直接检查目标源码。
- 报告-only 结论没有 Rust 源码改动，因此没有运行 `cargo fmt`、`cargo test` 或 Rust 1.99.0 编译门。没有连接运行时 PostgreSQL；不存在目标数据库迁移/权限实证。
- 提交前执行 `git diff --check`；结果见最终回报。测试未运行不表示 provider 通过。

## §3 已知缺口

1. 需要 owner-controlled、持久的 repository master/source，至少明确 tenant、Project、Repository identity 与不含敏感值的 display name/default branch；需定义无 branch repository 的完整列举语义。
2. 需要 host-owned repository-root registry：由可信 Rust Host/Manager 管理，路径不从请求、Agent、Plugin 或任务卡输入；需要稳定 identity、Project 授权关系、生命周期/失效规则与 DB 只读 ACL。
3. 需要明确未绑定 Run（`repository_id IS NULL`）的列表语义。当前验收要求不能将 Project 可见 repositories 推断成 Run 已绑定 repositories。
4. repository master 与 root registry 落库后，需授予 provider 最小 SELECT 权限，并在 provider 查询/输出边界复验当前 Project grant、Run grant、Run→Repository binding 和 registry currentness。
5. 实现后需新增 PG-backed route/provider 验证：合法 Project/Run 与绑定、撤权、跨 Project/tenant 隔离、缺失/过期 root fail-closed、repository URL/host path/credential 不出现在响应，以及 provider 缺失仍返回 503。应区分 fixture 证据与目标库 evidence。
6. 本阶段没有修改需求、基本设计或详细设计；新表、owner 契约和未绑定 Run 语义须在获准扩展 manifest 后同步到相应设计与验收记录。

## §4 子代理失败接手清单

本 lane 未派发子代理；没有子代理产出或失败项需要接手。未修改或覆盖其他 worker/claim 的文件。

## §5 守门规则（17 项）

| # | 守门项 | 本阶段结果 |
|---|---|---|
| 1 | 只在 brief 指定的 managed lane 工作 | 通过 |
| 2 | branch 与精确 base 与 brief 一致 | 通过 |
| 3 | 初始工作树状态已记录 | 通过，初始干净 |
| 4 | 仅修改 manifest 中五个文件之一 | 通过，仅报告文件 |
| 5 | 不编辑 D:\Star main、共享 dev 或其他 lane | 通过 |
| 6 | 不用 mock、seed 或内存队列证明生产能力 | 通过，未实现 provider |
| 7 | 不接受请求侧路径、URL 或任意 filesystem selector | 通过，未增加输入 |
| 8 | 不做隐式目录扫描 | 通过，未扫描运行时文件系统 |
| 9 | 不开放 create/import/archive 写侧 | 通过，provider 未安装 |
| 10 | Project→Repository 必须由 canonical 持久事实证明 | **阻断**，仅见 branch registration 与内存 SCM |
| 11 | Run scope 必须来自当前持久关系并处理 NULL binding | 部分通过，NULL Run 必须保持不可列出/未绑定 |
| 12 | Project grant、Run grant、creator/owner 来源必须核验 | 通过现有授权链；repo/root grant 尚无来源 |
| 13 | Repository root 必须来自可信、受控且持久的注册 | **阻断**，现有 registry 为进程内 PathBuf map |
| 14 | 返回字段不得包含 remote URL、主机路径或 credential | 未实现；当前 DTO 只含 id/name/default_branch |
| 15 | provider 缺失时 fail closed | 通过，state 默认 None，handler 503 |
| 16 | Rust 1.99.0 focused gates | 不适用；没有 Rust 变更，未运行 Cargo gates |
| 17 | commit 仅含 manifest 文件；不 rebase/merge/push/cleanup | 提交时核对；按 brief 执行 |

## §6 签字栏

| 角色 | 审阅结论 |
|---|---|
| 架构 | 架构师（Mavis 接手 agent per DEC-008）：审核通过；当前来源不完整，维持 fail-closed。 |
| SRE Lead | Mavis 接手审核：未连接目标 PostgreSQL；不声称运行时 schema/ACL 已验证。 |
| 平台 | Mavis 接手审核：当前启动未安装 provider；不启用 repository listing。 |
| 评审主持 | Mavis 接手审核：确认 repository master/root registry 为阻断缺口。 |
| PM | Mavis 接手审核：本阶段仅交付门禁审计，不标记 provider 可用。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容与触发 |
|---|---|---|---|
| v0.1 | 2026-10-23 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 按 WLD-1 brief 核验 Project/Repository、Run binding、授权与 trusted root 来源；因 repository master 与持久 trusted root registry 缺失，仅新增本报告，provider 保持未安装与 fail-closed。 |
