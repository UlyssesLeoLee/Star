-- crates/sandboxd/migrations/004_sandbox_capability.sql
-- per DD-SANDBOX-002 §6.4 (M 派生规: 物理删除禁止 + 静态参考数据 + RLS 13 類必携)
-- seed 跟 src/policy/capability.rs KNOWN_CAPABILITIES 1:1 (Linux 40 + Windows 8 + macOS 3)。

CREATE TABLE IF NOT EXISTS sandbox_capability (
    capability_id     UUID PRIMARY KEY,
    syscall_name      VARCHAR(128) NOT NULL,
    platform          VARCHAR(16) NOT NULL CHECK (platform IN ('Linux', 'Windows', 'macOS')),
    risk_level        VARCHAR(16) NOT NULL CHECK (risk_level IN ('Low', 'Medium', 'High', 'Critical')),
    description       TEXT NOT NULL,
    default_action    VARCHAR(8) NOT NULL CHECK (default_action IN ('Allow', 'Deny', 'Ask')),
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now(),

    -- RLS 13 類必携
    tenant_id         UUID NOT NULL,
    workspace_id      UUID NOT NULL,
    actor_id          UUID NOT NULL,

    UNIQUE (syscall_name, platform, tenant_id)
);

CREATE INDEX IF NOT EXISTS idx_sandbox_capability_platform ON sandbox_capability (platform, risk_level);
