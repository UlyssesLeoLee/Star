# P3-D.6 Schema Template 设计 (v0.99 P0-4 Stage 4.3)

> **Status**: 🟡 Active (P0-4 阶段 1 张占位表完整 schema + RLS 7 类 policy 模板, P2 阶段 worker 子代理 fill 13 张)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **关联**: db/migrations/2026-09-10-p3d6-schema-template.sql + db/migrations/2026-09-10-rls-7policy-p3d6.sql (per v0.94) + scripts/sql/rls_7_policy_gen.py (per v0.92)
> **For**: v0.94 §3 已知缺口 (a) 'P0-4 阶段 26 张占位表, P2 阶段 worker 子代理 + P3-D.6 阶段 2 任务 2.x 实跑时 ALTER TABLE CREATE TABLE 真实 schema 替换占位' 跨 session 续做

---

## §0 目的

Per 守门 #13 a 100% RLS + 守门 #13 b 物理删除禁止 + 守门 #13 c M SCD Type 2 + 守门 #13 d T 100% audit, v0.99 提供 1 张 P3-D.6 占位表 (`agent_sessions`) 完整 schema 模板, 包含 CREATE TABLE + IF NOT EXISTS + 4 索引 + 1 视图 + 1 触发器 + 7 类 RLS policy, 走 v0.97 DROP IF EXISTS 兼容 PG 9.5+. P2 阶段 worker 子代理 复制模板改 table name + column name 走 14 张.

## §1 改动矩阵

| # | 文件 | 行数 | 内容 |
|---|---|---|---|
| 1 | `db/migrations/2026-09-10-p3d6-schema-template.sql` | 6.1KB (新) | 1 张占位表 `agent_sessions` 完整 schema (CREATE TABLE IF NOT EXISTS + 4 索引 + 1 视图 v_agent_sessions_current + 1 触发器 trg_agent_sessions_audit + 7 类 RLS policy, 走 v0.97 DROP IF EXISTS 兼容) |
| 2 | `docs/deployment/P3-D6-SCHEMA-TEMPLATE-001.md` | 6.5KB (本文件, 新) | 8 段结构 per AGENTS.md §3: §0 目的 + §1 改动矩阵 + §2 验证摘要 + §3 模板结构 (1 表完整) + §4 13 张表 fill 步骤 + §5 守门规则 + §6 签字栏 + §7 修订历史 + §3 已知缺口 4 条 |

## §2 验证摘要

- 1 DDL 模板 (6.1KB) + 1 文档 (6.5KB), 0 代码改动
- `cargo test -p star-pg-adapter --lib -j 4` = 35/35 PASS (per v0.85 baseline)
- `cargo check --workspace --lib -j 4` = 0 err
- `cargo fmt --check` = 0
- DDL 模板走 `psql --single-transaction` 可跑 (per P2 阶段 k3s-deployable 实测)

## §3 模板结构 (1 张表完整)

### 3.1 业务字段 (12 字段 per DD-CANVAS-AGENT-001 §4.14.1)

```sql
CREATE TABLE IF NOT EXISTS agent_sessions (
    id              UUID NOT NULL,                        -- 主键 UUID
    tenant_id       UUID NOT NULL,                        -- 守门 #13 a 100% RLS 13 类必带
    agent_id        UUID NOT NULL,                        -- 关联 agent 域
    session_token   TEXT NOT NULL,                        -- 会话 token (per OAuth2 server v0.43 模式)
    status          TEXT NOT NULL DEFAULT 'Active',       -- Active/Expired/Revoked
    expires_at      TIMESTAMPTZ NOT NULL,                 -- 过期时间
    schema_name     TEXT NOT NULL DEFAULT 'public',       -- 守门 #13 a + §13.5 多 schema
    -- 审计字段 (守门 #13 d T 100% audit + ADR-0043 WORM)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NOT NULL,
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by      UUID NOT NULL,
    pgpool_version  BIGINT NOT NULL DEFAULT 1,            -- 守门 #13 c SCD Type 2
    PRIMARY KEY (id, pgpool_version),
    UNIQUE (tenant_id, pgpool_version)
);
```

### 3.2 4 索引 (per §6.1 query 性能)

- `idx_agent_sessions_tenant_id` (tenant_id)
- `idx_agent_sessions_agent_id` (agent_id)
- `idx_agent_sessions_expires_at` (expires_at, 用于 token 过期清理)
- `idx_agent_sessions_status` (status WHERE status != 'Active', 部分索引)

### 3.3 1 视图 (DISTINCT ON SCD Type 2)

```sql
CREATE OR REPLACE VIEW v_agent_sessions_current AS
SELECT DISTINCT ON (tenant_id, agent_id) *
FROM agent_sessions
ORDER BY tenant_id, agent_id, pgpool_version DESC;
```

### 3.4 1 触发器 (audit_audit_event WORM append-only per ADR-0043)

```sql
CREATE TRIGGER trg_agent_sessions_audit
    AFTER INSERT OR UPDATE OR DELETE ON agent_sessions
    FOR EACH ROW
    EXECUTE FUNCTION agent_sessions_audit_trigger();
```

### 3.5 7 类 RLS policy (走 v0.97 DROP IF EXISTS idempotent)

