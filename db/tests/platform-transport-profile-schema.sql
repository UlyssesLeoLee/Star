-- @cypher schema=1 source_sha256=d3c4914432414dc4d57ed2b7ab715e9370623f440c3a89e8daab9ba9284f9e44
-- MERGE (self:File {path:"db/tests/platform-transport-profile-schema.sql"})
-- MERGE (profile:Table {id:"platform_messaging.transport_profile_revision"})
-- MERGE (audit:Table {id:"platform_messaging.transport_profile_revision_audit"})
-- MERGE (outbox:Table {id:"platform_messaging.transport_profile_revision_outbox"})
-- MERGE (owner:DatabaseRole {id:"star_platform_profile_owner"})
-- MERGE (resolver:DatabaseRole {id:"star_platform_profile_resolver"})
-- MERGE (self)-[:TESTS]->(profile)
-- MERGE (self)-[:TESTS]->(audit)
-- MERGE (self)-[:TESTS]->(outbox)
-- MERGE (self)-[:TESTS]->(owner)
-- MERGE (self)-[:TESTS]->(resolver)
-- @endcypher

DO $assertions$
DECLARE
    table_name TEXT;
    rls_enabled BOOLEAN;
    rls_forced BOOLEAN;
    role_is_restricted BOOLEAN;
    profile_tables TEXT[] := ARRAY[
        'platform_messaging.transport_profile_revision',
        'platform_messaging.transport_profile_revision_audit',
        'platform_messaging.transport_profile_revision_outbox'
    ];
BEGIN
    SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolcreatedb AND NOT rolcreaterole
           AND NOT rolreplication AND NOT rolbypassrls AND NOT rolinherit
      INTO role_is_restricted
      FROM pg_roles
     WHERE rolname = 'star_platform_profile_owner';
    IF role_is_restricted IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'transport profile owner role is missing or unsafe';
    END IF;
    SELECT NOT rolcanlogin AND NOT rolsuper AND NOT rolcreatedb AND NOT rolcreaterole
           AND NOT rolreplication AND NOT rolbypassrls AND NOT rolinherit
      INTO role_is_restricted
      FROM pg_roles
     WHERE rolname = 'star_platform_profile_resolver';
    IF role_is_restricted IS DISTINCT FROM TRUE THEN
        RAISE EXCEPTION 'transport profile resolver role is missing or unsafe';
    END IF;

    FOREACH table_name IN ARRAY profile_tables LOOP
        SELECT c.relrowsecurity, c.relforcerowsecurity
          INTO rls_enabled, rls_forced
          FROM pg_class c
         WHERE c.oid = table_name::regclass;
        IF rls_enabled IS DISTINCT FROM TRUE OR rls_forced IS DISTINCT FROM TRUE THEN
            RAISE EXCEPTION 'transport profile table % must enable FORCE RLS', table_name;
        END IF;
    END LOOP;

    IF NOT has_table_privilege('star_platform_profile_owner',
            'platform_messaging.transport_profile_revision', 'SELECT')
       OR NOT has_table_privilege('star_platform_profile_owner',
            'platform_messaging.transport_profile_revision', 'INSERT')
       OR NOT has_column_privilege('star_platform_profile_owner',
            'platform_messaging.transport_profile_revision', 'valid_to', 'UPDATE')
       OR has_column_privilege('star_platform_profile_owner',
            'platform_messaging.transport_profile_revision', 'provider_id', 'UPDATE')
       OR has_table_privilege('star_platform_profile_owner',
            'platform_messaging.transport_profile_revision', 'DELETE') THEN
        RAISE EXCEPTION 'transport profile owner revision grants are not least privilege';
    END IF;

    IF has_table_privilege('star_platform_profile_owner',
            'platform_messaging.transport_profile_revision_audit', 'INSERT')
       OR has_table_privilege('star_platform_profile_owner',
            'platform_messaging.transport_profile_revision_outbox', 'INSERT')
       OR NOT has_table_privilege('star_platform_profile_owner',
            'platform_messaging.transport_profile_revision_audit', 'SELECT')
       OR NOT has_table_privilege('star_platform_profile_owner',
            'platform_messaging.transport_profile_revision_outbox', 'SELECT') THEN
        RAISE EXCEPTION 'profile audit and Outbox rows must be emitted only by the owner trigger';
    END IF;

    IF NOT has_table_privilege('star_platform_profile_resolver',
            'platform_messaging.transport_profile_revision', 'SELECT')
       OR has_table_privilege('star_platform_profile_resolver',
            'platform_messaging.transport_profile_revision', 'INSERT')
       OR has_table_privilege('star_platform_profile_resolver',
            'platform_messaging.transport_profile_revision_audit', 'SELECT')
       OR has_table_privilege('star_platform_profile_resolver',
            'platform_messaging.transport_profile_revision_outbox', 'SELECT') THEN
        RAISE EXCEPTION 'profile resolver must have bounded master read access only';
    END IF;

    IF EXISTS (
        SELECT 1 FROM information_schema.columns AS columns_catalog
         WHERE columns_catalog.table_schema = 'platform_messaging'
           AND columns_catalog.table_name = 'transport_profile_revision_outbox'
           AND columns_catalog.column_name IN ('secret_reference','credential','private_key','token')
    ) THEN
        RAISE EXCEPTION 'profile Outbox must not contain secret references or credential material';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_trigger
         WHERE tgrelid = 'platform_messaging.transport_profile_revision'::regclass
           AND tgname = 'transport_profile_revision_successor'
           AND tgdeferrable AND tginitdeferred
    ) THEN
        RAISE EXCEPTION 'closed profile revision must require a deferred immutable successor';
    END IF;

    IF NOT EXISTS (
        SELECT 1 FROM pg_proc p
         WHERE p.oid = 'platform_messaging.append_transport_profile_revision_facts()'::regprocedure
           AND p.prosecdef
           AND p.proconfig @> ARRAY['search_path=pg_catalog']
           AND NOT EXISTS (
                SELECT 1 FROM aclexplode(p.proacl) a
                 WHERE a.grantee = 0 AND a.privilege_type = 'EXECUTE'
           )
    ) THEN
        RAISE EXCEPTION 'profile revision fact trigger must be fixed-path and non-public';
    END IF;
END;
$assertions$;
