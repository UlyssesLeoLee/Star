# PHASE-W2-SCHEDULE-COMPILE-20261031

## §0 目的

修复 Run Schedule admission 中一个未定义的 branch binding 引用，使生产库可编译，同时保留现有“必须绑定远端 Branch”的 fail-closed 规则。本报告只覆盖本 lane 的最小代码修复；不代表稳定版 W2 或 W9 已关闭。

## §1 改动矩阵

| 项目 | 结果 | 证据 |
|---|---|---|
| Run Schedule 当前任务授权查询 | 将错误的局部变量 `branch_id` 替换为交易对象在开始 admission 时已固定的 `self.branch_id` | `crates/run-owner-runtime/src/schedule_admission.rs` 的 `ScheduleAdmissionTransaction.revalidate_task_binding` |
| Branchless Schedule 处理 | 未放宽。wire decoder 仍拒绝缺失/空 Branch，`required_schedule_branch_id(None)` 仍 fail closed | 同文件 `hostless_schedule_target_in_wire`、`decode_branch_bound_schedule_command` 与对应单元测试 |
| 数据库、权限、schema、锁文件 | 未修改 | 本 lane diff 仅涉及上表源码和本报告/brief/dispatch 记录 |
| CypherGraph header | 保留既有行为关系并刷新源文件 hash | `cgg_guard.py --changed --stamp` 检查通过 |

## §2 验证摘要

运行环境：Windows `x86_64-pc-windows-msvc`。诊断工具链为 `cargo 1.99.1 (5f94df478 2026-08-27)`、`rustc 1.99.1 (58cf8f9c3 2026-10-13)`。项目最低基线仍是 1.99.0；使用 1.99.1 仅作诊断，不会替代最低版本验收。

| 命令 | 退出 | 实际结果 |
|---|---:|---|
| `cargo +1.99.0 check -p run-owner-runtime --all-targets --locked -j 4` | 1 | 未进入编译。rustup 报告 rustc.exe 不适用于 `1.99.0-x86_64-pc-windows-msvc` toolchain。 |
| `cargo +1.99.1 check -p run-owner-runtime --all-targets --locked -j 4` | 1 | 新的 production-source 未定义引用已消失；检查被未修改的 `crates/run-owner-runtime/tests/schedule_admission_pg.rs` 阻断：第 1405、1436 行把 `Option<Uuid>` 以默认 Display 格式插入 SQL 字符串。该测试文件相对本 lane 基线无差异，本 lane 未越界修改。 |
| `cargo +1.99.1 check -p run-owner-runtime --lib --locked -j 4` | 0 | production library 编译通过。依赖 `domain-kms` 有 4 条已有 unused/deprecation warnings。 |
| `cargo +1.99.1 test -p run-owner-runtime --lib --locked -j 4` | 0 | 20 passed，0 failed；包括 branchless wire 拒绝和 Schedule branch 必需校验两项测试。 |
| `git diff --check` | 0 | 通过；Git 提示该源码工作副本的 CRLF 将在后续 Git 操作时转换为 LF。 |
| `python C:\Users\leo19\.codex\skills\CypherGraph-Guardian\skill\cypher-graph-guardian\scripts\cgg_guard.py --changed --stamp` | 0 | `PASS (1 source file checked)`。仓库内未提供 `.agents/skills/cypher-graph-guardian` 副本，因此按该 Skill 指示从安装目录运行。 |

**当时结论边界（历史快照）：**生产 library 与定向单元测试通过（诊断工具链 1.99.1）；Rust 1.99.0 all-targets 验收未通过，且 1.99.1 all-targets 当时被独立 PG test fixture 类型错误阻断。因此不能把此 lane 标记为稳定版 Rust gate 已通过。PG 集成测试没有运行；它的测试目标在编译期即失败。

## §2.1 2026-11-01 current-dev revalidation

本节补充当前 `dev@6e7d51a7088aa978d67f75197fadf0597db2925f` 的复核结果；§2 的原始输出保留为该 lane 当时的历史快照。

| 检查 | 结果 |
|---|---|
| `rtk proxy cargo +1.99.1 check -p run-owner-runtime --all-targets --locked -j 4` | ✅ exit 0，52.76 秒。此前报告记录的 PG fixture `Option<Uuid>` 插值问题已由当前 dev 中的提交 `eb1e91f4` 修复；该诊断编译现在通过。 |
| 编译 warning | ⚠️ `domain-kms` 输出 4 条 warning：1 条 unused import（`SecretDataKey`）和 3 条 `Nonce::from_slice` deprecation；它们未阻断本次检查，未在本 lane 修改。 |
| Rust 1.99.0 最低版本门 | ⛔ 未通过/未验证。最初 exact check 因 rustc 组件不适用于该 toolchain 而未进入编译。2026-11-01 再次运行 `rtk proxy rustup toolchain install 1.99.0-x86_64-pc-windows-msvc --profile minimal`；rustup 报告恢复部分安装，选择 2026-10-01 的 1.99.0 manifest，清理旧组件并开始下载 6 个组件。约 60 秒没有新输出后停止；遗留的同一安装子进程已按命令行核实并终止。随后检查确认 toolchain 目录仍缺 `multirust-channel-manifest.toml` 与 `bin/rustc.exe`，且没有该安装进程在运行。两个 `.broken` 备份仍在。观察到下载停滞，但不推断其根因。 |
| PostgreSQL 集成测试 | 未运行；本次只执行 `cargo check`，不得视为数据库验收。 |
| 源码/manifest/lock | 未修改。本 lane 仅更新证据文档与审计记录。 |

