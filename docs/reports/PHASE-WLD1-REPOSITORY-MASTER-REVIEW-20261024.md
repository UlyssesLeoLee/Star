# PHASE-WLD1-REPOSITORY-MASTER-REVIEW-20261024

> 状态：只读独立审查完成；发现 2 项 P1 和 1 项 P2；不建议当前集成。Cargo、Rust 1.99.0 与 PostgreSQL 验收未执行。
> 审查 lane：`codex/wld1-review-20261024`，基线 `dev@d4f26ee`
> 被审查 lane：`codex/wbs-repository-master-20261023`，HEAD `5059508e5dcea98cd5a7a3aceb99e85ada60da10`，相对基线有 15 个登记路径改动。
> 范围：只读；未修改实施 lane，未 commit/rebase/merge/push。

## §0 目的

独立审查 WLD1 Repository master source slice 的权限与身份边界、SCD2/CAS、审计/RLS/grants、migration 安全、PostgreSQL test assertions，以及需求/基本设计/详细设计/追踪文档的一致性。源码目录没有 CodeGraph 索引；依 brief 对具体文件直接读取。实施 lane 自述的 Cargo/MSRV 与 PostgreSQL 测试阻塞按未验证处理。

## §1 审查发现（按严重度）

### P1 — 允许单独关闭 current revision，提交后可使 Repository master 消失

- 位置：`C:\Users\leo19\.codex\worktrees\wbs-repository-master-20261023\Star\db\migrations\2026-10-23-scm-repository-master.sql:31,62`；调用路径：`C:\Users\leo19\.codex\worktrees\wbs-repository-master-20261023\Star\crates\star-pg-adapter\src\repository\scm_repository_master.rs:299-319`
- migration 只给 owner role `UPDATE(valid_to)`；触发器允许把当前 revision 的 `valid_to` 改成 `transaction_timestamp()`，但没有要求同一事务插入连续 successor，也没有延迟约束保证提交时至少存在一个 current revision。持有该 owner role 的调用者可执行合法的单独关闭并提交。current partial unique index 只限制“最多一个”，不保证“至少一个”。之后 `find_current` 返回 `None`，而 identity 已有历史会阻止 `create` 恢复。
- 这同时绕过了“每个 revision 与 append-only audit 同事务形成完整变更”的预期：单独关闭旧行不会触发 INSERT audit。
- 建议：撤销直接 `UPDATE(valid_to)` 能力并通过受控过程完成关闭+插入，或增加 deferred constraint trigger，在事务提交时强制连续 successor/current 唯一性；加入 owner-role 直接关闭后提交必须失败的 PostgreSQL 用例。

### P1 — 复用已有 owner role 时，SCM 以外的既有对象授权会被保留

- 位置：`C:\Users\leo19\.codex\worktrees\wbs-repository-master-20261023\Star\db\migrations\2026-10-23-scm-repository-master.sql:19-25,60-63`
- migration 将 `star_scm_repository_owner` 设为 NOLOGIN/NOBYPASSRLS/NOINHERIT，并清理该 role 在 `scm` schema 与当前 database 的 CONNECT/CREATE/TEMPORARY 权限；漂移检查只拒绝其成为其他 role 的 member 和拥有对象。它没有检查或撤销该 role 在同一 database 的其他 schema/table/function 上的既有直接 grants。之后被授予 owner-role membership 的登录角色可 `SET ROLE`，继承这些遗留权限；NOINHERIT 不会移除 `SET ROLE` 后的直接授权。
- 因而“仅 SCM owner capability”的隔离只在假设这个 cluster-global role 从未被跨 schema 复用时成立，而 migration 没有验证该假设。文档对权限收敛的描述没有覆盖这类 privilege drift。
- 建议：保持不跨 schema 清理的策略，但在 grant 新权限前枚举并 fail closed 拒绝 scm 以外的 schema/object/function grants；测试应先注入一项非 SCM grant，确认 migration 整体回滚且不留下状态变化。

