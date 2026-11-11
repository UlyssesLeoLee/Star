-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-11-09-platform-native-transport-pull-privilege.sql"})
-- MERGE (pull:Function {id:"platform_transport.pull_one"})
-- MERGE (pullAuthenticated:Function {id:"platform_transport.pull_authenticated"})
-- MERGE (runtimeRole:Config {id:"star_platform_transport_runtime",kind:"postgres_role"})
-- MERGE (self)-[:CONFIGURES]->(pull)
-- MERGE (self)-[:CONFIGURES]->(pullAuthenticated)
-- MERGE (self)-[:CONFIGURES]->(runtimeRole)
-- @endcypher

-- Keep the internal lease routine behind the certificate/profile-authenticated wrapper.
--
-- platform_transport.pull_one is SECURITY DEFINER and reads the caller-controlled
-- app.platform_deployment_id, app.platform_tenant_id and app.platform_service_id settings
-- to scope its lease claim. The 2026-10-24 lease-order migration granted the runtime
-- role EXECUTE on it, which lets a runtime caller supply those identity settings and
-- bypass the certificate/profile authorization that platform_transport.pull_authenticated
-- performs first.
--
-- The native server-auth contract (db/tests/platform-native-transport-server-auth.sql)
-- requires the runtime to execute only the authenticated wrappers, and that file fails
-- against the current migration set because of the direct grant. This migration removes
-- the direct grant so pull_one is reachable only through pull_authenticated.
BEGIN;

REVOKE EXECUTE ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    FROM PUBLIC, star_platform_transport_runtime;

-- The wrapper must keep its own EXECUTE grant: it is the only runtime entry point that
-- authorizes the certificate and profile before it calls the internal routine.
REVOKE EXECUTE ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    FROM star_platform_transport_owner;
GRANT EXECUTE ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER)
    TO star_platform_transport_owner;

COMMENT ON FUNCTION platform_transport.pull_one(TEXT, UUID, INTEGER) IS
    'Internal per-log lease claim routine. Not callable by the runtime role; reachable only through platform_transport.pull_authenticated, which authorizes the client certificate and transport profile first.';

COMMIT;
