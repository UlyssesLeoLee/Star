# Brief: STABLE-DESKTOP-CI-HARDENING-20261031

**Agent**: worker
**Phase**: STABLE-DESKTOP-CI-HARDENING-20261031
**Created**: 2026-10-31 11:28:56

---

Scope: update only the registered desktop stable artifact workflow, the lane report, and its existing dispatcher audit JSONL. Add the Vitest suite to the Node 24 job before the production build, so workflow evidence covers package install, tests, and build. Set persist-credentials: false on each actions/checkout used for pull-request code. Keep the workflow non-publishing and keep signing, notarization, install/update/rollback, hosted runner, Rust 1.99 and Tauri CSP gates explicitly open. Do not modify the legacy desktop workflow, Cargo workspace metadata, package manager outside the registered desktop frontend, or other lanes. Acceptance: inspect every workflow checkout and job permission; ensure supported Node 24 is used; run the registered desktop frontend install/test/build gates if available; validate YAML and diff; report tests and limitations truthfully. Do not publish, sign, tag, deploy, or push.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
