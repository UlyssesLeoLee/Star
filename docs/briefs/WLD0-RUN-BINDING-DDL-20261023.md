# Brief: WLD0-RUN-BINDING-DDL-20261023

**Agent**: wld0-run-binding-ddl
**Phase**: wld0-binding-ddl-reconciliation-20261023
**Created**: 2026-10-23 07:55:25

---

## Objective
Advance the planned WLD0 Run/Worktree binding DDL lane with a safe, testable migration-ledger reconciliation/preflight boundary for the non-replay-safe migration sequence. Own only:
- \`scripts/automation/engineering_run_directory.py\`
- \`scripts/automation/test_engineering_run_directory.py\`
- \`docs/automation-design.md\`
- \`scripts/automation/registry.md\`
- \`docs/reports/PHASE-WLD0-RUN-BINDING-DDL-HARNESS-20261020.md\`
- this registered brief

## Required approach
First inspect current runner behavior and tests. Reuse existing abstractions; do not add a parallel framework if the current code already covers the requirement.
The immediate WBS gap is safe target migration ledger/catalog inspection and ambiguous-commit reconciliation before deployment. Build a deterministic preflight/classifier and focused tests so the non-replay-safe 2026-10-15 migration is never blindly replayed. At minimum distinguish:
- unapplied with required predecessor chain present;
- exact applied state with matching catalog fingerprint;
- ledger/catalog disagreement or partial schema;
- ambiguous apply outcome where a later read can prove the exact migration committed;
- unknown/incomplete state, which must fail closed without applying.
Keep historical migration SQL immutable. Do not implement a generic production DB deployer or consume credentials. If target DB access is unavailable (expected), explicitly keep target acceptance unverified and use local/pure deterministic tests only; do not imply production readiness.

## Architecture constraints
Hostless Project-owned Run identity remains independent from remote Branch; remote metadata stays optional. Do not let deployment/branch assumptions enter owner authorization. Keep API execution, Task CLI, Schedule and cleanup fail-closed until their separate gates pass. No third-party brand references in product-facing copy.

## Acceptance
- Inspect the current source, exact callers and tests before editing.
- Add focused unit coverage for each classifier branch, especially ambiguous/partial/mismatch fail-closed paths; run only the focused tests and syntax/format/Guardian checks supported by this environment.
- Update automation design, registry and phase report with truthful scope and remaining production gates.
- Run \`git diff --check\`; record exact commands/results.
- Commit only owned files and include the brief path plus \`comments-read:\` in the commit message per repository rules.
- Do not rebase, merge, deploy, fetch, push, or delete branches/worktrees; coordinator owns integration.

## Authorization and safety
You are not alone in the repository. Do not revert or overwrite other lanes. No target/system database, secret inspection, remote API, migration apply, destructive cleanup, or execution enablement. Preserve all untracked evidence. No unsupported historical claims; verify Git before writing them.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
