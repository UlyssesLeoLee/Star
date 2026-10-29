# PHASE-W16-A-DEVICE-AUTH-IMPLEMENTATION-20261028

> 状态：🟡 Partial。已实现 Run/Worktree 写请求的设备 generation 事务内复验，以及 OAuth/device repository 的 owner-role 源码边界；真实 PostgreSQL/RLS 验收、所有共享读取/订阅/resolve 与 SSE 撤销联动仍未关闭。

## §0 目的

记录 W16-A 已完成的设备身份与数据库 owner 访问切片，按实际代码和验证结果界定完成范围。编译和非 PostgreSQL 测试不代替数据库授权、RLS、并发与 SSE 验收。

## §1 改动矩阵

| 文件 | 实施内容 | 状态 |
|---|---|---|
| `crates/star-api-rest/src/group_api/run_worktree_bindings.rs` | Run/Worktree 写事务以 Engineering Directory role 开始，设置 tenant/actor scope 与短超时；在同事务对 device generation 执行 `FOR SHARE` 检查，失败关闭。 | 源码实现；route 单测通过，PG acceptance 未运行 |
| `crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs` | 为受保护 mutation 提供 tenant/actor/device 对应 JWT fixture；增加隔离 PG probe 检查 NOINHERIT runtime 可 SET Directory role 并通过 scoped RLS 读取 device generation。 | 2 passed，probe 所在真实 PG 用例 ignored |
| `crates/star-pg-adapter/src/repository/device_bindings.rs` | 五个操作统一在事务内先 SET OAuth owner role，再绑定 tenant/actor scope；错误不降级。 | 源码实现；真实 PG/RLS 未运行 |
| `crates/star-pg-adapter/tests/device_auth_pg.rs` | 设备 owner-role membership 与 NOLOGIN/NOINHERIT/NOBYPASSRLS 属性探针；refresh fixture/断言也显式切换 owner role。 | all-targets 编译通过；真实 PG 测试未运行 |
| `db/migrations/2026-10-28-w16-device-auth.sql` | OAuth owner 最小 DML/列权限及 scoped RLS；Directory owner 对 device binding 仅 SELECT；refresh revoke 仅更新 `revoked_at` 并受 owner-scoped permissive + restrictive RLS 保护。 | SQL 静态审阅；未在真实 PostgreSQL 执行 |

## §2 验证摘要

- `rtk proxy rustup run 1.99.0 rustfmt --edition 2021 --check crates/star-api-rest/src/group_api/run_worktree_bindings.rs crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs crates/star-pg-adapter/src/repository/device_bindings.rs crates/star-pg-adapter/tests/device_auth_pg.rs`：通过，覆盖全部本 lane 修改的 Rust 源/测试文件。
- `rtk proxy python "C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py" --changed --stamp`：PASS，4 个 source file。
- `rtk proxy cargo +1.99.0 check -p star-pg-adapter --all-targets --locked --offline -j4`：通过，7m19s；仅有 domain-kms 的既有 unused/deprecation warnings。
- `rtk proxy cargo +1.99.0 test -p star-pg-adapter --lib --locked --offline -j4`：35 passed，0 failed，0 ignored。
- `rtk proxy cargo +1.99.0 test -p star-api-rest --test engineering_run_worktree_binding_routes --locked --offline -j4`：2 passed，0 failed，1 ignored；ignored case 要求隔离 PG HTTP acceptance runner。
- `rtk proxy cargo +1.99.0 fmt --package star-pg-adapter -- --check`：失败于包内其他未修改文件的既有格式差异：`automation_schedule.rs`、`repository/mod.rs`、`oauth_authorization_codes.rs`、`schedule_postgres.rs`。未格式化这些无关文件；改动文件的定向 rustfmt 检查通过。
- `rtk git diff --check`：通过。
- `STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL` 未设置；未运行迁移、device-auth PostgreSQL tests 或真实 RLS/角色探针。所有命令经 RTK 执行，未读取或输出任何 secret。

## §3 已知缺口

- 本次唯一标记为源码已实现的是 Run/Worktree 写请求在 binding mutation 同一事务中复验设备 generation，以及 OAuth/device repository 显式 owner-role/scope 顺序。该角色合同尚未经过真实 PostgreSQL effective ACL/RLS 验证。
- 设备 proof/enrollment/revoke 路由及 repository 源码存在，不代表认证 enrollment 已验收；PG ownership、挑战单次消费、撤销 refresh family 与并发行为仍需隔离 PostgreSQL 测试。
- 其他共享读取、订阅、OAuth credential resolve、SSE 撤销事件即时关闭/有界租约与跨节点失效未在本 lane 验收。E2E-COLLAB-12/13 保持 OPEN/NOT_RUN；OAuth 单赢家 E2E-14 也未由本 lane 关闭。
- KMS/provider、持久密文、durable drafts/conflicts、备份恢复与 PT-09/10 性能验收不属于本次闭环，继续保持 OPEN/NOT_RUN。

## §4 子代理失败接手清单

- 本 lane 未再派生子代理；独立 PostgreSQL/HTTP harness 在另一 lane 中等待本分支集成后运行。其后续输出由协调者审查，不在本报告中提前声明通过。

## §5 守门规则

- 应用登录身份不获 OAuth/device 表直接 ACL；部署配置必须显式配置所需 role membership，缺失时源码调用 fail closed。
- `star_oauth_credential_owner` 用于 OAuth/device owner 操作；`star_engineering_directory_owner` 对设备绑定只保留 Run/Worktree generation lock 所需 SELECT。
- 不把 Rust 编译、普通单测、mock 或 SQL 静态检查写成真实 PostgreSQL/RLS acceptance。
- 本提交仅属于当前隔离 W16-A lane；不得在此 lane rebase、merge、push 或触碰 `main`。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 / 本报告审阅 | 架构师（Mavis 接手审核）；只审阅已实现切片与状态边界 |
| SRE Lead | 待隔离 PostgreSQL 与部署角色/RLS acceptance |
| 平台 Lead | 待审查 role grants、owner membership 与真实数据库结果 |
| 评审主持 | Rust/route 局部验证已记录；E2E-COLLAB-12/13 未关闭 |
| PM | W16-A、W16、WLD-7 与 PT-09/10 保持 OPEN/PARTIAL |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 W16-A 的 Run/Worktree 写入 generation 复验、OAuth/device role 最小权限设计、验证结果与尚未完成的 PostgreSQL/请求面验收。 | 设备 owner-role/RLS 安全缺口修复。 |
