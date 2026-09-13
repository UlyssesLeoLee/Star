# ARG.8 Brief — 7 行为 + 5 产出成就 evaluator 落地

> **Task ID**: arg-08-behavior-output-evaluator
> **WT**: `wt-arg-08-behavior-output-evaluator` (branch: `wt-arg-08-behavior-output-evaluator`, base: main @ 1596f97 含 ARG.1-7 merge)
> **触发**: 2026-09-10 10:15 JST 用户发令"继续派" (per ARG.7 merge + 父会话自驱继续)
> **依赖**: 
> - ARG.1 (`crates/arg`) 已 merge @ `651117e`
> - ARG.2 (`crates/arg-bridge`) 已 merge @ `87e1618`
> - ARG.3 (`crates/arg-effect` 8 拓扑成就 Cypher 模板已落) 已 merge @ `f1207e2`
> - ARG.7 (10 IT + 8 E2E + 4 PT 端到端测试套件) 已 merge @ `a8ed5d0`
> **拍板来源**: per WBS-001 v0.57 §14.11 ARG.8 (2M tokens / 0.3 周) + v0.50 跨 session 续做规划 (估 4-6M / 3-5 session)
> **守门合规**: #1 v15 (第 50 次新事件) + #1 v19 ([M] 必先 `scripts/automation/arg_behavior_eval.py`) + #1 v25 + #3 + #5 + #6 + #7 + #9 + #10 + #12 + #13 + #14 v3 + #19 v19

---

## 1. Objective

落地 ARG 成就评估器剩余 12 个 evaluator（7 行为 + 5 产出），完整闭环 20 成就 (8 拓扑 + 7 行为 + 5 产出)。

ARG.8 完成后:
- P3-E W1 收官
- 20 成就 100% 闭环（拓扑 ARG.3 已落，行为 + 产出 ARG.8）
- evaluator 3 类（Topology / Behavior / Output）异步触发 + SSE 推送全跑通
- 守门 #1 v3 cargo test -p star-arg-effect 跨 sub-session 0 错收敛

---

## 2. Scope (per DD-AGENT-RELATIONSHIP-001.md v0.1.1 §3.2.4 + §4.9 + §6-§7)

### 2.1 必做 (per brief 缺标比错标)

#### A. `crates/arg-effect/src/achievement_engine/` 扩展 (3 evaluator 全异步)

**A.1 BehaviorEvaluator** (替换 ARG.3 占位, per brief `behavior_evaluator.rs`):
- 7 行为 event pattern (per SRS NFR §6 + WBS-001 §14.7):
  1. `first_dispatch_via_delegates_to` (首次通过 delegates_to 派发, 累计 1 次)
  2. `consults_decision_made_100_times` (consults 决策辅助 100 次)
  3. `collaborates_with_parallel_50_times` (collaborates_with 并行 50 次)
  4. `stand_in_for_takeover_5_times` (stand_in_for 接管 5 次)
  5. `peer_reviews_one_pass_100_percent` (peer_reviews 一次通过率 100%, 跑 10 次)
  6. `challenges_rebuttal_3_times` (challenges 反驳 3 次)
  7. `shadows_observation_24_hours` (shadows 观察 24h 累计)
- 每个 evaluator 接收 event pattern + 历史统计 → 命中 → AchievementUnlock

**A.2 OutputEvaluator** (替换 ARG.3 占位, per brief `output_evaluator.rs`):
- 5 产出聚合指标 (per SRS NFR §6 + WBS-001 §14.7):
  1. `trusts_skip_verify_save_100k_token` (trusts 跳过 verify 节省 100K token)
  2. `collaborate_save_1h_wall_clock` (协作并行节省 1h wall-clock)
  3. `5_domain_lead_consensus_reached` (5 域 Lead 共识达成)
  4. `zero_failure_100_collaborations` (0 失败 100 次协作)
  5. `achievement_chain_5_in_a_row` (5 个成就连环解锁)
- 每个 evaluator 查聚合表 + 阈值匹配 → 命中 → AchievementUnlock

**A.3 ARGAchievementEngine 集成**:
- 3 evaluator 并行 (tokio::join!)
- 命中 → 写 `achievement_unlocks` Transaction 表 (per 守门 #13)
- 触发 SSE 推送 (`ArgsSEHub.publish_achievement_unlocked`)
- 幂等: 同一 code 重复 unlock 返 false (per `unlock_achievement` 函数, ARG.1 已落)

#### B. `crates/arg-effect/tests/` 5 新增 IT (替换 ARG.3 既有 placeholder 测试)

- `behavior_test.rs` 7 UT (per 7 行为)
- `output_test.rs` 5 UT (per 5 产出)
- `evaluator_integration_test.rs` 5 UT (3 evaluator 集成 + 异步 + SSE)
- `unlock_idempotency_test.rs` 3 UT (幂等验证)
- `sse_publish_test.rs` 3 UT (SSE 推送验证)

#### C. 1 脚本 (per 守门 #1 v19 [M])

- `scripts/automation/arg_behavior_eval.py` v0.1 (~150 行):
  - 12 IT 端到端 (7 行为 + 5 产出)
  - 触发 mock event stream, 验证 unlock + SSE 推送
  - 守门 #9 RPC 不可靠: mock ARGSSEHub

#### D. 2 文档更新

- `docs/automation-design.md` §4 追加 ARG.8 行
- `scripts/automation/registry.md` §1 + §5 索引追加

#### E. 1 报告

- `docs/reports/PHASE-ARG-08-IMPL-REPORT.md` v0.1

#### F. 1 commit + merge

- 1 commit: `feat(arg-behavior-output): 7 行为 + 5 产出成就 evaluator 落地 (ARG.8 子项, P3-E W1)`
- author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)