### P2 — ignored PostgreSQL test 的 disposable-target guard 不验证实际数据库身份

- 位置：`C:\Users\leo19\.codex\worktrees\wbs-repository-master-20261023\Star\crates\star-pg-adapter\tests\scm_repository_master_postgres.rs:55-77,137-142`
- 测试只要求环境变量 `STAR_SCM_REPOSITORY_DISPOSABLE_DATABASE=1`、两条 URL 字符串不相等，然后在检查 runtime 登录身份前就用 admin pool 应用 migration 两次。URL 字符串不等于不同 PostgreSQL 登录角色，也不能证明目标为 disposable/loopback 数据库；migration 会创建/调整 cluster-global owner role 并修改目标数据库 schema/grants。显式 `--ignored` 与环境变量是 opt-in，但缺少机器可验证的目标保护。
- 建议：在任何 migration 前，连接两个 endpoint 并验证 `session_user/current_user` 确实不同、两者指向相同且明确标记的 disposable database/cluster；校验数据库身份及 host/port，或要求来自受控 fixture 的一次性数据库标识。负向测试覆盖 production-like/default target 被拒绝。

## §2 范围与标准核对矩阵

| 审查项 | 证据与结论 | 状态 |
|---|---|---|
| 稳定 Repository identity / provider metadata | `RepositoryMasterRevision` 不含 Project/Branch；provider key、remote identity、URL、default branch 可选；Rust 与 SQL 均有长度/格式约束，credential-bearing URL 常见形式被拒绝。 | 源码契约一致；未编译/未执行 |
| Tenant / authorization boundary | adapter 显式 `SET LOCAL ROLE` 与 tenant/actor/correlation scope；master 与 event FORCE RLS。adapter 本身只处理 tenant，不校验 Project membership、Run grant 或 Project↔Repository link；本阶段没有 API/service authorization 层。文档正确要求候选及绑定由上游 owner 复验。 | owner slice 可接受；不得视为用户/API 授权验收 |
| Project、Run、Branch、root 身份分离 | master 没有 Project/Run/Branch 字段或 FK；新增实现没有 root registry、Run binding 或 provider/lifecycle capability。需求、SRS、BD、DD 与 Traceability 均将后续 owner 工作和 Worktree/CLI admission 保持 fail closed。 | 对齐；WTG-026 仍未完成 |
| CAS / SCD2 | adapter 按 identity advisory lock 与 `FOR UPDATE` 读取 current，比较 expected version，关闭旧 revision 后插入下一版本；trigger 检查版本连续、creator 不变及时间相接。发现 §1 P1 的独立关闭缺口。 | 有实现；完整事务不变量不成立 |
| Idempotency | 本 master repository port 有 correlation ID，但没有 idempotency key/receipt；本次列出的 WTG-026 master contract 未要求 master create/revise 的请求重放，Run binding 的 CAS/idempotency 属后续 owner。不要把 correlation ID 宣称为幂等键。 | 本切片不做幂等；未来 command/API 需要独立契约 |
| Audit / Outbox | INSERT trigger 在同一事务写 append-only audit，支持 exact actor/correlation 记录；migration 明确 audit event 不承担 Outbox delivery。没有 Repository master Outbox/consumer；文档未冒称 delivery 已实现。 | 审计源码存在；未运行；Outbox 未实现 |
| RLS / role grants | FORCE RLS、tenant GUC policy 与 owner role最小表列权限源码存在；未验证实际登录 member grants/RLS。发现 §1 P1 role privilege drift 检查缺失。 | 部分源码一致；不具部署证明 |
| Migration safety | 双次执行是 test 工件的一部分；脚本按既有 physical-line runner 的约定写成每语句一行；target DB 未执行。发现 §1 P1 权限漂移和 §1 P2 test target guard。 | 未获运行时验证 |
| PostgreSQL assertions | ignored test 设计覆盖双次 migration、非 superuser/member、RLS、并发 create、跨 tenant 隔离、跨 actor revision、CAS、SCD2/audit 与 audit mutation 拒绝；未覆盖单独 close、非 SCM grants 漂移或 target 身份误配。 | `#[ignore]`，未运行 |
| 文档同步 | requirements v6.95、basic v6.98、SRS v1.6、BD v1.7、Group DD v5.20、Traceability v1.5 与阶段报告状态相互引用；均区分 source 存在和验收完成，并记录 Cargo.lock 未更改。 | 范围内一致；保留实现缺口 |
| 锁文件 | `star-pg-adapter/Cargo.toml` 新增 `domain-scm` direct dependency；当前 `Cargo.lock` 的 `star-pg-adapter` package dependency 列表未包含 `domain-scm`。实施报告说明 MAGOS 冲突阻止 lock 更新。 | 集成 blocker；先解决锁文件冲突，不能以 `--locked` 宣称通过 |

