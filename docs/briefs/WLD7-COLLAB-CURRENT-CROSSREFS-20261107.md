# WLD7-COLLAB-CURRENT-CROSSREFS-20261107

## Objective

Synchronize the current WLD-7 spec and owner-local data-design headers so each references the current counterpart revision.

## Baseline and scope

- Base: local `dev@c6261224ea067bc68b1ae5a870007b4ea342f9c7`.
- Current baseline versions: collaboration spec v1.4 and owner-local data design v0.6; both header references lag one revision.
- This lane updates only the two source documents plus this brief, its dispatch JSONL, and phase report. Keep test design at v2.6 and preserve all prior revision history.
- No schema, migration, runtime, code, database, host, deployment, or performance change is in scope.

## Acceptance

1. Current collaboration spec v1.5 references data design v0.7; current data design v0.7 references spec v1.5.
2. Append revision entries v1.5 and v0.7 and preserve the previous histories.
3. Both documents continue to cite test design v2.6; status remains Draft/logical design only and OPEN/NOT_RUN.
4. `git diff --check` passes; exact changed-path manifest is 5/5.
5. No implementation or target-environment pass is claimed.
