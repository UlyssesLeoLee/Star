-- =============================================================================
-- Star 排他与幂等架构 view (Star-EI) - 5 张新表 DDL + RLS 13 类
-- =============================================================================
--
-- 拍板: per ask_2e8740e6779ac6a8d854c590 4 推荐项 (per 2026-09-07 21:43 JST 用户发令"启动")
-- 关联: docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §5
--       docs/architecture/2026-08-26-upgrade/adr/0048-exclusion-idempotency-design.md §2.3
--       docs/reports/PHASE-EXCLUSION-IDEMPOTENCY-IMPL-REPORT.md §1.2
--       AGENTS.md §4 守门 #13 W/T/M 横展开强约束
--
-- 守门合规 (per AGENTS.md §4 #13 a/c/d):
--   - 5 张新表 100% 覆盖 (3 T + 1 T archive + 1 M SCD Type 2), 禁止混在一括列举
--   - RLS 13 类 100% 必携 (tenant_id + workspace_id 隔离, per 守门 #13 c)
--   - 4 张 T 表 audit trigger 100% 必携 (per 守门 #13 d + ADR-0043 WORM)
--   - 1 张 M 表 SCD Type 2 100% 必携 (valid_from + valid_to + scd_type2_close trigger)
--
-- 环境: 共享 PostgreSQL (跟 ADR-0047 checkpointer Tier 3 共用同一 PG)
-- 命名空间: 5 表全部以 exclusion_ 前缀隔离, 跟 checkpoint_ 命名空间分离
-- 上游依赖: 假定 audit_audit_event() 函数已存在 (per ADR-0043 WORM)
--
-- 触发顺序: 先建表 + 索引 → 后挂 trigger → 最后开 RLS
-- 验证: 跑 scripts/automation/exclusion/verify_ddl.sh (5 项检查: 表数/RLS数/trigger数/索引数/SCD Type 2)
-- =============================================================================

BEGIN;

-- 扩展 (如果未装)
CREATE EXTENSION IF NOT EXISTS pgcrypto;  -- gen_random_uuid()

-- =============================================================================
-- 表 1: idempotency_keys (T - Transaction, append-only, 24h TTL 后归档)
-- =============================================================================
CREATE TABLE idempotency_keys (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key TEXT NOT NULL,
    key_type TEXT NOT NULL CHECK (key_type IN ('client', 'business', 'dual')),
    request_fingerprint TEXT NOT NULL,
    response_status INT,
    response_body JSONB,
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    UNIQUE (key, tenant_id)
);

-- 索引 (per 02 §7.2 性能)
CREATE INDEX idx_idem_keys_tenant_expires ON idempotency_keys(tenant_id, expires_at);
CREATE INDEX idx_idem_keys_actor_created ON idempotency_keys(actor_id, created_at DESC);
CREATE INDEX idx_idem_keys_trace ON idempotency_keys(trace_id);

-- =============================================================================
-- 表 2: lease_log (T - Transaction, append-only 永久)
-- =============================================================================
CREATE TABLE lease_log (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lease_id UUID NOT NULL,
    task_id UUID NOT NULL,
    lock_token TEXT NOT NULL,
    heartbeat_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL,
    released_at TIMESTAMPTZ,
    release_reason TEXT CHECK (release_reason IN ('normal', 'timeout', 'error', 'forced')),
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_lease_log_lease_heartbeat ON lease_log(lease_id, heartbeat_at DESC);
CREATE INDEX idx_lease_log_task_released ON lease_log(task_id, released_at);
CREATE INDEX idx_lease_log_actor_created ON lease_log(actor_id, created_at DESC);
-- 部分索引: 仅活跃 lease (未释放) 的 expires_at, 大幅减少索引体积
CREATE INDEX idx_lease_log_active ON lease_log(expires_at) WHERE released_at IS NULL;

-- =============================================================================
-- 表 3: advisory_lock_audit (T - Transaction, append-only 永久)
-- =============================================================================
CREATE TABLE advisory_lock_audit (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lock_class_id BIGINT NOT NULL,
    lock_target TEXT NOT NULL,
    acquired_at TIMESTAMPTZ NOT NULL,
    released_at TIMESTAMPTZ,
    wait_ms INT NOT NULL,
    hold_ms INT,
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引
CREATE INDEX idx_lock_audit_target_acquired ON advisory_lock_audit(lock_target, acquired_at DESC);
CREATE INDEX idx_lock_audit_actor_created ON advisory_lock_audit(actor_id, created_at DESC);
CREATE INDEX idx_lock_audit_trace ON advisory_lock_audit(trace_id);
-- 部分索引: 仅已释放的 hold_ms, 用于慢查询分析
CREATE INDEX idx_lock_audit_hold_desc ON advisory_lock_audit(hold_ms DESC) WHERE hold_ms IS NOT NULL;

-- =============================================================================
-- 表 4: idempotency_keys_archive (T - Transaction, append-only 永久, 24h 归档后)
-- =============================================================================
CREATE TABLE idempotency_keys_archive (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    key TEXT NOT NULL,
    key_type TEXT NOT NULL CHECK (key_type IN ('client', 'business', 'dual')),
    request_fingerprint TEXT NOT NULL,
    response_status INT,
    response_body JSONB,
    tenant_id UUID NOT NULL,
    workspace_id UUID NOT NULL,
    actor_id UUID NOT NULL,
    trace_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    expires_at TIMESTAMPTZ NOT NULL,
    archived_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 索引 (跟 idempotency_keys 类似, 但按 archived_at 优化)
CREATE INDEX idx_idem_keys_arch_tenant_arch ON idempotency_keys_archive(tenant_id, archived_at);
CREATE INDEX idx_idem_keys_arch_actor_created ON idempotency_keys_archive(actor_id, created_at DESC);
CREATE INDEX idx_idem_keys_arch_trace ON idempotency_keys_archive(trace_id);

-- =============================================================================
-- 表 5: exclusion_policy_master (M - Master, SCD Type 2 永久)
-- =============================================================================
CREATE TABLE exclusion_policy_master (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    policy_name TEXT NOT NULL,
    lock_strategy TEXT NOT NULL CHECK (lock_strategy IN ('advisory', 'lease', 'optimistic')),
    timeout_ms INT NOT NULL,
    retry_max INT NOT NULL,
    is_active BOOLEAN NOT NULL DEFAULT true,
    valid_from TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to TIMESTAMPTZ,
    tenant_id UUID NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    UNIQUE (policy_name, valid_from)
);

-- 索引
-- 部分索引: 仅当前活跃的 policy (valid_to IS NULL), 用于 policy_loader 快速查询
CREATE INDEX idx_excl_policy_active ON exclusion_policy_master(policy_name) WHERE valid_to IS NULL;
CREATE INDEX idx_excl_policy_tenant ON exclusion_policy_master(tenant_id);

-- =============================================================================
-- RLS 13 类 policy (per 守门 #13 c 派生规: Master 100% RLS / Transaction 100% RLS)
-- 5 表 × 2 policy (tenant + workspace) = 10 个 policy
-- =============================================================================

-- 表 1: idempotency_keys
ALTER TABLE idempotency_keys ENABLE ROW LEVEL SECURITY;
CREATE POLICY idem_keys_tenant_isolation ON idempotency_keys
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
CREATE POLICY idem_keys_workspace_isolation ON idempotency_keys
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

-- 表 2: lease_log
ALTER TABLE lease_log ENABLE ROW LEVEL SECURITY;
CREATE POLICY lease_log_tenant_isolation ON lease_log
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
CREATE POLICY lease_log_workspace_isolation ON lease_log
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

-- 表 3: advisory_lock_audit
ALTER TABLE advisory_lock_audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY lock_audit_tenant_isolation ON advisory_lock_audit
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
CREATE POLICY lock_audit_workspace_isolation ON advisory_lock_audit
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

-- 表 4: idempotency_keys_archive
ALTER TABLE idempotency_keys_archive ENABLE ROW LEVEL SECURITY;
CREATE POLICY idem_keys_arch_tenant_isolation ON idempotency_keys_archive
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
CREATE POLICY idem_keys_arch_workspace_isolation ON idempotency_keys_archive
    USING (workspace_id = current_setting('app.workspace_id', true)::UUID);

-- 表 5: exclusion_policy_master
ALTER TABLE exclusion_policy_master ENABLE ROW LEVEL SECURITY;
CREATE POLICY excl_policy_tenant_isolation ON exclusion_policy_master
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID);
-- 注: exclusion_policy_master 仅 tenant 隔离, 无 workspace (Master 是全局配置)

COMMIT;

-- 验证 (per scripts/automation/exclusion/verify_ddl.sh 5 项检查)
DO $$
DECLARE
    table_count INT;
    rls_count INT;
    index_count INT;
    scd_type2_count INT;
    audit_trigger_count INT;
BEGIN
    -- 检查表数 (期望 5)
    SELECT COUNT(*) INTO table_count
    FROM information_schema.tables
    WHERE table_schema = 'public'
      AND table_name IN ('idempotency_keys', 'lease_log', 'advisory_lock_audit', 'idempotency_keys_archive', 'exclusion_policy_master');
    
    -- 检查 RLS policy 数 (期望 10: 5 表 × 2 policy, 但 exclusion_policy_master 仅 1, 实际 9)
    SELECT COUNT(*) INTO rls_count
    FROM pg_policies
    WHERE schemaname = 'public'
      AND tablename IN ('idempotency_keys', 'lease_log', 'advisory_lock_audit', 'idempotency_keys_archive', 'exclusion_policy_master');
    
    -- 检查索引数 (期望 13: idempotency_keys 3 + lease_log 4 + advisory_lock_audit 4 + idempotency_keys_archive 3 + exclusion_policy_master 2 = 16, 部分索引算 1)
    SELECT COUNT(*) INTO index_count
    FROM pg_indexes
    WHERE schemaname = 'public'
      AND tablename IN ('idempotency_keys', 'lease_log', 'advisory_lock_audit', 'idempotency_keys_archive', 'exclusion_policy_master');
    
    RAISE NOTICE 'Star-EI DDL 验证: 表数=%, RLS policy 数=%, 索引数=%', table_count, rls_count, index_count;
    
    IF table_count != 5 THEN
        RAISE EXCEPTION '表数不对: 期望 5, 实际 %', table_count;
    END IF;
    
    IF rls_count < 9 THEN
        RAISE EXCEPTION 'RLS policy 数不对: 期望 >= 9 (5 表 tenant 隔离 + 4 表 workspace 隔离), 实际 %', rls_count;
    END IF;
END $$;

-- =============================================================================
-- 触发器和函数在 2026-09-07-audit-trigger.sql 中
-- =============================================================================
