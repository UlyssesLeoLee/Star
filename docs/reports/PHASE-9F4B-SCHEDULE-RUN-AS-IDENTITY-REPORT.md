# Phase 9F4B — Schedule Run-as identity implementation report v0.1

> **状态**：🟢 Rule/Occurrence run-as identity persistence slice verified；🔴 Schedule worker 与 Run admission 未关闭，生产无人值守执行保持 fail closed
> **日期**：2026-10-04
> **基点**：`c1ce1630b6254a66c0cd3ebe0f65c494a25fd48a`
> **分支**：`codex/schedule-run-as-identity-20261004`
> **制定者**：Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核
> **签批**：🟢 Mavis 接手审核（仅本报告所述实现切片；不是生产执行验收）

---

## 0. 报告目的

本阶段按用户选择固定无人值守执行身份：创建 Schedule Rule 的已授权用户是永久 `run_as_actor_id`；所有后续 revision 和 occurrence 继承同一主体；每次将来发生触发、retry 或 resume 时，worker 必须重新校验该 actor 当前 Project/Run 权限，撤权或授权服务不可用即 fail closed。

本阶段落地并验证 Rule/Occurrence 身份 contract、持久化与历史 backfill。**没有实现 Schedule worker、实时 ACL reauthorization 或 TaskExecutionRun admission；不能将本阶段描述为无人值守执行可用。**

## 1. 改动矩阵

| # | 文件 | 改动与结果 |
|---:|---|---|
| 1 | `crates/domain-automation/src/schedule.rs` | Rule 与 occurrence snapshot 增加非空 run-as 身份；快照必须与其 pinned Rule revision 身份一致，并增加负例单测。 |
| 2 | `crates/star-api-rest/src/group_api/schedule_rules.rs` | create 从已授权 actor 派生身份；CAS successor 从当前锁定 revision 继承；body 拒绝调用者自选主体；`changed_by` 继续表示编辑者。 |
| 3 | `crates/star-pg-adapter/src/repository/automation_schedule.rs` | 读取和比较 Rule run-as，持久化独立 occurrence 身份列。 |
| 4 | `crates/star-pg-adapter/tests/schedule_postgres.rs` | 增加身份首版/后续版本/occurrence mismatch 负例及 RLS-scoped occurrence 主体持久化断言。 |
| 5 | `db/migrations/2026-10-04-schedule-run-as-actor.sql` | 为 Rule revision 和 occurrence 增加主体列、回填与 NOT NULL/non-nil；DB trigger 拒绝替换 Rule 创建者或写入与精确 Rule revision 不一致的 occurrence。回填时在同一事务/DDL 锁内受控停用旧 guard、临时恢复 schema owner 的 RLS bypass，随后恢复 guard 与 FORCE RLS。 |
| 6 | `scripts/automation/phase9f3_schedule.py` | 加入 Windows loopback Docker PostgreSQL backend、有限清理、legacy identity backfill fixture/assertion 和 API run-as 测试；API 链接测试使用 `-j 1`。 |
| 7 | `docs/requirements.md` | AC-LOOP-008 实施对账 v5.63，记录 legacy backfill 证据和未完成执行门。 |
| 8 | `docs/basic-design.md` | v5.60 同步 v5.63 需求与 backfill/迁移边界。 |
| 9 | `docs/data-design.md` | v1.1 定义 W/T/M identity 字段、owner backfill transaction/guard/RLS 恢复和目标 migration principal 未验收。 |
| 10 | `docs/design/DD-WORKTREE-GROUP-001.md` | v4.42 同步 occurrence identity、legacy fixture、migration 锁与未闭合 worker/admission。 |
| 11 | `docs/design/BD-AUTOMATION-SCHEDULE-API-001.md` | 同步 9F4B creator/run-as API contract（v0.7）。 |
| 12 | `docs/design/DD-AUTOMATION-SCHEDULE-API-001.md` | 同步 CAS successor、数据库 identity invariant、occurrence 与执行门（v0.7）。 |
| 13 | `docs/requirements/SRS-AUTOMATION-SCHEDULE-API-001.md` | 同步 SCHED-API 与 creator/run-as验收条件（v0.7）。 |
| 14 | `docs/implementation-plans/WORKTREE-GROUP-IMPL-PLAN-001.md` | v5.85 记录代码/DB fixture/验证矩阵和明确未关闭项。 |
| 15 | `docs/automation-design.md` | v2.0 更新 §4.45 runner 与执行边界。 |
| 16 | `scripts/automation/registry.md` | v0.50 更新 runner coverage 与开放门。 |

