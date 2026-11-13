# Brief: W8-CURSOR-MODE-BARRIER-REMEDIATION-20261025

**Agent**: worker
**Phase**: W8-CURSOR-MODE-BARRIER-REMEDIATION-20261025
**Created**: 2026-10-25 14:54:46

---

Fix the newly confirmed fail-open in the registered W8 source-position lane HEAD fd0472ffdf2ad5fe6385b071abbbf4b6024198f2. Current BEFORE INSERT trigger protects a new non-NULL cursor after NULL history, but a new NULL claim can still be inserted after an existing non-NULL claim. Under READ COMMITTED, advisory locking serializes the writes but the lock winner can leave a mixed generation, including an earlier sequence cursorless row accepted after a later known-cursor claim. In the existing manifest paths only: (1) enforce symmetric fail-closed mode consistency for newly inserted claims, with exact replay (inserted_count=0 + full payload match) preserved; known-cursor claims must fail after cursorless history, and newly cursorless claims must fail after any known-cursor claim. The shared/exclusive generation lock must guard both checks. (2) Extend the ignored PostgreSQL harness for known-first then cursorless-at-different-sequence rejection, cursorless-first then known rejection, unchanged receipt/ACK counts, and an actual concurrent race proving at most one mode commits. Ensure the seeded cursorless case is a fresh generation so it models legacy NULL history without contradicting the new transition rule. Review migration backfill behavior for mixed legacy generations: preserve historical rows but make all new unmatched claims fail closed; allow only exact existing replays. If this cannot be proved safely in these three files, record a precise blocker instead of weakening the rule. Update the W8 report with the P1 finding, remediation and unchanged PG/RLS NOT_RUN status. Keep exactly the three manifest paths, do not rebase/merge/push, and report commit SHA, exact verification commands/results, and remaining risks.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
