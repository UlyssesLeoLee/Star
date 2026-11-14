# Lane D: Rust duplication audit (2026-11-13)

Branch `dedupe/lane-d` (from `dev`). Method: `cargo metadata --offline` reverse-dependency graph over the 112 workspace members, a normalized function-body hash scan across `crates/**/*.rs` (fns >= 220 normalized chars, identical in >= 2 crates), and manual reading of crate headers/public items. `cargo-machete` is not installed, so unused-dependency analysis was NOT run.

## 1. Changes made (only provably identical code)

Commit `refactor(dedupe-d): hoist duplicated wire-string mappers into domain crates`.

| Duplicate (byte-identical in star-api-rest and star-mcp) | Now |
|---|---|
| `status_str` (x3: routes/validations, routes/submissions, tools/run_validation) | `domain_validation::ValidationStatus::as_str()` (same text as the existing `Display`) |
| `parse_kind` (x2) | `domain_validation::ValidationKind::from_wire()` (exact inverse of existing `as_str`) |
| `pipeline_status_str` (x2) | `domain_scm::PipelineStatus::as_api_str()` |
| `review_state_str` (x2) | `domain_scm::ReviewState::as_api_str()` |

7 local fns removed (-109 / +62 lines); output strings unchanged; the two domain crates only gained methods. `as_api_str` is named distinctly because both enums serialize as snake_case via serde while the API/MCP wire form is UPPER_CASE.

Evidence: `cargo check -p domain-validation -p domain-scm -p star-api-rest -p star-mcp -p star-pg-adapter -p api -p application --all-targets -j 4` (CARGO_TARGET_DIR=D:/Star-wt/target-lane-d) gave `Finished dev profile in 5m 39s`, EXIT 0, no new warnings in touched files (remaining warnings are pre-existing unused imports elsewhere). Direct dependents of domain-validation = {api, application, star-api-rest, star-mcp}; of domain-scm = {api, application, star-api-rest, star-mcp, star-pg-adapter}; all were checked. No tests were run (not requested; no behaviour change).

## 2. Overlap matrix (report only; nothing merged/renamed/deleted)

"Dependents" are workspace reverse dependencies from `cargo metadata`.

