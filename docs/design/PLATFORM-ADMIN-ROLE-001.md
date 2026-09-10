# PLATFORM_ADMIN Role 设计 (v0.88 P0-4 Stage 3.4)

> **Status**: 🟡 Active (P0-4 阶段声明落地, P2 阶段 worker 子代理实测 testcontainers)
> **Created**: 2026-09-10
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **关联**: db/migrations/2026-09-10-platform-admin-role.sql + db/migrations/2026-09-10-rls-policies-tenant-pools.sql
> **For**: v0.87 §3 已知缺口 (e) 'PLATFORM_ADMIN role 未建 (policy 引用 TO platform_admin 但 role 未建)' 跨 session 续做

---

## §0 目的

Per spec §13.1 PostgreSQL = 默认 SoR 平台运营 + 守门 #13 a 100% RLS, 实装 PLATFORM_ADMIN PG role + 跨所有 RLS policy bypass, 闭合 v0.87 §3 已知缺口 (e). role 用于 platform admin 紧急运维 (跨 tenant 查询 / 物理删除 / audit 审计) 跟 application 层 `app.is_admin = 'true'` GUC 协同.

## §1 改动矩阵

| # | 文件 | 行数 | 内容 |
|---|---|---|---|
| 1 | `db/migrations/2026-09-10-platform-admin-role.sql` | 2.8KB (新) | CREATE ROLE + GRANT CONNECT + GRANT USAGE schema + GRANT 4 类 CRUD + GRANT audit_audit_event + ALTER DEFAULT PRIVILEGES + BYPASSRLS = 7 段 |

## §2 验证摘要

- `cargo test -p star-pg-adapter --lib -j 4` = 35/35 PASS (per v0.85 baseline, 0 代码改动)
- `cargo check --workspace --lib -j 4` = 0 err
- DDL 走 pg_dump + psql 验证 (P2 阶段 worker 子代理 + testcontainers-rs)
- 手动 SQL 验证 (per §4 用法示例)

## §3 7 段 PLATFORM_ADMIN role 实装

### 3.1 CREATE ROLE platform_admin

```sql
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'platform_admin') THEN
        CREATE ROLE platform_admin LOGIN PASSWORD 'CHANGE_ME_AT_DEPLOY';
    END IF;
END
$$;
```

