# Brief: EX-01 — 5 张新表 DDL + RLS 13 类 + audit trigger

> **状态**: 🟡 Brief v0.1
> **拍板**: per `ask_2e8740e6779ac6a8d854c590` 4 推荐项 (A 4 阶段 8 wt 串行 / 每 wt 1 brief / 2.3M token 原估 / 阶段 merge cargo check)
> **wt-branch**: `wt-ex-01-5-tables-ddl`
> **base**: `main @ c32d876` (9/7 21:08 JST 推 origin 完成, ahead = 0)
> **触发**: 2026-09-07 21:43 JST 用户发令"启动,开子代理和worktree并行处理并在完成后merge到main"
> **关联**: [PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.1 EX-01](../../reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md) · [03-detailed-design.md §5 5 张新表完整 DDL](../architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md) · [ADR-0048 §2.3](../architecture/2026-08-26-upgrade/adr/0048-exclusion-idempotency-design.md) · [AGENTS.md §4 守门](../../AGENTS.md)

---

## 1. 目标 (Objective)

在 PostgreSQL (跟 ADR-0047 checkpointer Tier 3 共享) 上落档 **5 张新表 + RLS 13 类 + audit trigger**, per 守门 #13 a/c/d 派生规 100% 覆盖.

5 张新表 (per 守门 #13 W/T/M 严格分类, 禁止混在一括列举):

| # | 表名 | 分类 | 物理删除 | 审计 | RLS 13 类 |
|---|---|---|---|---|---|
| 1 | `idempotency_keys` | **T** Transaction | 禁止 (24h 后归档到 archive) | ✅ | ✅ |
| 2 | `lease_log` | **T** Transaction | 禁止 (append-only 永久) | ✅ | ✅ |
| 3 | `advisory_lock_audit` | **T** Transaction | 禁止 (append-only 永久) | ✅ | ✅ |
| 4 | `idempotency_keys_archive` | **T** Transaction | 禁止 (append-only 永久) | ✅ | ✅ |
| 5 | `exclusion_policy_master` | **M** Master | 禁止 (SCD Type 2 永久) | ✅ | ✅ |

## 2. 范围 (Scope)

### 2.1 In-Scope

- 5 张新表完整 DDL (含字段类型 / 主外键 / UNIQUE 约束 / CHECK 约束)
- 13 个索引 (含 3 个部分索引: `idx_lease_log_active` / `idx_excl_policy_active` / `idx_lock_audit_hold_desc`)
- 5 个 audit trigger (per ADR-0043 WORM, 挂 `audit_audit_event` 函数)
- 1 个 SCD Type 2 trigger (`scd_type2_close` 函数, 闭旧 valid_to)
- 10 个 RLS policy (5 表 × 2 policy = tenant + workspace 隔离, per 守门 #13 c)
- 2 个 SQL migration 文件落档 `docs/migrations/`
- 1 个 DDL 验证脚本 `scripts/automation/exclusion/verify_ddl.sh` (5 项检查: 表数 / RLS 数 / trigger 数 / 索引数 / SCD Type 2)

### 2.2 Out-of-Scope

- ❌ 任何应用层代码 (Rust / Python / TypeScript) — EX-02..EX-07 范围
- ❌ star-mutex crate 接入 — EX-02 范围
- ❌ DispatchLockManager / SubAgentLock / IdempotencyManager 调用 — EX-03/04/05 范围
- ❌ 16 tool 幂等改造 — EX-06 范围
- ❌ 可观测性 metric / 告警 / 归档 cron — EX-07 范围
- ❌ 18 UT + 8 IT + 12 E2E — EX-08 范围
- ❌ 真实 PG 实例 psql 跑 (本地无 PG, 仅 dry-run 语法验证)

## 3. 已知缺口 (per 守门 #11 缺标比错标)

- 本环境无 PG 实例, 只能 `psql --dry-run` 语法验证, 真实 PG 部署后跨 session 续实证
- `audit_audit_event` 函数依赖 ADR-0043 已落档的 WORM 触发器, 假设已存在 (per 02 §3 RLS policy 引用)
- SCD Type 2 `scd_type2_close` 函数假定 5 域 Lead 拍板在 Mavis 临时代签维持 (per 守门 #14 v2 拍板 D), 真人到位后追溯签字

## 4. 守门 (per 守门 #1 4 守门 + #9 + #10 + #11 + #13)

1. **守门 #1 4 守门**: 本子项不涉及 Rust, 仅 SQL, 走 `psql --dry-run` 语法验证 + `verify_ddl.sh` 5 项检查 0 错
2. **守门 #9 v3**: Mavis 直接落地, 不派 worker 子代理 (SQL DDL 简单任务, 子代理 RPC 不可靠实证下 Mavis 直做最稳)
3. **守门 #10 author=Ulysses**: 1 commit, author=`Ulysses <ulysses@mavis.local>`
4. **守门 #11 缺标比错标**: §3 显式列 3 缺口
5. **守门 #13 W/T/M 严格**: 5 表 100% 覆盖 (3 T + 1 T archive + 1 M SCD Type 2), 禁止混在一括列举
6. **守门 #13 a**: 5 表均无 L1↔L1 直通约束 (表层不涉及, 业务层 EX-02 实施)
7. **守门 #13 c Master 100% RLS**: `exclusion_policy_master` 100% RLS 必携 (tenant_id + workspace_id)
8. **守门 #13 d Transaction 100% audit**: 4 张 T 表 100% audit trigger 必携 (per ADR-0043 WORM)
9. **守门 #5 env 安全**: 不打印 DATABASE_URL, 仅引用 (per `docs/migrations/*.sql` 注释)

## 5. 依赖 (Dependencies)

### 5.1 上游依赖

- 无 (阶段 1 第一 wt, 阻塞 EX-02..EX-07)

### 5.2 下游阻塞

- EX-02 (star-mutex) — 依赖 `exclusion_policy_master` 表存在 (load policy)
- EX-03 (DispatchLockManager) — 依赖 `idempotency_keys` 表存在
- EX-04 (SubAgentLock) — 依赖 `lease_log` 表存在
- EX-06 (star-mcp 16 tool) — 依赖 `idempotency_keys` 表存在
- EX-07 (可观测性) — 依赖 5 表全部存在 (metric / 告警 / 归档)

## 6. 交付物 (Deliverables)

| # | 路径 | 描述 |
|---|---|---|
| 1 | `docs/migrations/2026-09-07-exclusion-rls.sql` | 5 张新表 DDL + 13 索引 + 10 RLS policy (~15KB) |
| 2 | `docs/migrations/2026-09-07-audit-trigger.sql` | 5 audit trigger + 1 SCD Type 2 trigger + 2 函数 (~5KB) |
| 3 | `scripts/automation/exclusion/verify_ddl.sh` | 5 项 DDL 验证脚本: 表数 (5) / RLS 数 (10) / trigger 数 (5) / 索引数 (13) / SCD Type 2 (1) (~2KB) |
| 4 | `docs/briefs/ex-01-5-tables-ddl.md` | 本 brief |

**总 4 文件, ~28KB raw**

## 7. 验收 (Acceptance Criteria)

### 7.1 守门实证

- [ ] `psql --dry-run -f docs/migrations/2026-09-07-exclusion-rls.sql` exit 0, 0 err
- [ ] `psql --dry-run -f docs/migrations/2026-09-07-audit-trigger.sql` exit 0, 0 err
- [ ] `bash scripts/automation/exclusion/verify_ddl.sh` exit 0, 5/5 项 PASS
  - 表数: 5 (idempotency_keys / lease_log / advisory_lock_audit / idempotency_keys_archive / exclusion_policy_master)
  - RLS 数: 10 (5 表 × 2 policy)
  - trigger 数: 5 (audit) + 1 (SCD Type 2) = 6
  - 索引数: 13 (含 3 部分索引)
  - SCD Type 2: 1 (exclusion_policy_master valid_from/valid_to)
- [ ] 5 表 100% 分类 (3 T + 1 T archive + 1 M) per 守门 #13
- [ ] 5 表 RLS 100% (10 policy) per 守门 #13 c
- [ ] 4 张 T 表 audit trigger 100% (per 守门 #13 d)
- [ ] 1 张 M 表 SCD Type 2 100% (valid_from + valid_to + scd_type2_close trigger) per 守门 #13 c

### 7.2 git 实证

- [ ] `git log -p --follow docs/migrations/2026-09-07-exclusion-rls.sql` 实证 DDL 完整
- [ ] `git log -p --follow docs/migrations/2026-09-07-audit-trigger.sql` 实证 trigger 完整
- [ ] commit author = `Ulysses <ulysses@mavis.local>` per 守门 #10
- [ ] 1 commit 含全部 4 文件, 不散落

## 8. 实施路径 (Implementation Path)

### 8.1 Mavis 直接落地 (per 守门 #9 v3 fallback)

1. 创建 wt branch: `git worktree add ../.worktrees/wt-ex-01-5-tables-ddl -b wt-ex-01-5-tables-ddl main`
2. 在 wt 内写 4 文件 (DDL / trigger / verify 脚本 / 本 brief 已落, 不重复)
3. `git add` + `git commit -m "..."` author=Ulysses
4. `git log -p --follow <file>` 实证 wt commit 完整
5. 切回 main: `git checkout main` + `git merge --no-ff wt-ex-01-5-tables-ddl -m "merge: EX-01 5 表 DDL 落地"`
6. 阶段 1 守门: `cargo check --workspace --lib -j 4` exit 0 (本子项不动 Rust, 应 0 err 快速通过) + 守门 #13 验证 (DDL grep 实证)
7. 推 origin: `git push origin main` (per 守门 #1 反转 9/7 21:08 JST 拍板)

### 8.2 文件写入顺序

1. `docs/migrations/2026-09-07-exclusion-rls.sql` (主 DDL, 5 表 + 13 索引 + 10 RLS policy)
2. `docs/migrations/2026-09-07-audit-trigger.sql` (5 audit trigger + 1 SCD Type 2 + 2 函数)
3. `scripts/automation/exclusion/verify_ddl.sh` (5 项检查脚本)
4. `git add` + `git commit` (author=Ulysses)

## 9. 风险 (Risks)

| 风险 | 概率 | 影响 | 缓解 |
|---|---|---|---|
| 本环境无 PG 实例, 真实部署后 DDL 报错 | 中 | 中 | 走 `psql --dry-run` 语法验证 + 部署阶段跨 session 续实证 |
| `audit_audit_event` 函数未在 PG 部署 | 低 | 高 | 在 DDL 文件顶部加注释, 引用 ADR-0043 |
| SCD Type 2 trigger 跟现有 trigger 命名冲突 | 低 | 中 | 用 `scd_type2_close` 独立函数, 不复用现有 trigger |
| 5 张表跟 ADR-0047 5 张 checkpointer 表命名空间冲突 | 极低 | 低 | 命名空间分离 (exclusion_ vs checkpoint_), 共享 PG 不共享表 |

## 10. 签字

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 |
| SRE Lead | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 平台 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| 评审主持 | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |
| PM | 🟢 架构师 (Mavis 接手 agent per DEC-008) | 2026-09-07 (per 守门 #14 v2 拍板 D 临时代签) |

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-07 21:43 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 | 初稿, 4 文件 28KB, 5 表 W/T/M 严格 + 16 守门 + 3 缺口 | per `ask_2e8740e6779ac6a8d854c590` 4 推荐项拍板 + 9/7 21:43 JST 用户发令"启动" |
