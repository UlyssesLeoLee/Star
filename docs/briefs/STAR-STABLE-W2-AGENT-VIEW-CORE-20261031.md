# Brief: STAR-STABLE-W2-AGENT-VIEW-CORE-20261031

**Agent**: worker
**Phase**: STABLE-RELEASE-W2-AGENT-VIEW-CORE
**Created**: 2026-10-31 15:24:41

---

Remove legacy game mechanics from the shipping Web Agent View path while preserving useful operational Star features.

Ownership is only the registered manifest: frontend/src/app/(app)/agent-view/page.tsx; frontend/src/components/agent-view/AgentCanvasView.tsx and AgentCanvasView.test.tsx; frontend/src/components/agent-game/; frontend/src/lib/agent-game/; frontend/src/lib/store.ts; the new neutral agent settings files frontend/src/components/agent/AgentSettingsTab.tsx, frontend/src/lib/agent/settings.ts and settings.test.ts; and this lane brief, phase report and dispatch JSONL. Do not edit workflows, package manifests/locks, Cargo files, WBS, Docker/Helm, D2/D4/W8, main, remotes, or any other worktree.

Evidence: the routable `frontend/src/app/(app)/agent-view/page.tsx` is a Next.js client route and directly imports GameHUD, PerkPicker, DeathModal, AgentSettingsTab, useAgentGame and agent-game modules. It drives XP, coins, HP, perks, deaths/revive and game state in the shared store. `AgentCanvasView.tsx` imports game levels, characters, enemies and game theme. The route is part of the production Web surface, so those static imports enter its client module graph. Context7 Next.js v16.2.9 docs consulted: a `use client` boundary includes directly imported/rendered modules in the client bundle; production `next build` and route output/trace can verify the actual result.

Remove game-only UI/state/assets and unreachable game modules from this Agent View route. Preserve Agent Canvas annotation/editing, agent/worktree/task context, relationships, and useful Agent Settings by moving settings code to neutral `agent` paths. Do not delete or rename unrelated Rust `star-game` files; Docker exclusion is a separate existing release gate. Inspect whether any preserved route data is seed/mock-only; remove obsolete demo fixtures only when they are in this manifest, and never present mock data as a working production agent workflow. Do not claim the Agent CLI is a real execution loop.

Because source paths change, maintain all affected Cypher graph headers using the Cypher Graph Guardian and Cypher Header skills. Update old file/symbol references, add accurate headers to new source files, and run the repository `cgg_guard.py --changed --stamp` gate if available. Verify all new edges against actual imports/calls; no guessed relationships.

Run focused frontend typecheck, relevant tests, and production Web build (using the repository's locked package manager); scan the resulting route/client trace or bundle for `agent-game`, XP/coins/HP/perk/death/revive/enemy/game-map code and verify the operational canvas/settings/relationships remain. Run diff check and record exact results and any pre-existing failures in the phase report. Commit only manifest files with author Ulysses <ulysses@mavis.local>. Do not merge, push, tag, publish, deploy, or clean anything.

CodeGraph evidence: two targeted calls identified the AgentViewContent → GameHUD path and listed AgentCanvasView/store references, but did not return all relevant source. No `check_index_coverage` tool is available; exact source fallback was used for the route and import/reference inventory. Do not claim graph coverage is complete.

## 6. Read COMMENTS (任务卡留言, 必读, per 守门 v33)

(无任务卡留言)
