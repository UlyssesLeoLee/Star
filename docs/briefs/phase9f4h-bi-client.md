# Brief: phase9f4h-bi-client

**Agent**: merge_code_review
**Phase**: 9F4H
**Created**: 2026-10-17 11:13:56

---

Own frontend/src/lib/run/runDirectoryApi.ts and its tests only. Add exported typed RunBiTaskEventCursor/Item/Page and listRunTaskEvents(run,cursor?,signal?). Reuse bounded authenticated request, 50 row page, bounded response, strict scope and projection validation, unknown coverage/freshness only. Cover scope mismatch, invalid page/cursor, 503 and abort. No mocks in product. Other agents edit UI/docs; do not revert others. No commits. Graph current candidate; Tier2 targeted source with missing coverage tool explicitly disclosed. No invented historical evidence.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
