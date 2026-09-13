# ARG.3 Brief — crates/arg-effect 5 子模块实装

> **Task ID**: arg-03-arg-effect-crate
> **WT**: `wt-arg-03-effect` (branch: `wt-arg-03-effect`, base: main @ 87e1618 含 ARG.1 + ARG.2 + ARG.4 merge)
> **触发**: 2026-09-10 07:21 JST 用户发令"按顺序推进" + ARG.2 merge 走守门后父会话自驱
> **依赖**: 
> - ARG.1 (`crates/arg`) 已 merge @ `651117e`
> - ARG.2 (`crates/arg-bridge`) 已 merge @ `87e1618` ← 刚 merge
> - ARG.4 (`crates/api/src/arg`) 已 merge @ `1d894ab`
> **拍板来源**: per WBS-001 v0.50 §14.11 ARG.3 (5M tokens / 0.8 周) + v0.50 跨 session 续做规划 (估 8M / 5-7 session)
> **守门合规**: #1 v15 (第 45 次新事件) + #1 v19 ([P] 必先 `scripts/automation/arg_dispatch_test.py`) + #1 v25 (cargo test -p star-arg-effect --lib -j 4 100%) + #3 (跨域强制 consults) + #5 + #6 + #7 + #9 + #10 + #12 + #13 + #14 v3 (Mavis 永久代签) + #19 v19

---

## 1. Objective

落地 ARG 4 新 crate 的**第 3 个 — Effect Tier**：`crates/arg-effect`。ARG.2 同步桥 + ARG.1 数据层 之上跑 4 effect 维度。

ARG.3 完成后:
- P3-C W3 完成
- 4 effect 维度实装 (dispatch_router / context_injector / trust_engine / output_evaluator)
- 8 拓扑成就 Cypher 模板实装到 evaluator
- 10 套 challenges 双向论证 prompt 模板实装
- L0↔L1 PyO3 协议实装 (per DD §5.3)

---

## 2. Scope (per DD-AGENT-RELATIONSHIP-001.md v0.1.1 §3.1 + §4.5-4.9 + §5.2-5.3 + §6-§7)

### 2.1 必做

#### A. `crates/arg-effect/` 新 crate 5 子模块 (per DD §3.1)

- `Cargo.toml` (10 dep: arg / arg-bridge / tokio / serde / serde_json / uuid / chrono / thiserror / async-trait / tracing + 5 dev-dep: tokio / tempfile / mockall / approx / proptest)
- `src/lib.rs` 模块入口
- `src/error.rs` ARGEffectError enum (5 variants, 引用 arg::ARGError)
- `src/dispatch_router.rs` `ARGDispatchRouter` (per DD §4.5):
  - `edge_ops: Arc<EdgeOps>`
  - `route(state, current_agent_id) -> DispatchRoute { delegates, stand_ins, collaborators }`
  - 5 Reducer + DispatchRoute struct
- `src/context_injector.rs` `ARGContextInjector` (per DD §4.6):
  - 注入 mentors / shadows 边 context 到 prompt
  - `inject_context(agent_id, base_prompt, tenant_id) -> String`
  - `query_recent_decisions` / `query_recent_events` (跨 crate 调用, 接受 stub)
- `src/trust_engine.rs` `ARGTrustEngine` (per DD §4.7):
  - `record_success` / `record_failure` 调整 trust_score ±0.01 / -0.05
  - `should_skip_verify(agent_id, tenant_id) -> bool` (trust ≥ 0.8 + trusts 边 weight ≥ 0.7)
- `src/output_evaluator.rs` `ARGOutputEvaluator` (per DD §4.8):
  - `challenge_round(from_agent, to_agent, decision, tenant_id) -> ChallengeVerdict` (10 套 prompts)
  - `peer_review(agent_a, agent_b, output, tenant_id) -> PeerReviewVerdict` (阈值 ≥ 0.8)
  - `find_challenges_edge` / `find_peer_review_edge` / `select_challenge_prompt` / `build_peer_review_prompt`
  - 10 套 challenges prompt HashMap (per DD §7 + §3.2.5)
