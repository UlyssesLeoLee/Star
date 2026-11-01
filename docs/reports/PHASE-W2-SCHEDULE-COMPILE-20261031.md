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

**结论边界：**生产 library 与定向单元测试通过（诊断工具链 1.99.1）；Rust 1.99.0 all-targets 验收未通过，且 1.99.1 all-targets 仍被独立 PG test fixture 类型错误阻断。因此不能把此 lane 标记为稳定版 Rust gate 已通过。PG 集成测试没有运行；它的测试目标在编译期即失败。

## §3 已知缺口

1. Rust 1.99.0 当前安装的 rustc 组件无法启动，需修复 toolchain 后重跑最低版本验收。
2. `schedule_admission_pg.rs` 的两处 `Option<Uuid>` 格式化问题位于 lane 外，需由其 owner 修复；修复后重跑 `--all-targets` 与 PostgreSQL 集成测试。
3. 真实 PostgreSQL 执行结果未知；本轮未声称数据库行为已验收。
4. Schedule 的 branchless/hostless target 支持仍未实现。本 lane 按当前契约 fail closed；实现它需要独立的需求、授权、SQL 和测试设计。
5. 根 Cargo metadata 与 MIT 权利闭包、cargo-audit workflow pin、桌面/Web/Canvas hosted build、制品证明与真实部署仍是稳定版 WBS 的独立门禁。

## §4 子代理失败接手清单

本 lane 未再派生子代理。后续交接项：

- Rust toolchain owner：恢复 Rust 1.99.0 编译器组件，并重跑本报告中的最低版本命令。
- PG test owner：修正 `schedule_admission_pg.rs` 第 1405、1436 行的 `Option<Uuid>` 插值，再运行 all-targets 与相应 PG 集成测试。
- 根协调者：审查本 lane commit 和报告后，决定是否进入 MAGOS integration；不得将 1.99.1 诊断结果记作 1.99.0 release gate 通过。

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
