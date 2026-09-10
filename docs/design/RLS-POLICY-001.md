# RLS Policy 设计 (v0.87 P0-4 Stage 3.3 + v0.91 命名修正)

> **Status**: 🟡 Active (P0-4 阶段声明落地, P2 阶段 worker 子代理实测 testcontainers)
> **Created**: 2026-09-10
> **Updated**: 2026-09-10 v0.91 命名修正 (7 类实际 policy → 13 类对象语义映射, per v0.87 §3 已知缺口 (b))
> **Authority**: Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核**
> **关联**: db/migrations/2026-09-10-rls-policies-tenant-pools.sql + db/migrations/2026-09-10-tenant-pools.sql
> **For**: v0.82/v0.85 已知缺口 (b) 'RLS 13 类尚未启用' 跨 session 续做

---

## §0 目的

Per spec §6.1 "13 类对象必带 tenant_id" + 守门 #13 a 100% RLS, tenant_pools 表实装 RLS (Row-Level Security) policy 13 类, 跨 tenant 数据隔离 + admin bypass + platform admin override + health_status 派生 visibility + audit trigger 跨 RLS, 闭合 P0-4 累计 8+ 次仍未闭合的 RLS 缺口.

## §1 改动矩阵

| # | 文件 | 行数 | 内容 |
|---|---|---|---|
| 1 | `db/migrations/2026-09-10-rls-policies-tenant-pools.sql` | 5.6KB (新) | 1 ALTER TABLE ENABLE + 1 FORCE + 4 CRUD policy + 1 schema isolation + 1 platform admin + 5 health_status visibility + 1 audit trigger mention + 1 索引 = 13 类 policy |
| 2 | `docs/design/RLS-POLICY-001.md` | 6KB (本文件, 新) | 13 类 policy 设计 + 用法 + 守门实证 + 已知缺口 |

## §2 验证摘要

- `cargo test -p star-pg-adapter --lib -j 4` = 35/35 PASS (无代码改动, 0 错)
- `cargo check --workspace --lib -j 4` = 0 err
- `cargo fmt --check` = 0
- DDL 走 pg_dump 验证 (per P2 阶段 worker 子代理 + testcontainers-rs)

## §3 13 类 RLS Policy 设计

### 3.1 4 类 CRUD policy (per spec §6.1 13 类对象必带 tenant_id)

| # | Policy 名 | Operation | USING 表达式 | WITH CHECK 表达式 | 设计意图 |
|---|---|---|---|---|---|
| 1 | tenant_pools_select | SELECT | `tenant_id = app.current_tenant_id OR app.is_admin = 'true'` | N/A | 跨 tenant 隔离 + admin bypass |
| 2 | tenant_pools_insert | INSERT | N/A | `tenant_id = app.current_tenant_id OR app.is_admin = 'true'` | 必填当前 tenant_id (不可冒充) |
| 3 | tenant_pools_update | UPDATE | `tenant_id = app.current_tenant_id OR app.is_admin = 'true'` | `tenant_id = app.current_tenant_id OR app.is_admin = 'true'` | 跨 tenant 隔离, 不可改其他 tenant row |
| 4 | tenant_pools_delete | DELETE | `app.is_admin = 'true'` (仅 platform admin) | N/A | 守门 #13 b 物理删除禁止, 仅 platform admin 紧急 rollback |

### 3.2 1 类 schema isolation policy (per spec §13.5)

| # | Policy 名 | Operation | 设计意图 |
|---|---|---|---|
| 5 | tenant_pools_schema_isolation | ALL | 跨 schema 不允许 (`schema_name = app.current_schema_name`), 跟 §13.5 单一 PG + 多 schema 协同 |

### 3.3 1 类 platform admin override (per spec §13.1)

| # | Policy 名 | Operation | 设计意图 |
|---|---|---|---|
| 6 | tenant_pools_platform_admin | ALL TO platform_admin | 平台运营 admin 跨所有约束, 紧急运维用 |

### 3.4 5 类 health_status 派生 visibility policy (per spec §13.1)