### 2.2 不做

- 不写 ARG.9 收官报告 (后续)
- 不写 ARG.10 DDD Review (后续)
- 不重写 8 拓扑成就 (ARG.3 已落)
- 不写真实 SSE 客户端 (ARG.4 backend 已就绪, mock fallback)

---

## 3. Acceptance Criteria

### AC-1: 3 evaluator 落地
- [ ] `BehaviorEvaluator` 7 行为 event pattern 全部实现
- [ ] `OutputEvaluator` 5 产出聚合指标 全部实现
- [ ] `ARGAchievementEngine` 3 evaluator 并行 (tokio::join!)

### AC-2: 异步 + SSE + 幂等
- [ ] 触发 event pattern 后异步跑 3 evaluator
- [ ] 命中后写 `achievement_unlocks` Transaction 表
- [ ] 触发 SSE `ArgsSEHub.publish_achievement_unlocked` (per ARG.2 §4.11 + ARG.4 §4.12)
- [ ] `unlock_achievement` 幂等 (同 code 重复 unlock 返 false, per ARG.1 §3.2.5)

### AC-3: 5 守门实证 (per 守门 #1 v25)
- [ ] `cargo check --workspace --lib -j 4` 0 err
- [ ] `cargo test -p star-arg-effect --tests -j 4` 0 err (ARG.3 既有 47 + ARG.8 新增 23 = 70+ pass)
- [ ] `python scripts/automation/arg_behavior_eval.py` 12 IT 全过
- [ ] `cargo build --release -p star-arg-effect` 0 err
- [ ] `cargo test -p star-arg-bridge --tests -j 4` 0 err (跨 crate 兼容)

### AC-4: 自动化档守门
- [ ] `scripts/automation/arg_behavior_eval.py` 存在
- [ ] `docs/automation-design.md` §4 追加 1 行
- [ ] `scripts/automation/registry.md` 追加 1 行

### AC-5: 守门 v25
- [ ] workspace 兼容: 0 回归
- [ ] author = Ulysses
- [ ] 不推 origin

---

## 4. 已知约束

| 约束 | 处理 |
|---|---|
| 守门 #1 v19 [M] | 落 `arg_behavior_eval.py` |
| 守门 #7 unsafe | 0 unsafe 块 |
| 守门 #10 代签 | Ulysses |
| 守门 #9 RPC 不可靠 | mock ARGSSEHub |
| 守门 #6 PowerShell | PowerShell only |
| ARG.1 G-1 MemGraph stub | 接受 stub 返回, 不报错 |
| ARG.3 占位 BehaviorEvaluator / OutputEvaluator | 替换为完整实现 (不重写 8 拓扑) |

---

## 5. Deliverable

- `crates/arg-effect/src/achievement_engine/behavior_evaluator.rs` (7 行为)
- `crates/arg-effect/src/achievement_engine/output_evaluator.rs` (5 产出)
- `crates/arg-effect/src/achievement_engine/mod.rs` (3 evaluator 并行)
- 5 新增 tests
- 1 脚本
- 2 文档更新
- 1 报告
- 1 commit

---

## 6. Token 预算

- 软预算 2M, v0.50 估 4-6M
- 实际预期 3-4M
- 触发熔断 7M
- 超出记录 (per 2026-09-10 用户拍板"超预算可接受")

---

## 7. 子代理执行守门

- 不调任何 RPC
- 不用 RPC 派子代理
- 不调 git push
- 不写无 git 实证叙事
- 每步必跑守门

---

## 8. 父会话职责

- 接收子代理 final report
- self-review 子代理产出
- 跑守门 #1 v3 + v25
- merge 走 `git merge --no-ff wt-arg-08-behavior-output-evaluator`
- WBS 升版
- 触发 ARG.9 (PHASE-ARG-IMPL-REPORT.md 收官报告)

---

## 9. 失败接手

| 失败 | 接手 |
|---|---|
| 子代理 RPC 失败 | task_stop, 父会话接手 (per 守门 #9 实证 #7, ARG.6 验证成功) |
| cargo test 失败 | 本地 fix, 不重试超 2 次 |
| 12 IT < 100% | 必补到 100% |
| 守门 #7 unsafe 触发 | 移除 |
| 跨 crate 循环 import | 必用 trait 抽象拆解 |
| Token 超 7M | task_stop, 报告 |

---

## 10. 引用

- `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 §3.2.4 + §4.9 + §6-§7
- `docs/architecture/2026-09-03-arg/07-arg-03-5-sa-impl.md` v0.50 (428 行, 5 module + 9 SA + 8 拓扑 Cypher + 10 challenges + L0↔L1 PyO3)
- `docs/reports/STAR-P3-WBS-001.md` v0.57 §14.11 ARG.8
- `crates/arg/` ARG.1 commit `651117e` (ARGError + AchievementUnlock 类型)
- `crates/arg-bridge/` ARG.2 commit `87e1618` (ARGSSEHub 5 协议)
- `crates/arg-effect/` ARG.3 commit `f1207e2` (AchievementEngine 骨架 + 8 拓扑)
- `crates/api/src/arg/` ARG.4 commit `1d894ab` (ARGSSEHub wire)
- `docs/briefs/arg-01-arg-crate-skeleton.md` (模板)
- `AGENTS.md` §4 守门 + §3 7 段报告
- `docs/automation-design.md` v0.1
- `scripts/automation/registry.md` v0.6+
