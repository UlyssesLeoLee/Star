# W0 — Version and Release Owner Evidence Audit

**Lane:** `codex/star-stable-w0-version-owner-audit-20261031`
**Baseline:** `dev@2a57fa0d329be830735d6657677236e195673362`
**Disposition:** bounded evidence audit only; W0 remains PARTIAL and the stable release remains OPEN.

## Objective

Establish the current version declarations and release workflow inputs for the user-approved Desktop, Web/Vercel, and three Star Canvas image scope. Identify what must be decided before W0 can close. Do not choose a canonical stable version, name an accountable release owner, create a tag/release, publish, deploy, or alter package metadata.

## Questions answered

1. Which version values are currently declared by the workspace and shipped application manifests?
2. How do the current Desktop, Web evidence, and Canvas image workflows select a version?
3. Which version and ownership decisions remain open, and what evidence would close W0?

## Evidence scope

- `Cargo.toml`, `bff/Cargo.toml`, the root and Desktop frontend `package.json` files, Tauri configuration, and the three Canvas service manifests.
- `star-stable-desktop.yml`, `star-stable-web.yml`, and `publish-star-canvas-services.yml`.
- Local `v*` Git tags and their relation to the audit baseline. This does not establish GitHub Release objects or remote publication state.

## Boundaries

- W0 only; the root Cargo license mismatch, legal/NOTICE closure, artifact attestations, target platform verification, and Multipass/K3s readiness remain their own WBS gates.
- Read-only audit; only this brief, its phase report, and its dispatcher record may be written.
- Report source facts separately from recommendations. Historical tags are not evidence of a release created in this lane.

## Acceptance

- Record exact version values and source paths/lines.
- Distinguish Git tags from user-facing package versions and deployment API/schema versions.
- Record the latest locally visible release-style tag and whether it is an ancestor of the audit baseline; do not infer remote release state.
- Identify the specific user decisions and machine-checkable mapping criteria needed to close W0.
- Run `git diff --check` and parse the lane JSONL before committing. Do not run builds/tests for this documentation-only audit.
