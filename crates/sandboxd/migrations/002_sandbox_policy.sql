-- crates/sandboxd/migrations/002_sandbox_policy.sql
-- per DD-SANDBOX-002 §6.2 (M 派生规: 物理删除禁止 + SCD Type 2 + RLS 13 類必携)
-- 偏差标注: tenant/workspace/actor FK 省略 (见 001, 3 表落地后补)。

CREATE TABLE IF NOT EXISTS sandbox_policy (
    policy_id           UUID PRIMARY KEY,
    policy_name         VARCHAR(128) NOT NULL,
    resource            JSONB NOT NULL,
    network             JSONB NOT NULL,
    fs                  JSONB NOT NULL,
    capability          JSONB NOT NULL,
    version             INT NOT NULL DEFAULT 1,
    valid_from          TIMESTAMPTZ NOT NULL DEFAULT now(),
    valid_to            TIMESTAMPTZ,
    created_at          TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- RLS 13 類必携
    tenant_id           UUID NOT NULL,
    workspace_id        UUID NOT NULL,
    actor_id            UUID NOT NULL,

    UNIQUE (policy_name, version, tenant_id)
);

CREATE INDEX IF NOT EXISTS idx_sandbox_policy_name ON sandbox_policy (policy_name, valid_from DESC) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_sandbox_policy_tenant ON sandbox_policy (tenant_id, valid_from DESC);