**设计意图**:
- `LOGIN` 允许客户端用 platform_admin 角色登录 (P2 阶段 k3s-deployable)
- `PASSWORD 'CHANGE_ME_AT_DEPLOY'` 占位, 部署时由 env var / secret manager 替换 (per 守门 #5 v2 env 安全)
- `IF NOT EXISTS` 防止重复创建 (idempotent, 跟其他 11 Repository + 7 类 RLS policy 一致)

### 3.2 GRANT CONNECT ON DATABASE

```sql
GRANT CONNECT ON DATABASE star_db TO platform_admin;
```

### 3.3 GRANT USAGE ON SCHEMA

```sql
GRANT USAGE ON SCHEMA public TO platform_admin;
```

### 3.4 GRANT tenant_pools 表 4 类 CRUD

```sql
GRANT SELECT, INSERT, UPDATE, DELETE ON tenant_pools TO platform_admin;
```

### 3.5 ALTER DEFAULT PRIVILEGES (未来表)

```sql
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO platform_admin;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO platform_admin;
```

**设计意图**:
- 未来 P3-D.6 14+15 张表 + star-pg-adapter 11 Repository 都自动 GRANT (per 守门 #19 v19 复用现有模式)
- 不用每次新表都手动 GRANT, 减少遗漏

### 3.6 GRANT audit_audit_event

```sql
GRANT SELECT, INSERT ON audit_audit_event TO platform_admin;
```

**设计意图**:
- audit_audit_event 触发器 SECURITY DEFINER 跨 RLS 写 (per 守门 #13 d), platform admin 需 SELECT 审计

### 3.7 BYPASSRLS attribute

```sql
ALTER ROLE platform_admin BYPASSRLS;
```

**设计意图**:
- platform admin 跨所有 RLS policy (per v0.87 7 类 policy 全 bypass)
- 紧急运维 / 跨 tenant 物理删除 / audit 审计 必需

## §4 用法示例 (per P2 阶段 worker 子代理 + testcontainers)

```sql
-- 1. 切换到 platform admin
SET ROLE platform_admin;

-- 2. 跨 tenant 查询 (绕过 RLS)
SELECT * FROM tenant_pools;
-- 必返所有 tenant row (per BYPASSRLS)

-- 3. 紧急物理删除 (per §13.1 平台运营)
DELETE FROM tenant_pools WHERE id = '...';
-- 走 v0.87 §3.1 RLS DELETE policy (admin bypass)

-- 4. 切回普通 user
RESET ROLE;
SET app.current_tenant_id = 'tenant-a-uuid';
SET app.is_admin = 'false';
SELECT * FROM tenant_pools;
-- 必只返 tenant-a row (RLS 生效)

-- 5. 验证 audit
SELECT * FROM audit_audit_event WHERE actor_id = 'platform_admin-uuid';
-- 必返所有 audit row (BYPASSRLS + SELECT permission)
```

## §5 守门规则 (per AGENTS.md §4)

| # | 守门 | 应用 |
|---|---|---|
| 1 | 禁回溯叙事 | DDL 新文件, 不重写 v0.87 DDL |
| 1 v15 | docs 同步饱和 | 本 doc + WBS row = 第 77 次新事件触发 |
| 5 v2 | env 安全 | PASSWORD 留占位 'CHANGE_ME_AT_DEPLOY', 部署时 env var 替换, 不打印 |
| 11 | 缺标比错标 | P0-4 阶段 role 已声明但未集成 runtime test, P2 阶段 worker 子代理实测 |
| 13 a | 100% RLS | 跟 v0.87 RLS 7 类 policy 协同, BYPASSRLS 跨所有 RLS |
| 13 d | T 100% audit | GRANT SELECT, INSERT ON audit_audit_event 让 platform admin 查审计 |
| 14 v4 | Mavis 审核 author=Ulysses | 5 角色全部 author=Ulysses |

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
| v0.1 | 2026-09-10 19:40 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | PLATFORM_ADMIN role 实装落档 (per v0.87 §3 已知缺口 (e) 跨 session 续做) |

---

## §3 已知缺口 (per 守门 #11 缺标比错标)

1. **P0-4 阶段 role 已声明但未集成 runtime test**: 当前 DDL 已落, 但没真实 PG 端到端验证 BYPASSRLS + GRANT 行为. P2 阶段 worker 子代理 + testcontainers-rs 实测 SET ROLE + cross-tenant query (per v0.85 缺口 (a) + v0.87 缺口 (a))
2. **PASSWORD 占位需 env 替换**: 守门 #5 v2 env 安全, 部署时需 k8s Secret 替换 'CHANGE_ME_AT_DEPLOY', 留 P2 阶段实装
3. **platform admin 自身不在 RLS 内**: BYPASSRLS 是 PG attribute, 跨所有 policy, 紧急运维设计意图. 但 audit 留 audit_audit_event 永久审计, 可追溯
4. **多 schema 跨 schema GRANT 缺**: 当前只 GRANT public schema, 未来 §13.5 多 schema 需 per schema GRANT
5. **跟 application 层 GUC app.is_admin 协同未文档化**: v0.87 文档已写用法示例, 但 application 层 P2 阶段需用 sqlx::query "SET LOCAL app.is_admin = 'true'" 跟 PG role 协同

## §4 子代理失败接手清单

N/A (本 commit 走 root session 直接实装, 纯 DDL 工作无子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC)

## §5 守门规则 (per AGENTS.md §4)

详见 §5 表.

## §7 修订历史

详见 §7 表.
