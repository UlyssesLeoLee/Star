-- @cypher schema=1
-- CREATE (self:File {path:"2026-10-16-platform-transport-profile-event-kind.sql",type:"file",language:"sql"}),(m:Module {name:"platform_transport_profile_event_kind_migration",type:"module",language:"sql"}),(profile:Table {name:"platform_messaging.transport_profile_revision",type:"table",language:"sql"}),(allowed:Constraint {name:"ck_transport_profile_revision_allowed_message_kinds_v2",type:"constraint"});
-- CREATE (self)-[:CONTAINS]->(m),(m)-[:ALTERS]->(profile),(profile)-[:HAS_CONSTRAINT]->(allowed);
-- @endcypher

-- Add a separately authorized message kind for owner-scoped business events.
-- Existing profile digests remain byte-for-byte stable because the new kind has a new tag.
BEGIN;

ALTER TABLE platform_messaging.transport_profile_revision
    DROP CONSTRAINT IF EXISTS transport_profile_revision_allowed_message_kinds_check,
    DROP CONSTRAINT IF EXISTS ck_transport_profile_revision_allowed_message_kinds_v2;

ALTER TABLE platform_messaging.transport_profile_revision
    ADD CONSTRAINT ck_transport_profile_revision_allowed_message_kinds_v2
    CHECK (
        cardinality(allowed_message_kinds) BETWEEN 1 AND 4
        AND allowed_message_kinds <@ ARRAY['request','result','resolution','event']::TEXT[]
    );

COMMENT ON CONSTRAINT ck_transport_profile_revision_allowed_message_kinds_v2
    ON platform_messaging.transport_profile_revision IS
    'Versioned Transport Profile authorization categories; event is independent of Schedule request/result/resolution.';

COMMIT;