## 2. 验证摘要

### 2.1 最终 runner

实际通过的命令（Rust 工具均来自仓库锁定的 1.98.1 toolchain）：

```powershell
python scripts/automation/phase9f3_schedule.py `
  --cargo C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin\cargo.exe `
  --rustfmt C:\Users\leo19\.rustup\toolchains\1.98.1-x86_64-pc-windows-msvc\bin\rustfmt.exe `
  --postgres-docker-image postgres:18.6 `
  --docker 'C:\Program Files\Docker\Docker\resources\bin\docker.exe'
```

最终 runner `exit 0`，包含：

- Rustfmt check（2024 与 2021 edition）；`domain-automation` 26/26 tests；Schedule API 的“请求不可指定 run-as”测试 1/1。
- `star-pg-adapter` 与 `star-api-rest` all-targets checks、Schedule PostgreSQL test binary compile、domain/adapter Clippy 均通过。
- PostgreSQL 18.6 一次性 loopback 容器：9F2/9F4A/9F4B migration chain 首次和重复应用通过；migration 前插入两版旧 Rule（首版作者与 successor 编辑者不同）及一条旧 occurrence，迁移后验证所有 revision 使用首版作者、occurrence 从精确 pinned Rule revision 继承主体。
- 七张 Schedule 表最终全部 `FORCE ROW LEVEL SECURITY`；non-superuser、`NOBYPASSRLS` runtime role 下 5/5 adapter scenarios 通过，覆盖 tenant scope、幂等 materialization、并发/fencing、retry/terminal TTL 与 Rule/Occurrence run-as 不可替换/不匹配拒绝。
- 本次 runner 创建的 PostgreSQL 容器已清理；没有遗留 `star-schedule-*` 容器。
- `python -m py_compile scripts/automation/phase9f3_schedule.py` 通过；CypherGraph Guardian `--changed --stamp` 通过（5 个 source files）。

自审过程中，一次初始检查发现 test 的新字段查询没有设置 RLS tenant scope；查询移入当前 tenant transaction 后，完整 runner 通过。增加历史数据 fixture 后，首次运行又实际暴露迁移 UPDATE 被既有 SCD2/append-only trigger 拒绝；migration 已修复为事务内受控回填并恢复 guards/FORCE RLS，之后完整 runner 再次通过。

`git diff --check` 最终通过（仅提示四个既有 Rust 文件的 CRLF/LF 转换 warning，不含 whitespace error）。当前 `rtk` shim 不在 PATH，按绝对路径使用仓库 Rust 1.98.1 工具链执行。CodeRabbit Windows shim 指向 WSL `/root/.local/bin/coderabbit`，执行时报 `/bin/bash: Permission denied`（exit 126）；未获得独立 CodeRabbit 结果，已完成本地代码、SQL、迁移和文档自审。

### 2.2 未运行的验证

未运行整个 Cargo workspace/release 全门、frontend/Tauri desktop gates、目标 PostgreSQL migration/grants、真实宿主运行时或 Schedule worker/E2E。这些范围不属于本次 disposable runner 结果。

## 3. 已知缺口

