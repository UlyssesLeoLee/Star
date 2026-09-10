-- 2026-09-10-tenant-pools.sql
-- v0.82 P0-4 Stage 3.1 = RealPostgresAdapterRegistry 多租户 routing DDL 持久化
-- (per v0.81 已知缺口 (a) 'multi-tenant 路由只支持内存 HashMap, 跨 session persist 需 DDL' 跨 session 续做)
--
-- 1 表 DDL, 累计 30+ 张表 (per WBS W/T/M 100% 覆盖)
-- Migration: 跟 F-01/F-02/F-03/OAuth2 server 6/6 ops 4/4 oauth 同 pattern
-- Per 守门 #13 a/d Transaction 100% audit + 守门 #13 c M SCD Type 2 + 守门 #13 b 物理删除禁止
-- Per spec §6.1 13 类 RLS (tenant_id-based 隔离, P0-4 集成 RLS 留缺口 d)

BEGIN;

-- ==========================================
-- 1. tenant_pools (M 类, SCD Type 2 配置参考数据 + AuditEventTrail)
-- 守门 #13 c: 物理删除禁止 + SCD Type 2 (per pgpool_version 递增) + 100% RLS 必携
-- 守门 #13 d: 100% audit (created_at + updated_at + AuditEvent WORM 触发器)
-- ==========================================
CREATE TABLE IF NOT EXISTS tenant_pools (
    -- 业务字段
    id              UUID NOT NULL,
    tenant_id       UUID NOT NULL,                    -- 13 类对象必带 (§6.1)
    pg_url          TEXT NOT NULL,                    -- PG 连接 URL (masked secret)
    pool_size       INTEGER NOT NULL DEFAULT 10,      -- sqlx 默认 10
    ssl_mode        TEXT NOT NULL DEFAULT 'prefer',   -- disable/allow/prefer/require/verify-ca/verify-full
    schema_name     TEXT NOT NULL DEFAULT 'public',   -- per §13.5 多 schema 隔离
    health_status   TEXT NOT NULL DEFAULT 'NotChecked', -- Healthy/Degraded/Down/NotChecked
    pgpool_version  BIGINT NOT NULL DEFAULT 1,        -- SCD Type 2 version 字段
    -- 审计字段 (守门 #13 d 100% audit)
    created_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    created_by      UUID NOT NULL,                    -- actor.user_id
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_by      UUID NOT NULL,                    -- actor.user_id
    -- 显式主键
    PRIMARY KEY (id, pgpool_version),
    -- 同一 tenant 同一 version 必唯一
    UNIQUE (tenant_id, pgpool_version)
);

-- 索引 (per §6.1 query 优化)
CREATE INDEX IF NOT EXISTS idx_tenant_pools_tenant_id
    ON tenant_pools (tenant_id);
CREATE INDEX IF NOT EXISTS idx_tenant_pools_health_status
    ON tenant_pools (health_status)
    WHERE health_status != 'Healthy';
CREATE INDEX IF NOT EXISTS idx_tenant_pools_updated_at
    ON tenant_pools (updated_at DESC);

-- 当前 SCD Type 2 "有效" version 视图 (查询时 WHERE pgpool_version = MAX(pgpool_version) GROUP BY tenant_id)
CREATE OR REPLACE VIEW v_tenant_pools_current AS
SELECT DISTINCT ON (tenant_id) *
FROM tenant_pools
ORDER BY tenant_id, pgpool_version DESC;

-- AuditEvent WORM 触发器 (守门 #13 d 100% audit + ADR-0043 WORM)
-- 任何 INSERT/UPDATE/DELETE 必写 audit_audit_event 表 (append-only)
CREATE OR REPLACE FUNCTION tenant_pools_audit_trigger() RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_audit_event (
        id, tenant_id, source_kind, source_id, action, row_data, actor_id, created_at
    ) VALUES (
        gen_random_uuid(),
        COALESCE(NEW.tenant_id, OLD.tenant_id),
        'TenantPool',
        COALESCE(NEW.id, OLD.id),
        TG_OP,
        to_jsonb(COALESCE(NEW, OLD)),
        COALESCE(NEW.updated_by, OLD.updated_by, NEW.created_by, OLD.created_by),
        NOW()
    );
    RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trg_tenant_pools_audit
    AFTER INSERT OR UPDATE OR DELETE ON tenant_pools
    FOR EACH ROW
    EXECUTE FUNCTION tenant_pools_audit_trigger();

-- ==========================================
-- 2. RLS 13 类必携 (per §6.1 跨 tenant 隔离)
-- P0-4 当前还没集成 RLS, 此处预声明 policy 占位 (P2 阶段 worker 子代理实装)
-- 守门 #11 缺标比错标: 当前 P0-4 阶段 RLS 未启用, 待 ADR-0043/§6.1 拍板后落地
-- ==========================================
-- ALTER TABLE tenant_pools ENABLE ROW LEVEL SECURITY;
-- CREATE POLICY tenant_pools_isolation ON tenant_pools
--     USING (tenant_id = current_setting('app.current_tenant_id')::UUID);

-- ==========================================
-- 3. schema_migrations 记录 (per star-pg-adapter::apply_migrations 自动)
-- ==========================================
-- 此处不需要手动 INSERT, apply_migrations 会自动检测本文件 + INSERT schema_migrations row

COMMIT;