| A vs B | Evidence | Risk | Recommendation | Effort |
|---|---|---|---|---|
| star-task / domain-task / shared-task / star-taskgraph / star-taskqueue | Five different jobs under one name family. star-task = 7-state task lifecycle + review gate (1043 LOC); domain-task = in-memory L0 `TaskQueue` trait (212 LOC); star-taskqueue = SQLite WAL queue + `Compactor` (about 1.9k LOC). domain-task and star-taskqueue both implement an "L0 dispatch queue". shared-task = cross-view schema (From/Into); star-taskgraph = TaskCard<->Worktree + react-flow. `TaskId(pub Uuid)` is defined separately in shared-task, star-task, star-game, star-scheduler (the latter three already depend on shared-task, so could re-export it). `TaskState`/`TaskStatus` enums exist in 9 crates (domain-batch, domain-cli, graph-core, shared-task, star-agent-runtime, star-dispatcher, star-task, star-taskgraph, star-taskqueue) with differing variants. star-task, domain-task, star-taskgraph, star-taskqueue have 0 dependents. | Med: variant sets differ, a naive merge changes semantics | Choose one queue owner (domain-task trait + star-taskqueue SQLite backend is the natural split); have star-task/star-game/star-scheduler re-export `shared_task::TaskId`; leave status enums until an ADR picks the canonical lifecycle | M-L |
| star-eventbus / event-bus / bi-native-event-adapter | Not duplicates in code. star-eventbus = in-process pub/sub + Mailbox + Valkey (0 dependents); event-bus = worktree-canvas Event Bus (15 events, debouncer, Redis Streams abstraction; dependent: worktree-canvas); bi-native-event-adapter = durable receive adapter for the BI owner (dependent: star-api-rest). Overlap is conceptual: two generic pub/sub traits and two Redis/Valkey stream stubs. | Low/Med | Keep. Long term: one `EventBus` trait behind the transport-agnostic messaging contract in AGENTS.md (NATS only as adapter) | L |
| agent-domain / domain-agent / domain-agent-windows / agent-bridge | agent-domain (A1-A10 cluster/handoff/topology, about 1.6k LOC, only dependent: `api`) vs domain-agent (about 13k LOC; dependents api, application, run-owner-runtime, star-api-rest, star-mcp): different layers, both claim the "agent domain". domain-agent-windows: 0 dependents. agent-bridge = `AgentRuntime` trait for worktree-canvas (dependents api, worktree-canvas), unrelated to agent-domain despite the name. | Med | agent-domain is reachable only through the orphan `api`; fold into domain-agent or retire with `api` once owners confirm. Confirm domain-agent-windows against the frontend agent-windows page | M |
| star-game (retired demo/game service) | AGENTS.md section 0 says retired demo/game services must not be in the active build/deploy surface. Yet `Cargo.toml` members (line 138) include `crates/star-game`, and `ci.yml` (L125/135/167) runs `--workspace` check/clippy/test, so star-game IS built and tested in CI. Mitigations present: `.dockerignore` excludes it; `publish-star-canvas-services.yml` L320/341 asserts the canvas package closure excludes it. 0 dependents (shared-task docs still list it as a "view"). Frontend `canvas-game/` also exists. No hits in `deploy/`. | High (policy) | Remove from workspace members and CI (owner decision; crate deletion is out of scope for this lane) | S |
| api / application / infrastructure | `api` and `application`: 0 dependents (confirmed orphans, only built via `--workspace`). `infrastructure` is used by api, application and star-api-rest, so it is NOT an orphan. `actor_context_skeleton` is byte-identical in api, application and infrastructure; `new` identical in api and application. Copy-paste skeletons. | Low | Delete api + application (and agent-domain behind them) after owner confirmation, or move the skeleton helper into star-context | S |
| star-workflow / domain-workflow | star-workflow (762 LOC, shared-task WBS issue + transition rules) vs domain-workflow (1.5k LOC + visualize, spec'd in docs/specs/domain-workflow-spec.md); both 0 dependents. Same concept, different models. domain-workflow is the DDD-sanctioned name. | Med | Pick domain-workflow as canonical; port star-workflow's WBS-issue rules or retire it | M |
| star-context / star-context-tiering | Different concerns: star-context = shared `ActorContext` used by about 38 crates; star-context-tiering = L0/L1/L2 context tiers (380 LOC, 0 dependents). Name collision only. | Low | Keep; consider renaming tiering | S |
| ActorContext / ID types | Only 2 `pub struct ActorContext`: `star-context/src/actor.rs` (canonical) and `star-mutex/src/lib.rs:144` (four Uuids actor/tenant/workspace/trace). The duplicated machinery is the per-domain `define_uuid_id!` macro: 5 separate `macros.rs` copies (domain-validation/theme/integration/feedback/comment) plus roughly 30 crates using it. | Med | Hoist the macro into star-context and let star-mutex convert from the canonical ActorContext; touches many crates, needs staged checks | L |
| Error boilerplate | `fmt` of a Display error impl is identical in 9 crates (action-engine, agent-bridge, git-adapter, git-observer, graph-core, health-engine, relationship-engine, risk-engine, worktree-service); `new` in 5. | Low | Move those error enums to `thiserror`; no shared crate needed | S |
| `record` helper | Identical in platform-messaging-runtime/admission_receive.rs and run-owner-runtime/native_capacity_result_consumer.rs. Owners are intentionally isolated, so sharing needs care. | Low | Leave, or move into a test util | S |

## 3. Other findings

* `Cargo.toml` line 138 lists `star-workflow`, `star-canvas`, `star-scheduler`, `shared-task`, `leads` twice in `members` (cargo tolerates it). Left untouched; a one-line cleanup.
* 52 of 112 members have no workspace dependents. Many are legitimate binaries/services/WASM; suspicious library leaves: star-task, star-taskgraph, star-taskqueue, domain-task, star-eventbus, star-memory, star-mutex, star-context-tiering, star-dispatcher, star-workflow, domain-workflow, star-game, api, application, domain-report/dashboard/form/ai/mcp/agent-windows. Confirm each is intended scaffolding before pruning.
* Unused dependency check not performed (no cargo-machete installed).

## 4. Deferred (needs owner decision)

1. Remove star-game from workspace/CI (policy violation, highest priority).
2. Retire or wire `api` / `application` / `agent-domain`.
3. Pick a canonical task queue (domain-task vs star-taskqueue) and one `TaskId`.
4. star-workflow vs domain-workflow; ActorContext in star-mutex; `define_uuid_id!` hoist.
5. Install cargo-machete and run per crate.
