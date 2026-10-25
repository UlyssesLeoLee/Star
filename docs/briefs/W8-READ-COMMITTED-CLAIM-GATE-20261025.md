# Brief: W8-READ-COMMITTED-CLAIM-GATE-20261025

**Agent**: worker
**Phase**: W8-READ-COMMITTED-CLAIM-GATE-20261025
**Created**: 2026-10-25 15:15:00

---

Review and, if needed, harden W8 commit 7f96a066dc227ae42731884304135760c687ffc5 in its original three manifest paths. The source-position generation lock now uses shared/exclusive advisory transaction locks and the trigger is explicitly VOLATILE. Verify the proof under PostgreSQL transaction isolation. Official PostgreSQL docs state REPEATABLE READ uses the snapshot established by the first statement throughout the transaction, so VOLATILE must not be claimed as refreshing cross-transaction visibility for arbitrary isolation levels. Either (A) enforce and test that unmatched new claims run only at READ COMMITTED (exact replay may fail closed outside it), or (B) provide a sound mechanism valid at supported higher isolation levels. Add a focused regression/contract assertion if practical, update the phase report with the isolation precondition and NOT_RUN database gate, and preserve the P1 classification pending independent review. Do not broaden manifest, do not rebase/merge/push, and do not use CodeRabbit or send source externally. Return exact commands/results and limitations.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
