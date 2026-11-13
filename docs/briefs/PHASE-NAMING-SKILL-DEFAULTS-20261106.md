# Brief: neutral Skill Registry defaults

## Task

Close naming audit A4 without breaking explicit adapter compatibility. The
Skill Registry's implicit `agent_compat` default must expose only Star-neutral
runtime identities. Legacy provider IDs may remain accepted when callers
explicitly supply them, but must not be injected into default CLI JSON.

## Ownership

- Lane: `codex/naming-skill-defaults-20261106`
- Base: current local `dev` at lane creation
- Owned files: `crates/star-cli/src/skill_registry.rs`, this brief, and the
  phase report.
- Do not touch other source files, database IDs, historical documents,
  `main`, remotes, user-dirty files, or other worktrees.

## Required result

1. Define a documented neutral default compatibility set (including the Star
   runtime identity) and use it only when the caller passes no list.
2. Keep explicit compatibility values round-trippable so adapter/API migration
   can be handled outside the default user-facing path.
3. Update tests and public comments/help to reflect the neutral default and
   prove explicit values are preserved.
4. Add a §0–§7 report with the exact focused test and CLI JSON checks, and
   state the compatibility boundary explicitly.

## Acceptance

- `cargo +1.99.1 test --locked -p star-cli --all-targets -j 4` passes.
- Default JSON contains no legacy provider names; explicit values remain
  unchanged and are covered by tests.
- `git diff --check` passes and only the three manifest files change.
