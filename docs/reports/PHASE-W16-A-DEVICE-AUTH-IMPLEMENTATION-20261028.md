# PHASE-W16-A-DEVICE-AUTH-IMPLEMENTATION-20261028

> 状态：🟡 Partial。已实现 Run/Worktree 写请求的设备 generation 事务内复验，以及 OAuth/device repository 的 owner-role 源码边界；真实 PostgreSQL/RLS 验收、所有共享读取/订阅/resolve 与 SSE 撤销联动仍未关闭。

## §0 目的

记录 W16-A 已完成的设备身份与数据库 owner 访问切片，按实际代码和验证结果界定完成范围。编译和非 PostgreSQL 测试不代替数据库授权、RLS、并发与 SSE 验收。

## §1 改动矩阵

| 文件 | 实施内容 | 状态 |
|---|---|---|
| `crates/star-api-rest/src/group_api/run_worktree_bindings.rs` | Run/Worktree 写事务以 Engineering Directory role 开始，设置 tenant/actor scope 与短超时；在同事务对 device generation 执行 `FOR SHARE` 检查，失败关闭。 | 源码实现；route 单测通过，PG acceptance 未运行 |
| `crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs` | 为受保护 mutation 提供 tenant/actor/device 对应 JWT fixture；隔离 PG probe 检查 NOINHERIT runtime 可 SET Directory role、对 generation 执行 `FOR SHARE`，且直接 UPDATE 受 RLS 拒绝并保持 generation 不变。 | Rust route tests 2 passed，含 PG probe 的真实数据库用例 ignored；新数据库断言未运行 |
| `crates/star-pg-adapter/src/repository/device_bindings.rs` | 五个操作统一在事务内先 SET OAuth owner role，再绑定 tenant/actor scope；错误不降级。 | 源码实现；真实 PG/RLS 未运行 |
| `crates/star-pg-adapter/tests/device_auth_pg.rs` | 设备 owner-role membership 与 NOLOGIN/NOINHERIT/NOBYPASSRLS 属性探针；refresh fixture/断言也显式切换 owner role。 | all-targets 编译通过；真实 PG 测试未运行 |
| `db/migrations/2026-10-28-w16-device-auth.sql` | OAuth owner 最小 DML/列权限及 scoped RLS；Directory owner 对 device binding 获得 SELECT 与 `UPDATE(generation)` 列 ACL，以满足 PostgreSQL `SELECT ... FOR SHARE` 的权限检查；restrictive `USING(false) WITH CHECK(false)` 阻止实际更新。refresh generation 配对约束显式禁止非空 device_id 搭配 NULL/非正 generation，并替换已存在的同名弱约束。 | SQL 静态审阅；未在真实 PostgreSQL 执行 |

## §2 验证摘要

- Rust 1.99.0 rustfmt --edition 2024 --check：全部四个本 lane 修改的 Rust 源/测试文件 PASS。
- CypherGraph Guardian --changed --stamp：PASS，重新协调 3 个 source file 的关系头与 freshness hash。
- `rtk proxy cargo +1.99.0 check -p star-pg-adapter --all-targets --locked --offline -j4`：通过，7m19s；仅有 domain-kms 的既有 unused/deprecation warnings。
- `rtk proxy cargo +1.99.0 test -p star-pg-adapter --lib --locked --offline -j4`：35 passed，0 failed，0 ignored。
- rtk proxy cargo +1.99.0 test -p star-api-rest --test engineering_run_worktree_binding_routes --locked --offline -j4：2 passed，0 failed，1 ignored；ignored 用例包含 FOR SHARE/RLS denied-update probe，需独立 PostgreSQL acceptance runner 执行。
- `rtk proxy cargo +1.99.0 fmt --package star-pg-adapter -- --check`：失败于包内其他未修改文件的既有格式差异：`automation_schedule.rs`、`repository/mod.rs`、`oauth_authorization_codes.rs`、`schedule_postgres.rs`。未格式化这些无关文件；改动文件的定向 rustfmt 检查通过。
- `rtk git diff --check`：通过。
- `STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL` 未设置；未运行迁移、device-auth PostgreSQL tests 或真实 RLS/角色探针。所有命令经 RTK 执行，未读取或输出任何 secret。

## §3 已知缺口

- 本次唯一标记为源码已实现的是 Run/Worktree 写请求在 binding mutation 同一事务中复验设备 generation，以及 OAuth/device repository 显式 owner-role/scope 顺序。Directory role 的 `UPDATE(generation)` 仅用于通过 PostgreSQL 对 `FOR SHARE` 的 ACL 检查；restrictive RLS 仍应令直接 UPDATE 影响零行。该 effective ACL/RLS 与配对约束迁移尚未经过真实 PostgreSQL 验证。
- 设备 proof/enrollment/revoke 路由及 repository 源码存在，不代表认证 enrollment 已验收；PG ownership、挑战单次消费、撤销 refresh family 与并发行为仍需隔离 PostgreSQL 测试。
- 其他共享读取、订阅、OAuth credential resolve、SSE 撤销事件即时关闭/有界租约与跨节点失效未在本 lane 验收。E2E-COLLAB-12/13 保持 OPEN/NOT_RUN；OAuth 单赢家 E2E-14 也未由本 lane 关闭。
- KMS/provider、持久密文、durable drafts/conflicts、备份恢复与 PT-09/10 性能验收不属于本次闭环，继续保持 OPEN/NOT_RUN。

## §4 子代理失败接手清单

- 原实现 worker 与替补 worker 因用量限制退出；其代码由协调者接手。随后两个独立只读审查分别复核 source/SQL 和 PG harness，未发现 P1/P2；source review 发现 migration Cypher header 的 policy id 与实际策略名不一致，已修正并重新通过 CGG。PG harness 仍须等本修复提交后 rebase 并实跑。CodeRabbit CLI 已安装但执行返回 permission denied，未视为通过。

## §5 守门规则

- 应用登录身份不获 OAuth/device 表直接 ACL；部署配置必须显式配置所需 role membership，缺失时源码调用 fail closed。
- `star_oauth_credential_owner` 用于 OAuth/device owner 操作；`star_engineering_directory_owner` 仅有 `UPDATE(generation)` 列 ACL 来取得 generation `FOR SHARE` 锁。restrictive `USING(false) WITH CHECK(false)` RLS policy 阻止这个角色写入任何行；真实 PostgreSQL probe 还需证明 SELECT FOR SHARE 成功、UPDATE 影响零行且 generation 不变。
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
| v0.2 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 说明 `UPDATE(generation)` 是 PostgreSQL `FOR SHARE` ACL 所需的列级授权，restrictive RLS 继续禁止实际更新；补录配对约束 NULL 修复和锁/拒写探针状态，真实 PostgreSQL 结果仍 NOT_RUN。 | 安全审查发现 row-lock ACL 与 CHECK NULL 语义缺口。 |
| v0.3 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Rust 1.99.0 route test 2 passed/1 ignored、四个 changed Rust files rustfmt PASS、CGG PASS，以及两个只读审查的结果；真实 PostgreSQL/HTTP acceptance 仍等待专用 harness。 | 复核工作树修复并完成源码门禁。 |
