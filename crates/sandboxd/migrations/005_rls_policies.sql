-- crates/sandboxd/migrations/005_rls_policies.sql
-- per DD-SANDBOX-002 §4.6 RLS 策略 (13 類必携 per 守门 #13c 派生), 4 表 100% 覆盖
-- current_setting(..., true) missing_ok: 未设置时 policy 求值为 NULL → fail-closed 不可见。
-- 写入侧由 PgAuditSink 在事务内 set_config (is_local=true) 注入三元组。

ALTER TABLE sandbox_session ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_session_tenant_isolation ON sandbox_session
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID
        AND workspace_id = current_setting('app.workspace_id', true)::UUID);

ALTER TABLE sandbox_policy ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_policy_tenant_isolation ON sandbox_policy
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID
        AND workspace_id = current_setting('app.workspace_id', true)::UUID);

ALTER TABLE sandbox_audit ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_audit_tenant_isolation ON sandbox_audit
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID
        AND workspace_id = current_setting('app.workspace_id', true)::UUID);

ALTER TABLE sandbox_capability ENABLE ROW LEVEL SECURITY;
CREATE POLICY sandbox_capability_tenant_isolation ON sandbox_capability
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID
        AND workspace_id = current_setting('app.workspace_id', true)::UUID);
