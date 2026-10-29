# Brief: W16-A-OAUTH-PG-ACCEPTANCE-20261029

**Agent**: Codex worker OAuth PG acceptance
**Phase**: W16-A OAuth PostgreSQL transaction acceptance
**Created**: 2026-10-29 13:20:23

---

Objective: make the existing OAuth authorization-code and refresh-token PostgreSQL integration tests a reproducible, fail-closed acceptance gate, and repair the refresh-token consume contract exposed by running that gate. Baseline is dev@8704466be335d2a3e9c12ba7a8d7da44e9e6767c.

Evidence/context: Parent CodeGraph found atomic repository paths for authorization-code consumption/token-pair issue and refresh rotation, but did not reliably index the integration test module. A targeted read confirmed ignored real PostgreSQL tests in crates/star-api-rest/tests/oauth_device_auth_pg.rs: postgres_oauth_credentials_have_one_winner_when_test_database_is_configured, postgres_auth_code_redemption_persists_tokens_with_consumption, and postgres_refresh_rotation_commits_one_successor_with_the_consume. They use STAR_W16_DEVICE_AUTH_TEST_DATABASE_URL. Mocks are not production evidence.

Implement only the manifest files. Build a small acceptance runner (plus unit tests) that invokes the real ignored PG tests and fails closed when database/test prerequisites are unavailable; it must never report success from mocks or silently skip ignored tests. Prefer established local isolated PostgreSQL harness conventions, and do not modify scripts/automation/phase9f2_binding_http_acceptance.py because another active lane owns it. Do not use Docker Desktop, consume or create Multipass inventory instances, or touch external/target databases. If the suite can safely include both existing W16 device and canvas PG binaries via explicit suite selection, keep the runner API bounded and document exactly what it executes. Strengthen OAuth PG tests only if needed to prove actual single-winner behavior and atomic token-pair persistence under concurrent requests; use barriers and database row assertions, not sleeps.

Scope update (2026-10-29): the first completed local PostgreSQL run exposed that `PgOAuthRefreshTokenRepository::consume_if_active` uses eight SQL placeholders but its method contract supplied only tenant, token hash, and timestamp. Amend only the MAGOS-registered lane paths to pass an expected `OAuthRefreshToken` snapshot through the repository and mock, compare the client/user/scope/device identity fields atomically, and exercise a stale-snapshot rejection before the 24-way single-winner race. The manifest was amended and allowed before these edits. This does not alter the HTTP/OAuth handler contract or loosen identity predicates.

Write the phase report and dispatch audit listed in the manifest, recording exact commands/outcomes and open evidence. Do not claim W16-A complete. Do not edit shared WBS, test-design, requirements, basic/detailed design, automation registry, other files, or any other worktree. Commit only after focused formatting/static checks and runnable tests; provide the commit SHA and clearly label any unavailable database acceptance as open. You are not alone in the repository: stay inside this lane, preserve all unrelated work, and never revert other agents' edits.

The repository-required dispatcher brief has been created before dispatch. Include exact commands you can run without external credentials.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
