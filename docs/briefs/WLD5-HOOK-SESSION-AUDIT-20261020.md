# Brief: WLD5-HOOK-SESSION-AUDIT-20261020

**Agent**: worker
**Phase**: WLD5-HOOK-SESSION-AUDIT-20261020
**Created**: 2026-10-20 06:43:51

---

WLD-5 session integration audit only. Work only in this worktree and modify only docs/reports/WLD5-HOOK-SESSION-AUDIT-20261020.md. Trace the real application authentication/session provider, how Advanced Settings → Hooks is mounted, the existing Run-directory session pattern, and WorktreeGroupApiProvider token/session-generation requirements. Identify the smallest safe wiring, expiry/revocation behavior, tests and remaining Rust-native Hook/DB gates. Do not introduce mock/seed tokens or change application source. Base is dev 1136a24c709e944a538d28a2feff7e72b2b29f4c. Graph context from coordinator: HookPolicyPage in frontend/src/app/(app)/settings/advanced/[tab]/HookPolicyPage.tsx has saveDraft (376), publishDraft (419), rollback (439), but at 468 disables itself because app root does not provide WorktreeGroupApiProvider token/session. The API client methods for draft/publish/rollback exist. Tests are in HookPolicyPage.test.tsx; Run-directory session pattern is frontend/src/lib/run/runDirectorySession.tsx. Coordinator used CodeGraph Tier 2, index reports 1,891 files but current generation/coverage API unavailable, then focused rg fallback; no index coverage tool is available. Read exact current files before conclusions.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