- `src/achievement_engine.rs` `ARGAchievementEngine` (per DD §4.9):
  - 3 evaluator: `TopologyEvaluator` / `BehaviorEvaluator` / `OutputEvaluator` (注: 跟 §6/§7 8 拓扑成就 + 7 行为 + 5 产出 1:1)
  - 异步触发: 收到 edge.changed event 后跑 3 evaluator
  - 命中 → 写 `achievement_unlocks` Transaction 表 + SSE 推送

#### B. 8 拓扑成就 Cypher 模板 (per DD §6 闭环 G-6)

- `src/achievement_engine/topology_evaluator.rs` 含 8 Cypher:
  - TOP-001 跨 5 域全连接
  - TOP-002 Hub-and-Spoke
  - TOP-003 Mesh 10 节点
  - TOP-004 Chain 5 步
  - TOP-005 Hierarchical 3 层
  - TOP-006 Review-Council 4 节点
  - TOP-007 自环检测
  - TOP-008 孤岛检测
- 7 行为 event pattern + 5 产出聚合指标 (placeholder, ARG.8 续做)

#### C. 10 套 challenges 双向论证 prompt 模板 (per DD §7 闭环 G-5)

- `src/prompts/challenge.rs`:
  - 5 decision_type × 2 trust_tier = 10 套
  - 每套含 `self_justify_prompt` (B 自证) + `evaluate_prompt` (A 接受/reject/escalate)
  - CHALLENGE_PROMPTS HashMap (DecisionType, TrustTier) -> ChallengePrompt

#### D. 1 脚本 (per 守门 #1 v19 [P])

- `scripts/automation/arg_dispatch_test.py` v0.1 (~250 行):
  - 18 UT 覆盖 (dispatch 3 + context 3 + trust 4 + output 3 + achievement 5)
  - mock LLMClient, mock EdgeOps, 跑所有 effect 维度

#### E. 2 文档更新

- `docs/automation-design.md` §4 追加 ARG.3 行
- `scripts/automation/registry.md` §1 + §5 索引追加 arg_dispatch_test.py

#### F. 1 报告 (per AGENTS.md §3 7 段)

- `docs/reports/PHASE-ARG-03-IMPL-REPORT.md` v0.1

#### G. 1 commit + merge

- 1 commit: `feat(arg-effect): crates/arg-effect 5 子模块 + 18 UT + 8 拓扑成就 Cypher + 10 challenges prompt (ARG.3 子项, P3-C W3)`
- author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)
- 不推 origin

### 2.2 不做