1. Schedule clock/worker 尚不存在。每次 trigger、retry、resume 的 Project membership、Run grant、执行 capability 和 target recheck 尚未实现；撤权后的调度执行尚无生产行为可验证。
2. Occurrence 到 `TaskExecutionRun`、reservation、quota、RunEvent 的同事务 admission 未实现；缺失时必须不创建 Run。
3. Outbox consumer、Run BI/Benchmark 投影和 Schedule 可观测拒绝结果未接入。
4. Migration backfill 由 disposable DB 的 schema owner/superuser 运行验证；目标环境 migration principal、表 owner、锁窗口与 grants 未验收。
5. Backfill fixture 覆盖代表性双 revision 和匹配 occurrence；未读取任何目标环境已有数据，也未覆盖孤立/损坏历史行。找不到精确 Rule revision 时，本 migration 会在 NOT NULL 收敛处失败并回滚；上线前仍需审查目标数据。
6. 目标 PostgreSQL runtime grants/RLS policy 与真实部署流水线未验收；本机 runner 仅证明隔离容器结果。

## 4. 子代理失败接手清单

本阶段未派发子代理；因此没有子代理失败或未收回产出。最终改动均由当前隔离 worktree 自审并运行阶段 runner。

## 5. 守门规则

| # | 守门条件 | 状态 |
|---:|---|---|
| 1 | Rule 创建请求必须经过当前 actor 身份校验，并在当前 Project/Run 写权限通过后继续。 | ✅ |
| 2 | Project 与 Engineering Run 必须属于同一 Project；授权失败不得因幂等 replay 返回写结果。 | ✅ |
| 3 | Rule target 必须仍绑定当前 Run 的 Worktree、Task、Profile 与 HookSet 快照。 | ✅ |
| 4 | API body 严格拒绝 caller 自选 `run_as_actor_id`；首版主体只能来自已授权创建者。 | ✅ |
| 5 | CAS successor 从锁定当前 revision 继承同一 run-as；编辑者仅写入 `changed_by`。 | ✅ |
| 6 | DB 首版 Rule trigger 校验 `run_as_actor_id = changed_by`。 | ✅ |
| 7 | Rule identity trigger 对同 Rule 插入串行化并拒绝 successor 替换主体。 | ✅ |
| 8 | Domain Rule 与 occurrence snapshot 拒绝 nil 主体；occurrence 主体必须与 snapshot Rule 相同。 | ✅ |
| 9 | Adapter 先精确匹配已持久化 Rule revision，再将主体写入独立 occurrence 列。 | ✅ |
| 10 | 历史 Rule 所有 revision 使用最早版本 `changed_by`；旧 occurrence 从精确 pinned Rule version 回填。 | ✅（disposable fixture） |
| 11 | 回填事务仅修改新身份列，且在事务提交前恢复 SCD2/append-only guards 与两表 FORCE RLS。 | ✅（PostgreSQL 18.6 fixture） |
| 12 | 完整 Schedule migration chain 可重复应用，7 张表最终均 FORCE RLS。 | ✅（disposable DB） |
| 13 | 目标环境 migration owner/principal 与 grants 满足锁定回填需要。 | ⏳ 未在目标 DB 验收 |
| 14 | Worker 每次 trigger/retry/resume 重新授权当前 Project/Run/capability；撤权或无法读取 ACL 时不创建 Run。 | ⏳ Worker 未实现 |
| 15 | Run/reservation/RunEvent 原子 admission、Outbox consumer 与 BI outcome 可观测并验收。 | ⏳ 未实现 |

## 6. 签字栏

| 角色 | 修订/审核 | 结论 |
|---|---|---|
| 架构 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 🟢 身份归属与 fail-closed 边界符合本阶段设计；无人值守执行不验收 |
| SRE Lead | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 🟡 disposable migration/RLS 通过；目标 DB migration principal 与恢复演练开放 |
| 平台 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 🟢 Rust/API/adapter/PG fixture 验证通过；worker/runtime provider 未实现 |
| 评审主持 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 🟢 已对照 AC-LOOP-008、SRS、BD、DD 与阶段 runner；CodeRabbit 未能启动 |
| PM | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 🟡 9F4B identity persistence slice 可继续集成；整体 Schedule Loop 不关闭 |

## 7. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-04 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 建立 Phase 9F4B run-as identity 阶段报告；记录 legacy backfill 迁移修复、最终 PostgreSQL/Rust 验证与未实现 worker/Run admission 门禁 | 完成身份持久化切片、自审和文档同步 |
