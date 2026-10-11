-- @cypher schema=1 source_sha256=586cea67522b52472d2c7cfe7684552db9febe080b41f96108f4cc2e338c082b
-- MERGE (self:File {path:"db/migrations/2026-10-12-admission-transport-profile-receipts.sql"})
-- MERGE (claim_v2:Symbol {id:"db/migrations/2026-10-12-admission-transport-profile-receipts.sql::automation.claim_admission_delivery_v2",kind:"function"})
-- MERGE (settle_v2:Symbol {id:"db/migrations/2026-10-12-admission-transport-profile-receipts.sql::automation.settle_admission_delivery_v2",kind:"function"})
-- MERGE (run_claim_v2:Symbol {id:"db/migrations/2026-10-12-admission-transport-profile-receipts.sql::multica.claim_admission_delivery_v2",kind:"function"})
-- MERGE (run_settle_v2:Symbol {id:"db/migrations/2026-10-12-admission-transport-profile-receipts.sql::multica.settle_admission_delivery_v2",kind:"function"})
-- MERGE (schedule_work:Table {id:"automation.admission_message_delivery"})
-- MERGE (schedule_receipt:Table {id:"automation.admission_message_delivery_receipt"})
-- MERGE (run_work:Table {id:"multica.admission_message_delivery"})
-- MERGE (run_receipt:Table {id:"multica.admission_message_delivery_receipt"})
-- MERGE (self)-[:DEFINES]->(claim_v2)
-- MERGE (self)-[:DEFINES]->(settle_v2)
-- MERGE (self)-[:DEFINES]->(run_claim_v2)
-- MERGE (self)-[:DEFINES]->(run_settle_v2)
-- MERGE (claim_v2)-[:WRITES]->(schedule_work)
-- MERGE (claim_v2)-[:READS]->(schedule_source:View {id:"automation.admission_message_source"})
-- MERGE (settle_v2)-[:WRITES]->(schedule_work)
-- MERGE (settle_v2)-[:WRITES]->(schedule_receipt)
-- MERGE (run_claim_v2)-[:WRITES]->(run_work)
-- MERGE (run_claim_v2)-[:READS]->(run_source:View {id:"multica.admission_message_source"})
-- MERGE (run_settle_v2)-[:WRITES]->(run_work)
-- MERGE (run_settle_v2)-[:WRITES]->(run_receipt)
-- @endcypher
-- Provider-neutral, profile-pinned delivery receipts. The old broker columns are retained
-- only as legacy evidence; old claim/settle entry points are removed to prevent unpinned writes.
BEGIN;
DO $migration$
DECLARE
    s TEXT;
    c RECORD;
    table_oid REGCLASS;
