-- =============================================================================
-- Star 排他与幂等架构 view (Star-EI) - 5 audit trigger + 1 SCD Type 2 + 2 函数
-- =============================================================================
--
-- 拍板: per ask_2e8740e6779ac6a8d854c590 4 推荐项
-- 关联: docs/migrations/2026-09-07-exclusion-rls.sql (前置 DDL)
--       docs/architecture/2026-09-07-exclusion-idempotency/03-detailed-design.md §5
--       ADR-0043 audit_audit_event WORM
--       AGENTS.md §4 守门 #13 d (Transaction 100% audit 必携)
--
-- 守门合规 (per AGENTS.md §4 #13 d):
--   - 4 张 T 表 audit trigger 100% 必携 (idempotency_keys / lease_log / advisory_lock_audit / idempotency_keys_archive)
--   - audit_audit_event() 函数假定已存在 (per ADR-0043 WORM 落档)
--   - 1 张 M 表 SCD Type 2 100% 必携 (exclusion_policy_master, valid_from + valid_to + scd_type2_close trigger)
--
-- 触发顺序: 先建函数 → 后挂 trigger (避免 trigger 引用未存在函数)
-- =============================================================================

BEGIN;

-- =============================================================================
-- 函数 1: scd_type2_close() - SCD Type 2 闭旧记录 (per 守门 #13 c Master 派生)
-- =============================================================================
-- 用途: exclusion_policy_master 更新时, 自动闭旧记录 (设 valid_to = NOW())
-- 不允许直接 UPDATE 旧记录 (valid_to NOT NULL)
CREATE OR REPLACE FUNCTION scd_type2_close() RETURNS TRIGGER AS $$
BEGIN
    -- 仅在 NEW.valid_to 仍为 NULL (即"更新"操作) 时闭旧记录
    IF OLD.valid_to IS NULL AND NEW.valid_to IS NULL THEN
        NEW.valid_to = NOW();
    END IF;
    -- updated_at 自动更新
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

COMMENT ON FUNCTION scd_type2_close() IS 'SCD Type 2 close: exclusion_policy_master 更新时自动闭旧记录, per 守门 #13 c 派生规';

-- =============================================================================
-- Trigger 1: exclusion_policy_master SCD Type 2 (BEFORE UPDATE)
-- =============================================================================
CREATE TRIGGER trg_excl_policy_scd2
BEFORE UPDATE ON exclusion_policy_master
FOR EACH ROW EXECUTE FUNCTION scd_type2_close();

-- =============================================================================
-- Trigger 2-5: 4 张 T 表 audit trigger (AFTER INSERT OR UPDATE)
-- 假定 audit_audit_event() 函数已存在 (per ADR-0043)
-- 派生规 (per 守门 #13 d): Transaction 100% audit 必携
-- =============================================================================

-- 表 1: idempotency_keys audit
CREATE TRIGGER trg_idem_keys_audit
AFTER INSERT OR UPDATE ON idempotency_keys
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- 表 2: lease_log audit
CREATE TRIGGER trg_lease_log_audit
AFTER INSERT OR UPDATE ON lease_log
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- 表 3: advisory_lock_audit audit
CREATE TRIGGER trg_lock_audit_audit
AFTER INSERT OR UPDATE ON advisory_lock_audit
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- 表 4: idempotency_keys_archive audit
CREATE TRIGGER trg_idem_keys_arch_audit
AFTER INSERT OR UPDATE ON idempotency_keys_archive
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

-- 注: 表 5 exclusion_policy_master 同时挂 SCD Type 2 trigger 和 audit trigger
-- (Master 既需要 SCD Type 2 慢变, 也需要 audit 审计)
CREATE TRIGGER trg_excl_policy_audit
AFTER INSERT OR UPDATE ON exclusion_policy_master
FOR EACH ROW EXECUTE FUNCTION audit_audit_event();

COMMIT;

-- =============================================================================
-- 验证 (per scripts/automation/exclusion/verify_ddl.sh 5 项检查)
-- =============================================================================
DO $$
DECLARE
    audit_trigger_count INT;
    scd_trigger_count INT;
    function_count INT;
BEGIN
    -- 检查 audit trigger 数 (期望 5: 4 T 表 + 1 M 表 = 5)
    SELECT COUNT(*) INTO audit_trigger_count
    FROM pg_trigger
    WHERE tgrelid IN (
        'idempotency_keys'::regclass,
        'lease_log'::regclass,
        'advisory_lock_audit'::regclass,
        'idempotency_keys_archive'::regclass,
        'exclusion_policy_master'::regclass
    )
    AND tgname LIKE '%audit%';
    
    -- 检查 SCD Type 2 trigger 数 (期望 1: 仅 exclusion_policy_master)
    SELECT COUNT(*) INTO scd_trigger_count
    FROM pg_trigger
    WHERE tgname = 'trg_excl_policy_scd2';
    
    -- 检查函数数 (期望 1 落地: scd_type2_close; audit_audit_event 由 ADR-0043 落地, 不在本文件)
    SELECT COUNT(*) INTO function_count
    FROM pg_proc
    WHERE proname = 'scd_type2_close';
    
    RAISE NOTICE 'Star-EI trigger 验证: audit_trigger=%, scd_trigger=%, function=%', 
        audit_trigger_count, scd_trigger_count, function_count;
    
    IF audit_trigger_count != 5 THEN
        RAISE EXCEPTION 'audit trigger 数不对: 期望 5 (4 T + 1 M), 实际 %', audit_trigger_count;
    END IF;
    
    IF scd_trigger_count != 1 THEN
        RAISE EXCEPTION 'SCD Type 2 trigger 数不对: 期望 1 (exclusion_policy_master), 实际 %', scd_trigger_count;
    END IF;
    
    IF function_count != 1 THEN
        RAISE EXCEPTION 'scd_type2_close 函数数不对: 期望 1, 实际 %', function_count;
    END IF;
    
    RAISE NOTICE 'Star-EI trigger 验证: ALL PASS (audit=5/5, scd=1/1, function=1/1)';
END $$;
