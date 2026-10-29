# PHASE-W16-A-DEVICE-AUTH-IMPLEMENTATION-20261028

> 状态：🟡 Partial。已实现 Run/Worktree 写请求的设备 generation 事务内复验及 OAuth/device repository 的 owner-role 源码边界；隔离 PostgreSQL 18 owner/RLS 切片已通过（run `6af04e5a765446c8b3baaffff126bd52`）。所有共享读取/订阅/resolve、OAuth 并发单赢家与 SSE 撤销联动仍未关闭。

## §0 目的

记录 W16-A 已完成的设备身份与数据库 owner 访问切片，按实际代码和验证结果界定完成范围。编译和非 PostgreSQL 测试不代替数据库授权、RLS、并发与 SSE 验收。

## §1 改动矩阵

| 文件 | 实施内容 | 状态 |
|---|---|---|
| `crates/star-api-rest/src/group_api/run_worktree_bindings.rs` | Run/Worktree 写事务以 Engineering Directory role 开始，设置 tenant/actor scope 与短超时；在同事务对 device generation 执行 `FOR SHARE` 检查，失败关闭。 | 源码实现；本地 PG18 `FOR SHARE`/拒写行为通过；并发撤销争用仍未验证 |
| `crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs` | 为受保护 mutation 提供 tenant/actor/device 对应 JWT fixture；隔离 PG probe 检查 NOINHERIT runtime 可 SET Directory role、对 generation 执行 `FOR SHARE`，并用 savepoint 断言直接 UPDATE 返回 SQLSTATE 42501 且 generation 不变。 | Rust route tests 2 passed/1 ignored；隔离 runner 的 binding、race、rollback、detach HTTP 场景通过；设备锁/RLS SQL probe 另行通过 |
| `crates/star-pg-adapter/src/repository/device_bindings.rs` | 五个操作统一在事务内先 SET OAuth owner role，再绑定 tenant/actor scope；错误不降级。 | PG18 repository behavior tests passed; end-to-end enrollment/reauthorization remains partial |
| `crates/star-pg-adapter/tests/device_auth_pg.rs` | 设备 owner-role membership 与 NOLOGIN/NOINHERIT/NOBYPASSRLS 属性探针；refresh fixture/断言也显式切换 owner role。 | 隔离 PostgreSQL 18 runner 实际执行两项 PG tests，2 passed；refresh fixture 通过测试专用 helper 创建 |
| `db/migrations/2026-10-28-w16-device-auth.sql` | OAuth owner 最小 DML/列权限及 scoped RLS；Directory owner 对 device binding 获得 SELECT 与 `UPDATE(generation)` 列 ACL。UPDATE RLS `USING` 限定同一 tenant/actor、`WITH CHECK(false)` 拒绝直接写入；这同时让 PostgreSQL `SELECT ... FOR SHARE` 锁住被授权行。OAuth owner 的 refresh-token SELECT 另加 tenant+actor restrictive RLS，收窄旧 tenant-wide permissive policy；refresh generation 配对约束显式禁止非空 device_id 搭配 NULL/非正 generation，并替换已存在的同名弱约束。 | 最新隔离 PostgreSQL 18 run 验证 owner/ACL、generation `FOR SHARE` 与拒写、refresh tenant+actor RLS、弱约束升级与非法配对回滚通过；并发撤销等待及生产审计契约仍开放 |

## §2 验证摘要