BEGIN
    FOREACH s IN ARRAY ARRAY['automation','multica'] LOOP
        table_oid := format('%I.admission_message_delivery_receipt', s)::REGCLASS;
        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema=s AND table_name='admission_message_delivery_receipt'
              AND column_name='broker_stream'
        ) THEN
            EXECUTE format('ALTER TABLE %I.admission_message_delivery_receipt RENAME COLUMN broker_stream TO legacy_broker_stream', s);
        END IF;
        IF EXISTS (
            SELECT 1 FROM information_schema.columns
            WHERE table_schema=s AND table_name='admission_message_delivery_receipt'
              AND column_name='broker_sequence'
        ) THEN
            EXECUTE format('ALTER TABLE %I.admission_message_delivery_receipt RENAME COLUMN broker_sequence TO legacy_broker_sequence', s);
        END IF;

        EXECUTE format($ddl$
            ALTER TABLE %1$I.admission_message_delivery
                ADD COLUMN IF NOT EXISTS transport_provider_id TEXT,
                ADD COLUMN IF NOT EXISTS transport_provider_version TEXT,
                ADD COLUMN IF NOT EXISTS transport_profile_digest BYTEA,
                ADD COLUMN IF NOT EXISTS transport_endpoint_identity TEXT
        $ddl$, s);
        EXECUTE format($ddl$
            ALTER TABLE %1$I.admission_message_delivery_receipt
                ADD COLUMN IF NOT EXISTS receipt_version SMALLINT NOT NULL DEFAULT 1,
                ADD COLUMN IF NOT EXISTS transport_provider_id TEXT,
                ADD COLUMN IF NOT EXISTS transport_provider_version TEXT,
                ADD COLUMN IF NOT EXISTS transport_profile_digest BYTEA,
                ADD COLUMN IF NOT EXISTS transport_endpoint_identity TEXT,
                ADD COLUMN IF NOT EXISTS transport_log_id TEXT,
                ADD COLUMN IF NOT EXISTS transport_log_generation UUID,
                ADD COLUMN IF NOT EXISTS transport_cursor BYTEA,
                ADD COLUMN IF NOT EXISTS payload_sha256 BYTEA
        $ddl$, s);

        -- Replace only the old delivered-vs-broker-pair constraint; keep per-column legacy checks.
        FOR c IN
            SELECT conname FROM pg_constraint
            WHERE conrelid=table_oid AND contype='c'
              AND pg_get_constraintdef(oid) LIKE '%legacy_broker_stream%'
              AND pg_get_constraintdef(oid) LIKE '%legacy_broker_sequence%'
        LOOP
            EXECUTE format('ALTER TABLE %I.admission_message_delivery_receipt DROP CONSTRAINT %I', s, c.conname);
        END LOOP;
        FOR c IN
            SELECT conname FROM pg_constraint
            WHERE conrelid=table_oid AND contype='c'
              AND pg_get_constraintdef(oid) LIKE '%terminal_reason%'
              AND pg_get_constraintdef(oid) LIKE '%broker_ack%'
        LOOP
            EXECUTE format('ALTER TABLE %I.admission_message_delivery_receipt DROP CONSTRAINT %I', s, c.conname);
        END LOOP;

        IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid=format('%I.admission_message_delivery',s)::REGCLASS AND conname='admission_message_transport_profile_ck') THEN
            EXECUTE format($ddl$
                ALTER TABLE %1$I.admission_message_delivery ADD CONSTRAINT admission_message_transport_profile_ck CHECK (
                    (transport_provider_id IS NULL AND transport_provider_version IS NULL AND transport_profile_digest IS NULL AND transport_endpoint_identity IS NULL)
                    OR
                    (transport_provider_id IS NOT NULL AND transport_provider_id ~ '^[a-z][a-z0-9-]{0,63}$'
                     AND transport_provider_version IS NOT NULL
                     AND transport_provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'
                     AND transport_profile_digest IS NOT NULL
                     AND octet_length(transport_profile_digest)=32
                     AND transport_endpoint_identity IS NOT NULL
                     AND transport_endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$')
                )
            $ddl$,s);
        END IF;
        IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid=table_oid AND conname='admission_message_receipt_version_ck') THEN
            EXECUTE format('ALTER TABLE %I.admission_message_delivery_receipt ADD CONSTRAINT admission_message_receipt_version_ck CHECK(receipt_version IN (1,2))',s);
        END IF;
        IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid=table_oid AND conname='admission_message_receipt_transport_ck') THEN
            EXECUTE format($ddl$
                ALTER TABLE %1$I.admission_message_delivery_receipt ADD CONSTRAINT admission_message_receipt_transport_ck CHECK (
                    (outcome='dead' AND receipt_version=1
                     AND legacy_broker_stream IS NULL AND legacy_broker_sequence IS NULL
                     AND transport_provider_id IS NULL AND transport_provider_version IS NULL
                     AND transport_profile_digest IS NULL AND transport_endpoint_identity IS NULL
                     AND transport_log_id IS NULL AND transport_log_generation IS NULL
                     AND transport_cursor IS NULL AND payload_sha256 IS NULL)
                    OR
                    (outcome='delivered' AND (
                        (receipt_version=1 AND legacy_broker_stream IS NOT NULL AND legacy_broker_sequence IS NOT NULL
                         AND transport_provider_id IS NULL AND transport_provider_version IS NULL
                         AND transport_profile_digest IS NULL AND transport_endpoint_identity IS NULL
                         AND transport_log_id IS NULL AND transport_log_generation IS NULL
                         AND transport_cursor IS NULL AND payload_sha256 IS NULL)
                        OR
                        (receipt_version=2 AND legacy_broker_stream IS NULL AND legacy_broker_sequence IS NULL
                         AND transport_provider_id IS NOT NULL AND transport_provider_id ~ '^[a-z][a-z0-9-]{0,63}$'
                         AND transport_provider_version IS NOT NULL
                         AND transport_provider_version ~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'
                         AND transport_profile_digest IS NOT NULL
                         AND octet_length(transport_profile_digest)=32
                         AND transport_endpoint_identity IS NOT NULL
                         AND transport_endpoint_identity ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
                         AND transport_log_id IS NOT NULL
                         AND transport_log_id ~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
                         AND transport_log_generation IS NOT NULL
                         AND transport_log_generation<>'00000000-0000-0000-0000-000000000000'::UUID
                         AND transport_cursor IS NOT NULL
                         AND octet_length(transport_cursor) BETWEEN 1 AND 128
                         AND payload_sha256 IS NOT NULL
                         AND octet_length(payload_sha256)=32)
                    ))
                )
            $ddl$,s);
        END IF;
        IF NOT EXISTS (SELECT 1 FROM pg_constraint WHERE conrelid=table_oid AND conname='admission_message_receipt_terminal_reason_v2_ck') THEN
            EXECUTE format($ddl$
                ALTER TABLE %1$I.admission_message_delivery_receipt
                ADD CONSTRAINT admission_message_receipt_terminal_reason_v2_ck
                CHECK(terminal_reason IN ('broker_ack','durable_acceptance','invalid_wire','attempts_exhausted','retention_elapsed'))
            $ddl$,s);
        END IF;

        -- Old clients cannot bypass profile pinning by calling the unversioned procedures.
        EXECUTE format('DROP FUNCTION IF EXISTS %I.claim_admission_delivery(UUID,INTEGER,INTEGER)', s);
        EXECUTE format('DROP FUNCTION IF EXISTS %I.settle_admission_delivery(UUID,TEXT,UUID,INTEGER,TEXT,TEXT,BIGINT)', s);

        EXECUTE format($ddl$
            CREATE OR REPLACE FUNCTION %1$I.claim_admission_delivery_v2(
                p_worker UUID,p_limit INTEGER,p_lease INTEGER,
                p_provider_id TEXT,p_provider_version TEXT,p_profile_digest BYTEA,p_endpoint_identity TEXT
            ) RETURNS TABLE(
                message_id UUID,request_id UUID,message_kind TEXT,target_service_id UUID,
                wire BYTEA,attempt_count INTEGER,lease_expires_at TIMESTAMPTZ,
                transport_provider_id TEXT,transport_provider_version TEXT,
                transport_profile_digest BYTEA,transport_endpoint_identity TEXT
            ) LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $body$
            DECLARE tenant UUID:=NULLIF(current_setting('app.tenant_id',true),'')::UUID;
            BEGIN
                IF tenant IS NULL OR tenant='00000000-0000-0000-0000-000000000000'::UUID
                   OR p_worker IS NULL OR p_worker='00000000-0000-0000-0000-000000000000'::UUID
                   OR p_limit IS NULL OR p_limit NOT BETWEEN 1 AND 8 OR p_lease IS NULL OR p_lease NOT BETWEEN 30 AND 300
                   OR p_provider_id IS NULL OR p_provider_id !~ '^[a-z][a-z0-9-]{0,63}$'
                   OR p_provider_version IS NULL OR p_provider_version !~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'
                   OR p_profile_digest IS NULL OR octet_length(p_profile_digest)<>32
                   OR p_endpoint_identity IS NULL OR p_endpoint_identity !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
                THEN RAISE EXCEPTION 'Invalid profile-pinned delivery claim boundary'; END IF;

                WITH expired AS MATERIALIZED (
                    SELECT d.* FROM %1$I.admission_message_delivery d WHERE d.tenant_id=tenant
                      AND (d.expires_at<=clock_timestamp()
                           OR (d.state='leased' AND d.lease_expires_at<=clock_timestamp() AND d.attempt_count>=12))
                    ORDER BY d.expires_at,d.message_id LIMIT p_limit FOR UPDATE SKIP LOCKED
                ), audited AS (
                    INSERT INTO %1$I.admission_message_delivery_receipt AS receipt(
                        tenant_id,message_id,request_id,message_kind,outcome,attempts,terminal_reason)
                    SELECT e.tenant_id,e.message_id,e.request_id,e.message_kind,'dead',e.attempt_count,
                        CASE WHEN e.expires_at<=clock_timestamp() THEN 'retention_elapsed' ELSE 'attempts_exhausted' END
                    FROM expired e RETURNING receipt.tenant_id,receipt.message_kind,receipt.message_id
                ) DELETE FROM %1$I.admission_message_delivery d USING audited a
                    WHERE d.tenant_id=a.tenant_id AND d.message_kind=a.message_kind AND d.message_id=a.message_id;

                RETURN QUERY WITH due AS MATERIALIZED (
                    SELECT d.tenant_id,d.message_kind,d.message_id FROM %1$I.admission_message_delivery d
                    WHERE d.tenant_id=tenant AND d.expires_at>clock_timestamp() AND d.attempt_count<12
                      AND (d.transport_provider_id IS NULL OR
                           (d.transport_provider_id=p_provider_id
                            AND d.transport_provider_version=p_provider_version
                            AND d.transport_profile_digest=p_profile_digest
                            AND d.transport_endpoint_identity=p_endpoint_identity))
                      AND ((d.state IN ('pending','retry') AND d.next_attempt_at<=clock_timestamp())
                           OR (d.state='leased' AND d.lease_expires_at<=clock_timestamp()))
                    ORDER BY d.next_attempt_at,d.message_id LIMIT p_limit FOR UPDATE SKIP LOCKED
                ), claimed AS (
                    UPDATE %1$I.admission_message_delivery d SET state='leased',attempt_count=d.attempt_count+1,
                        lease_owner_id=p_worker,lease_expires_at=LEAST(clock_timestamp()+p_lease*interval '1 second',d.expires_at),
                        transport_provider_id=COALESCE(d.transport_provider_id,p_provider_id),
                        transport_provider_version=COALESCE(d.transport_provider_version,p_provider_version),
                        transport_profile_digest=COALESCE(d.transport_profile_digest,p_profile_digest),
                        transport_endpoint_identity=COALESCE(d.transport_endpoint_identity,p_endpoint_identity)
                    FROM due q WHERE d.tenant_id=q.tenant_id AND d.message_kind=q.message_kind AND d.message_id=q.message_id
                    RETURNING d.*
                ) SELECT c.message_id,c.request_id,c.message_kind,v.target_service_id,v.wire,c.attempt_count,c.lease_expires_at,
                         c.transport_provider_id,c.transport_provider_version,c.transport_profile_digest,c.transport_endpoint_identity
                    FROM claimed c JOIN %1$I.admission_message_source v
                      ON v.tenant_id=c.tenant_id AND v.message_id=c.message_id AND v.message_kind=c.message_kind;
            END $body$;
            REVOKE ALL ON FUNCTION %1$I.claim_admission_delivery_v2(UUID,INTEGER,INTEGER,TEXT,TEXT,BYTEA,TEXT) FROM PUBLIC;
            GRANT EXECUTE ON FUNCTION %1$I.claim_admission_delivery_v2(UUID,INTEGER,INTEGER,TEXT,TEXT,BYTEA,TEXT) TO %2$I;

            CREATE OR REPLACE FUNCTION %1$I.settle_admission_delivery_v2(
                p_message UUID,p_kind TEXT,p_worker UUID,p_attempt INTEGER,p_action TEXT,
                p_provider_id TEXT,p_provider_version TEXT,p_profile_digest BYTEA,p_endpoint_identity TEXT,
                p_log_id TEXT,p_log_generation UUID,p_cursor BYTEA,p_payload_sha256 BYTEA
            ) RETURNS VOID LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $body$
            DECLARE tenant UUID:=NULLIF(current_setting('app.tenant_id',true),'')::UUID;
                d %1$I.admission_message_delivery%%ROWTYPE;
            BEGIN
                IF tenant IS NULL OR p_action IS NULL OR p_action NOT IN ('delivered','retry','dead')
                THEN RAISE EXCEPTION 'Invalid delivery settlement'; END IF;
                SELECT * INTO STRICT d FROM %1$I.admission_message_delivery q
                    WHERE q.tenant_id=tenant AND q.message_kind=p_kind AND q.message_id=p_message FOR UPDATE;
                IF d.state<>'leased' OR d.lease_owner_id IS DISTINCT FROM p_worker OR d.attempt_count IS DISTINCT FROM p_attempt
                   OR d.lease_expires_at<=clock_timestamp() OR d.expires_at<=clock_timestamp()
                THEN RAISE EXCEPTION 'Delivery fence lost'; END IF;
                IF p_action='retry' AND d.attempt_count<12 THEN
                    IF p_provider_id IS NOT NULL OR p_provider_version IS NOT NULL OR p_profile_digest IS NOT NULL
                       OR p_endpoint_identity IS NOT NULL OR p_log_id IS NOT NULL OR p_log_generation IS NOT NULL
                       OR p_cursor IS NOT NULL OR p_payload_sha256 IS NOT NULL
                    THEN RAISE EXCEPTION 'Retry cannot carry a terminal transport receipt'; END IF;
                    UPDATE %1$I.admission_message_delivery q SET state='retry',lease_owner_id=NULL,lease_expires_at=NULL,
                        next_attempt_at=LEAST(clock_timestamp()+LEAST(300,power(2,d.attempt_count)::INTEGER)*interval '1 second',q.expires_at)
                    WHERE q.tenant_id=tenant AND q.message_kind=p_kind AND q.message_id=p_message;
                    RETURN;
                END IF;
                IF p_action='delivered' THEN
                    IF p_provider_id IS DISTINCT FROM d.transport_provider_id
                       OR p_provider_version IS DISTINCT FROM d.transport_provider_version
                       OR p_profile_digest IS DISTINCT FROM d.transport_profile_digest
                       OR p_endpoint_identity IS DISTINCT FROM d.transport_endpoint_identity
                       OR p_provider_id IS NULL OR p_provider_id !~ '^[a-z][a-z0-9-]{0,63}$'
                       OR p_provider_version IS NULL OR p_provider_version !~ '^[A-Za-z0-9][A-Za-z0-9._+-]{0,63}$'
                       OR p_profile_digest IS NULL OR octet_length(p_profile_digest)<>32
                       OR p_endpoint_identity IS NULL OR p_endpoint_identity !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
                       OR p_log_id IS NULL OR p_log_id !~ '^[A-Za-z0-9][A-Za-z0-9._:-]{0,127}$'
                       OR p_log_generation IS NULL OR p_log_generation='00000000-0000-0000-0000-000000000000'::UUID
                       OR p_cursor IS NULL OR octet_length(p_cursor) NOT BETWEEN 1 AND 128
                       OR p_payload_sha256 IS NULL OR octet_length(p_payload_sha256)<>32
                    THEN RAISE EXCEPTION 'Delivery requires a matching versioned durable transport proof'; END IF;
                    INSERT INTO %1$I.admission_message_delivery_receipt(
                        tenant_id,message_id,request_id,message_kind,outcome,attempts,terminal_reason,receipt_version,
                        transport_provider_id,transport_provider_version,transport_profile_digest,transport_endpoint_identity,
                        transport_log_id,transport_log_generation,transport_cursor,payload_sha256)
                    VALUES(tenant,d.message_id,d.request_id,d.message_kind,'delivered',d.attempt_count,'durable_acceptance',2,
                        p_provider_id,p_provider_version,p_profile_digest,p_endpoint_identity,p_log_id,p_log_generation,p_cursor,p_payload_sha256);
                ELSE
                    IF p_provider_id IS NOT NULL OR p_provider_version IS NOT NULL OR p_profile_digest IS NOT NULL
                       OR p_endpoint_identity IS NOT NULL OR p_log_id IS NOT NULL OR p_log_generation IS NOT NULL
                       OR p_cursor IS NOT NULL OR p_payload_sha256 IS NOT NULL
                    THEN RAISE EXCEPTION 'Dead settlement cannot carry a transport receipt'; END IF;
                    INSERT INTO %1$I.admission_message_delivery_receipt(
                        tenant_id,message_id,request_id,message_kind,outcome,attempts,terminal_reason)
                    VALUES(tenant,d.message_id,d.request_id,d.message_kind,'dead',d.attempt_count,
                        CASE WHEN p_action='dead' THEN 'invalid_wire' ELSE 'attempts_exhausted' END);
                END IF;
                DELETE FROM %1$I.admission_message_delivery q
                    WHERE q.tenant_id=tenant AND q.message_kind=p_kind AND q.message_id=p_message;
                IF d.lease_expires_at<=clock_timestamp() THEN RAISE EXCEPTION 'Delivery fence expired during settlement'; END IF;
            END $body$;
            REVOKE ALL ON FUNCTION %1$I.settle_admission_delivery_v2(UUID,TEXT,UUID,INTEGER,TEXT,TEXT,TEXT,BYTEA,TEXT,TEXT,UUID,BYTEA,BYTEA) FROM PUBLIC;
            GRANT EXECUTE ON FUNCTION %1$I.settle_admission_delivery_v2(UUID,TEXT,UUID,INTEGER,TEXT,TEXT,TEXT,BYTEA,TEXT,TEXT,UUID,BYTEA,BYTEA) TO %2$I;
        $ddl$,s,CASE s WHEN 'automation' THEN 'star_automation_schedule_owner' ELSE 'star_multica_run_owner' END);
    END LOOP;
END
$migration$;
COMMIT;
