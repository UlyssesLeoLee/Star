-- crates/sandboxd/migrations/001_sandbox_session.sql
-- per DD-SANDBOX-002 §6.1 (T 派生规: 物理删除禁止 + 审计必须 + RLS 13 類必携)
-- 偏差标注: DD 引用 tenant/workspace/actor 表 FK, 本仓 db/migrations 尚无该 3 表,
--   故 FK 约束省略 (仅保留 13 類 UUID 字段), 待 3 表落地后补 FK (per 守门 #11 显式标注)。

CREATE TABLE IF NOT EXISTS sandbox_session (
    session_id          UUID PRIMARY KEY,
    client_id           VARCHAR(64) NOT NULL,
    task_id             VARCHAR(128) NOT NULL,
    parent_session_id   UUID REFERENCES sandbox_session(session_id) DEFERRABLE INITIALLY DEFERRED,
    policy_snapshot     JSONB NOT NULL,
    status              VARCHAR(16) NOT NULL CHECK (status IN ('Created', 'Running', 'Destroyed', 'Failed')),
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),
    destroyed_at        TIMESTAMPTZ,
    destroy_reason      TEXT,
    resource_peak       JSONB,

    -- RLS 13 類必携 (per 守门 #13c)
    tenant_id           UUID NOT NULL,
    workspace_id        UUID NOT NULL,
    actor_id            UUID NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_sandbox_session_client_id ON sandbox_session (client_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_sandbox_session_status ON sandbox_session (status) WHERE status IN ('Created', 'Running');
CREATE INDEX IF NOT EXISTS idx_sandbox_session_tenant ON sandbox_session (tenant_id, created_at DESC);
