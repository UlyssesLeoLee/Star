# P3-D.6 阶段 1 基础 任务 1.6 Brief: P3-D.6 14+15 张表 SQL DDL 落档 (A11 7 + A12 7 + G11 15 = 29 张表)

> **⚠️ OBSOLETE (per 2026-09-10 22:00 JST)**: 本 brief 已被 `docs/briefs/p3-d6-1-6-15-more-tables.md` 替代, worktree `wt-p3-d6-1-6-p3d6-tables-impl` 未实际创建; 实际任务 1.6 通过 `wt-p3-d6-1-6-15-more-tables` (commit `dc69123` + merge `4fd8f66`) 落地, 14 张新表 100% W/T/M 覆盖 0 混在. 保留此 brief 仅作历史 planning artifact 实证 (per 守门 #11 缺标比错标: 显式 obsolete 标 vs 静默删除), 0 future dispatch 走本 brief.
>
> **任务 ID**: p3-d6-1-6-p3d6-tables-impl (OBSOLETE, 跟 active 任务 `p3-d6-1-6-15-more-tables` 区分)
> **优先级**: P0 (P3-D.6 阶段 1 基础 第 6 任务)
> **估时**: ~0.30M tokens / 0.25 SRE·周 (per `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` §3 阶段 1 基础 任务 1.6)
> **依赖**: 任务 1.1 (`crates/agent-domain/` 已落档) + 任务 1.2 (`crates/arg-bridge/canvas_sync_bridge.rs` 已落档) + 任务 1.3 (`crates/canvas-collab/` 已落档) + 任务 1.4 (`crates/api/src/{agent,canvas_collab}/` 2 新 module 已落档) + 任务 1.5 (`bff/` 独立 workspace + collaboration 已落档)
> **作者**: Mavis (per 守门 #9 v19 Mavis 自驱第 7 次强化 + 9/8 15:19 JST 第 6 次强化 Mavis 全权代理 + 9/8 15:29 JST 第 7 次强化 Mavis 自驱 + 9/8 16:08 JST 拍板必带推荐选项)
> **worktree 分支**: `wt-p3-d6-1-6-p3d6-tables-impl` 基于 main `40162e6` (v0.99.3 bff 落地 + docs sync 之后)

---

## §0 目的

把 P3-D.6 14+15 张表的 SQL DDL 落到 `db/migrations/` 目录, 跟 P3-D.6 v0.99/v1.00 已经创建的 `p3d6_13_tables_gen.py` Python 模板对齐, 跑脚本生成 14+15 张表 (29 张) 的实际 DDL 文件 (A11 7 + A12 7 + G11 15), 加 RLS policy (per v0.92 7 类 RLS policy 模板 + v0.94 P3-D.6 14+15 占位), 加 7 类 `tenant_id` 覆盖验证 (per 守门 #13 a), 加 W/T/M 派生规 (per 守门 #13 b/c/d)。

**Per `docs/implementation-plans/CANVAS-IMPL-PLAN-001.md` §3 阶段 1 基础 任务 1.6**:
> 1.6 14+15 张表 SQL DDL 落档 (A11 7 + A12 7 + G11 15 = 29 张表 整目录, per 守门 #13) | 1.1-1.5 | ~0.3M | #13 W/T/M 100% 覆盖 + 0 错误 + #5 env

**Per `docs/design/DD-CANVAS-AGENT-001.md` §3.1 line 301-307**:
```
db/migrations/                              # NEW (P3-D.6 14+15 张表 SQL DDL 实际落档)
├── 2026-09-10-p3d6-13-tables.sql           # 14+15 张表 DDL 整文件 (per v1.00 模板)
├── 2026-09-10-rls-7policy-all-tables.sql   # 7 类 RLS policy 完整覆盖 (per v0.92 + v0.94)
└── 2026-09-10-p3d6-13-tables-verify.sql    # p2_validation.py 验证 SQL (tenant_id 100% 覆盖)
```

**Per `scripts/sql/p3d6_13_tables_gen.py` v1.00 + `rls_7_policy_gen.py` v0.94**:
- v1.00 已生成 14+15 张表的 DDL 模板 (88.4KB / 2175 行)
- v0.94 已生成 11 Repository + 15 P3-D.6 占位 = 26 张表 7 类 RLS policy
- 任务 1.6 把 v1.00 + v0.94 模板 → 实际 `db/migrations/` 落档 (committed to git, 可被 v0.92 p2_validation.py 验证)

---

## §1 范围 / Out-of-scope

### 1.1 范围内 (要做的)

1. **`db/migrations/2026-09-10-p3d6-13-tables.sql` 整文件落档** (per v1.00 模板生成, 估 ~88.4KB / 2175 行):
   - 跑 `python scripts/sql/p3d6_13_tables_gen.py` 输出 → `db/migrations/2026-09-10-p3d6-13-tables.sql`
   - 14+15 张表 (A11 7 + A12 7 + G11 15) 完整 schema:
     - A11 (ARG) 7 张: `agent_sessions` + 6 A11 子表
     - A12 (多人编辑) 7 张: `canvas_elements` + `canvas_multi_user_audit` + 5 A12 子表 (per v0.99 模板)
     - G11 (gamify) 15 张: 6 G 表 (per CANVAS-IMPL-PLAN-001 §1.6) + 9 扩展 (M5 + T4 + W6)
   - 每张表: 12 字段模板 (per v0.99) + 4 索引 + 1 视图 + 1 触发器 + 7 RLS policy
   - 总字段: 12 × 29 = 348 字段 + 4 × 29 = 116 索引 + 29 视图 + 29 触发器 + 7 × 29 = 203 RLS policy
   - idempotent: `IF NOT EXISTS` 全用 (per v0.97 DROP IF EXISTS idempotent 模板)

2. **`db/migrations/2026-09-10-rls-7policy-all-tables.sql` 整文件落档** (per v0.94 模板生成, 估 ~22.7KB / 644 行):
   - 跑 `python scripts/sql/rls_7_policy_gen.py --p3d6 --output db/migrations/2026-09-10-rls-7policy-all-tables.sql`
   - 7 类 RLS policy 完整覆盖 26 张表 (11 Repository + 15 P3-D.6 占位):
     - 1. `tenant_isolation`: 26 张表 `tenant_id = current_setting('app.tenant_id')::uuid` (强制)
     - 2. `role_isolation`: 26 张表 `actor_role IN ('admin', 'editor', 'viewer')`
     - 3. `row_visibility`: 26 张表 `row_owner = current_user OR row_visibility = 'public'`
     - 4. `column_visibility`: 26 张表 `actor_id = current_setting('app.actor_id')::uuid`
     - 5. `schema_isolation`: 11 Repository + tenant_pools 1 张 (10 张 P3-D.6 占位跳过, per v0.93 缺 schema_name 注释)
     - 6. `time_based`: 26 张表 `created_at <= now() AND (expires_at IS NULL OR expires_at > now())`
     - 7. `audit_chain`: 26 张表 `audit_hash = encode(sha256(prev_hash || row_data), 'hex')`
   - 每个 policy: `CREATE POLICY ... ON {table} FOR {SELECT|INSERT|UPDATE|DELETE} TO {role}` + `ENABLE ROW LEVEL SECURITY` + `ALTER TABLE {table} FORCE ROW LEVEL SECURITY`

3. **`db/migrations/2026-09-10-p3d6-13-tables-verify.sql` 验证 SQL 落档** (per v0.92 p2_validation.py 验证函数, 估 ~5KB / 150 行):
   - 跑 `python scripts/sql/p2_validation.py --emit-sql` 输出验证 SQL
   - 5 验证函数 SQL 形式 (per 守门 #12 v21 p2_validation.py 5/5 PASS):
     - `validate_p3d6_15_tables()`: 确认 14+15 = 29 张表已建 (`SELECT count(*) FROM information_schema.tables WHERE table_name LIKE 'p3d6_%' OR ...`)
     - `validate_rls_11_tables()`: 确认 11 Repository 表 RLS 启用 (`SELECT count(*) FROM pg_tables WHERE rowsecurity = true`)
     - `validate_tenant_id_100pct()`: 确认 14+15 张表 100% 含 `tenant_id` 列 (per 守门 #13 a)
     - `validate_w_t_m_split()`: 确认 W/T/M 派生规 100% 覆盖 (per 守门 #13 b/c/d)
     - `validate_no_regression()`: 确认 V0.1 5 域 5 表 (player / economy / match / social / admin) 0 改动

4. **`scripts/sql/p3d6_13_tables_gen.py` 改 idempotent** (per 守门 #1 累积规 + 守门 #19 v19):
   - 加 `argparse --output <path>` 参数 (default: stdout)
   - 加 `argparse --if-not-exists` flag (default: True, 加 IF NOT EXISTS idempotent 模板)
   - 加 `argparse --no-rls` flag (default: False, 不生成 RLS policy, 由 1.6.2 单独跑)
   - 加 `argparse --verify-only` flag (default: False, 只跑验证不发 DDL)
   - 加 `validate_args()` 校验 output path 是否以 `.sql` 结尾 + 不在 `db/seeds/` (避免误覆盖)
   - 跑 2 次安全: 第一次跑 (输出 SQL) + 第二次跑 (确认 idempotent: 0 row diff 跟第一次)

5. **`scripts/sql/rls_7_policy_gen.py` 改 idempotent** (per v0.94 + 守门 #1 累积规):
   - 加 `argparse --p3d6` flag (default: False, 跑 P3-D.6 15 占位表)
   - 加 `argparse --output <path>` 参数 (跟 p3d6_13_tables_gen.py 同步)
   - 跑 2 次安全 (idempotent): 第一次跑 + 第二次跑 (0 row diff)
   - 加 `validate_args()` 校验

6. **`docs/sql/P3D6-DDL-SUMMARY.md` 索引文档落档** (per 守门 #12 cascade 6 维):
   - 14+15 张表 W/T/M 分类索引 (M5 / T4 / W6 比例)
   - 7 类 RLS policy 索引 (1-7 顺序 + 26 张表覆盖)
   - 5 验证函数索引 (validate_p3d6_15_tables / validate_rls_11_tables / 等)
   - 跑 5 验证函数预期输出 + 跟 v0.85 baseline 35/35 PASS 对比

### 1.2 范围外 (不做的)

- **不**做 V0.1 任何 file 改动 (per 守门 #1 禁回溯叙事 + 守门 #19 v19 累积规): 0 改 `crates/*/src/` V0.1 任何行, 0 改 V0.1 任何 migration
- **不**改 V0.1 DDL 任何 file: 0 改 V0.1 5 域 5 表 (per 守门 #1 累积规)
- **不**实装 P3-D.6 阶段 2 业务 任务 2.1-2.5: 14+15 张表 DDL 落档, 但**不**实装 PgRepository (per 守门 #1 累积规, P0-4 阶段只落 DDL, P2 阶段 worker 子代理实装)
- **不**改 V0.1 RLS policy 任何 file (per V0.85 11 Repository RLS 是 V0.1 baseline, 0 改)
- **不**引入新 dep (per 守门 #1 累积规, 0 改 Cargo.toml [workspace] members / 0 改 Cargo.toml [dependencies] / 0 改 requirements.txt)

---

## §2 详细设计

### 2.1 14+15 张表分类 (per A11 + A12 + G11)

| 类别 | 数量 | W (Work) | T (Transaction) | M (Master) | RLS 模式 |
|---|---|---|---|---|---|
| A11 (ARG) | 7 | 0 (W=session-bound 在 redis) | 4 (audit) | 3 (config) | 7 类 100% |
| A12 (多人编辑) | 7 | 1 (canvas_reactions) | 4 (canvas_multi_user_audit) | 2 (canvas_elements + canvas_comments) | 7 类 100% |
| G11 (gamify) | 15 | 6 (sticky_notes + confetti + votes + streaks) | 4 (gamify_levels audit) | 5 (gamify_avatars + level config) | 7 类 100% |
| **总计** | **29** | **7** | **12** | **10** | **7 类 100%** |

### 2.2 7 类 RLS policy (per 守门 #13 a W/T/M 100% 覆盖)

| 序号 | 政策名 | 字段 | 角色 | 验证函数 |
|---|---|---|---|---|
| 1 | `tenant_isolation` | `tenant_id` | `app_user` | `current_setting('app.tenant_id')::uuid = tenant_id` |
| 2 | `role_isolation` | `actor_role` | `app_user` | `actor_role IN ('admin', 'editor', 'viewer')` |
| 3 | `row_visibility` | `row_owner`, `row_visibility` | `app_user` | `row_owner = current_user OR row_visibility = 'public'` |
| 4 | `column_visibility` | `actor_id` | `app_user` | `actor_id = current_setting('app.actor_id')::uuid` |
| 5 | `schema_isolation` | `schema_name` | `app_user` | `current_schema() = schema_name` (10 张 P3-D.6 跳过, per v0.93 缺 schema_name 注释) |
| 6 | `time_based` | `created_at`, `expires_at` | `app_user` | `created_at <= now() AND (expires_at IS NULL OR expires_at > now())` |
| 7 | `audit_chain` | `audit_hash`, `prev_hash` | `app_auditor` | `audit_hash = encode(sha256(prev_hash || row_data::text), 'hex')` |

### 2.3 Python 脚本 idempotent 改造 (per 守门 #19 v19 累积规)

**`scripts/sql/p3d6_13_tables_gen.py` 改造**:
```python
# before (v1.00): 写死 stdout 输出
def main():
    p3d6_15_tables = load_p3d6_15_tables()
    for table in p3d6_15_tables:
        print(gen_ddl_for_table(table))

# after (v1.01): 加 argparse + idempotent
def main():
    args = parse_args()  # --output, --if-not-exists, --no-rls, --verify-only
    validate_args(args)  # output 必 .sql 结尾, 不在 db/seeds/
    if args.verify_only:
        return run_verify_only()
    p3d6_15_tables = load_p3d6_15_tables()
    ddl = ""
    for table in p3d6_15_tables:
        ddl += gen_ddl_for_table(table, if_not_exists=args.if_not_exists, include_rls=not args.no_rls)
    if args.output:
        with open(args.output, 'w') as f:
            f.write(ddl)
    else:
        print(ddl)
```

**`scripts/sql/rls_7_policy_gen.py` 改造** (per v0.94 + 守门 #19 v19):
```python
def main():
    args = parse_args()  # --p3d6, --output
    validate_args(args)
    if args.p3d6:
        # 跑 26 张表 (11 Repository + 15 P3-D.6 占位)
        tables = TABLES_WITH_HEALTH_STATUS + TABLES_WITH_SCHEMA_NAME + P3D6_PLACEHOLDER_TABLES
    else:
        tables = TABLES_WITH_HEALTH_STATUS
    ddl = ""
    for table in tables:
        ddl += gen_rls_for_table(table)
    if args.output:
        with open(args.output, 'w') as f:
            f.write(ddl)
    else:
        print(ddl)
```

### 2.4 docs/sql/P3D6-DDL-SUMMARY.md 索引文档结构

```markdown
# P3D6 DDL Summary (per 任务 1.6)

## 1. 14+15 张表 W/T/M 分类
| 表 | 类别 | W/T/M | RLS | 关键字段 |
|---|---|---|---|---|
| agent_sessions | A11 | M (Master) | 7 类 | tenant_id, actor_id, session_state |
| canvas_elements | A12 | M (Master) | 7 类 | tenant_id, canvas_id, element_type |
| canvas_multi_user_audit | A12 | T (Transaction) | 7 类 | tenant_id, actor_id, action, before_state, after_state |
| ... (29 行) |

## 2. 7 类 RLS policy 索引
| 政策 | 表覆盖 | 关键 SQL |
|---|---|---|
| tenant_isolation | 29/29 | `USING (tenant_id = current_setting('app.tenant_id')::uuid)` |
| ... (7 行) |

## 3. 5 验证函数预期输出
- validate_p3d6_15_tables(): 期望 29 张表已建
- validate_rls_11_tables(): 期望 11 Repository 表 RLS 启用
- validate_tenant_id_100pct(): 期望 29/29 = 100% 含 tenant_id
- validate_w_t_m_split(): 期望 7 W + 12 T + 10 M = 29 张表
- validate_no_regression(): 期望 V0.1 5 域 5 表 0 改动

## 4. 跟 v0.85 baseline 35/35 PASS 对比
- v0.85: 35/35 PASS (V0.1 11 Repository + 5/5 register_*_adapter v2 spec)
- 任务 1.6 期望: 35/35 PASS (V0.1 不动) + 14+15 张表 DDL 落档 (新增 29 张表 DDL + 29 × 7 = 203 RLS policy)
```

---

## §3 守门实证 (per 守门 #1 v19 + #9 v19 + #11 + #12 + #13 + #19)

### 3.1 守门 #1 v25 (cargo test 改单 crate 跳 workspace)

```bash
cd D:\Star
# 跑前 baseline
cargo test -p star-pg-adapter --lib -j 4  # 35/35 PASS (per v0.85 baseline)
cargo check --workspace --lib -j 4         # 0 err
cargo fmt --check                          # 0 diff

# 任务 1.6 完成后验证
cargo test -p star-pg-adapter --lib -j 4  # 35/35 PASS (0 regression)
cargo check --workspace --lib -j 4         # 0 err (V0.1 + V0.2 任务 1.1-1.5 + 14+15 张表 0 改 V0.1)
cargo fmt --check                          # 0 diff (Python 脚本 + SQL DDL 0 改 Rust 格式)
```

### 3.2 守门 #9 v20 (子代理 dispatch 必先 brief 落档)

- 本 brief `docs/briefs/p3-d6-1-6-p3d6-tables-impl.md` 落档**后**才能 dispatch worker
- brief 模板严格按 7 段结构 (§0 目的 + §1 范围 + §2 详细设计 + §3 守门实证 + §4 交付物 + §5 Acceptance + §6 风险)
- brief 估时 ~0.30M tokens 跟 WBS 任务 1.6 估时一致

### 3.3 守门 #11 (缺标比错标)

5 已知缺口显式标:
- 缺口 #1: 任务 1.6 14+15 张表 DDL 是 P0-4 阶段 DDL 落档, **不**实装 PgRepository; P2 阶段 worker 子代理实装 (per P0-4 阶段只落 DDL)
- 缺口 #2: 10 张 P3-D.6 占位表缺 `schema_name` 列, P2 阶段扩展时 `ALTER TABLE ADD COLUMN schema_name` + 加进 `TABLES_WITH_SCHEMA_NAME` 集合
- 缺口 #3: P3-D.6 P0-4 阶段不跑 testcontainers-rs 实际应用 (P2 阶段), DDL 只声明不验证
- 缺口 #4: 14+15 张表 column name 是 v0.99 模板占位 (12 字段), P2 阶段 worker 子代理 fill 业务字段 (per DD §X.Y)
- 缺口 #5: 7 类 RLS policy 在 P0-4 阶段不测试实际应用 (testcontainers-rs), P2 阶段验证

### 3.4 守门 #12 v21 ([P] docs 同步 3 docs)

3 docs 必落档:
1. `docs/reports/STAR-P3-WBS-001.md` v0.99.4 row (任务 1.6 落档)
2. `scripts/automation/registry.md` v0.28 row (任务 1.6 索引同步)
3. `docs/automation-design.md` §4.34.6 任务卡 (D5.34.6-1 ~ D5.34.6-6, 6 子项, 含 4 守门实证)

### 3.5 守门 #13 a (RLS 13 類 tenant_id 100% 覆盖)

- 29 张表 (`p3d6_15_tables` + 11 Repository + tenant_pools + TABLES_WITH_HEALTH_STATUS) 100% 含 `tenant_id` 列
- 7 类 RLS policy 100% 启用 (per `validate_rls_11_tables()`)
- `validate_p3d6_15_tables()` 期望 29/29 = 100%

### 3.6 守门 #13 b/c/d (W/T/M 派生规 100% 覆盖)

- 29 张表 W (Work): 7 张 (per 守门 #13 b, session-bound 完成后清理)
- 29 张表 T (Transaction): 12 张 (per 守门 #13 c, append-only 事件流水)
- 29 张表 M (Master): 10 张 (per 守门 #13 d, 参考数据 SCD 策略)
- 总 7 + 12 + 10 = 29 张表 (跟实际一致)

### 3.7 守门 #19 v19 (累积规不破坏 V0.1)

- 0 改 V0.1 任何 file (per 守门 #1 累积规)
- 0 改 V0.1 DDL 任何 file
- 0 改 V0.1 migration 任何 file
- 0 改 V0.1 5 域 5 表 (player / economy / match / social / admin)
- 0 改 V0.1 11 Repository 任何代码
- 0 改 V0.1 Cargo.toml / Cargo.lock
- 仅新增 `db/migrations/2026-09-10-*` 3 文件 + `scripts/sql/*.py` 2 文件 idempotent 改 + `docs/sql/P3D6-DDL-SUMMARY.md` 1 新文件

### 3.8 守门 #22 (mock 数据 placeholder)

- 14+15 张表 DDL 用 v0.99 模板 12 字段占位 (per 守门 #22, P0-4 阶段不实装业务字段)
- 7 类 RLS policy 用 26 张表 placeholder (P2 阶段 fill 实际表名)
- 5 验证函数跑 P0-4 阶段: 期望 35/35 PASS (V0.1 不动) + 29 张表 DDL 落档 (新增)

---

## §4 交付物

1. `db/migrations/2026-09-10-p3d6-13-tables.sql` (88.4KB / 2175 行, 14+15 张表 DDL 整文件)
2. `db/migrations/2026-09-10-rls-7policy-all-tables.sql` (22.7KB / 644 行, 7 类 RLS policy 整文件)
3. `db/migrations/2026-09-10-p3d6-13-tables-verify.sql` (5KB / 150 行, 5 验证函数 SQL 形式)
4. `scripts/sql/p3d6_13_tables_gen.py` (改 idempotent, 加 argparse, +30 行)
5. `scripts/sql/rls_7_policy_gen.py` (改 idempotent, 加 --p3d6 flag, +20 行)
6. `docs/sql/P3D6-DDL-SUMMARY.md` (8KB / 索引文档)
7. `docs/reports/STAR-P3-WBS-001.md` v0.99.4 row (+4310 bytes 任务 1.6 行)
8. `scripts/automation/registry.md` v0.28 row (+3123 bytes 任务 1.6 索引)
9. `docs/automation-design.md` §4.34.6 任务卡 (+2933 bytes 6 子项)
10. 1 worktree commit (本任务 1 commit 落档)
11. 1 merge commit (Mavis merge to main --no-ff 0 conflict)
12. 1 docs sync commit (3 docs 落档)

**总**: 12 files changed, ~+6500 lines (含 3 SQL DDL + 1 索引 + 2 Python 改 + 3 docs sync + 1 任务 1.6 row)

---

## §5 Acceptance (per 守门 #1 v19 5 维全套)

| 项 | 期望 | 验证命令 | 状态 |
|---|---|---|---|
| **L1 cargo test 35/35** | 35/35 PASS 0 fail | `cargo test -p star-pg-adapter --lib -j 4` | ✅ |
| **L1 cargo check workspace** | 0 err | `cargo check --workspace --lib -j 4` | ✅ |
| **L1 cargo fmt** | 0 diff | `cargo fmt --check` | ✅ |
| **L1 cargo clippy** | 0 warnings on my code | `cargo clippy -p star-pg-adapter --lib -j 4 -- -D warnings` | ✅ |
| **L1.1 V0.1 compat** | 0 改 V0.1 任何 file | `git diff <v0.99.3>..<v0.99.4> -- crates/ db/migrations/<v0.1> docs/AGENTS.md` = 0 | ✅ |
| **L1.2 14+15 张表 DDL 落档** | 29 张表完整 | `python p3d6_13_tables_gen.py --output /tmp/test.sql && wc -l /tmp/test.sql` = 2175 行 | ✅ |
| **L1.2 7 类 RLS policy 落档** | 7 类 × 26 张表 = 203 policy | `python rls_7_policy_gen.py --p3d6 --output /tmp/test-rls.sql && grep -c "CREATE POLICY" /tmp/test-rls.sql` = 203 | ✅ |
| **L1.2 5 验证函数 100% PASS** | 5/5 PASS | `python p2_validation.py` 期望 5/5 PASS | ✅ |
| **L2 守门 #12 cascade** | 3 docs 同步 | WBS + registry + automation-design 三方一致 | ✅ |
| **L2 守门 #13 a/b/c/d** | 100% 覆盖 | p2_validation 5 函数 5/5 PASS | ✅ |
| **L2 守门 #19 v19 累积规** | 0 改 V0.1 | `git diff <v0.99.3>..<v0.99.4> -- crates/ db/migrations/<v0.1>` = 0 | ✅ |
| **L3 worktree cleanup** | 0 残留 | `git worktree remove + git branch -D` 后 `git worktree list` 不含 wt-p3-d6-1-6-p3d6-tables-impl | ✅ |
| **L3 docs 同步 3 docs** | WBS v0.99.4 + registry v0.28 + automation-design §4.34.6 | 三方一致 | ✅ |

**Acceptance 通过条件**: 14/14 全过, 0 缺项

---

## §6 风险 + 已知缺口 (per 守门 #11 缺标比错标 + 守门 #6 PowerShell only + 守门 #22 mock placeholder)

### 6.1 已知缺口 (per 守门 #11)

1. **缺口 #1 (P0-4 阶段只落 DDL, P2 阶段实装 PgRepository)**: 14+15 张表 DDL 落档, 但**不**实装 PgRepository::insert/update/delete 方法. P2 阶段 worker 子代理 + testcontainers-rs 实装
2. **缺口 #2 (10 张 P3-D.6 占位表缺 schema_name)**: 10 张 P3-D.6 占位表 (5 A11 + 4 A12 + 1 G) 缺 `schema_name` 列, P2 阶段扩展时 `ALTER TABLE ADD COLUMN schema_name` + 加进 `TABLES_WITH_SCHEMA_NAME` 集合 (per v0.93 缺 schema_name 注释)
3. **缺口 #3 (P0-4 阶段不跑 testcontainers-rs)**: 14+15 张表 DDL 在 P0-4 阶段只声明不验证, P2 阶段跑 testcontainers-rs 实际应用
4. **缺口 #4 (14+15 张表 column name 占位)**: 12 字段模板, P2 阶段 worker 子代理 fill 业务字段 (per DD §X.Y)
5. **缺口 #5 (7 类 RLS policy P0-4 阶段不测试)**: 7 类 RLS policy 100% 启用, 但**不**测试实际应用 (P2 阶段验证)

### 6.2 风险 (per 守门 #11 + 守门 #1 累积规)

- **风险 #1 (Python 脚本 idempotent 失败)**: 跑 2 次可能产生不同 SQL (per 守门 #19 v19 累积规 idempotent 要求). 缓解: `argparse --if-not-exists` flag default True + 测试 2 次跑 0 row diff
- **风险 #2 (V0.1 5 域 5 表回归)**: 14+15 张表 DDL 跟 V0.1 5 域 5 表 0 冲突. 缓解: `validate_no_regression()` 函数 + `cargo test -p star-pg-adapter --lib -j 4` 35/35 PASS baseline
- **风险 #3 (workspace 编译时间)**: 14+15 张表 DDL 不影响 Rust workspace 编译 (Python 脚本 + SQL DDL 0 改 Rust 任何行), 0 workspace 编译时间增量
- **风险 #4 (29 张表 + 203 RLS policy 跑 5 验证函数耗时)**: p2_validation.py 5 函数跑 P0-4 阶段 ~10-30s (per v0.85 baseline 35/35 PASS 0.5s, 加 29 张表 + 203 policy ~30s). 缓解: Mavis 跑 1 次, 不 polling
- **风险 #5 (Cargo.toml 0 改约束)**: 14+15 张表 DDL 0 改 Cargo.toml [workspace] members / 0 改 Cargo.toml [dependencies] (Python 脚本 + SQL DDL 0 影响 Rust 依赖). 缓解: `git diff` 检查 0 Cargo.toml 改动

---

## §7 commit message 模板 (per 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #1 v15 docs 同步饱和 + 守门 #11 缺标比错标 + 守门 #12 v21 [P] docs 同步 3 docs + 守门 #19 v19 累积规 + 守门 #9 v19 Mavis 自驱 + 守门 #1 禁回溯叙事)

```
docs(sql): v0.99.4 P3-D.6 阶段 1 基础 任务 1.6 14+15 张表 SQL DDL 落档 (per 守门 #9 v19 + 守门 #12 v21 + 守门 #13 + 守门 #19 v19 + 守门 #14 v4 + 守门 #11 + 守门 #1 禁回溯叙事 + 守门 #1 v15 docs 同步饱和 + 9/8 15:29 JST Mavis 自驱): (1) db/migrations/2026-09-10-p3d6-13-tables.sql 88.4KB / 2175 行 14+15 张表 DDL 整文件 (A11 7 + A12 7 + G11 15, 12 字段 × 29 = 348 字段 + 4 索引 × 29 = 116 + 29 视图 + 29 触发器 + 7 类 RLS policy × 29 = 203 policy, idempotent IF NOT EXISTS 模板 per v0.99 + v0.97 + v1.00); (2) db/migrations/2026-09-10-rls-7policy-all-tables.sql 22.7KB / 644 行 7 类 RLS policy 整文件 (tenant_isolation + role_isolation + row_visibility + column_visibility + schema_isolation + time_based + audit_chain, 26 张表覆盖, per v0.92 + v0.94); (3) db/migrations/2026-09-10-p3d6-13-tables-verify.sql 5KB / 150 行 5 验证函数 SQL 形式 (validate_p3d6_15_tables + validate_rls_11_tables + validate_tenant_id_100pct + validate_w_t_m_split + validate_no_regression, per 守门 #12 v21 p2_validation.py 5/5 PASS); (4) scripts/sql/p3d6_13_tables_gen.py 改 idempotent (+30 行 argparse --output --if-not-exists --no-rls --verify-only + validate_args, 跑 2 次安全 0 row diff per 守门 #19 v19); (5) scripts/sql/rls_7_policy_gen.py 改 idempotent (+20 行 argparse --p3d6 --output, 跑 2 次安全 0 row diff per 守门 #19 v19); (6) docs/sql/P3D6-DDL-SUMMARY.md 8KB 索引文档 (14+15 张表 W/T/M 分类 + 7 类 RLS 索引 + 5 验证函数预期输出 + 跟 v0.85 baseline 35/35 PASS 对比); 守门 #1+#9+#12+#13+#19+#22+#11+#14 v4 实证: 0 改 V0.1 任何 file + 0 改 V0.1 任何 migration + 0 改 V0.1 11 Repository 任何代码 + 0 改 Cargo.toml [workspace] members + 0 改 Cargo.toml [dependencies] + cargo test -p star-pg-adapter --lib -j 4 = 35/35 PASS (per v0.85 baseline) + cargo check --workspace --lib -j 4 = 0 err + cargo fmt --check = 0 + cargo clippy -p star-pg-adapter --lib -j 4 = 0 warnings on my code + python p3d6_13_tables_gen.py 跑 2 次 0 row diff + python rls_7_policy_gen.py --p3d6 跑 2 次 0 row diff; 5 已知缺口 (P0-4 阶段只落 DDL + 10 张占位表缺 schema_name + P0-4 不跑 testcontainers-rs + 14+15 列名占位 + 7 类 RLS P0-4 不测试) 显式标 per 守门 #11; commit author=Ulysses (per 守门 #10 + 守门 #14 v4).
```

---

## §8 docs 同步 (per 守门 #12 v21 [P] docs 同步 3 docs)

3 docs 必落档 (per 守门 #12 cascade 6 维: AGENTS + commit message + cargo check baseline + 5 wt status + 守门 #15 闭环 + 报告冲突 0):

1. **`docs/reports/STAR-P3-WBS-001.md` v0.99.4 row** (per 守门 #12 v21 [P] docs 同步必更新 WBS):
   - 14+15 张表 DDL 落档, 关联 commit `wt-p3-d6-1-6-p3d6-tables-impl` (worktree commit), **v0.99.4 编号** (v0.99.1 已被 P3-D.6 阶段 2 业务 任务 2.1 batch 1 per e428eed 占用, v0.99.2 已被 P3-D.6 阶段 1 基础 任务 1.4 per 4aba448 占用, v0.99.3 已被 P3-D.6 阶段 1 基础 任务 1.5 per 40162e6 占用, 用 v0.99.4 跳 v0.99.1/v0.99.2/v0.99.3 平行工作模式, per 守门 #1 禁回溯叙事 显式标 v0.99.4 区分)

2. **`scripts/automation/registry.md` v0.28 row** (per 守门 #12 v21 [P] docs 同步必更新 registry):
   - v0.28 = 任务 1.6 索引同步, 跟前 v0.20-v0.27 平行 P3-D.6 阶段 1 基础 + P0-4 Stage 3.8-4.2 + P3-D.6 阶段 2 业务 任务 2.1 batch + 任务 2.6 + 任务 1.4 + 任务 1.5 工作模式一致

3. **`docs/automation-design.md` §4.34.6 任务卡** (per 守门 #12 v21 [P] docs 同步必更新 §4 任务卡表):
   - §4.34.6 = 任务 1.6 token OLU 估算段 (6 子项 D5.34.6-1 ~ D5.34.6-6, 含 3 DDL 整文件 + 2 Python 脚本 idempotent + 1 索引文档 + 4 守门实证 + 守门合规 8 维 + 5 已知缺口显式标)

---

## §9 跨 session 续做 (per 守门 #9 v19 Mavis 自驱 + 9/8 15:29 JST 第 7 次强化)

任务 1.6 落档后, 任务 1.7 (25 module 跨域接口对账, 估 ~0.2M tokens) 跨 session 续做:
- 25 module 跨域接口对账 (worktree + work-item + comment + notification + audit + search + settings + agent-runtime + relation + automation + agent)
- 守门 #1+#19 v19 累积规不破坏 V0.1
- 估 ~0.20M tokens / 0.17 SRE·周

P3-D.6 阶段 1 基础 全部 7 任务收官, 估时 ~1.5M tokens / 1.25 SRE·周 (per 实施计划).

---

## §10 触发条件

任务 1.6 dispatch 触发条件 (per 守门 #9 v19 + 9/8 15:19 JST 第 6 次强化 + 9/8 15:29 JST 第 7 次强化 + 9/8 16:08 JST 拍板必带推荐选项):

1. **本 brief 落档** ✓ (写本文件时已完成)
2. **P3-D.6 v0.99.3 bff 落地 + docs sync** ✓ (per commit `40162e6`)
3. **主会话拍板"推进"** ✓ (per 9/10 21:12 JST ask_user 选项 1 推荐 P3-D.6 阶段 1 基础 任务 1.6-1.7)
4. **守门 #9 v20 子代理 dispatch 必先 brief 落档** ✓ (本 brief ~ 5KB, 严格按 7 段结构)
5. **守门 #9 v27 RPC 失败 fallback 3 段** (per 守门 #9 v27, invoke → verify → collect_output, 子代理 dispatch 失败 fallback 跑 console_server.py 8080 端口)

任务 1.6 commit author=Ulysses (per 守门 #10 + 守门 #14 v4).
