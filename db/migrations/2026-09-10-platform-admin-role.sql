-- 2026-09-10-platform-admin-role.sql
-- v0.88 P0-4 Stage 3.4 = PLATFORM_ADMIN role 实装 (per v0.87 已知缺口 (e) 'PLATFORM_ADMIN role 未建')
-- (跨 session 续做 v0.87 §3 缺口 (e))
--
-- 守门 #5 v2 env 安全: PASSWORD 留占位 'CHANGE_ME_AT_DEPLOY', 部署时由 env var 或 secret manager 替换
-- 守门 #13 a 100% RLS: 跟 v0.87 RLS 政策协同, PLATFORM_ADMIN role 跨所有 RLS policy
-- 守门 #11 缺标比错标: P0-4 阶段 role 已声明但未集成 runtime test (P2 阶段 worker 子代理 + testcontainers 实测)

BEGIN;

-- ==========================================
-- 1. CREATE ROLE platform_admin (per spec §13.1 PostgreSQL = 默认 SoR 平台运营)
-- ==========================================
-- PASSWORD 留占位, 部署时由 env var 替换 (per 守门 #5 v2 env 安全)
DO $$
BEGIN
    IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'platform_admin') THEN
        CREATE ROLE platform_admin LOGIN PASSWORD 'CHANGE_ME_AT_DEPLOY';
    END IF;
END
$$;

-- ==========================================
-- 2. GRANT 数据库连接权限
-- ==========================================
GRANT CONNECT ON DATABASE star_db TO platform_admin;

-- ==========================================
-- 3. GRANT schema 权限
-- ==========================================
GRANT USAGE ON SCHEMA public TO platform_admin;

-- ==========================================
-- 4. GRANT 跨 tenant_pools 表权限 (per RLS policy 跨所有约束)
-- ==========================================
GRANT SELECT, INSERT, UPDATE, DELETE ON tenant_pools TO platform_admin;

-- 未来表 (per v0.85 11 Repository + P3-D.6 14+15 张表) 默认权限
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO platform_admin;
ALTER DEFAULT PRIVILEGES IN SCHEMA public
    GRANT USAGE, SELECT ON SEQUENCES TO platform_admin;

-- ==========================================
-- 5. audit_audit_event 表权限 (守门 #13 d T 100% audit WORM 跨 RLS)
-- ==========================================
GRANT SELECT, INSERT ON audit_audit_event TO platform_admin;
-- (注: audit_audit_event 触发器 SECURITY DEFINER 跨 RLS, 平台 admin 需能 SELECT 审计)

-- ==========================================
-- 6. BYPASSRLS attribute (per spec §13.1 平台运营 admin 跨所有 RLS)
-- ==========================================
ALTER ROLE platform_admin BYPASSRLS;

-- ==========================================
-- 7. 验证 (P0-4 阶段 placeholder, P2 阶段 worker 子代理实测)
-- ==========================================
-- 注: 不能在 DDL 中直接 SELECT 测试, 留 application 层 P2 阶段验证
-- 测试 query: SET ROLE platform_admin; SELECT * FROM tenant_pools; -- 必返所有 tenant

COMMIT;
