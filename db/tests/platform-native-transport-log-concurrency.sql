-- @cypher schema=1 source_sha256=b4f80e8cc6854a5b062ddfc7a13e60a6e1703b4998d269e5b47c823d46767adf
-- MERGE (self:File {path:"db/tests/platform-native-transport-log-concurrency.sql"})
-- MERGE (log_state:Table {id:"platform_transport.log_state"})
-- MERGE (message_work:Table {id:"platform_transport.message_work"})
-- MERGE (receipt:Table {id:"platform_transport.delivery_receipt"})
-- MERGE (outbox:Table {id:"platform_transport.receipt_outbox"})
-- MERGE (append:Function {id:"platform_transport.append_message"})
-- MERGE (self)-[:TESTS]->(log_state)
-- MERGE (self)-[:TESTS]->(message_work)
-- MERGE (self)-[:TESTS]->(receipt)
-- MERGE (self)-[:TESTS]->(outbox)
-- MERGE (self)-[:TESTS]->(append)
-- @endcypher

DO $parallel_idempotency$
DECLARE
    accepted_count INTEGER;
    work_count INTEGER;
    outbox_count INTEGER;
    next_sequence BIGINT;
    bytes_in_use BIGINT;
BEGIN
    SELECT count(*)::INTEGER INTO accepted_count
      FROM platform_transport.delivery_receipt
     WHERE deployment_id='10000000-0000-0000-0000-000000000001'
       AND tenant_id='20000000-0000-0000-0000-000000000001'
       AND sender_service_id='30000000-0000-0000-0000-000000000001'
       AND recipient_service_id='30000000-0000-0000-0000-000000000002'
       AND message_kind='schedule.concurrent-test'
       AND message_id='50000000-0000-0000-0000-000000000011'
       AND action='accepted'
       AND payload_sha256=decode('7fd52a02a5983c33419862e872c92918b83adb66be8f09001eca131c43b65d35','hex');
    SELECT count(*)::INTEGER INTO work_count
      FROM platform_transport.message_work
     WHERE deployment_id='10000000-0000-0000-0000-000000000001'
       AND tenant_id='20000000-0000-0000-0000-000000000001'
       AND log_id='test.schedule.concurrent' AND generation='40000000-0000-0000-0000-000000000003'
       AND message_id='50000000-0000-0000-0000-000000000011'
       AND wire=convert_to('concurrent-identical-wire','UTF8')
       AND payload_sha256=decode('7fd52a02a5983c33419862e872c92918b83adb66be8f09001eca131c43b65d35','hex');
    SELECT count(*)::INTEGER INTO outbox_count
      FROM platform_transport.receipt_outbox o
      JOIN platform_transport.delivery_receipt r ON r.receipt_id=o.receipt_id
     WHERE r.deployment_id='10000000-0000-0000-0000-000000000001'
       AND r.tenant_id='20000000-0000-0000-0000-000000000001'
       AND r.message_kind='schedule.concurrent-test'
       AND r.message_id='50000000-0000-0000-0000-000000000011'
       AND r.action='accepted';
    SELECT s.next_sequence, s.payload_bytes_in_use INTO next_sequence, bytes_in_use
      FROM platform_transport.log_state s
     WHERE s.deployment_id='10000000-0000-0000-0000-000000000001'
       AND s.tenant_id='20000000-0000-0000-0000-000000000001'
       AND s.log_id='test.schedule.concurrent' AND s.generation='40000000-0000-0000-0000-000000000003';
    IF accepted_count <> 1 OR work_count <> 1 OR outbox_count <> 1
       OR next_sequence <> 2 OR bytes_in_use <> octet_length(convert_to('concurrent-identical-wire','UTF8')) THEN
        RAISE EXCEPTION 'parallel exact append must create one receipt, work item, outbox fact and cursor allocation';
    END IF;
END
$parallel_idempotency$;