- 不写 crates/arg-bridge 改动 (ARG.2 已收官)
- 不写 frontend (ARG.5 后续)
- 不写 behavior / output achievement evaluator (ARG.8 后续)
- 不写 真实 LLM 调用 (守门 #5 v2 + #23, mock LLMClient)
- 不写 PyO3 真实 Python LangGraph state module (ARG.1 stub 接受, L0↔L1 协议留 ARG.3 §5.3 实证)

---

## 3. Acceptance Criteria (per DD §10.1.3 18 UT 模式)

### AC-1: 编译守门
- [ ] `cargo check --workspace --lib -j 4` exit 0
- [ ] `cargo fmt --all -- --check` exit 0
- [ ] `cargo clippy --workspace --lib -j 4 -- -D warnings` exit 0
- [ ] `cargo build --release -p star-arg-effect` exit 0

### AC-2: 测试守门
- [ ] `cargo test -p star-arg-effect --tests -j 4` exit 0
- [ ] **18 UT 100% pass** (per DD §10.1.3: dispatch 3 + context 3 + trust 4 + output 3 + achievement 5)
- [ ] 0 failed, 0 ignored

### AC-3: 集成守门
- [ ] `crates/arg-effect/Cargo.toml` 10+ dep + `arg` + `arg-bridge` workspace path
- [ ] `[workspace].members` 追加 `"crates/arg-effect"`
- [ ] 0 unsafe 块
- [ ] 0 missing_docs warn

### AC-4: 4 effect 维度实装
- [ ] `ARGDispatchRouter.route` 返回 3 Vec (delegates/stand_ins/collaborators)
- [ ] `ARGContextInjector.inject_context` 返回 injected String (含 mentors/shadows context)
- [ ] `ARGTrustEngine.should_skip_verify` 验 trust ≥ 0.8 + trusts 边 weight ≥ 0.7
- [ ] `ARGOutputEvaluator.challenge_round` 返回 ChallengeVerdict (含 verdict: Accept/Reject/Escalate)

### AC-5: 8 拓扑成就 Cypher (G-6 闭环)
- [ ] 8 个 TOP-001..TOP-008 Cypher 全 (per DD §6)
- [ ] apoc.coll 不用 (per self-review F-11 修复)

### AC-6: 10 套 challenges prompt (G-5 闭环)
- [ ] 5 decision × 2 tier = 10 套
- [ ] CHALLENGE_PROMPTS HashMap 含 10 键

### AC-7: 自动化档守门
- [ ] `scripts/automation/arg_dispatch_test.py` 存在
- [ ] `docs/automation-design.md` §4 追加 1 行
- [ ] `scripts/automation/registry.md` 追加 1 行

### AC-8: 守门 v25
- [ ] 单 crate 模式实证: `cargo test -p star-arg-effect --lib -j 4` exit 0
- [ ] workspace 兼容: `cargo check --workspace --lib -j 4` 0 err
- [ ] author = Ulysses

---

## 4. 已知约束

| 约束 | 处理 |
|---|---|
| 守门 #1 v19 [P] | 落 `arg_dispatch_test.py` + 注册 |
| 守门 #7 unsafe | 0 unsafe 块 |
| 守门 #12 docs 同步 [P] | automation-design.md §4 + registry.md |
| 守门 #10 代签 | `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local'` |
| 守门 #9 RPC 不可靠 | 不调任何 RPC, mock EdgeOps / LLMClient |
| 守门 #3 5 域 Lead 跨域边 | `select_challenge_prompt` 强制 consults 而非 delegates_to |
| ARG.1 MemGraphClient stub | effect 调用 ops 接受 stub |
| ARG.2 bridge 4 子模块 | effect 接受 bridge 事件流 |

---

## 5. Deliverable

- `crates/arg-effect/Cargo.toml` + ~8 src + 18 tests = ~27 文件
- `Cargo.toml` 末尾追加 `"crates/arg-effect"`
- `scripts/automation/arg_dispatch_test.py` (~250 行)
- 2 文档更新
- 1 报告 (~250 行)
- 1 commit (author=Ulysses)

---

## 6. Token 预算

- 软预算 5M (per WBS-001 v0.18 §14.11 ARG.3)
- v0.50 跨 session 估 8M
- 本子代理预期 5-7M
- 触发熔断 9M
- 超出记录即可 (per 2026-09-10 用户拍板)

---

## 7. 子代理执行守门

- 不调任何 RPC
- 不用 RPC 派子代理
- 不写跨 crate 改动
- 不调 cargo publish / git push
- 不写无 git 实证叙事
- 每步必跑守门

---

## 8. 父会话职责

- 接收子代理 final report
- self-review 子代理产出
- 跑守门 #1 v3 + v25 全套
- merge 走 `git merge --no-ff wt-arg-03-effect`
- WBS 升版
- 触发 ARG.5 派新子代理 (frontend 5 UI 组件)

---

## 9. 失败接手

| 失败 | 接手 |
|---|---|
| 子代理 RPC 失败 | task_stop, 父会话接手 |
| cargo test 失败 | 本地 fix, 跑 cargo test |
| 18 UT < 100% | 必补到 100% |
| 守门 #7 unsafe | 移除 |
| 守门 #12 docs 同步漏 | 补 |
| Token 超 9M | task_stop, 报告 |
| merge 冲突 | 父会话手动 rebase |

---

## 10. 引用

- `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1
- `docs/design/BD-AGENT-RELATIONSHIP-001.md` v0.1
- `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 §3.1 + §4.5-4.9 + §5.2-5.3 + §6-§7
- `docs/architecture/2026-09-03-arg/07-arg-03-5-sa-impl.md` v0.50 (428 行, 5 module + 9 SA + 8 拓扑 Cypher + 10 challenges + L0↔L1 PyO3)
- `docs/reports/STAR-P3-WBS-001.md` v0.50 §14.11 (commit `f476147`)
- `crates/arg/` ARG.1 commit `651117e`
- `crates/arg-bridge/` ARG.2 commit `87e1618` (刚 merge)
- `docs/briefs/arg-01-arg-crate-skeleton.md` (模板)
- `AGENTS.md` §4 守门 + §3 7 段报告
- `docs/automation-design.md` v0.1
- `scripts/automation/registry.md` v0.6+
