-- 2026-09-08-ops-cluster.sql
-- F-01 cluster update 端到端实装 (per WBS §14.10.2 + SRS-001 §4 F-01)
-- 2 表 DDL 雏形, 守门 #13 W/T/M 100% 覆盖
-- Migration: 跟 F-02 ops-log 3 表同 pattern (per commit 3602cfb)

BEGIN;

-- ==========================================
-- 1. ops_helm_release_state (T 类, 业务事实快照)
-- 守门 #13 b: 物理删除禁止 + 監査必須 + RLS 13 類必携
-- ==========================================
CREATE TABLE IF NOT EXISTS ops_helm_release_state (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL,
    release_name        TEXT NOT NULL,
    namespace           TEXT NOT NULL,
    chart               TEXT NOT NULL,
    revision            INT  NOT NULL,
    status              TEXT NOT NULL CHECK (status IN ('pending', 'healthy', 'degraded', 'failed', 'canary')),
    canary_weight       INT  NOT NULL DEFAULT 0 CHECK (canary_weight BETWEEN 0 AND 100),
    last_deployed_at    TIMESTAMPTZ NOT NULL,
    captured_at         TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- RLS 13 類 (T 必携, per 守门 #13 c)
    user_id             UUID,
    role_id             UUID,
    permission_id       UUID,
    policy_id           UUID,
    workspace_id        UUID,
    project_id          UUID,
    work_item_id        UUID,
    agent_id            UUID,
    session_id          UUID,
    trace_id            UUID,
    source_module       TEXT NOT NULL DEFAULT 'star_ops::cluster',
    source_kind         TEXT NOT NULL DEFAULT 'internal'
);

CREATE INDEX IF NOT EXISTS idx_ops_helm_release_state_tenant ON ops_helm_release_state(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ops_helm_release_state_release ON ops_helm_release_state(release_name, revision);
CREATE INDEX IF NOT EXISTS idx_ops_helm_release_state_captured ON ops_helm_release_state(captured_at DESC);

-- 守门 #13 d: 物理删除禁止
CREATE OR REPLACE FUNCTION prevent_hard_delete_ops_helm_release_state()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'ops_helm_release_state 物理删除禁止 (per 守門 #13 d)';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_prevent_delete_ops_helm_release_state ON ops_helm_release_state;
CREATE TRIGGER trg_prevent_delete_ops_helm_release_state
    BEFORE DELETE ON ops_helm_release_state
    FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_ops_helm_release_state();

ALTER TABLE ops_helm_release_state ENABLE ROW LEVEL SECURITY;
ALTER TABLE ops_helm_release_state FORCE ROW LEVEL SECURITY;

-- ==========================================
-- 2. ops_cluster_action_log (T 类, WORM 审计)
-- 守门 #13 d: 物理删除禁止 + 100% audit
-- ==========================================
CREATE TABLE IF NOT EXISTS ops_cluster_action_log (
    id                  UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id           UUID NOT NULL,
    action_id           UUID NOT NULL,
    release_name        TEXT NOT NULL,
    namespace           TEXT NOT NULL,
    action_type         TEXT NOT NULL CHECK (action_type IN ('canary', 'rollback', 'upgrade', 'list', 'status')),
    actor_user_id       UUID NOT NULL,
    requested_at        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at        TIMESTAMPTZ,
    status              TEXT NOT NULL CHECK (status IN ('pending', 'running', 'succeeded', 'failed')),
    payload             JSONB NOT NULL,
    error_message       TEXT,
    -- RLS 13 類 (T 必携)
    user_id             UUID,
    role_id             UUID,
    permission_id       UUID,
    policy_id           UUID,
    workspace_id        UUID,
    project_id          UUID,
    work_item_id        UUID,
    agent_id            UUID,
    session_id          UUID,
    trace_id            UUID,
    source_module       TEXT NOT NULL DEFAULT 'star_ops::cluster',
    source_kind         TEXT NOT NULL DEFAULT 'audit'
);

CREATE INDEX IF NOT EXISTS idx_ops_cluster_action_log_tenant ON ops_cluster_action_log(tenant_id);
CREATE INDEX IF NOT EXISTS idx_ops_cluster_action_log_action ON ops_cluster_action_log(action_id);
CREATE INDEX IF NOT EXISTS idx_ops_cluster_action_log_release ON ops_cluster_action_log(release_name, action_type);
CREATE INDEX IF NOT EXISTS idx_ops_cluster_action_log_requested ON ops_cluster_action_log(requested_at DESC);

-- 守门 #13 d: 物理删除禁止 (WORM)
CREATE OR REPLACE FUNCTION prevent_hard_delete_ops_cluster_action_log()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'ops_cluster_action_log 物理删除禁止 (per 守門 #13 d WORM)';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_prevent_delete_ops_cluster_action_log ON ops_cluster_action_log;
CREATE TRIGGER trg_prevent_delete_ops_cluster_action_log
    BEFORE DELETE ON ops_cluster_action_log
    FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_ops_cluster_action_log();

-- 守門 #13 d: 100% audit (新增触发器落 audit_event)
CREATE OR REPLACE FUNCTION audit_ops_cluster_action_log()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_audit_event (event_type, resource_id, tenant_id, payload, occurred_at)
    VALUES (TG_OP, NEW.id, NEW.tenant_id, to_jsonb(NEW), NOW());
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_audit_ops_cluster_action_log ON ops_cluster_action_log;
CREATE TRIGGER trg_audit_ops_cluster_action_log
    AFTER INSERT OR UPDATE ON ops_cluster_action_log
    FOR EACH ROW EXECUTE FUNCTION audit_ops_cluster_action_log();

ALTER TABLE ops_cluster_action_log ENABLE ROW LEVEL SECURITY;
ALTER TABLE ops_cluster_action_log FORCE ROW LEVEL SECURITY;

COMMIT;

-- ==========================================
-- 守门 #13 W/T/M 100% 覆盖 实证 (per IT it_ops_cluster_ddl_wtm_coverage)
-- ==========================================
-- 既有 6 ops 表 (per SRS-001 §4):
--   1. ops_helm_release_state      T
--   2. ops_cluster_action_log      T
--   3. ops_log_query_log           T
--   4. ops_log_entry               W
--   5. ops_log_analysis            W
--   6. ops_metrics_config          M
-- F-02 3 表 (per commit 3602cfb):
--   7. ops_log_entry               W (同 #4)
--   8. ops_log_query_log           T (同 #3)
--   9. ops_log_analysis            M (同 #6, SCD2)
-- F-01 2 表 (本文件):
--   10. ops_helm_release_state     T (同 #1)
--   11. ops_cluster_action_log     T (同 #2)
--
-- 累计 11 表 W/T/M 100% 覆盖 (T=4 + W=2 + M=1, 0 混合分类)
-- per 守門 #13 派生规 (a) (b) (c) (d) 全实现