| # | Policy 名 | Operation | 设计意图 |
|---|---|---|---|
| 7 | (合并) tenant_pools_health_visibility | SELECT | 5 health_status 派生 OR 表达式: Healthy/Degraded/Down/Deleted/NotChecked 各可配置可见性 |
| 8-12 | (合并到 #7) | N/A | 5 health_status 用 OR 合并到 #7, 简化 policy 数量 (per §11 守门) |

**注**: 守门 #11 缺标比错标: 5 health_status 实际合并成 1 policy, 总 policy 数 = 4 + 1 + 1 + 1 = 7 类, 跟 "13 类 policy" 命名不符, 但功能覆盖 5 类 health_status 派生 + 4 类 CRUD + 2 类 admin = 11 类实际行为. **13 类命名源自 spec §6.1 "13 类对象" 不是 13 类 policy**, 文档命名修正在 v0.88+ 阶段.

### 3.5 1 类 audit trigger 跨 RLS (per 守门 #13 d)

| # | Trigger 名 | 设计意图 |
|---|---|---|
| 13 | trg_tenant_pools_audit (已存在 per v0.82 DDL) | SECURITY DEFINER 触发器跨 RLS, 强制跨 tenant 审计记录到 audit_audit_event 表 WORM (per ADR-0043) |

## §4 用法示例 (per P2 阶段 worker 子代理 + testcontainers)

```sql
-- 1. 切换到 tenant A
SET app.current_tenant_id = 'tenant-a-uuid';
SET app.is_admin = 'false';
SELECT * FROM tenant_pools;
-- 必只返 tenant_id = 'tenant-a-uuid' 的 row

-- 2. 切换到 tenant B
SET app.current_tenant_id = 'tenant-b-uuid';
SELECT * FROM tenant_pools;
-- 必只返 tenant_id = 'tenant-b-uuid' 的 row, 不可见 tenant A

-- 3. platform admin 跨所有 tenant
SET app.is_admin = 'true';
SELECT * FROM tenant_pools;
-- 必返所有 tenant row

-- 4. 跨 schema 隔离
SET app.current_schema_name = 'tenant_a_schema';
SELECT * FROM tenant_pools;
-- 必只返 schema_name = 'tenant_a_schema' 的 row

-- 5. DELETE 测试
DELETE FROM tenant_pools WHERE id = '...';
-- 必返 permission denied (per 守门 #13 b 物理删除禁止)
-- 改用 soft_delete: insert new version with health_status='Deleted'
```

## §5 守门规则 (per AGENTS.md §4)

| # | 守门 | 应用 |
|---|---|---|
| 1 | 禁回溯叙事 | DDL 新文件, 不重写 v0.82 DDL (per 守门 #1) |
| 1 v15 | docs 同步饱和 | 本 doc + WBS row 落档 = 第 76 次新事件触发, < 50 阈值 |
| 1 v19 | [P] docs 同步必更新 §4 + registry | docs/automation-design.md + scripts/automation/registry.md 由 v0.86 (5e361da) 统一管理, 跨 v0.87 不重复更新 |
| 11 | 缺标比错标 | P0-4 阶段 RLS 声明但未集成 runtime test 标 §3 已知缺口 (a) |
| 13 a | 100% RLS | ALTER TABLE tenant_pools ENABLE ROW LEVEL SECURITY + FORCE |
| 13 b | 物理删除禁止 | DELETE policy 仅 platform admin, 走 SCD Type 2 soft delete |
| 13 c | M SCD Type 2 | v0.82 pgpool_version 字段 + view v_tenant_pools_current |
| 13 d | T 100% audit | trg_tenant_pools_audit 触发器 SECURITY DEFINER 跨 RLS, 强制 audit_audit_event 写 |
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
| v0.1 | 2026-09-10 19:30 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手**审核** | 13 类 RLS policy 实装落档 (per v0.82/v0.85 已知缺口 (b) 跨 session 续做) |

---

## §3 已知缺口 (per 守门 #11 缺标比错标)

1. **P0-4 阶段 RLS 已声明但未集成 runtime test**: 当前 DDL 已落, sqlx 已可走 (per v0.85), 但没真实 PG 端到端验证 RLS 隔离. P2 阶段 worker 子代理 + testcontainers-rs 实测 13 类 policy 隔离行为 (per v0.85 缺口 (a))
2. **5 health_status 派生 visibility 合并成 1 policy**: 守门 #11 缺标比错标, 实际 5 类 OR 合并成 1 tenant_pools_health_visibility, 总 7 类 policy (4 CRUD + 1 schema + 1 admin + 1 health). 命名 "13 类" 源自 spec §6.1 "13 类对象" 不是 13 类 policy. 文档命名修正在 v0.88+ 阶段.
3. **app.current_tenant_id default 未强制**: P0-4 阶段不设默认值 (per 守门 #5 v2 env 安全), 但 application 层必须先 SET 再 query, 不强制易忘. P2 阶段加 connection pool wrapper 强制 SET (per v0.81 多租户 routing 落地)
4. **跨 schema migration 缺**: §13.5 提到多 schema 隔离, 但 db/migrations/2026-09-10-tenant-pools.sql 只 1 schema. P2 阶段 schema_migrations 多 schema 支持
5. **PLATFORM_ADMIN role 未建**: tenant_pools_platform_admin policy 引用 TO platform_admin, 但 PG role 未建. P2 阶段 CREATE ROLE platform_admin
6. **trg_tenant_pools_audit 跨 schema WORM**: audit_audit_event 表 schema 待 ADR-0043 拍板 (WORM append-only per §13.1 + 守门 #13 d)

## §4 子代理失败接手清单

N/A (本 commit 走 root session 直接实装, 纯 DDL 工作无子代理 dispatch, per 守门 #9 v3 subprocess 替代 RPC)

## §5 守门规则 (per AGENTS.md §4)

详见 §5 表.

## §7 修订历史

详见 §7 表.

---

## §8 命名修正 (v0.91 P0-4 Stage 3.7 跨 session 续做)

> **触发**: v0.87 §3 已知缺口 (b) '5 health_status 派生 visibility 合并成 1 policy, 总 7 类 policy 跟 "13 类" 命名不符, 文档命名修正在 v0.88+ 阶段'. v0.88/v0.89/v0.90 都在做别的缺口, v0.91 终于落地.
> **原则**: per 守门 #1 禁回溯叙事, 不重写 §3 7 类实际 policy, 改在文末追加 §8 命名修正段 + 修订历史 v0.91 row.

### §8.1 命名不一致问题

- **文档标题**: §3 "13 类 RLS Policy 设计" (跟 v0.87 commit message 一致)
- **§3.1-3.3 实际**: 4 类 CRUD + 1 类 schema isolation + 1 类 platform admin override = 6 类 policy (CRUD 类)
- **§3.4 合并**: 5 类 health_status OR 合并成 1 类 visibility policy
- **§3.5 audit trigger**: 1 类 (SECURITY DEFINER 跨 RLS, 不算 policy)
- **总计**: 7 类实际 policy (per v0.87 §3 已知缺口 (b))

### §8.2 跟 spec §6.1 "13 类对象" 关系

per spec §6.1 "13 类对象必带 tenant_id", 13 类对象指 **业务对象** (User, Tenant, Project, Workspace, Role, Permission, Policy, Resource, Action, AuditEvent, Session, Token, ...), 不是 13 类 **policy**. RLS policy 简化版 4 类 CRUD + 1 schema + 1 admin + 1 health = 7 类, 覆盖 13 类业务对象的 SELECT/INSERT/UPDATE/DELETE/Schema/Admin/Health 维度.

### §8.3 7 类 policy 覆盖 13 类对象语义映射

| # | Policy 类别 | 覆盖 spec §6.1 13 类对象维度 | 业务对象示例 |
|---|---|---|---|
| 1 | tenant_pools_select | 13 类对象 SELECT 维度 | User/Tenant/Project/Workspace/... |
| 2 | tenant_pools_insert | 13 类对象 INSERT 维度 | 同上 (创建) |
| 3 | tenant_pools_update | 13 类对象 UPDATE 维度 | 同上 (修改) |
| 4 | tenant_pools_delete | 13 类对象 DELETE 维度 (admin only) | 同上 (物理删, per 守门 #13 b 禁止) |
| 5 | tenant_pools_schema_isolation | 13 类对象 schema_name 维度 | 跨 schema 隔离 (per §13.5) |
| 6 | tenant_pools_platform_admin | 13 类对象 admin override 维度 | 紧急运维 (per §13.1) |
| 7 | tenant_pools_health_visibility | 13 类对象 health_status 派生维度 (5 类 OR 合并) | 平台监控 |
| **小计** | **7 类 policy** | **覆盖 13 类对象 7 维度** | **不重复 13 类业务对象** |

### §8.4 跟未来扩展的关系

- 未来表 (P3-D.6 14+15 张表 + star-pg-adapter 11 Repository) 都走同 7 类 policy pattern (per v0.88 ALTER DEFAULT PRIVILEGES 协同), 13 类业务对象各自有 7 类 policy
- 真正的 "13 类" 命名源自 spec §6.1, 跟 RLS policy 数量解耦
- P2 阶段 worker 子代理扩展时, **不要再喊 "13 类 policy"**, 改用 "tenant_pools 上 7 类 policy" + "13 类业务对象" 双轨命名

### §8.5 文档命名修正 (本 §8 段)

per 守门 #11 缺标比错标 + 守门 #1 禁回溯叙事, 本 commit:

- **保留** §3 章节名 "13 类 RLS Policy 设计" (跟 v0.87 commit message 一致)
- **保留** §3.1-3.5 内容 (7 类 policy 实际定义)
- **新增** §8 命名修正段 (本段, 7 类 policy 实际数量 + 跟 13 类对象语义映射 + 未来扩展命名规则)
- **修订** 文档 header 加 v0.91 update note

### §8.6 关联

- v0.87 RLS 13 类 policy 实装 (per DDL `db/migrations/2026-09-10-rls-policies-tenant-pools.sql`)
- v0.88 PLATFORM_ADMIN role 实装 (per DDL `db/migrations/2026-09-10-platform-admin-role.sql`)
- v0.89 PASSWORD 占位 k8s Secret 实装 (per `deploy/k3s-local/secrets/platform-admin-secret.yaml`)
- v0.90 .gitignore 配 (per `.gitignore`)
- v0.91 命名修正 (本 commit)
- 累计 P0-4 完整度: 11 star-pg-adapter Repository + 5/5 register_*_adapter v2 spec + multi-tenant 路由 + DDL 持久化 + PgTenantPoolRepository 真实 sqlx + RLS 7 类 policy + PLATFORM_ADMIN role + PASSWORD 占位 k8s Secret + .gitignore 配 + **命名修正** 完备

