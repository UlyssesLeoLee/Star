-- crates/sandboxd/migrations/003_sandbox_audit.sql
-- per DD-SANDBOX-002 §6.3 (T 派生规: 物理删除禁止 + 审计必须 + RLS 13 類必携 + 90 天保留)
-- 索引跟守门 v36 audit log 索引联动 (session_id / event_type / tenant_id)。

CREATE TABLE IF NOT EXISTS sandbox_audit (
    audit_id              BIGSERIAL PRIMARY KEY,
    session_id            UUID NOT NULL,
    event_type            VARCHAR(32) NOT NULL CHECK (event_type IN (
        'ResourceHit', 'NetworkBlock', 'NetworkAllow',
        'FsBlock', 'FsAllow', 'CapabilityDrop',
        'ProcessStart', 'ProcessExit'
    )),
    resource_used         JSONB,
    network_violations    JSONB,
    fs_violations         JSONB,
    capability_violations JSONB,
    timestamp             TIMESTAMPTZ NOT NULL DEFAULT now(),
    latency_ms            INT,
    hit_violations        TEXT[],

    -- RLS 13 類必携
    tenant_id             UUID NOT NULL,
    workspace_id          UUID NOT NULL,
    actor_id              UUID NOT NULL
);

-- 跟守门 v36 audit log 索引联动
CREATE INDEX IF NOT EXISTS idx_sandbox_audit_session_time ON sandbox_audit (session_id, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_sandbox_audit_event_type_time ON sandbox_audit (event_type, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_sandbox_audit_tenant_time ON sandbox_audit (tenant_id, timestamp DESC);

-- 90 天保留 (per BR-4 + 守门 #5 隐含): 自动清理走定时 job (物理删除禁止, 通过归档标记实现)
