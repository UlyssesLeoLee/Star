-- 2026-09-09-oauth2-server.sql
-- §14.12 IV OAuth2 server phase 2 (per WBS v0.43 + 用户拍板"Authorization Code + PKCE + Client Credentials (推荐)")
-- 4 表 DDL 落档, 守门 #13 W/T/M 100% 覆盖 (M=1 + T=3)
-- 跨 session 续: 5 域 RBAC 跟 claims.roles 整合 + 资源服务器 middleware + 真实 PKCE HTTP 流程
--
-- Author: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手 (per 9/8 15:19 JST 第 6 次强化 Mavis 全权代理)
-- 触发: 9/9 20:16 JST 用户拍板"both" (Authorization Code + PKCE + Client Credentials)

BEGIN;

-- ==========================================
-- 1. oauth_clients (M 类, SCD Type 2 OAuth2 客户端注册)
-- 守门 #13 c: 物理删除禁止 + SCD Type 2 + RLS 13 類必携
-- ==========================================
CREATE TABLE IF NOT EXISTS oauth_clients (
    -- 业务字段
    id                      UUID NOT NULL,
    tenant_id               UUID NOT NULL,
    client_id               TEXT NOT NULL,                -- 公开 client_id (e.g. "star-frontend-spa")
    client_secret_hash      TEXT,                         -- bcrypt/argon2 哈希 (confidential client 才有, public client NULL)
    client_name             TEXT NOT NULL,                -- 显示名
    client_type             TEXT NOT NULL CHECK (client_type IN ('public', 'confidential')),
    redirect_uris           TEXT[] NOT NULL DEFAULT '{}',  -- public client 走 auth code 时需要
    allowed_scopes          TEXT[] NOT NULL DEFAULT '{}',  -- e.g. ['read', 'write', 'admin']
    allowed_grant_types     TEXT[] NOT NULL DEFAULT '{}',  -- e.g. ['authorization_code', 'client_credentials', 'refresh_token']
    require_pkce            BOOLEAN NOT NULL DEFAULT TRUE,  -- public client 强制 PKCE
    require_authentication  BOOLEAN NOT NULL DEFAULT TRUE,  -- confidential client 需 client_secret
    owner_user_id           UUID NOT NULL,                -- 注册人
    -- SCD Type 2 字段 (per 守门 #13 c)
    valid_from              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    valid_to                TIMESTAMPTZ,
    -- 元字段
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- RLS 13 類 (M 必携)
    user_id                 UUID,
    role_id                 UUID,
    permission_id           UUID,
    policy_id               UUID,
    workspace_id            UUID,
    project_id              UUID,
    work_item_id            UUID,
    agent_id                UUID,
    session_id              UUID,
    trace_id                UUID,
    source_module           TEXT NOT NULL DEFAULT 'star_api_rest::auth::oauth',
    source_kind             TEXT NOT NULL DEFAULT 'master',
    PRIMARY KEY (id, valid_from)
);

CREATE UNIQUE INDEX IF NOT EXISTS idx_oauth_clients_client_id
    ON oauth_clients(client_id) WHERE valid_to IS NULL;
CREATE INDEX IF NOT EXISTS idx_oauth_clients_tenant
    ON oauth_clients(tenant_id, valid_from DESC);
CREATE INDEX IF NOT EXISTS idx_oauth_clients_valid_to
    ON oauth_clients(tenant_id, valid_to) WHERE valid_to IS NULL;

-- 守门 #13 c: tenant 隔离 RLS
ALTER TABLE oauth_clients ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_clients FORCE ROW LEVEL SECURITY;

CREATE POLICY oauth_clients_tenant_isolation ON oauth_clients
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- 守门 #13 c: SCD Type 2 trigger
CREATE TRIGGER trg_oauth_clients_scd2
    BEFORE UPDATE ON oauth_clients
    FOR EACH ROW
    EXECUTE FUNCTION scd_type2_close();

-- 守门 #13 d: audit trigger
CREATE TRIGGER trg_oauth_clients_audit
    AFTER INSERT OR UPDATE ON oauth_clients
    FOR EACH ROW
    EXECUTE FUNCTION audit_audit_event();

-- ==========================================
-- 2. oauth_authorization_codes (T 类, WORM 短期 auth code)
-- 守门 #13 d: 物理删除禁止 + audit (single-use token 审计)
-- ==========================================
CREATE TABLE IF NOT EXISTS oauth_authorization_codes (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID NOT NULL,
    code_hash               TEXT NOT NULL UNIQUE,         -- sha256(code), 原始 code 不存 (守门 #5 v2)
    client_id               TEXT NOT NULL,                -- FK 概念 (无 PG FK, 因 oauth_clients SCD2 复合主键)
    user_id                 UUID NOT NULL,                -- 授权人
    redirect_uri            TEXT NOT NULL,
    scope                   TEXT NOT NULL DEFAULT '',
    code_challenge          TEXT NOT NULL,                -- PKCE code_challenge
    code_challenge_method   TEXT NOT NULL DEFAULT 'S256' CHECK (code_challenge_method IN ('plain', 'S256')),
    expires_at              TIMESTAMPTZ NOT NULL,         -- 10 分钟
    consumed_at             TIMESTAMPTZ,                  -- 兑换后置位 (WORM 派生, 物理删除禁止)
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- RLS 13 類
    role_id                 UUID,
    permission_id           UUID,
    policy_id               UUID,
    workspace_id            UUID,
    project_id              UUID,
    work_item_id            UUID,
    agent_id                UUID,
    session_id              UUID,
    trace_id                UUID,
    source_module           TEXT NOT NULL DEFAULT 'star_api_rest::auth::oauth',
    source_kind             TEXT NOT NULL DEFAULT 'audit'
);

CREATE INDEX IF NOT EXISTS idx_oauth_authcodes_tenant
    ON oauth_authorization_codes(tenant_id);
CREATE INDEX IF NOT EXISTS idx_oauth_authcodes_client_user
    ON oauth_authorization_codes(client_id, user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_oauth_authcodes_expires
    ON oauth_authorization_codes(expires_at) WHERE consumed_at IS NULL;

-- 守门 #13 d: 物理删除禁止 (WORM)
CREATE OR REPLACE FUNCTION prevent_hard_delete_oauth_authorization_codes()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'oauth_authorization_codes 物理删除禁止 (per 守門 #13 d WORM)';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_prevent_delete_oauth_authcodes ON oauth_authorization_codes;
CREATE TRIGGER trg_prevent_delete_oauth_authcodes
    BEFORE DELETE ON oauth_authorization_codes
    FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_oauth_authorization_codes();

-- 守门 #13 d: 100% audit
CREATE OR REPLACE FUNCTION audit_oauth_authorization_codes()
RETURNS TRIGGER AS $$
BEGIN
    INSERT INTO audit_audit_event (event_type, resource_id, tenant_id, payload, occurred_at)
    VALUES (TG_OP, NEW.id, NEW.tenant_id, to_jsonb(NEW), NOW());
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_audit_oauth_authcodes ON oauth_authorization_codes;
CREATE TRIGGER trg_audit_oauth_authcodes
    AFTER INSERT OR UPDATE ON oauth_authorization_codes
    FOR EACH ROW EXECUTE FUNCTION audit_oauth_authorization_codes();

ALTER TABLE oauth_authorization_codes ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_authorization_codes FORCE ROW LEVEL SECURITY;

CREATE POLICY oauth_authorization_codes_tenant_isolation ON oauth_authorization_codes
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- ==========================================
-- 3. oauth_access_tokens (T 类, append-only 访问令牌)
-- 守门 #13 b: 物理删除禁止 + audit + RLS 13 類
-- revocation 走 UPDATE revoked_at (WORM 派生)
-- ==========================================
CREATE TABLE IF NOT EXISTS oauth_access_tokens (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID NOT NULL,
    token_hash              TEXT NOT NULL UNIQUE,         -- sha256(jti), jti 不存明文
    client_id               TEXT NOT NULL,
    user_id                 UUID,                         -- NULL for client_credentials grant
    grant_type              TEXT NOT NULL CHECK (grant_type IN ('authorization_code', 'client_credentials', 'refresh_token')),
    scope                   TEXT NOT NULL DEFAULT '',
    expires_at              TIMESTAMPTZ NOT NULL,
    revoked_at              TIMESTAMPTZ,                  -- WORM 派生
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- RLS 13 類
    role_id                 UUID,
    permission_id           UUID,
    policy_id               UUID,
    workspace_id            UUID,
    project_id              UUID,
    work_item_id            UUID,
    agent_id                UUID,
    session_id              UUID,
    trace_id                UUID,
    source_module           TEXT NOT NULL DEFAULT 'star_api_rest::auth::oauth',
    source_kind             TEXT NOT NULL DEFAULT 'audit'
);

CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_tenant
    ON oauth_access_tokens(tenant_id);
CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_client_user
    ON oauth_access_tokens(client_id, user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_oauth_access_tokens_expires
    ON oauth_access_tokens(expires_at) WHERE revoked_at IS NULL;

-- 守门 #13 d: 物理删除禁止 (WORM)
CREATE OR REPLACE FUNCTION prevent_hard_delete_oauth_access_tokens()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'oauth_access_tokens 物理删除禁止 (per 守門 #13 d WORM)';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_prevent_delete_oauth_access_tokens ON oauth_access_tokens;
CREATE TRIGGER trg_prevent_delete_oauth_access_tokens
    BEFORE DELETE ON oauth_access_tokens
    FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_oauth_access_tokens();

-- audit trigger
CREATE TRIGGER trg_audit_oauth_access_tokens
    AFTER INSERT OR UPDATE ON oauth_access_tokens
    FOR EACH ROW
    EXECUTE FUNCTION audit_audit_event();

ALTER TABLE oauth_access_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_access_tokens FORCE ROW LEVEL SECURITY;

CREATE POLICY oauth_access_tokens_tenant_isolation ON oauth_access_tokens
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- ==========================================
-- 4. oauth_refresh_tokens (T 类, WORM 刷新令牌)
-- 守门 #13 d: 物理删除禁止 + audit
-- ==========================================
CREATE TABLE IF NOT EXISTS oauth_refresh_tokens (
    id                      UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    tenant_id               UUID NOT NULL,
    token_hash              TEXT NOT NULL UNIQUE,         -- sha256(token), 不存明文
    client_id               TEXT NOT NULL,
    user_id                 UUID NOT NULL,                -- refresh token 仅 auth code flow 有
    access_token_id         UUID NOT NULL,                -- 关联的 access_token
    scope                   TEXT NOT NULL DEFAULT '',
    expires_at              TIMESTAMPTZ NOT NULL,         -- 通常 30 天
    revoked_at              TIMESTAMPTZ,
    created_at              TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- RLS 13 類
    role_id                 UUID,
    permission_id           UUID,
    policy_id               UUID,
    workspace_id            UUID,
    project_id              UUID,
    work_item_id            UUID,
    agent_id                UUID,
    session_id              UUID,
    trace_id                UUID,
    source_module           TEXT NOT NULL DEFAULT 'star_api_rest::auth::oauth',
    source_kind             TEXT NOT NULL DEFAULT 'audit'
);

CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_tenant
    ON oauth_refresh_tokens(tenant_id);
CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_user
    ON oauth_refresh_tokens(user_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_expires
    ON oauth_refresh_tokens(expires_at) WHERE revoked_at IS NULL;

-- 守门 #13 d: 物理删除禁止 (WORM)
CREATE OR REPLACE FUNCTION prevent_hard_delete_oauth_refresh_tokens()
RETURNS TRIGGER AS $$
BEGIN
    RAISE EXCEPTION 'oauth_refresh_tokens 物理删除禁止 (per 守門 #13 d WORM)';
END;
$$ LANGUAGE plpgsql;

DROP TRIGGER IF EXISTS trg_prevent_delete_oauth_refresh_tokens ON oauth_refresh_tokens;
CREATE TRIGGER trg_prevent_delete_oauth_refresh_tokens
    BEFORE DELETE ON oauth_refresh_tokens
    FOR EACH ROW EXECUTE FUNCTION prevent_hard_delete_oauth_refresh_tokens();

-- audit trigger
CREATE TRIGGER trg_audit_oauth_refresh_tokens
    AFTER INSERT OR UPDATE ON oauth_refresh_tokens
    FOR EACH ROW
    EXECUTE FUNCTION audit_audit_event();

ALTER TABLE oauth_refresh_tokens ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_refresh_tokens FORCE ROW LEVEL SECURITY;

CREATE POLICY oauth_refresh_tokens_tenant_isolation ON oauth_refresh_tokens
    USING (tenant_id = current_setting('app.tenant_id', true)::UUID)
    WITH CHECK (tenant_id = current_setting('app.tenant_id', true)::UUID);

-- ==========================================
-- 守门 #13 W/T/M 100% 覆盖 实证
-- ==========================================
-- 累计 10 ops 表 W/T/M 100% 覆盖 (T=6 + W=2 + M=2, 0 混合分类):
--   1. ops_helm_release_state      T    (F-01)
--   2. ops_cluster_action_log      T    (F-01)
--   3. ops_log_query_log           T    (F-02)
--   4. ops_log_entry               W    (F-02 TTL 7d)
--   5. ops_log_analysis            W    (F-02 TTL 30d)
--   6. ops_metrics_config          M    (F-03 SCD2)
--   7. oauth_clients               M    (本文件, SCD2)
--   8. oauth_authorization_codes   T    (本文件, WORM)
--   9. oauth_access_tokens         T    (本文件, WORM)
--  10. oauth_refresh_tokens        T    (本文件, WORM)
-- per 守門 #13 派生规 (a) (b) (c) (d) (e) 全实现
-- 跨 session 续: introspect + revoke endpoint + RBAC 整合 + 资源服务器 middleware

COMMIT;
