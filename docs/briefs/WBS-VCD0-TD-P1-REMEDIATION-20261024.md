# Brief: WBS-VCD0-TD-P1-REMEDIATION-20261024

**Agent**: codex-worker
**Phase**: WBS-VCD0-TD-REMEDIATION
**Created**: 2026-10-24 23:00:25

---

Modify only docs/test-design/vercel/TD-STAR-VERCEL-DEPLOY-001.md to address the independent review P1 for FR-VC-013. Add a NOT_RUN integration case proving an authorized Preview can connect through Star-owned Gateway to the required long-lived terminal/session, file/ARG/Canvas and chat SSE paths; verify current authorization is checked, revocation denies or closes access, and the connection is carried by the Star Gateway rather than relying on a platform request handler. Keep all test statuses NOT_RUN and VCD-0 OPEN; do not claim production behavior or run tests or deployments. Keep the test traceable to FR-VC-013 and include expected negative and recovery assertions. Do not modify SRS, ADR, Basic Design, DD, implementation code, briefs, or WBS. Run only document-level diff and format checks if available and report exact changes.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