**状态更新：**此前 §3 “PG fixture 两处插值错误待修复”已被 `eb1e91f4` 消除；这只关闭该编译阻塞。W2 仍 OPEN/BLOCKED：Rust 1.99.0 精确版本门、BFF lockfile 的 locked metadata/review、其他发行目标依赖闭包与 PostgreSQL 集成验收均未完成。整体稳定版保持 NO-GO。

## §3 已知缺口

1. Rust 1.99.0 最低版本验收仍未通过：toolchain directory 存在，但缺少可用 `rustc.exe` 和 channel manifest。两次官方 rustup 安装/恢复均未完成；最新 minimal-profile 尝试下载 6 个组件后约 60 秒无进展而停止，两个 `.broken` 备份目录保留。需恢复可运行的 1.99.0 compiler 后重跑验收；下载停滞原因未知。
2. 此前发现的 `schedule_admission_pg.rs` 两处 `Option<Uuid>` 格式化问题已由 dev 提交 `eb1e91f4` 修复；当前 dev 的 1.99.1 `--all-targets` 诊断编译通过。PostgreSQL 集成测试仍需执行。
3. 真实 PostgreSQL 执行结果未知；本轮未声称数据库行为已验收。
4. Schedule 的 branchless/hostless target 支持仍未实现。本 lane 按当前契约 fail closed；实现它需要独立的需求、授权、SQL 和测试设计。
5. 根 Cargo metadata 与 MIT 权利闭包、cargo-audit workflow pin、桌面/Web/Canvas hosted build、制品证明与真实部署仍是稳定版 WBS 的独立门禁。

## §4 子代理失败接手清单

本 lane 未再派生子代理。后续交接项：

- Rust toolchain owner：恢复 Rust 1.99.0 编译器组件，并重跑本报告中的最低版本命令。
- PG test owner：fixture 编译错误已由 dev 提交 `eb1e91f4` 修复；仍需运行相应 PostgreSQL 集成测试并记录结果。
- 根协调者：W2 文档 lane commit `2f30c52c49f0994e2f37d394a8e04f3ef2b9f88b` 已经审查并快进合入本地 dev，MAGOS claim 已释放且 lane 已清理；不得将 1.99.1 诊断结果记作 1.99.0 release gate 通过。W2 BFF lockfile 是独立 lane，继续保留其未提交内容等待 owner 的 locked recheck。

## §5 守门规则

1. lane 基线为 `dev@a1fed3437660be80d75c2741e62837ab1bbd2501`。
2. 仅修改已登记的 Schedule 源文件和 lane 报告/dispatch 文件。
3. Schedule admission transaction 中的 branch binding 在受理时固定，不从不可信 wire 或临时局部变量重新推导。
4. 缺失 Branch 的 wire 仍被拒绝；branchless schedule 测试保持有效。
5. 未改变 run-as actor、Project/Run 权限复验、SQL 参数顺序或数据 schema。
6. 未改 `Cargo.toml`、`Cargo.lock`、迁移或其他 owner 文件。
7. Rust 1.99.0 命令已实际运行，失败如实记录。
8. Rust 1.99.1 仅为诊断工具链，不充当最低支持版本证据。
9. `--all-targets` 的失败完整保留，不以 lib-only 成功覆盖。
10. PG fixture 错误与生产源文件错误分开归因。
11. 未修改 PG fixture 以绕过本 lane 范围。
12. lib-only 检查使用 `--locked` 与 `-j 4`。
13. 20 项 lib 单测全部实际运行并记录。
14. 未将 PG integration test 或真实数据库行为标为已验证。
15. CypherGraph semantic edge 未变化，只刷新了 source hash。
16. CypherGraph Guardian 对一个变更源文件检查通过。
17. `git diff --check` 通过；稳定版总 WBS 保持 OPEN。

## §6 签字栏

| 角色 | 记录 |
|---|---|
| 架构审核 | Mavis / 根协调者需在集成前复核此 lane 的提交；此 worker 报告不宣称已完成根协调者审查。 |
| SRE Lead | 本 lane 未执行部署或运行环境变更；不构成 SRE 发布批准。 |
| 平台 | 本 lane 未改平台基础设施或 owner schema。 |
| 评审主持 | 由根协调者在接受 lane 前完成独立集成审查。 |
| PM | 稳定版范围/MIT 决策来自本对话；本报告不构成稳定版发布批准。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-31 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 agent | 记录 Schedule admission 单行 branch binding 修复、运行证据与尚未解除的工具链/PG fixture 门禁。 | W2 stable release follow-up lane |
| v0.2 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 追加当前 dev 的 1.99.1 all-targets 诊断通过证据；说明旧 PG fixture 编译阻塞已由 eb1e91f4 修复，同时保留 Rust 1.99.0、PostgreSQL 与发行依赖门禁开放。 | 当前 dev 重新执行 W2 Schedule all-targets 编译检查 |
| v0.3 | 2026-11-01 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 记录第二次 Rust 1.99.0 minimal-profile 安装恢复尝试、下载停滞后的安全停止、toolchain manifest/rustc 缺失复查及 W2 文档 lane 已集成清理的事实；W2 与稳定发行仍未关闭。 | 当前 toolchain 重试结果与 W2 lane 集成回执 |
