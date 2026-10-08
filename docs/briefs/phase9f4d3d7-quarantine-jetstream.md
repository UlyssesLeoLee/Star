# Phase 9F4D-3D7 brief — JetStream quarantine redelivery gate

## Scope

Close the bounded verification gap left by Phase 9F4D-3D6: exercise the existing Run-owner permanent-poison hook against a real disposable JetStream server and PostgreSQL database. This is a test and documentation phase; it does not add a production poller, enable Schedule or Task Card execution, change any owner schema, or change the Worktree/Run navigation model.

## Required behavior

1. Use an explicit `metadata_only` quarantine policy and a trusted configured deployment/stream generation; do not infer Platform scope from event tenant or headers.
2. Inject a PostgreSQL insert failure while a malformed result is delivered. Verify that no quarantine receipt is committed and the JetStream delivery remains ack-pending.
3. Remove the fault, wait the configured short `AckWait`, and verify that the actual redelivery writes exactly one Platform receipt before a confirmed Term acknowledgement; verify no later delivery.
4. After receipt commit but before source ACK, drop the unacknowledged JetStream `Message` and fetch handles to model consumer loss. Verify the real redelivery matches the immutable receipt, returns the duplicate outcome, receives Term ACK, and is not delivered again. This does not kill or restart an OS process.
5. Verify metadata-only mode stores payload length/digest without a payload Work row. Keep Run Inbox, Schedule, BI, reservations, readiness, and execution unchanged.

## Acceptance evidence

- Focused Rust 1.99.0 format, check, test and strict Clippy gates for the changed runtime/test targets.
- Disposable PostgreSQL 18.6 and loopback NATS JetStream integration runner; no Multipass inventory changes or VM creation.
- The runner applies the existing quarantine migration and schema assertions, preserves logs in a new unique `target/` output folder, and stops/cleans only its owned test services/data.
- The Rust harness runs this shared-subject JetStream integration target with `--test-threads=1`; otherwise separate tests race while creating the same subject.
- Requirements, basic design, Schedule SRS/BD/DD, Group DD, Infrastructure DD, implementation plan, Automation Design, registry and phase report state the exact test boundary and keep production gates open.

## Explicitly out of scope

Production credentials/consumer provisioning, KMS, operator replay UI, production database rollout, Workspace/RSS benchmarking, provider/Manager lifecycle, and TaskExecutionRun/CLI execution remain separate gates.
