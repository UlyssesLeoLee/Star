# Brief: static and package naming remediation

## Task

Finish the user-visible naming remediation that remains after the web and CLI
lanes: neutralize the standalone demo/delivery surfaces and the descriptions
of non-published Rust packages, while preserving real connector identifiers,
compatibility IDs, and historical audit documents.

## Ownership

- Lane: `codex/naming-static-remediation-20261106`
- Base: `dev` at lane creation
- Owned files: the explicitly declared demo/delivery files, seven Cargo
  manifests, this brief, and the phase report.
- Do not touch source files outside the manifest, database identifiers,
  historical surveys/ADRs, `main`, remotes, user-dirty content, or other
  worktrees.

## Required result

1. Replace user-facing competitor/analogy labels in `demo/index.html` and the
   `kanban-vmodel-jp` delivery page with neutral Star terms such as task
   workflow, collaboration canvas, and project scheduling.
2. Update the generator assertion to validate the neutral behavior and
   preserve the task lifecycle semantics (add, complete, abort, delete,
   return-to-backlog).
3. Change only the `description` fields in the seven `publish = false`
   Cargo manifests; preserve package names, dependencies, features, and
   publication flags.
4. Add a §0–§7 report with static scans and targeted tests. Keep compatibility
   IDs and historical document mentions explicitly out of scope.

## Acceptance

- `git diff --check` passes.
- Targeted generator tests and a static scan of the owned user-facing files
  pass; no new mock or unrestricted execution path is introduced.
- Cargo metadata still parses with the same package count and all seven
  manifests remain `publish = false`.
- The report separates verified local changes from browser, release, and
  remote checks not run.