- Rust 1.99.0 rustfmt --edition 2024 --check：全部四个本 lane 修改的 Rust 源/测试文件 PASS。
- CypherGraph Guardian --changed --stamp：本轮 route-test 变更 PASS，1 个 source file。
- `rtk proxy cargo +1.99.0 check -p star-pg-adapter --all-targets --locked --offline -j4`：通过，7m19s；仅有 domain-kms 的既有 unused/deprecation warnings。
- `rtk proxy cargo +1.99.0 test -p star-pg-adapter --lib --locked --offline -j4`：35 passed，0 failed，0 ignored。
- rtk proxy cargo +1.99.0 test -p star-api-rest --test engineering_run_worktree_binding_routes --locked --offline -j4：2 passed，0 failed，1 ignored；ignored 用例包含 FOR SHARE/RLS denied-update probe，需独立 PostgreSQL acceptance runner 执行。
- `rtk proxy cargo +1.99.0 fmt --package star-pg-adapter -- --check`：失败于包内其他未修改文件的既有格式差异：`automation_schedule.rs`、`repository/mod.rs`、`oauth_authorization_codes.rs`、`schedule_postgres.rs`。未格式化这些无关文件；改动文件的定向 rustfmt 检查通过。
- `rtk proxy rustup run 1.99.0 rustfmt --edition 2024 --check crates/star-api-rest/tests/engineering_run_worktree_binding_routes.rs`：通过；`rtk proxy cargo +1.99.0 check -p star-api-rest --test engineering_run_worktree_binding_routes --locked --offline -j4`：通过。
- `rtk proxy cargo +1.99.0 test -p star-api-rest --test engineering_run_worktree_binding_routes --locked --offline -j4`：2 passed、0 failed、1 ignored。首次链接因临时 LNK1104 未生成测试可执行文件，立即重试通过；ignored PostgreSQL 用例仍由独立 runner 执行。
- `rtk git diff --check`：通过。
- 隔离 PostgreSQL 18 acceptance run `6af04e5a765446c8b3baaffff126bd52` 完成 70/70 步骤、失败 0；迁移、owner role/ACL、Directory device generation `FOR SHARE`/拒写、tenant+actor refresh RLS、旧弱同名 CHECK 升级与非法 NULL 配对回滚均通过。`device_auth_pg` 两项 PG 测试通过；hostless Run/Worktree binding、race、rollback 和 detach HTTP 场景通过；数据库仅监听 `127.0.0.1:52449`，目标数据库未触碰，未使用 Multipass/Docker。所有历史失败日志保留在 harness `.cache`。

## §3 已知缺口

- 只将 Run/Worktree 写事务内 device-generation 复验、OAuth/device repository owner-role/scope 顺序及 W16-A 已列明的 PG18 行为切片标为通过。真实 PostgreSQL 证明旧 `USING(false)` 会过滤 `SELECT FOR SHARE` 行，也证明旧 permissive refresh SELECT policy 泄露同租户其他 actor token；两项修正现在均由最终 run 复验通过。单事务 probe 不验证第二连接上的并发 revoke 等待。
- 设备 proof/enrollment/revoke 路由及 repository 源码存在，不代表认证 enrollment 已验收；PG ownership、挑战单次消费、撤销 refresh family 与并发行为仍需隔离 PostgreSQL 测试。
- 其他共享读取、订阅、OAuth credential resolve、SSE 撤销事件即时关闭/有界租约与跨节点失效未在本 lane 验收。E2E-COLLAB-12/13 保持 OPEN/NOT_RUN；OAuth 单赢家 E2E-14 也未由本 lane 关闭。
- KMS/provider、持久密文、durable drafts/conflicts、备份恢复与 PT-09/10 性能验收不属于本次闭环，继续保持 OPEN/NOT_RUN。

## §4 子代理失败接手清单

- 原实现 worker 与替补 worker因用量限制退出；协调者接手后修复的 ACL/CHECK/RLS 缺陷均由最终 PostgreSQL 18 run 复验通过。source/harness 复审未发现新增 P1/P2；CodeRabbit CLI 执行返回 permission denied，未视为通过。最终本地 PG18/HTTP harness 已通过，但结论仅限本地选定边界，不能外推到生产部署或完整 W16。

## §5 守门规则

