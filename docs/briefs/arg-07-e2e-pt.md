# ARG.7 Brief — 10 IT + 8 E2E + 4 PT 端到端实装

> **Task ID**: arg-07-e2e-pt
> **WT**: `wt-arg-07-e2e-pt` (branch: `wt-arg-07-e2e-pt`, base: main @ cee5459 含 ARG.1-6 merge)
> **触发**: 2026-09-10 09:30 JST 用户发令"要" (per ARG.6 收官后父会话自驱继续推进)
> **依赖**: ARG.1-6 全部已 merge ✅ (crates/arg + crates/arg-bridge + crates/arg-effect + crates/api/src/arg + frontend/agent-relationships + 30 集成 UT)
> **拍板来源**: per WBS-001 v0.56 §14.11 ARG.7 (3M tokens / 0.5 周) + v0.50 跨 session 续做规划 (估 5-7M / 4-5 session)
> **守门合规**: #1 v15 (第 49 次新事件) + #1 v19 ([M] 必先 `scripts/automation/arg_e2e_test.py`) + #1 v25 + #3 + #5 + #6 + #7 + #9 + #10 + #12 + #13 + #14 v3 + #19 v19

---

## 1. Objective

落地 ARG 端到端测试套件: **10 IT + 8 E2E + 4 PT** = 22 端到端测试, 跨 3 层 (Rust backend + Python IT + Playwright E2E + 性能 PT).

ARG.7 完成后:
- P3-D W2 完成
- 守门 #1 v3 跨 sub-session 收敛 0 错 实证
- ARG Rust 栈从"代码完成" → "端到端可观测可压测"

---

## 2. Scope (per DD-AGENT-RELATIONSHIP-001.md v0.1.1 §10.2-§10.4)

### 2.1 必做

#### A. 10 IT (integration test, 跨 crate 调用, 已有 8 升级 + 2 新增, per DD §10.2)

