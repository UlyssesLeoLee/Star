# Brief: ERUN-P3-RUST-RUNTIME-SCOUT

**Agent**: hook_deeplink_finish
**Phase**: ERUN-P3
**Created**: 2026-10-02 09:55:57

---

Read AGENTS.md first. Read-only scout for the next production CLI phase; do not edit source, commit, run Cargo, alter processes/configuration or dispatch workers. You are not alone in this repository; preserve all others' edits.
Parent graph evidence: no .codegraph directory; graph/index tools unavailable, source fallback only, no coverage/completeness claim. Known paths: crates/star-api-rest/src/group_api/cli_sessions.rs; crates/domain-local-runtime/src/task_execution.rs; star-dto/src/task_run.rs. ERUN-P2 adds server-resolved identity/typed snapshots, V2 signed fence and attachment recheck but no production TaskCliSessionProvisioner. Host session adapter in frontend exists but RootLayout has no provider. P3 Task owner sequence already scouted; do not repeat Task/Canvas mapping.
Question: which existing Rust PTY/process/sandbox/nonce/ticket/lifecycle and real authenticated desktop session components can form a production TaskCliSessionProvisioner, and precisely what prevents safely wiring them? Trace production startup/registration/callers, authority sources and cancel/drain/independent validation writeback; identify smallest bounded next implementation with dependencies/acceptance, and unsafe capability gaps that must remain disabled. Give exact paths/symbols/line evidence; no historical narrative without git evidence. Distinguish absent registration from unavailable external environment and complete source vs draft. Worktree: erun-p2-hook-deep-link/Star (read-only). Return findings to root only.


## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