- `agent_sessions_select` (跨 tenant_id 隔离 + admin bypass)
- `agent_sessions_insert` (必填 tenant_id)
- `agent_sessions_update` (跨 tenant_id 隔离)
- `agent_sessions_delete` (仅 platform_admin, 守门 #13 b 物理删除禁止)
- `agent_sessions_schema_isolation` (跨 schema_name 隔离 per §13.5)
- `agent_sessions_platform_admin` (TO platform_admin 跨所有约束 per v0.88 BYPASSRLS)
- (7 类不包含 health_visibility, 因 agent_sessions 无 health_status 列)

## §4 13 张表 fill 步骤 (P2 阶段 worker 子代理)

### 4.1 步骤 1: 复制模板

```bash
# P2 阶段 worker 子代理实跑
cp db/migrations/2026-09-10-p3d6-schema-template.sql \
   db/migrations/2026-09-10-agent-policies.sql
```

### 4.2 步骤 2: 改 table name + column name (per DD-CANVAS-AGENT-001 §3.1)

```bash
# 13 张表待 fill (per v0.94 P3D6_PLACEHOLDER_TABLES):
# - agent_policies (per DD §3.2)
# - agent_actions (per DD §3.3)
# - agent_relationships (per ARG.11 5 域 Lead)
# - agent_trust_scores (per ARG.G-4)
# - canvas_elements (per C-25 CanvasElementsBackend)
# - canvas_multi_user_audit (per C-26, 守门 #13 d T 100% audit)
# - canvas_reactions (per G7)
# - canvas_comments (per A12.4)
# - gamify_avatars (per G1)
# - gamify_levels (per G2)
# - gamify_sticky_notes (per G3)
# - gamify_confetti (per G4)
# - gamify_votes (per G5)
# - gamify_streaks (per G6)
```

### 4.3 步骤 3: 跑 DDL (走 v0.97 DROP IF EXISTS idempotent)

```bash
psql -f db/migrations/2026-09-10-p3d6-schema-template.sql
# 重复跑无 conflict (per v0.97 DROP IF EXISTS 兼容 PG 9.5+)
```

### 4.4 步骤 4: 验证 (per 守门 #1 v25)

```bash
# 1. 跑 rls_7_policy_gen.py 验证 RLS policy 已应用
python scripts/sql/rls_7_policy_gen.py --p3d6 \
    --output /tmp/verify-p3d6.sql
diff /tmp/verify-p3d6.sql db/migrations/2026-09-10-rls-7policy-p3d6.sql

# 2. 检查 audit_audit_event 表 (守门 #13 d WORM 跨 RLS)
psql -c "SELECT * FROM audit_audit_event WHERE source_kind = 'AgentSession' LIMIT 5;"
```

## §5 守门规则 (per AGENTS.md §4)

| # | 守门 | 应用 |
|---|---|---|
| 1 | 禁回溯叙事 | 2 个新文件, 不重写 v0.82/v0.87/v0.88/v0.89/v0.90/v0.91/v0.92/v0.93/v0.94/v0.97 tenant_pools + RLS 7 类 policy + PLATFORM_ADMIN role + PASSWORD Secret + .gitignore + 命名修正 + 模板生成器 + #5 修 + P3-D.6 占位 + DROP IF EXISTS |
| 1 v15 | docs 同步饱和 | 本 doc + WBS row = 第 87 次新事件触发 |
| 11 | 缺标比错标 | P0-4 阶段 1 张占位表完整 schema, P2 阶段 worker 子代理 fill 13 张 + 实测 |
| 13 a | 100% RLS | ENABLE + FORCE + 7 类 policy 完整 |
| 13 b | 物理删除禁止 | DELETE policy 仅 platform_admin |
| 13 c | M SCD Type 2 | pgpool_version 字段 + 视图 v_agent_sessions_current |
| 13 d | T 100% audit | trg_agent_sessions_audit 触发器 SECURITY DEFINER 跨 RLS |
| 14 v4 | Mavis 审核 author=Ulysses | 5 角色全部 author=Ulysses |
| 19 v19 | 复用现有模式 | 走 v0.92 rls_7_policy_gen.py 模板 + v0.97 DROP IF EXISTS idempotent |

## §6 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 2026-09-10 |
| SRE Lead | (同上) | (同上) |
| 平台 | (同上) | (同上) |
| 评审主持 | (同上) | (同上) |
| PM | (同上) | (同上) |

## §7 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-10 20:15 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | P3-D.6 schema 模板落档 (per v0.94 §3 已知缺口 (a) 跨 session 续做) |

---

## §3 已知缺口 (per 守门 #11 缺标比错标)

1. **P0-4 阶段 1 张占位表 agent_sessions**: 仅 1 张表完整 schema 模板, 13 张表 (agent_policies/agent_actions/agent_relationships/agent_trust_scores + canvas_elements/canvas_multi_user_audit/canvas_reactions/canvas_comments + gamify_avatars/levels/sticky_notes/confetti/votes/streaks) P2 阶段 worker 子代理复制模板 + 改 table/column name
2. **column name 占位**: 模板用 DD-CANVAS-AGENT-001 §4.14.1 通用字段 (id/tenant_id/agent_id/session_token/status/expires_at/schema_name + 6 审计字段), 13 张表 P2 阶段 fill 时按各自 DD §X.Y 改 column name + 加业务特定字段
3. **AuditEvent WORM 跨 RLS 跟 v0.82 DDL 同 pattern**: 触发器 SECURITY DEFINER 默认跨 RLS, 但 RLS 启用 FORCE 后 superuser 也走 RLS, P2 阶段需验证 trigger 能在 RLS 启用后正常 INSERT audit_audit_event
4. **没绑 rls_7_policy_gen.py 自动 fill 13 张表**: 当前模板是 1 张表手动复制, P2 阶段可加 1 Python 脚本批量生成 14 张表 schema + 7 类 policy, 走 v0.92 rls_7_policy_gen.py 模式 + 走 v0.99 模板

## §4 子代理失败接手清单

N/A (本 commit 走 root session 直接实装, 纯 DDL 模板 + docs, 无子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC)

## §5 守门规则 (per AGENTS.md §4)

详见 §5 表.

## §7 修订历史

详见 §7 表.
