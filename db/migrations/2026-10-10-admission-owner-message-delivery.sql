-- Each owner manages only its own delivery Work queue and immutable Transaction receipts.
-- @cypher schema=1
-- MERGE (self:File {path:"db/migrations/2026-10-10-admission-owner-message-delivery.sql"})
-- MERGE (schedule_queue:Table {id:"automation.admission_message_delivery"})
-- MERGE (run_queue:Table {id:"multica.admission_message_delivery"})
-- MERGE (schedule_receipt:Table {id:"automation.admission_message_delivery_receipt"})
-- MERGE (run_receipt:Table {id:"multica.admission_message_delivery_receipt"})
-- MERGE (self)-[:DEFINES]->(schedule_queue)
-- MERGE (self)-[:DEFINES]->(run_queue)
-- MERGE (self)-[:DEFINES]->(schedule_receipt)
-- MERGE (self)-[:DEFINES]->(run_receipt)
-- MERGE (schedule_enqueue:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::automation.enqueue_admission_delivery",kind:"function"})
-- MERGE (run_enqueue:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::multica.enqueue_admission_delivery",kind:"function"})
-- MERGE (schedule_claim:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::automation.claim_admission_delivery",kind:"function"})
-- MERGE (run_claim:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::multica.claim_admission_delivery",kind:"function"})
-- MERGE (schedule_settle:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::automation.settle_admission_delivery",kind:"function"})
-- MERGE (run_settle:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::multica.settle_admission_delivery",kind:"function"})
-- MERGE (self)-[:DEFINES]->(schedule_enqueue)
-- MERGE (self)-[:DEFINES]->(run_enqueue)
-- MERGE (self)-[:DEFINES]->(schedule_claim)
-- MERGE (self)-[:DEFINES]->(run_claim)
-- MERGE (self)-[:DEFINES]->(schedule_settle)
-- MERGE (self)-[:DEFINES]->(run_settle)
-- MERGE (schedule_enqueue)-[:WRITES]->(schedule_queue)
-- MERGE (run_enqueue)-[:WRITES]->(run_queue)
-- MERGE (schedule_claim)-[:WRITES]->(schedule_queue)
-- MERGE (run_claim)-[:WRITES]->(run_queue)
-- MERGE (schedule_claim)-[:WRITES]->(schedule_receipt)
-- MERGE (run_claim)-[:WRITES]->(run_receipt)
-- MERGE (schedule_settle)-[:WRITES]->(schedule_receipt)
-- MERGE (run_settle)-[:WRITES]->(run_receipt)
-- MERGE (schedule_settle)-[:WRITES]->(schedule_queue)
-- MERGE (run_settle)-[:WRITES]->(run_queue)
-- MERGE (schedule_reconcile:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::automation.reconcile_admission_delivery",kind:"function"})
-- MERGE (run_reconcile:Symbol {id:"db/migrations/2026-10-10-admission-owner-message-delivery.sql::multica.reconcile_admission_delivery",kind:"function"})
-- MERGE (self)-[:DEFINES]->(schedule_reconcile)
-- MERGE (self)-[:DEFINES]->(run_reconcile)
-- MERGE (schedule_reconcile)-[:WRITES]->(schedule_queue)
-- MERGE (run_reconcile)-[:WRITES]->(run_queue)
-- MERGE (schedule_source:View {id:"automation.admission_message_source"})
-- MERGE (run_source:View {id:"multica.admission_message_source"})
-- MERGE (self)-[:DEFINES]->(schedule_source)
-- MERGE (self)-[:DEFINES]->(run_source)
-- MERGE (schedule_claim)-[:READS]->(schedule_source)
-- MERGE (run_claim)-[:READS]->(run_source)
-- MERGE (schedule_reconcile)-[:READS]->(schedule_source)
-- MERGE (run_reconcile)-[:READS]->(run_source)
-- @endcypher
BEGIN;
CREATE OR REPLACE VIEW automation.admission_message_source AS
 SELECT q.tenant_id,q.request_id AS message_id,q.request_id,'request'::TEXT AS message_kind,
 q.target_run_service_id AS target_service_id,q.command_wire AS wire,q.recorded_at
 FROM automation.run_admission_request_outbox q
 UNION ALL
 SELECT x.tenant_id,x.resolution_id,x.request_id,'resolution',q.target_run_service_id,x.resolution_wire,x.recorded_at
 FROM automation.run_admission_resolution_outbox x JOIN automation.run_admission_request_outbox q USING(tenant_id,request_id);
CREATE OR REPLACE VIEW multica.admission_message_source AS
 SELECT o.tenant_id,o.result_id AS message_id,o.request_id,'result'::TEXT AS message_kind,
 i.source_schedule_service_id AS target_service_id,o.result_wire AS wire,o.recorded_at
 FROM multica.schedule_admission_result_outbox o JOIN multica.schedule_admission_inbox i USING(tenant_id,request_id,result_id);
REVOKE ALL ON automation.admission_message_source,multica.admission_message_source FROM PUBLIC;
DO $migration$
DECLARE s TEXT; owner_role TEXT; source_table TEXT; identity_column TEXT; kind TEXT;
BEGIN
 FOREACH s IN ARRAY ARRAY['automation','multica'] LOOP
  owner_role:=CASE s WHEN 'automation' THEN 'star_automation_schedule_owner' ELSE 'star_multica_run_owner' END;
  EXECUTE format($ddl$
   CREATE TABLE IF NOT EXISTS %1$I.admission_message_delivery (
    tenant_id UUID NOT NULL,message_id UUID NOT NULL,request_id UUID NOT NULL,
    message_kind TEXT NOT NULL CHECK(message_kind IN ('request','resolution','result')),
    state TEXT NOT NULL DEFAULT 'pending' CHECK(state IN ('pending','leased','retry')),
    attempt_count INTEGER NOT NULL DEFAULT 0 CHECK(attempt_count BETWEEN 0 AND 12),
    lease_owner_id UUID,lease_expires_at TIMESTAMPTZ,next_attempt_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),
    created_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp(),
    retention_period INTERVAL NOT NULL DEFAULT interval '7 days' CHECK(retention_period=interval '7 days'),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT statement_timestamp()+interval '7 days',
    PRIMARY KEY(tenant_id,message_kind,message_id),
    CHECK((state='leased')=(lease_owner_id IS NOT NULL AND lease_expires_at IS NOT NULL)),
    CHECK(expires_at=created_at+retention_period),CHECK(lease_expires_at IS NULL OR lease_expires_at<=expires_at)
   );
   CREATE INDEX IF NOT EXISTS admission_message_due ON %1$I.admission_message_delivery(tenant_id,next_attempt_at,message_id);
   CREATE TABLE IF NOT EXISTS %1$I.admission_message_delivery_receipt (
    tenant_id UUID NOT NULL,message_id UUID NOT NULL,request_id UUID NOT NULL,message_kind TEXT NOT NULL,
    outcome TEXT NOT NULL CHECK(outcome IN ('delivered','dead')),
    attempts INTEGER NOT NULL CHECK(attempts BETWEEN 0 AND 12),
    terminal_reason TEXT NOT NULL CHECK(terminal_reason IN ('broker_ack','invalid_wire','attempts_exhausted','retention_elapsed')),
    broker_stream TEXT,broker_sequence BIGINT,
    recorded_at TIMESTAMPTZ NOT NULL DEFAULT clock_timestamp(),PRIMARY KEY(tenant_id,message_kind,message_id),
    CHECK((outcome='delivered')=(broker_stream IS NOT NULL AND broker_sequence IS NOT NULL)),
    CHECK(broker_stream IS NULL OR length(broker_stream) BETWEEN 1 AND 128),
    CHECK(broker_sequence IS NULL OR broker_sequence>0)
   );
   ALTER TABLE %1$I.admission_message_delivery ENABLE ROW LEVEL SECURITY;
   ALTER TABLE %1$I.admission_message_delivery FORCE ROW LEVEL SECURITY;
   ALTER TABLE %1$I.admission_message_delivery_receipt ENABLE ROW LEVEL SECURITY;
   ALTER TABLE %1$I.admission_message_delivery_receipt FORCE ROW LEVEL SECURITY;
   DROP POLICY IF EXISTS delivery_tenant ON %1$I.admission_message_delivery;
   CREATE POLICY delivery_tenant ON %1$I.admission_message_delivery USING(tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID);
   DROP POLICY IF EXISTS delivery_tenant ON %1$I.admission_message_delivery_receipt;
   CREATE POLICY delivery_tenant ON %1$I.admission_message_delivery_receipt USING(tenant_id=NULLIF(current_setting('app.tenant_id',true),'')::UUID);
   REVOKE ALL ON %1$I.admission_message_delivery,%1$I.admission_message_delivery_receipt FROM PUBLIC;
   GRANT SELECT ON %1$I.admission_message_delivery,%1$I.admission_message_delivery_receipt TO %2$I;
   DROP TRIGGER IF EXISTS delivery_receipt_worm ON %1$I.admission_message_delivery_receipt;
   CREATE TRIGGER delivery_receipt_worm BEFORE UPDATE OR DELETE OR TRUNCATE ON %1$I.admission_message_delivery_receipt
   FOR EACH STATEMENT EXECUTE FUNCTION automation.reject_schedule_history_mutation();
   COMMENT ON TABLE %1$I.admission_message_delivery IS 'Work: bounded per-tenant delivery queue; 7-day hard retention; audited dead/delivered receipt before physical removal.';
   COMMENT ON TABLE %1$I.admission_message_delivery_receipt IS 'Transaction: immutable terminal delivery receipt; no TTL/deletion; tenant FORCE RLS; broker PubAck is not consumer/execution completion.';
  $ddl$,s,owner_role);
  EXECUTE format($ddl$
   CREATE OR REPLACE FUNCTION %1$I.enqueue_admission_delivery() RETURNS trigger
   LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $body$
   DECLARE identity UUID; BEGIN
    identity:=(to_jsonb(NEW)->>TG_ARGV[1])::UUID;
    PERFORM pg_advisory_xact_lock(hashtextextended('admission-delivery:%1$s:'||NEW.tenant_id::TEXT,0));
    IF EXISTS(SELECT 1 FROM %1$I.admission_message_delivery_receipt WHERE tenant_id=NEW.tenant_id AND message_kind=TG_ARGV[0] AND message_id=identity)
    THEN RETURN NEW; END IF;
    IF (SELECT count(*) FROM %1$I.admission_message_delivery WHERE tenant_id=NEW.tenant_id)>=1000
    THEN RAISE EXCEPTION 'Admission owner delivery queue budget exceeded'; END IF;
    INSERT INTO %1$I.admission_message_delivery(tenant_id,message_id,request_id,message_kind)
    VALUES(NEW.tenant_id,identity,NEW.request_id,TG_ARGV[0]); RETURN NEW;
   END $body$;
   REVOKE ALL ON FUNCTION %1$I.enqueue_admission_delivery() FROM PUBLIC;
   CREATE OR REPLACE FUNCTION %1$I.claim_admission_delivery(p_worker UUID,p_limit INTEGER,p_lease INTEGER)
   RETURNS TABLE(message_id UUID,request_id UUID,message_kind TEXT,target_service_id UUID,wire BYTEA,attempt_count INTEGER,lease_expires_at TIMESTAMPTZ)
   LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $body$
   DECLARE tenant UUID:=NULLIF(current_setting('app.tenant_id',true),'')::UUID; BEGIN
    IF tenant IS NULL OR tenant='00000000-0000-0000-0000-000000000000'::UUID OR p_worker IS NULL
    OR p_worker='00000000-0000-0000-0000-000000000000'::UUID OR p_limit IS NULL OR p_limit NOT BETWEEN 1 AND 8
    OR p_lease IS NULL OR p_lease NOT BETWEEN 30 AND 300 THEN RAISE EXCEPTION 'Invalid delivery claim boundary'; END IF;
    WITH expired AS MATERIALIZED (
     SELECT d.* FROM %1$I.admission_message_delivery d WHERE d.tenant_id=tenant
     AND (d.expires_at<=clock_timestamp() OR (d.state='leased' AND d.lease_expires_at<=clock_timestamp() AND d.attempt_count>=12))
     ORDER BY d.expires_at,d.message_id LIMIT p_limit FOR UPDATE SKIP LOCKED
    ), audited AS (
     INSERT INTO %1$I.admission_message_delivery_receipt AS receipt(tenant_id,message_id,request_id,message_kind,outcome,attempts,terminal_reason)
     SELECT e.tenant_id,e.message_id,e.request_id,e.message_kind,'dead',e.attempt_count,
     CASE WHEN e.expires_at<=clock_timestamp() THEN 'retention_elapsed' ELSE 'attempts_exhausted' END FROM expired e RETURNING receipt.tenant_id,receipt.message_kind,receipt.message_id
    ) DELETE FROM %1$I.admission_message_delivery d USING audited a WHERE d.tenant_id=a.tenant_id AND d.message_kind=a.message_kind AND d.message_id=a.message_id;
    RETURN QUERY WITH due AS MATERIALIZED (
     SELECT d.tenant_id,d.message_kind,d.message_id FROM %1$I.admission_message_delivery d WHERE d.tenant_id=tenant
     AND d.expires_at>clock_timestamp() AND d.attempt_count<12
     AND ((d.state IN ('pending','retry') AND d.next_attempt_at<=clock_timestamp()) OR (d.state='leased' AND d.lease_expires_at<=clock_timestamp()))
     ORDER BY d.next_attempt_at,d.message_id LIMIT p_limit FOR UPDATE SKIP LOCKED
    ), claimed AS (
     UPDATE %1$I.admission_message_delivery d SET state='leased',attempt_count=d.attempt_count+1,lease_owner_id=p_worker,
      lease_expires_at=LEAST(clock_timestamp()+p_lease*interval '1 second',d.expires_at)
     FROM due q WHERE d.tenant_id=q.tenant_id AND d.message_kind=q.message_kind AND d.message_id=q.message_id RETURNING d.*
    ) SELECT c.message_id,c.request_id,c.message_kind,v.target_service_id,v.wire,c.attempt_count,c.lease_expires_at
      FROM claimed c JOIN %1$I.admission_message_source v ON v.tenant_id=c.tenant_id AND v.message_id=c.message_id AND v.message_kind=c.message_kind;
   END $body$;
   REVOKE ALL ON FUNCTION %1$I.claim_admission_delivery(UUID,INTEGER,INTEGER) FROM PUBLIC;
   GRANT EXECUTE ON FUNCTION %1$I.claim_admission_delivery(UUID,INTEGER,INTEGER) TO %2$I;
   CREATE OR REPLACE FUNCTION %1$I.settle_admission_delivery(p_message UUID,p_kind TEXT,p_worker UUID,p_attempt INTEGER,p_action TEXT,p_stream TEXT,p_sequence BIGINT)
   RETURNS VOID LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $body$
   DECLARE tenant UUID:=NULLIF(current_setting('app.tenant_id',true),'')::UUID;d %1$I.admission_message_delivery%%ROWTYPE;
   BEGIN
    IF tenant IS NULL OR p_action IS NULL OR p_action NOT IN ('delivered','retry','dead') THEN RAISE EXCEPTION 'Invalid delivery settlement'; END IF;
    SELECT * INTO STRICT d FROM %1$I.admission_message_delivery q WHERE q.tenant_id=tenant AND q.message_kind=p_kind AND q.message_id=p_message FOR UPDATE;
    IF d.state<>'leased' OR d.lease_owner_id IS DISTINCT FROM p_worker OR d.attempt_count IS DISTINCT FROM p_attempt
    OR d.lease_expires_at<=clock_timestamp() OR d.expires_at<=clock_timestamp() THEN RAISE EXCEPTION 'Delivery fence lost'; END IF;
    IF p_action='retry' AND d.attempt_count<12 THEN
     UPDATE %1$I.admission_message_delivery q SET state='retry',lease_owner_id=NULL,lease_expires_at=NULL,
      next_attempt_at=LEAST(clock_timestamp()+LEAST(300,power(2,d.attempt_count)::INTEGER)*interval '1 second',q.expires_at)
     WHERE q.tenant_id=tenant AND q.message_kind=p_kind AND q.message_id=p_message; RETURN;
    END IF;
    IF p_action='delivered' AND (p_stream IS NULL OR length(p_stream) NOT BETWEEN 1 AND 128 OR p_sequence IS NULL OR p_sequence<=0)
    THEN RAISE EXCEPTION 'Delivery requires a positive broker receipt'; END IF;
    INSERT INTO %1$I.admission_message_delivery_receipt(tenant_id,message_id,request_id,message_kind,outcome,attempts,terminal_reason,broker_stream,broker_sequence)
    VALUES(tenant,d.message_id,d.request_id,d.message_kind,CASE WHEN p_action='delivered' THEN 'delivered' ELSE 'dead' END,d.attempt_count,
     CASE WHEN p_action='delivered' THEN 'broker_ack' WHEN p_action='dead' THEN 'invalid_wire' ELSE 'attempts_exhausted' END,
     CASE WHEN p_action='delivered' THEN p_stream END,CASE WHEN p_action='delivered' THEN p_sequence END);
    DELETE FROM %1$I.admission_message_delivery q WHERE q.tenant_id=tenant AND q.message_kind=p_kind AND q.message_id=p_message;
    IF d.lease_expires_at<=clock_timestamp() THEN RAISE EXCEPTION 'Delivery fence expired during settlement'; END IF;
   END $body$;
   REVOKE ALL ON FUNCTION %1$I.settle_admission_delivery(UUID,TEXT,UUID,INTEGER,TEXT,TEXT,BIGINT) FROM PUBLIC;
   GRANT EXECUTE ON FUNCTION %1$I.settle_admission_delivery(UUID,TEXT,UUID,INTEGER,TEXT,TEXT,BIGINT) TO %2$I;
   CREATE OR REPLACE FUNCTION %1$I.reconcile_admission_delivery(p_limit INTEGER) RETURNS INTEGER
   LANGUAGE plpgsql SECURITY DEFINER SET search_path=pg_catalog AS $body$
   DECLARE tenant UUID:=NULLIF(current_setting('app.tenant_id',true),'')::UUID;available INTEGER;added INTEGER; BEGIN
    IF tenant IS NULL OR tenant='00000000-0000-0000-0000-000000000000'::UUID OR p_limit IS NULL OR p_limit NOT BETWEEN 1 AND 8
    THEN RAISE EXCEPTION 'Invalid delivery reconciliation boundary'; END IF;
    PERFORM pg_advisory_xact_lock(hashtextextended('admission-delivery:%1$s:'||tenant::TEXT,0));
    SELECT 1000-count(*) INTO available FROM %1$I.admission_message_delivery d WHERE d.tenant_id=tenant;
    IF available<=0 THEN RAISE EXCEPTION 'Admission owner delivery queue budget exceeded'; END IF;
    INSERT INTO %1$I.admission_message_delivery(tenant_id,message_kind,message_id,request_id)
    SELECT v.tenant_id,v.message_kind,v.message_id,v.request_id FROM %1$I.admission_message_source v
    WHERE v.tenant_id=tenant AND NOT EXISTS(SELECT 1 FROM %1$I.admission_message_delivery d
      WHERE d.tenant_id=v.tenant_id AND d.message_kind=v.message_kind AND d.message_id=v.message_id)
    AND NOT EXISTS(SELECT 1 FROM %1$I.admission_message_delivery_receipt r
      WHERE r.tenant_id=v.tenant_id AND r.message_kind=v.message_kind AND r.message_id=v.message_id)
    ORDER BY v.recorded_at,v.message_id LIMIT LEAST(p_limit,available);
    GET DIAGNOSTICS added=ROW_COUNT; RETURN added;
   END $body$;
   REVOKE ALL ON FUNCTION %1$I.reconcile_admission_delivery(INTEGER) FROM PUBLIC;
   GRANT EXECUTE ON FUNCTION %1$I.reconcile_admission_delivery(INTEGER) TO %2$I;
  $ddl$,s,owner_role);
 END LOOP;
END $migration$;
DROP TRIGGER IF EXISTS enqueue_request_delivery ON automation.run_admission_request_outbox;
CREATE TRIGGER enqueue_request_delivery AFTER INSERT ON automation.run_admission_request_outbox FOR EACH ROW EXECUTE FUNCTION automation.enqueue_admission_delivery('request','request_id');
DROP TRIGGER IF EXISTS enqueue_resolution_delivery ON automation.run_admission_resolution_outbox;
CREATE TRIGGER enqueue_resolution_delivery AFTER INSERT ON automation.run_admission_resolution_outbox FOR EACH ROW EXECUTE FUNCTION automation.enqueue_admission_delivery('resolution','resolution_id');
DROP TRIGGER IF EXISTS enqueue_result_delivery ON multica.schedule_admission_result_outbox;
CREATE TRIGGER enqueue_result_delivery AFTER INSERT ON multica.schedule_admission_result_outbox FOR EACH ROW EXECUTE FUNCTION multica.enqueue_admission_delivery('result','result_id');
CREATE INDEX IF NOT EXISTS admission_request_delivery_source ON automation.run_admission_request_outbox(tenant_id,recorded_at,request_id);
CREATE INDEX IF NOT EXISTS admission_resolution_delivery_source ON automation.run_admission_resolution_outbox(tenant_id,recorded_at,resolution_id);
CREATE INDEX IF NOT EXISTS admission_result_delivery_source ON multica.schedule_admission_result_outbox(tenant_id,recorded_at,result_id);
-- No bulk seed: each owner reconciles at most 8 missing immutable sources per call.
-- Permanent receipts prevent migration replay or reconciliation resurrecting terminal messages.
COMMIT;