## §3 验证摘要

| 检查 | 结果 |
|---|---|
| 差异清单 | `rtk git status --short` 显示恰好 15 个登记路径：9 个已跟踪修改、6 个新增文件；与 WLD1 brief/report 的 15-path manifest 一致。 |
| 基线/HEAD | 实施 lane HEAD 为 `5059508e5dcea98cd5a7a3aceb99e85ada60da10`；review lane 基于 `dev@d4f26ee`。 |
| `rtk git diff --check 5059508e5dcea98cd5a7a3aceb99e85ada60da10` | exit 0。它不覆盖未跟踪文件，不能当作全部 15 路径的检查。 |
| Cargo / Rust / PostgreSQL | 未运行。WLD1 report 记录 RTK 无法启动 Cargo，Rust 1.99.0 不可用，PostgreSQL 工件为 ignored 且未启动数据库；本审查没有安装或绕过依赖。 |
| CodeRabbit | 本次 `coderabbit --version` 解析到 `/root/.local/bin/coderabbit` 后返回 Permission denied；未向外部服务发送源代码，结论来自定向静态审查。 |

## §4 子代理失败接手清单

本只读审查未派生子代理，没有子代理产物需要接手。

## §5 守门规则与集成建议

1. **当前不集成此 lane**：先修复 §1 两项 P1，并补充明确回归测试。
2. PostgreSQL test 需在受保护的 disposable fixture 上运行，并先修复 §1 P2 的身份预检；不能因 `#[ignore]` 工件存在就标记 PG/RLS/grants/CAS 通过。
3. 解决 `Cargo.lock` 与 `domain-scm` 直接依赖的冲突后，使用 Rust 1.99.0 执行格式化、聚焦 test、adapter integration、workspace/all-targets 与目标 Clippy；本报告不替代这些门禁。
4. 重新审阅 migration 的回放和 privilege fingerprint，再决定是否 rebase/merge。Project authorization link、Run binding、Host root registry、provider/API 与 Worktree/CLI lifecycle 仍须后续 WBS 实施；本切片不能让它们进入可用状态。
5. 报告仅审查 WLD1 候选差异，不改变 dev、main、来源分支、其他 worktree 与用户数据。

## §6 审核签字栏

| 角色 | 审核者 | 备注 |
|---|---|---|
| 架构 | 架构师（Mavis 接手审核） | owner 分离和文档范围基本一致；两个权限/事务不变量阻止集成。 |
| SRE Lead | Mavis 接手审核 | PostgreSQL/MSRV 与目标身份门禁未运行。 |
| 平台 | Mavis 接手审核 | Rust 编译与锁文件同步未验证。 |
| 评审主持 | Mavis 接手审核 | 静态只读审查；未使用外部代码审查服务。 |
| PM | Mavis 接手审核 | 建议修复后复审，不将 source slice 当产品闭环。 |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-10-24 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手审核 | 完成 WLD1 Repository master 的独立只读审查，记录 current revision 可被孤立关闭、owner role 非 SCM grant 漂移、PG test target guard 与验证/集成边界 | WBS-WLD1-REVIEW-20261024 |