- 应用登录身份不获 OAuth/device 表直接 ACL；部署配置必须显式配置所需 role membership，缺失时源码调用 fail closed。
- `star_oauth_credential_owner` 用于 OAuth/device owner 操作；`star_engineering_directory_owner` 仅有 `UPDATE(generation)` 列 ACL 来取得 generation `FOR SHARE` 锁。该 role 的 restrictive UPDATE policy 仅以 tenant/actor scope 决定锁行可见性，并用 `WITH CHECK(false)` 拒绝真实更新；probe 需证明匹配行可锁、非本 actor 行不可见、直接 UPDATE 返回 SQLSTATE 42501 且 generation 不变。
- 不把 Rust 编译、普通单测、mock 或 SQL 静态检查写成真实 PostgreSQL/RLS acceptance。
- 本提交仅属于当前隔离 W16-A lane；不得在此 lane rebase、merge、push 或触碰 `main`。

## §6 签字栏

| 角色 | 状态 |
|---|---|
| 架构 / 本报告审阅 | 架构师（Mavis 接手审核）；只审阅已实现切片与状态边界 |
| SRE Lead | 本地隔离 PG18 owner/RLS acceptance 已通过；Neon 与 Multipass/K3s 目标部署验收仍待执行 |
| 平台 Lead | 本地 role grants、owner membership 与所选 RLS 行为已通过；生产身份/审计契约仍待审查 |
| 评审主持 | PG18 与绑定 API 局部验证已记录；E2E-COLLAB-12/13 及 OAuth 单赢家仍未关闭 |
| PM | W16-A、W16、WLD-7 与 PT-09/10 保持 OPEN/PARTIAL |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 W16-A 的 Run/Worktree 写入 generation 复验、OAuth/device role 最小权限设计、验证结果与尚未完成的 PostgreSQL/请求面验收。 | 设备 owner-role/RLS 安全缺口修复。 |
| v0.2 | 2026-10-28 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 说明 `UPDATE(generation)` 是 PostgreSQL `FOR SHARE` ACL 所需的列级授权，restrictive RLS 继续禁止实际更新；补录配对约束 NULL 修复和锁/拒写探针状态，真实 PostgreSQL 结果仍 NOT_RUN。 | 安全审查发现 row-lock ACL 与 CHECK NULL 语义缺口。 |
| v0.3 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录 Rust 1.99.0 route test 2 passed/1 ignored、四个 changed Rust files rustfmt PASS、CGG PASS，以及两个只读审查的结果；真实 PostgreSQL/HTTP acceptance 仍等待专用 harness。 | 复核工作树修复并完成源码门禁。 |
| v0.4 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 真实 PostgreSQL 首轮发现旧 restrictive `USING(false)` 会过滤 `SELECT FOR SHARE` 设备行；修正为 tenant/actor scoped `USING` + `WITH CHECK(false)`，并将 Rust PG probe 改为 savepoint 捕获 SQLSTATE 42501。记录 runner 已发现并修正迁移顺序与跨租户 fixture context 问题；最终完整 DB/HTTP 验收仍待通过。 | PostgreSQL 18 本地实跑覆盖到 owner RLS 与设备行锁。 |
| v0.5 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 补录 PostgreSQL 行为 probe 发现 OAuth owner 可经旧 tenant-wide permissive SELECT policy 看到同租户另一 actor 的 refresh token（2 行而非预期 1 行）；新增 owner-targeted restrictive tenant+actor SELECT policy 和对应 catalog assertion。该修正尚未重跑验证，W16-A 仍 OPEN/PARTIAL。 | 隔离 PostgreSQL 18 RLS 行为实跑暴露 refresh-token 读取 scope 缺口。 |
| v0.6 | 2026-10-29 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录最终隔离 PostgreSQL 18 run `6af04e5a765446c8b3baaffff126bd52` 70/70 步骤通过、device repository 2/2 PG tests 与选定绑定 HTTP 场景；保留并发 revoke contention、Canvas 全读写/订阅/resolve、OAuth 单赢家及 SSE 撤销传播等开放边界。 | owner refresh RLS 修复已由最终运行复验。 |
