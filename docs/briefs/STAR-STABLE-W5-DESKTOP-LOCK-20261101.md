# Brief: STAR-STABLE-W5-DESKTOP-LOCK-20261101

**Agent**: worker
**Phase**: PHASE-STAR-STABLE-W5-DESKTOP-LOCK-20261101
**Created**: 2026-10-31 23:42:36

---

Implement the W5 Desktop locked-build correction in this registered lane only.

Base: dev@2a57fa0d329be830735d6657677236e195673362
Owned product file: .github/workflows/star-stable-desktop.yml
Registered report files:
- docs/briefs/STAR-STABLE-W5-DESKTOP-LOCK-20261101.md
- docs/reports/PHASE-STAR-STABLE-W5-DESKTOP-LOCK-20261101.md
- docs/reports/WBS-DISPATCH-STAR-STABLE-W5-DESKTOP-LOCK-20261101.jsonl

Goal: make the Linux, Windows, and macOS stable bundle build invocations fail if Cargo.lock would change. Verify the exact passthrough syntax using the pinned Tauri CLI version/help or current official Tauri documentation before editing. Context7 research found tauri build runner passthrough on iOS and tauri-apps/tauri-action args forwarding, but did not yet verify the desktop tauri build reference; do not infer desktop support from iOS. If desktop runner passthrough cannot be proven, leave the workflow unchanged and report the blocker with the strongest evidence and a supported alternative. Do not run native builds, create tags/releases, upload artifacts, or deploy.

Use only the owned product file plus registered brief/report/log paths. Validate the resulting workflow with available static tooling and git diff --check; report commands and exact results. Do not claim hosted evidence. The report must include the repository-required sections: purpose, change matrix, verification, known gaps, subagent handoff, guard rules, sign-off, revision history.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
