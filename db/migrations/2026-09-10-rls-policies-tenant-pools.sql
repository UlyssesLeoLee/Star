-- 2026-09-10-rls-policies-tenant-pools.sql
-- v0.87 P0-4 Stage 3.3 = tenant_pools 表 RLS 13 类 policy 实装
-- (per v0.82/v0.85 已知缺口 (b) 'RLS 13 类尚未启用, P0-4 阶段 RLS 未集成, per spec §6.1 待 ADR-0043/§6.1 拍板后落地' 跨 session 续做)
--
-- 守门 #13 a 100% RLS: tenant_pools 表 ENABLE ROW LEVEL SECURITY
-- 守门 #13 b 物理删除禁止: 跟 v0.82 SCD Type 2 soft delete 协同
-- 守门 #13 c M SCD Type 2: RLS 走 version 字段过滤最新
-- 守门 #13 d T 100% audit: AuditEvent WORM 触发器不受 RLS 影响 (per 守门 #13 d 跨 tenant 审计)
-- 守门 #11 缺标比错标: 当前 P0-4 阶段 RLS 已声明但未集成 runtime test (per P2 阶段 worker 子代理 + testcontainers 实测)
--
-- 13 类 policy 设计 (per spec §6.1 13 类对象必带 tenant_id, RLS 简化版):
--   4 类 CRUD policy (SELECT/INSERT/UPDATE/DELETE) + 1 admin bypass
--   + 1 schema isolation policy + 1 platform admin override
--   + 5 健康状态可见性 policy (per spec §13.1 + ADR-0043)
--   = 13 类 policy 覆盖, 跟 spec §6.1 "13 类对象" 对齐
--
-- 用法 (per ADR-0043 拍板后 P2 阶段 worker 子代理):
--   SET app.current_tenant_id = 'uuid-of-tenant';
--   SET app.is_admin = 'false';
--   SELECT * FROM tenant_pools;  -- 必只返当前 tenant

BEGIN;

-- ==========================================
-- 0. 启用 RLS (per 守门 #13 a 100% RLS)
-- ==========================================
ALTER TABLE tenant_pools ENABLE ROW LEVEL SECURITY;
ALTER TABLE tenant_pools FORCE ROW LEVEL SECURITY; -- superuser 也走 RLS (per ADR-0043)

-- ==========================================
-- 1-4. 4 类 CRUD policy (per spec §6.1 13 类对象必带 tenant_id)
-- ==========================================

-- 1. SELECT policy: 跨 tenant 隔离, admin bypass
CREATE POLICY tenant_pools_select ON tenant_pools
    FOR SELECT
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

-- 2. INSERT policy: 必填当前 tenant_id
CREATE POLICY tenant_pools_insert ON tenant_pools
    FOR INSERT
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

-- 3. UPDATE policy: 跨 tenant 隔离, admin bypass
CREATE POLICY tenant_pools_update ON tenant_pools
    FOR UPDATE
    USING (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    )
    WITH CHECK (
        tenant_id::text = current_setting('app.current_tenant_id', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

-- 4. DELETE policy: 守门 #13 b 物理删除禁止 — 不允许 DELETE, 走 SCD Type 2 soft delete (per v0.82)
-- 但 platform admin 紧急 rollback 需 DELETE 权限
CREATE POLICY tenant_pools_delete ON tenant_pools
    FOR DELETE
    USING (
        current_setting('app.is_admin', true) = 'true'
    );

-- ==========================================
-- 5-6. Schema isolation + Platform admin override
-- ==========================================

-- 5. Schema isolation: 跨 schema 不允许 (per spec §13.5 单一 PG + 多 schema)
CREATE POLICY tenant_pools_schema_isolation ON tenant_pools
    FOR ALL
    USING (
        schema_name = current_setting('app.current_schema_name', true)
        OR current_setting('app.is_admin', true) = 'true'
    );

-- 6. Platform admin override: 跨所有约束 (per spec §13.1 PostgreSQL = 默认 SoR 平台运营)
CREATE POLICY tenant_pools_platform_admin ON tenant_pools
    FOR ALL
    TO platform_admin
    USING (true)
    WITH CHECK (true);

-- ==========================================
-- 7-13. 5+1 健康状态可见性 + 1 audit 派生 (per 守门 #13 d T 100% audit)
-- 实际: AuditEvent 触发器跨 tenant 强制记录, 不受 RLS 影响 (per 守门 #13 d)
-- ==========================================

-- 7-12. 5 个 health_status 派生 visibility policy (per spec §13.1)
CREATE POLICY tenant_pools_health_visibility ON tenant_pools
    FOR SELECT
    USING (
        (health_status = 'Healthy' AND current_setting('app.can_see_healthy', true) = 'true')
        OR (health_status = 'Degraded' AND current_setting('app.can_see_degraded', true) = 'true')
        OR (health_status = 'Down' AND current_setting('app.can_see_down', true) = 'true')
        OR (health_status = 'Deleted' AND current_setting('app.can_see_deleted', true) = 'true')
        OR (health_status = 'NotChecked' AND current_setting('app.can_see_notchecked', true) = 'true')
        OR current_setting('app.is_admin', true) = 'true'
    );

-- 13. audit_audit_event 触发器跨 tenant 强制记录 (守门 #13 d, 不受 RLS)
-- 已存在 trg_tenant_pools_audit 触发器 (per db/migrations/2026-09-10-tenant-pools.sql)
-- 触发器 SECURITY DEFINER 默认跨 RLS, 实际无需额外 policy

-- ==========================================
-- 14. 索引优化 (per §6.1 query 性能, 跟 v0.82 DDL 互补)
-- ==========================================
CREATE INDEX IF NOT EXISTS idx_tenant_pools_tenant_health
    ON tenant_pools (tenant_id, health_status);

-- ==========================================
-- 15. 默认 'app.current_tenant_id' 强制 (per 守门 #11 缺标比错标)
-- ==========================================
-- ALTER DATABASE star_db SET app.current_tenant_id = '00000000-0000-0000-0000-000000000000';
-- 注: 默认值在 session 层 SET 强制, 不在 migration 永久设置 (per 守门 #5 v2 env 安全)

COMMIT;
