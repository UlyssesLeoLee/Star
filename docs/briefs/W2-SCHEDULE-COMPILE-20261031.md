# Brief: W2-SCHEDULE-COMPILE-20261031

**Agent**: worker schedule-compile
**Phase**: W2
**Created**: 2026-10-31 17:57:35

---

Implement only the Schedule admission compile fix in crates/run-owner-runtime/src/schedule_admission.rs. The undefined identifier is inside ScheduleAdmissionTransaction::revalidate_task_binding; this transaction already stores an immutable branch_id captured from decode_branch_bound_schedule_command. Bind that field. Preserve the current branch-bound behavior: hostless_schedule_target_in_wire and required_schedule_branch_id intentionally fail closed when branch_id is absent. Do not implement hostless Schedule support or change auth, SQL, schema, Cargo.toml, Cargo.lock, or other files. Read and reconcile the Cypher header for the edited source. Validate with cargo +1.99.0 check -p run-owner-runtime --all-targets --locked -j 4 first; if the installed 1.99.0 compiler cannot run, record the exact error and then try 1.99.1 without changing the project baseline. Run focused tests only where available, and record missing PostgreSQL prerequisites rather than claiming they passed. Write the required phase report at docs/reports/PHASE-W2-SCHEDULE-COMPILE-20261031.md with the repository report sections and exact commands, toolchain, exits, evidence and limitations. Keep your changes within the registered manifest. Do not merge, rebase, push, or edit another lane. Commit only your manifest files when ready.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
