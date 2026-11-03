# Brief: STAR-STABLE-W3-CONSUMER-VERIFY-20261103

**Agent**: codex subagent
**Phase**: STAR-STABLE-W3-CONSUMER-VERIFY-20261103
**Created**: 2026-11-03 04:31:13

---

Implement a small, standard-library-only consumer verifier for Star release attestations. W3 requires each selected Desktop file artifact, Web archive and Canvas OCI image to have a SHA-256-bound SLSA provenance attestation, scoped to the Star repository, exact release tag/source commit and approved signer workflow. Read the current stable desktop/web/Canvas workflows and official GitHub CLI documentation first. For files, compute and compare SHA-256 locally; for OCI, require an immutable @sha256 reference matching the declared digest. Invoke gh attestation verify with --repo, --source-digest, --source-ref refs/tags/<tag>, --signer-workflow github.com/<owner>/<repo>/<workflow>, --predicate-type https://slsa.dev/provenance/v1, and --format json. Fail closed for malformed inputs, missing executable, timeout/nonzero exit, malformed/empty JSON result, or digest mismatch. Keep subprocess shell=False and bounded; do not capture or log credentials. Add focused unit tests using mocked subprocess results (success and failure paths); do not claim hosted verification or run against real v1.0.6 assets. Use only Python standard library and the registered paths scripts/release/verify_attested_release.py and scripts/release/tests/test_verify_attested_release.py. Do not edit WBS main, workflows, other source files, D2/W8 lanes, main, remotes or user dirty files. Run focused unittest, py_compile, git diff --check, and CypherGraph Guardian; commit only the two code files on codex/star-stable-w3-consumer-verify-20261103 and report exact validation evidence.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