`scripts/automation/arg_e2e_test.py` v0.1 (per 守门 #1 v19 [M]):
- 10 IT 跑完整 ARG 栈, 端到端验证 5 类协作影响:
  1. `test_drag_create_delegates_edge` (E2E 模拟拖拽建 delegates_to 边)
  2. `test_dispatch_route_e2e` (Lead → Worker 自动 dispatch, 节省 token ≥ 30%)
  3. `test_consults_e2e` (关键决策时自动调 Reviewer, 决策矩阵可追溯)
  4. `test_collaborates_parallel_e2e` (2 agent 並行, wall-clock 节省 ≥ 20%)
  5. `test_stand_in_fallback_e2e` (Worker A failed → Worker B 接管, 任务不中断)
  6. `test_trust_skip_verify_e2e` (trusts 边 weight ≥ 0.7 + trust_score ≥ 0.8 跳过 verify, 节省 token ≥ 15%)
  7. `test_achievement_unlock_e2e` (跨 5 域全连接 → TOP-001 解锁, SSE 推送)
  8. `test_offline_reconnect_e2e` (Memgraph 不可达 → sled 缓存 → 重连 flush)
  9. `test_5_relationship_types_create_e2e` (delegates/consults/collaborates/reports/trusts 各 1 条边, 验证 10 类关系都能建)
  10. `test_template_instantiate_e2e` (Hub-and-Spoke 5 agent → 实例化, 验证 4 边自动建)

#### B. 8 E2E (Playwright 浏览器测试, frontend, per DD §10.3)

`frontend/e2e/arg-relationships.spec.ts` v0.1:
- 8 E2E 跑 frontend 5 UI 组件 + zustand 5 channel:
  1. `test_relationship_editor_drag` (拖拽 2 agent 建边, 选 EdgeTypeSelector)
  2. `test_relationship_view_zoom_pan` (图谱缩放/平移)
  3. `test_node_detail_panel` (点节点显示 trust_score + edge 数)
  4. `test_achievement_wall_filter` (按稀有度 COMMON/RARE/EPIC/LEGENDARY 筛选)
  5. `test_template_gallery_instantiate` (5 模板卡片, 点 Hub-and-Spoke)
  6. `test_websocket_event_push` (建边后 UI 实时更新, 不需刷新)
  7. `test_5_team_templates_render` (5 模板缩略图都显示)
  8. `test_20_achievements_display` (20 成就 4 稀有度分组显示)

#### C. 4 PT (performance test, per DD §10.4)

`scripts/automation/arg_perf_bench.py` v0.1:
- 4 PT 性能指标 (per SRS NFR-ARG-01..04):
  1. `test_edge_create_latency` (边创建 P95 < 200ms, 1k 边批量)
  2. `test_cypher_query_latency` (Cypher 查询 P95 < 500ms, ≤ 1k 节点)
  3. `test_event_push_latency` (WebSocket 推送 < 100ms, 10 并发客户端)
  4. `test_achievement_eval_latency` (成就评估 P95 < 1s, 1 万边规模)

#### D. 2 文档更新

- `docs/automation-design.md` §4 追加 ARG.7 行
- `scripts/automation/registry.md` §1 + §5 索引追加

#### E. 1 报告

- `docs/reports/PHASE-ARG-07-IMPL-REPORT.md` v0.1

#### F. 1 commit + merge

- 1 commit: `feat(arg-e2e-pt): 10 IT + 8 E2E + 4 PT 端到端测试套件 (ARG.7 子项, P3-D W2)`
- author = `Ulysses <ulysses@mavis.local>` (per 守门 #10)

### 2.2 不做

- 不写 ARG.8 行为/产出成就 evaluator (后续)
- 不写 ARG.9 收官报告 (后续)
- 不写 frontend build 集成 (ARG.5 已落地, 本任务只跑 e2e)
- 不写 真实 Memgraph Bolt 客户端 (ARG.1 G-1 缺口, 仍 stub path, 接受 stub 返回)

---

## 3. Acceptance Criteria

### AC-1: 端到端测试套件落地
- [ ] `scripts/automation/arg_e2e_test.py` v0.1 10 IT 端到端通过
- [ ] `frontend/e2e/arg-relationships.spec.ts` v0.1 8 E2E 端到端通过
- [ ] `scripts/automation/arg_perf_bench.py` v0.1 4 PT 性能指标达成

### AC-2: 4 维度守门 (per 守门 #1 v19 [M] 4 维打分)
- [ ] R (Rerunnable): 3 个测试脚本可重复跑, exit code 0
- [ ] V (Volume): 10 + 8 + 4 = 22 端到端测试
- [ ] S (Structural): 3 类别 (IT / E2E / PT) + 3 工具 (Python requests / Playwright / Python psutil)
- [ ] A (Audit-trail): 报告含 22 测试列表 + pass/fail 详情 + 守门合规

### AC-3: 性能指标达成 (per SRS NFR-ARG-01..04)
- [ ] 边创建 P95 < 200ms (1k 边)
- [ ] Cypher 查询 P95 < 500ms (1k 节点)
- [ ] WebSocket 推送 < 100ms (10 并发)
- [ ] 成就评估 P95 < 1s (1 万边)

### AC-4: 5 守门实证 (per 守门 #1 v25)
- [ ] `cargo check --workspace --lib -j 4` 0 err
- [ ] `cargo test -p star-arg* --lib -j 4` 0 err
- [ ] `python scripts/automation/arg_e2e_test.py` exit 0
- [ ] `pnpm playwright test frontend/e2e/arg-relationships.spec.ts` 8/8 pass
- [ ] `python scripts/automation/arg_perf_bench.py` 4/4 性能指标达成

### AC-5: 守门 v25
- [ ] workspace 兼容: 0 回归
- [ ] author = Ulysses
- [ ] 不推 origin

---

## 4. 已知约束

| 约束 | 处理 |
|---|---|
| 守门 #1 v19 [M] | 落 `arg_e2e_test.py` |
| 守门 #7 unsafe | 0 unsafe 块 |
| 守门 #10 代签 | Ulysses |
| 守门 #9 RPC 不可靠 | mock Memgraph + mock WebSocket |
| 守门 #6 PowerShell | 部署脚本 PowerShell only |
| 守门 #5 env 安全 | 不打印 secret |
| ARG.1 MemGraph stub | 接受 stub 返回, 不报错 |
| Playwright 浏览器 | 用 headless 模式 (per existing pnpm 栈) |
| MemGraph 不可达 | e2e 用 mock server (per 守门 #9 RPC 不可靠) |

---

## 5. Deliverable

- `scripts/automation/arg_e2e_test.py` v0.1 (~200 行, 10 IT)
- `frontend/e2e/arg-relationships.spec.ts` v0.1 (~150 行, 8 E2E)
- `scripts/automation/arg_perf_bench.py` v0.1 (~120 行, 4 PT)
- 2 文档更新
- 1 报告
- 1 commit

---

## 6. Token 预算

- 软预算 3M (per WBS-001 v0.18 §14.11 ARG.7), v0.50 估 5-7M
- 实际预期 4-5M
- 触发熔断 8M
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
- merge 走 `git merge --no-ff wt-arg-07-e2e-pt`
- WBS 升版
- 触发 ARG.8 (7 行为 + 5 产出成就 evaluator)

---

## 9. 失败接手

| 失败 | 接手 |
|---|---|
| 子代理 RPC 失败 | task_stop, 父会话接手 (per 守门 #9 实证 #7, ARG.6 验证成功) |
| 10 IT 失败 | 必补到 100% |
| 8 E2E 失败 | 必补到 100% |
| 4 PT 性能不达标 | 必优化或显式列 G-ARG7-1..4 缺口 |
| Token 超 8M | task_stop, 报告 |

---

## 10. 引用

- `docs/requirements/SRS-AGENT-RELATIONSHIP-001.md` v0.1
- `docs/design/BD-AGENT-RELATIONSHIP-001.md` v0.1
- `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 §10.2 + §10.3 + §10.4
- `docs/architecture/2026-09-03-arg/09-arg-05-frontend-e2e.md` v0.50 (411 行, 8 E2E 详细)
- `docs/reports/STAR-P3-WBS-001.md` v0.56 §14.11 ARG.7
- `crates/arg-bridge/` ARG.2 commit `87e1618`
- `crates/arg-effect/` ARG.3 commit `f1207e2`
- `crates/api/src/arg/` ARG.4 commit `1d894ab`
- `frontend/(app)/agent-relationships/` ARG.5 commit `b89391e`
- 30 集成 UT ARG.6 commit `f6e98ec`
- `docs/briefs/arg-01-arg-crate-skeleton.md` (模板)
- `AGENTS.md` §4 守门 + §3 7 段报告
- `docs/automation-design.md` v0.1
- `scripts/automation/registry.md` v0.6+
