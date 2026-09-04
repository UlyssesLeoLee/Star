{
  "fixture_version": "v1",
  "module": "sa-05",
  "type": "doc-sync",
  "method": "POST",
  "scenario": "sync-docs-AGENTS-and-test-design",
  "description": "SA-05 doc-sync: 同步 AGENTS.md / test-design.md / HANDOFF (per 守门 #12 commit-time docs 同步 + #15 死循环饱和边界)",
  "request": {
    "sync_target_docs": ["AGENTS.md", "docs/test-design.md", "docs/reports/HANDOFF-ST-001.md"],
    "sync_source": "git log --oneline -10 + diff --stat"
  },
  "response_200": {
    "sync_id": "docsync-uuid-005",
    "docs_updated": ["AGENTS.md", "docs/test-design.md"],
    "docs_skipped": ["docs/reports/HANDOFF-ST-001.md (饱和边界, 不动)"],
    "saturation_check": "12 ahead origin/main > saturation threshold 10, skip HANDOFF"
  },
  "fixture_assertion": {
    "guard_15_saturation_respected": true,
    "docs_updated": 2
  }
}
