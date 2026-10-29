-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-28-w16-device-auth.sql"})
-- MERGE (challenges:Table {name:"oauth_device_enrollment_challenges",type:"table",class:"W_short_ttl"})
-- MERGE (bindings:Table {name:"oauth_device_bindings",type:"table",class:"T_mutable_security_state"})
-- MERGE (audit:Table {name:"oauth_device_binding_audit",type:"table",class:"T_append_only"})
-- MERGE (refresh:Table {name:"oauth_refresh_tokens",type:"table",class:"T_worm"})
-- MERGE (engineeringOwner:Type {id:"star_engineering_directory_owner",kind:"database_role"})
-- MERGE (oauthOwner:Type {id:"star_oauth_credential_owner",kind:"database_role"})
-- MERGE (deviceLockPolicy:Policy {id:"oauth_device_bindings::engineering_directory_owner_no_oauth_device_binding_update",kind:"row_level_security"})
-- MERGE (refreshRevokePolicy:Policy {id:"oauth_refresh_tokens::oauth_owner_device_revoke_scope",kind:"row_level_security"})
-- MERGE (self)-[:CREATES]->(challenges),(self)-[:CREATES]->(bindings),(self)-[:CREATES]->(audit)
-- MERGE (self)-[:CONFIGURES]->(refresh)
-- MERGE (self)-[:GRANTS_TO]->(engineeringOwner),(self)-[:GRANTS_TO]->(oauthOwner)
-- MERGE (self)-[:CONFIGURES]->(deviceLockPolicy),(self)-[:CONFIGURES]->(refreshRevokePolicy)
-- MERGE (deviceLockPolicy)-[:PROTECTS]->(bindings)
-- MERGE (refreshRevokePolicy)-[:PROTECTS]->(refresh)
-- @endcypher
-- One physical SQL statement per line for the repository migration runner.
ALTER TABLE oauth_refresh_tokens ADD COLUMN IF NOT EXISTS device_id UUID;
ALTER TABLE oauth_refresh_tokens ADD COLUMN IF NOT EXISTS device_generation BIGINT;
DO $constraint$ DECLARE current_definition TEXT; current_is_validated BOOLEAN; BEGIN SELECT pg_get_constraintdef(oid), convalidated INTO current_definition, current_is_validated FROM pg_constraint WHERE conrelid='oauth_refresh_tokens'::regclass AND conname='oauth_refresh_tokens_device_generation_pair'; IF current_definition IS NULL THEN ALTER TABLE oauth_refresh_tokens ADD CONSTRAINT oauth_refresh_tokens_device_generation_pair CHECK ((device_id IS NULL AND device_generation IS NULL) OR (device_id IS NOT NULL AND device_generation IS NOT NULL AND device_generation > 0)); ELSIF NOT current_is_validated OR position('device_generation is not null' in lower(current_definition))=0 THEN ALTER TABLE oauth_refresh_tokens DROP CONSTRAINT oauth_refresh_tokens_device_generation_pair; ALTER TABLE oauth_refresh_tokens ADD CONSTRAINT oauth_refresh_tokens_device_generation_pair CHECK ((device_id IS NULL AND device_generation IS NULL) OR (device_id IS NOT NULL AND device_generation IS NOT NULL AND device_generation > 0)); END IF; END $constraint$;
CREATE INDEX IF NOT EXISTS idx_oauth_refresh_tokens_device_active ON oauth_refresh_tokens(tenant_id,user_id,device_id) WHERE revoked_at IS NULL AND device_id IS NOT NULL;
CREATE TABLE IF NOT EXISTS oauth_device_enrollment_challenges (challenge_id UUID PRIMARY KEY, tenant_id UUID NOT NULL CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'), actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'), challenge_hash BYTEA NOT NULL CHECK (octet_length(challenge_hash)=32), public_key BYTEA NOT NULL CHECK (octet_length(public_key)=32), public_key_fingerprint TEXT NOT NULL CHECK (public_key_fingerprint ~ '^[0-9a-f]{64}$'), expires_at TIMESTAMPTZ NOT NULL, consumed_at TIMESTAMPTZ, created_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(), CHECK (expires_at > created_at), CHECK (consumed_at IS NULL OR consumed_at >= created_at));
CREATE INDEX IF NOT EXISTS idx_oauth_device_enrollment_challenges_expiry ON oauth_device_enrollment_challenges(expires_at) WHERE consumed_at IS NULL;
CREATE INDEX IF NOT EXISTS idx_oauth_device_enrollment_challenges_owner_expiry ON oauth_device_enrollment_challenges(tenant_id,actor_id,expires_at) WHERE consumed_at IS NULL;
CREATE TABLE IF NOT EXISTS oauth_device_bindings (tenant_id UUID NOT NULL CHECK (tenant_id <> '00000000-0000-0000-0000-000000000000'), actor_id UUID NOT NULL CHECK (actor_id <> '00000000-0000-0000-0000-000000000000'), device_id UUID NOT NULL CHECK (device_id <> '00000000-0000-0000-0000-000000000000'), public_key_fingerprint TEXT NOT NULL CHECK (public_key_fingerprint ~ '^[0-9a-f]{64}$'), generation BIGINT NOT NULL CHECK (generation > 0), status TEXT NOT NULL CHECK (status IN ('active','revoked')), created_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(), revoked_at TIMESTAMPTZ, PRIMARY KEY (tenant_id,device_id), CHECK ((status='active' AND revoked_at IS NULL) OR (status='revoked' AND revoked_at IS NOT NULL)));
CREATE UNIQUE INDEX IF NOT EXISTS idx_oauth_device_bindings_active_fingerprint ON oauth_device_bindings(tenant_id,actor_id,public_key_fingerprint) WHERE status='active';
CREATE TABLE IF NOT EXISTS oauth_device_binding_audit (event_id UUID PRIMARY KEY DEFAULT gen_random_uuid(), tenant_id UUID NOT NULL, actor_id UUID NOT NULL, device_id UUID NOT NULL, event_kind TEXT NOT NULL CHECK (event_kind IN ('enrolled','revoked')), generation BIGINT NOT NULL CHECK (generation > 0), public_key_fingerprint TEXT NOT NULL CHECK (public_key_fingerprint ~ '^[0-9a-f]{64}$'), occurred_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(), source_module TEXT NOT NULL DEFAULT 'star_api_rest::auth::oauth::device_enrollment');
CREATE OR REPLACE FUNCTION prevent_oauth_device_binding_audit_mutation() RETURNS TRIGGER AS $$ BEGIN RAISE EXCEPTION 'oauth_device_binding_audit is append-only'; END; $$ LANGUAGE plpgsql;
DROP TRIGGER IF EXISTS trg_oauth_device_binding_audit_no_mutation ON oauth_device_binding_audit;
CREATE TRIGGER trg_oauth_device_binding_audit_no_mutation BEFORE UPDATE OR DELETE OR TRUNCATE ON oauth_device_binding_audit FOR EACH STATEMENT EXECUTE FUNCTION prevent_oauth_device_binding_audit_mutation();
CREATE OR REPLACE FUNCTION public.w16_notify_oauth_device_binding_change() RETURNS TRIGGER LANGUAGE plpgsql SET search_path=pg_catalog AS $function$ BEGIN PERFORM pg_catalog.pg_notify('star_device_binding_wake','changed'); RETURN NEW; END; $function$;
REVOKE ALL ON FUNCTION public.w16_notify_oauth_device_binding_change() FROM PUBLIC;
DROP TRIGGER IF EXISTS w16_oauth_device_binding_generation_wake ON oauth_device_bindings;
CREATE TRIGGER w16_oauth_device_binding_generation_wake AFTER UPDATE OF generation,status ON oauth_device_bindings FOR EACH ROW EXECUTE FUNCTION public.w16_notify_oauth_device_binding_change();
ALTER TABLE oauth_device_enrollment_challenges ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_device_enrollment_challenges FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS oauth_device_enrollment_challenges_owner_scope ON oauth_device_enrollment_challenges;
CREATE POLICY oauth_device_enrollment_challenges_owner_scope ON oauth_device_enrollment_challenges USING (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND actor_id=NULLIF(current_setting('app.actor_id',true),'')::UUID) WITH CHECK (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND actor_id=NULLIF(current_setting('app.actor_id',true),'')::UUID);
ALTER TABLE oauth_device_bindings ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_device_bindings FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS oauth_device_bindings_owner_scope ON oauth_device_bindings;
CREATE POLICY oauth_device_bindings_owner_scope ON oauth_device_bindings USING (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND actor_id=NULLIF(current_setting('app.actor_id',true),'')::UUID) WITH CHECK (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND actor_id=NULLIF(current_setting('app.actor_id',true),'')::UUID);
ALTER TABLE oauth_device_binding_audit ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_device_binding_audit FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS oauth_device_binding_audit_owner_insert ON oauth_device_binding_audit;
CREATE POLICY oauth_device_binding_audit_owner_insert ON oauth_device_binding_audit FOR INSERT WITH CHECK (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND actor_id=NULLIF(current_setting('app.actor_id',true),'')::UUID);
REVOKE UPDATE, DELETE, TRUNCATE ON oauth_device_binding_audit FROM PUBLIC;
COMMENT ON TABLE oauth_device_enrollment_challenges IS 'W: short-lived, single-use device proof challenge; only challenge hash and public credential are stored.';
COMMENT ON TABLE oauth_device_bindings IS 'T: mutable security owner state; device identity and generation are tenant/actor scoped and are rechecked for shared collaboration.';
COMMENT ON TABLE oauth_device_binding_audit IS 'T: append-only device enrollment/revocation facts; contains fingerprint only, never proof material or private credentials.';
-- Dedicated, non-login roles: the owner can read only credential rows needed by hash resolution; runtime receives EXECUTE only after the deployment login is explicitly granted membership.
DO $w16_oauth_roles$ BEGIN IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='star_oauth_credential_owner') THEN CREATE ROLE star_oauth_credential_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT; ELSE ALTER ROLE star_oauth_credential_owner NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT; END IF; IF NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname='star_oauth_token_runtime') THEN CREATE ROLE star_oauth_token_runtime NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT; ELSE ALTER ROLE star_oauth_token_runtime NOLOGIN NOSUPERUSER NOCREATEDB NOCREATEROLE NOREPLICATION NOBYPASSRLS NOINHERIT; END IF; END $w16_oauth_roles$;
-- Device/OAuth repository transactions explicitly SET LOCAL ROLE star_oauth_credential_owner; deployment login membership is provisioned separately.
REVOKE ALL ON TABLE oauth_device_enrollment_challenges, oauth_device_bindings, oauth_device_binding_audit FROM PUBLIC;
REVOKE ALL ON TABLE oauth_device_enrollment_challenges, oauth_device_bindings, oauth_device_binding_audit FROM star_oauth_credential_owner;
GRANT SELECT, INSERT, DELETE ON TABLE oauth_device_enrollment_challenges TO star_oauth_credential_owner;
GRANT UPDATE (consumed_at) ON TABLE oauth_device_enrollment_challenges TO star_oauth_credential_owner;
GRANT SELECT, INSERT ON TABLE oauth_device_bindings TO star_oauth_credential_owner;
GRANT UPDATE (generation, status, revoked_at) ON TABLE oauth_device_bindings TO star_oauth_credential_owner;
GRANT INSERT ON TABLE oauth_device_binding_audit TO star_oauth_credential_owner;
-- SELECT ... FOR SHARE requires an UPDATE column ACL and applies UPDATE RLS USING; scope locks to this actor, while WITH CHECK(false) rejects every actual Directory-role update.
REVOKE ALL ON TABLE oauth_device_bindings FROM star_engineering_directory_owner;
GRANT SELECT ON TABLE oauth_device_bindings TO star_engineering_directory_owner;
GRANT UPDATE (generation) ON TABLE oauth_device_bindings TO star_engineering_directory_owner;
DROP POLICY IF EXISTS engineering_directory_owner_no_oauth_device_binding_update ON oauth_device_bindings;
CREATE POLICY engineering_directory_owner_no_oauth_device_binding_update ON oauth_device_bindings AS RESTRICTIVE FOR UPDATE TO star_engineering_directory_owner USING (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND actor_id=NULLIF(current_setting('app.actor_id',true),'')::UUID) WITH CHECK (false);
CREATE TABLE IF NOT EXISTS oauth_credential_tenant_locator (credential_kind TEXT NOT NULL CHECK (credential_kind IN ('authorization_code','refresh_token')), token_hash TEXT NOT NULL, tenant_id UUID NOT NULL, actor_id UUID NOT NULL, client_id TEXT NOT NULL, expires_at TIMESTAMPTZ NOT NULL, created_at TIMESTAMPTZ NOT NULL DEFAULT transaction_timestamp(), PRIMARY KEY (credential_kind,token_hash));
CREATE INDEX IF NOT EXISTS idx_oauth_credential_tenant_locator_expiry ON oauth_credential_tenant_locator(expires_at);
ALTER TABLE oauth_credential_tenant_locator ENABLE ROW LEVEL SECURITY;
ALTER TABLE oauth_credential_tenant_locator FORCE ROW LEVEL SECURITY;
DROP POLICY IF EXISTS oauth_credential_locator_select_exact_hash ON oauth_credential_tenant_locator;
CREATE POLICY oauth_credential_locator_select_exact_hash ON oauth_credential_tenant_locator FOR SELECT USING (credential_kind=NULLIF(current_setting('app.oauth_credential_kind',true),'') AND token_hash=NULLIF(current_setting('app.oauth_credential_hash',true),''));
DROP POLICY IF EXISTS oauth_credential_locator_insert_owner ON oauth_credential_tenant_locator;
CREATE POLICY oauth_credential_locator_insert_owner ON oauth_credential_tenant_locator FOR INSERT WITH CHECK (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND actor_id=NULLIF(current_setting('app.actor_id',true),'')::UUID);
REVOKE ALL ON oauth_credential_tenant_locator FROM PUBLIC;
GRANT USAGE ON SCHEMA public TO star_oauth_credential_owner, star_oauth_token_runtime;
DO $w16_oauth_no_table_ownership$ BEGIN IF EXISTS (SELECT 1 FROM pg_class c JOIN pg_namespace n ON n.oid=c.relnamespace WHERE c.relowner='star_oauth_credential_owner'::regrole AND n.nspname='public' AND c.relname IN ('oauth_device_enrollment_challenges','oauth_device_bindings','oauth_device_binding_audit','oauth_refresh_tokens','oauth_authorization_codes','oauth_credential_tenant_locator')) THEN RAISE EXCEPTION 'star_oauth_credential_owner must not own OAuth/device tables'; END IF; END $w16_oauth_no_table_ownership$;
REVOKE ALL ON TABLE oauth_credential_tenant_locator FROM star_oauth_credential_owner;
GRANT SELECT, INSERT ON oauth_credential_tenant_locator TO star_oauth_credential_owner;
REVOKE ALL ON TABLE oauth_authorization_codes FROM star_oauth_credential_owner;
GRANT SELECT ON oauth_authorization_codes TO star_oauth_credential_owner;
REVOKE ALL ON TABLE oauth_refresh_tokens FROM star_oauth_credential_owner;
GRANT SELECT ON oauth_refresh_tokens TO star_oauth_credential_owner;
GRANT UPDATE (revoked_at) ON oauth_refresh_tokens TO star_oauth_credential_owner;
DROP POLICY IF EXISTS oauth_credential_owner_device_revoke_refresh ON oauth_refresh_tokens;
CREATE POLICY oauth_credential_owner_device_revoke_refresh ON oauth_refresh_tokens FOR UPDATE TO star_oauth_credential_owner USING (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND user_id=NULLIF(current_setting('app.actor_id',true),'')::UUID) WITH CHECK (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND user_id=NULLIF(current_setting('app.actor_id',true),'')::UUID);
DROP POLICY IF EXISTS oauth_credential_owner_device_revoke_refresh_scope ON oauth_refresh_tokens;
CREATE POLICY oauth_credential_owner_device_revoke_refresh_scope ON oauth_refresh_tokens AS RESTRICTIVE FOR UPDATE TO star_oauth_credential_owner USING (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND user_id=NULLIF(current_setting('app.actor_id',true),'')::UUID) WITH CHECK (tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID AND user_id=NULLIF(current_setting('app.actor_id',true),'')::UUID);
DROP FUNCTION IF EXISTS public.w16_resolve_oauth_authorization_code(UUID,TEXT,TEXT);
DROP FUNCTION IF EXISTS public.w16_resolve_oauth_refresh_token(UUID,TEXT);
CREATE OR REPLACE FUNCTION public.w16_index_oauth_authorization_code() RETURNS TRIGGER LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $function$ BEGIN PERFORM set_config('app.tenant_id',NEW.tenant_id::text,true); PERFORM set_config('app.actor_id',NEW.user_id::text,true); PERFORM set_config('app.oauth_credential_kind','authorization_code',true); PERFORM set_config('app.oauth_credential_hash',NEW.code_hash,true); INSERT INTO public.oauth_credential_tenant_locator (credential_kind,token_hash,tenant_id,actor_id,client_id,expires_at) VALUES ('authorization_code',NEW.code_hash,NEW.tenant_id,NEW.user_id,NEW.client_id,NEW.expires_at); RETURN NEW; END; $function$;
CREATE OR REPLACE FUNCTION public.w16_index_oauth_refresh_token() RETURNS TRIGGER LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $function$ BEGIN PERFORM set_config('app.tenant_id',NEW.tenant_id::text,true); PERFORM set_config('app.actor_id',NEW.user_id::text,true); PERFORM set_config('app.oauth_credential_kind','refresh_token',true); PERFORM set_config('app.oauth_credential_hash',NEW.token_hash,true); INSERT INTO public.oauth_credential_tenant_locator (credential_kind,token_hash,tenant_id,actor_id,client_id,expires_at) VALUES ('refresh_token',NEW.token_hash,NEW.tenant_id,NEW.user_id,NEW.client_id,NEW.expires_at); RETURN NEW; END; $function$;
CREATE OR REPLACE FUNCTION public.w16_resolve_oauth_authorization_code(p_code_hash TEXT,p_client_id TEXT) RETURNS SETOF public.oauth_authorization_codes LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $function$ DECLARE v_tenant UUID; v_actor UUID; v_client TEXT; BEGIN IF p_code_hash IS NULL OR p_client_id IS NULL THEN RETURN; END IF; PERFORM set_config('app.oauth_credential_kind','authorization_code',true); PERFORM set_config('app.oauth_credential_hash',p_code_hash,true); SELECT l.tenant_id,l.actor_id,l.client_id INTO v_tenant,v_actor,v_client FROM public.oauth_credential_tenant_locator AS l WHERE l.credential_kind='authorization_code' AND l.token_hash=p_code_hash AND l.client_id=p_client_id AND l.expires_at>clock_timestamp(); IF NOT FOUND OR v_client IS DISTINCT FROM p_client_id THEN RETURN; END IF; PERFORM set_config('app.tenant_id',v_tenant::text,true); PERFORM set_config('app.actor_id',v_actor::text,true); RETURN QUERY SELECT c.* FROM public.oauth_authorization_codes AS c WHERE c.tenant_id=v_tenant AND c.user_id=v_actor AND c.client_id=v_client AND c.code_hash=p_code_hash AND c.consumed_at IS NULL AND c.expires_at>clock_timestamp(); END; $function$;
CREATE OR REPLACE FUNCTION public.w16_resolve_oauth_refresh_token(p_refresh_hash TEXT,p_client_id TEXT) RETURNS SETOF public.oauth_refresh_tokens LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $function$ DECLARE v_tenant UUID; v_actor UUID; v_client TEXT; BEGIN IF p_refresh_hash IS NULL OR p_client_id IS NULL THEN RETURN; END IF; PERFORM set_config('app.oauth_credential_kind','refresh_token',true); PERFORM set_config('app.oauth_credential_hash',p_refresh_hash,true); SELECT l.tenant_id,l.actor_id,l.client_id INTO v_tenant,v_actor,v_client FROM public.oauth_credential_tenant_locator AS l WHERE l.credential_kind='refresh_token' AND l.token_hash=p_refresh_hash AND l.client_id=p_client_id AND l.expires_at>clock_timestamp(); IF NOT FOUND OR v_client IS DISTINCT FROM p_client_id THEN RETURN; END IF; PERFORM set_config('app.tenant_id',v_tenant::text,true); PERFORM set_config('app.actor_id',v_actor::text,true); RETURN QUERY SELECT r.* FROM public.oauth_refresh_tokens AS r WHERE r.tenant_id=v_tenant AND r.user_id=v_actor AND r.client_id=v_client AND r.token_hash=p_refresh_hash AND r.revoked_at IS NULL AND r.expires_at>clock_timestamp(); END; $function$;
ALTER FUNCTION public.w16_index_oauth_authorization_code() OWNER TO star_oauth_credential_owner;
ALTER FUNCTION public.w16_index_oauth_refresh_token() OWNER TO star_oauth_credential_owner;
ALTER FUNCTION public.w16_resolve_oauth_authorization_code(TEXT,TEXT) OWNER TO star_oauth_credential_owner;
ALTER FUNCTION public.w16_resolve_oauth_refresh_token(TEXT,TEXT) OWNER TO star_oauth_credential_owner;
REVOKE ALL ON FUNCTION public.w16_index_oauth_authorization_code() FROM PUBLIC;
REVOKE ALL ON FUNCTION public.w16_index_oauth_refresh_token() FROM PUBLIC;
REVOKE ALL ON FUNCTION public.w16_resolve_oauth_authorization_code(TEXT,TEXT) FROM PUBLIC;
REVOKE ALL ON FUNCTION public.w16_resolve_oauth_refresh_token(TEXT,TEXT) FROM PUBLIC;
GRANT EXECUTE ON FUNCTION public.w16_resolve_oauth_authorization_code(TEXT,TEXT) TO star_oauth_token_runtime;
GRANT EXECUTE ON FUNCTION public.w16_resolve_oauth_refresh_token(TEXT,TEXT) TO star_oauth_token_runtime;
DROP TRIGGER IF EXISTS trg_w16_index_oauth_authorization_code ON oauth_authorization_codes;
CREATE TRIGGER trg_w16_index_oauth_authorization_code AFTER INSERT ON oauth_authorization_codes FOR EACH ROW EXECUTE FUNCTION public.w16_index_oauth_authorization_code();
DROP TRIGGER IF EXISTS trg_w16_index_oauth_refresh_token ON oauth_refresh_tokens;
CREATE TRIGGER trg_w16_index_oauth_refresh_token AFTER INSERT ON oauth_refresh_tokens FOR EACH ROW EXECUTE FUNCTION public.w16_index_oauth_refresh_token();
-- Backfill requires a migration principal that can actually see FORCE-RLS rows; row_security=off makes missing privilege an error instead of a silent zero-row migration.
DO $w16_oauth_locator_backfill$ DECLARE r RECORD; BEGIN PERFORM set_config('row_security','off',true); FOR r IN SELECT tenant_id,user_id,client_id,code_hash,expires_at FROM public.oauth_authorization_codes WHERE consumed_at IS NULL AND expires_at>clock_timestamp() LOOP PERFORM set_config('app.tenant_id',r.tenant_id::text,true); PERFORM set_config('app.actor_id',r.user_id::text,true); PERFORM set_config('app.oauth_credential_kind','authorization_code',true); PERFORM set_config('app.oauth_credential_hash',r.code_hash,true); INSERT INTO public.oauth_credential_tenant_locator(credential_kind,token_hash,tenant_id,actor_id,client_id,expires_at) VALUES('authorization_code',r.code_hash,r.tenant_id,r.user_id,r.client_id,r.expires_at) ON CONFLICT DO NOTHING; END LOOP; FOR r IN SELECT tenant_id,user_id,client_id,token_hash,expires_at FROM public.oauth_refresh_tokens WHERE revoked_at IS NULL AND expires_at>clock_timestamp() LOOP PERFORM set_config('app.tenant_id',r.tenant_id::text,true); PERFORM set_config('app.actor_id',r.user_id::text,true); PERFORM set_config('app.oauth_credential_kind','refresh_token',true); PERFORM set_config('app.oauth_credential_hash',r.token_hash,true); INSERT INTO public.oauth_credential_tenant_locator(credential_kind,token_hash,tenant_id,actor_id,client_id,expires_at) VALUES('refresh_token',r.token_hash,r.tenant_id,r.user_id,r.client_id,r.expires_at) ON CONFLICT DO NOTHING; END LOOP; END $w16_oauth_locator_backfill$;
COMMENT ON TABLE oauth_credential_tenant_locator IS 'W: internal owner-scoped index for exact random OAuth credential hashes; FORCE RLS limits reads to the supplied hash and expiry is the retention boundary.';
