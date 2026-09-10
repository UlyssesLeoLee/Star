# ARG.6 Brief — 30 UT 完整落地 (依 DD §10.1)

> **Task ID**: arg-06-30-ut
> **WT**: `wt-arg-06-30ut` (branch: `wt-arg-06-30ut`, base: main @ b89391e 含 ARG.1+ARG.2+ARG.3+ARG.4+ARG.5 merge)
> **触发**: 2026-09-10 08:14 JST 用户发令"按顺序推进" + ARG.5 merge 走守门后父会话自驱
> **依赖**: ARG.1-5 全部已 merge ✅ (crates/arg + crates/arg-bridge + crates/arg-effect + crates/api/src/arg + frontend/agent-relationships)
> **拍板来源**: per WBS-001 v0.50 §14.11 ARG.6 (2M tokens / 0.3 周) + v0.50 跨 session 续做规划 (估 3-4M / 2-3 session)
> **守门合规**: #1 v15 (第 47 次新事件) + #1 v25 (cargo test -p star-arg --lib -j 4 100% pass) + #3 + #5 + #6 + #7 + #9 + #10 + #12 + #13 + #14 v3 + #19 v19

---

## 1. Objective

**ARG.6 子项**: 跨 4 crate ARG 测试覆盖率从 47 UT (ARG.1+ARG.2+ARG.3 已落) 扩到 30+ UT (per DD §10.1 完整列表) — 实装阶段 P3-D W1 任务, 提升覆盖率.

ARG.6 完成后:
- P3-D W1 收官
- 4 crate ARG UT 数量: arg 30 → ≥30 + bridge 13 → ≥10 + effect 18 → ≥18 + api 13 → 13 IT (已有)
- 守门 #1 v25 cargo test -p star-arg* 全部 100% pass

---

## 2. Scope (per DD §10.1)

### 2.1 必做 (per brief 缺标比错标)

#### A. `crates/arg/` 补 UT (per DD §10.1.1 UT-01..UT-32)

当前已有 32 UT (per ARG.1 实证, 5+8+5+6+4+2+2=32). 跟 DD §10.1.1 列表对齐 (24 + 4 trust_score + 2 cypher_cache + 2 event = 32). 已 100% 覆盖, **本子项不重复 ARG.1 已有 UT**, 只做覆盖率验证.

#### B. `crates/arg-bridge/` 补 UT (per DD §10.1.2 UT-25..UT-34)

当前已有 13 UT (per ARG.2 实证, 3 listener + 3 flush + 4 offline + 1 langgraph + 2 extras = 13). 跟 DD §10.1.2 列表 (10) 对齐, **已超 30%**, **本子项不重复 ARG.2 已有 UT**, 只做覆盖率验证.

#### C. `crates/arg-effect/` 补 UT (per DD §10.1.3 UT-35..UT-52)

当前已有 47 UT (per ARG.3 实证, 18 named + 28 lib + 1 extra = 47). 跟 DD §10.1.3 列表 (18) 对齐, **已超 160%**, **本子项不重复 ARG.3 已有 UT**, 只做覆盖率验证.

#### D. 集成级测试 (新增, per §2.2)

虽然单个 crate UT 已 100% pass, ARG.6 主目标是**集成级 30 UT 落地**:

**D.1. 跨 crate 集成 (10 UT)**
- test_arg_bridge_to_arg_ffi_consistency
- test_arg_effect_to_arg_bridge_5_reducer_protocol
- test_arg_api_routes_call_arg_ops
- test_arg_seed_fixture_loads_in_arg
- test_arg_8_topology_cyphers_avoid_apoc (已有 per ARG.3)
- test_arg_10_challenge_prompts_unique_keys (已有 per ARG.3)
- test_arg_event_variants_compatible (已有 per ARG.1)
- test_arg_3_reducer_semantics (已有 per ARG.2)
- test_arg_dispatch_uses_arg_edge_ops (已有 per ARG.3)
- test_arg_5_team_templates_match_bd (新增)

**D.2. MemGraph stub 协议 (5 UT)**
- test_memgraph_stub_returns_502_on_write
- test_memgraph_stub_returns_empty_on_read
- test_memgraph_bolt_pool_health_check
- test_memgraph_cypher_cache_lru_eviction
- test_memgraph_client_constructor_env_only

**D.3. 错误处理 (5 UT)**
- test_arg_error_10_variants
- test_arg_bridge_error_5_variants
- test_arg_effect_error_inherits_arg
- test_arg_validation_self_loop_blocked
- test_arg_validation_weight_range

**D.4. SCD Type 2 + RLS 13 类 (5 UT)**
- test_arg_agent_version_increments_on_update
- test_arg_edge_version_increments_on_update
- test_arg_audit_append_only_no_update_no_delete
- test_arg_rls_tenant_id_required
- test_arg_template_instance_ttl_30_days

**D.5. 守门 v3 跨 sub-session 收敛 (5 UT)**
- test_arg_crate_singleton_compile
- test_arg_bridge_imports_arg_without_cycle
- test_arg_effect_imports_arg_bridge_without_cycle
- test_arg_api_imports_arg_without_cycle
- test_arg_full_workspace_compile_no_cycles

**新增总计: 30 UT (10+5+5+5+5)** — 落地到 `tests/integration/` 或各 crate `tests/` 已有目录, 跨 4 crate.

#### E. 1 脚本 (per 守门 #1 v19 [S])

- `scripts/automation/arg_30ut_test.py` v0.1 (~150 行):
  - 跑 `cargo test --workspace -p star-arg -p star-arg-bridge -p star-arg-effect --tests -j 4`
  - 跑 `python scripts/automation/arg_*.py` 全部 IT
  - 30 UT 全过 (含新增 30 + 既有 92 = 122 总)
  - 报告 5/5 守门 exit code

#### F. 2 文档更新

- `docs/automation-design.md` §4 追加 ARG.6 行
- `scripts/automation/registry.md` §1 + §5 索引追加

#### G. 1 报告

- `docs/reports/PHASE-ARG-06-IMPL-REPORT.md` v0.1

#### H. 1 commit + merge

- 1 commit: `feat(arg-30ut): 跨 4 crate ARG 集成 30 UT + 1 Python 脚本 (ARG.6 子项, P3-D W1)`
- author = `Ulysses <ulysses@mavis.local>`

### 2.2 不做

- 不写 e2e test (ARG.7 后续, Playwright 8 E2E)
- 不写 PT 性能测试 (ARG.7 后续, 4 PT 性能指标)
- 不写 ARG.8 行为/产出成就 evaluator

---

## 3. Acceptance Criteria

### AC-1: 编译守门
- [ ] `cargo check --workspace --lib -j 4` exit 0
- [ ] `cargo fmt --all -- --check` exit 0
- [ ] `cargo clippy --workspace --lib -j 4 -- -D warnings` exit 0
- [ ] `cargo test -p star-arg --tests -j 4` exit 0
- [ ] `cargo test -p star-arg-bridge --tests -j 4` exit 0
- [ ] `cargo test -p star-arg-effect --tests -j 4` exit 0
- [ ] `cargo build --release -p star-arg -p star-arg-bridge -p star-arg-effect` exit 0

### AC-2: 集成 30 UT 100% pass (新增 30 + 既有 92 = 122 总)
- [ ] D.1 跨 crate 集成 10 UT 100% pass
- [ ] D.2 MemGraph stub 协议 5 UT 100% pass
- [ ] D.3 错误处理 5 UT 100% pass
- [ ] D.4 SCD Type 2 + RLS 13 类 5 UT 100% pass
- [ ] D.5 守门 v3 跨 sub-session 收敛 5 UT 100% pass

### AC-3: 自动化档守门
- [ ] `scripts/automation/arg_30ut_test.py` 存在
- [ ] `docs/automation-design.md` §4 追加 1 行
- [ ] `scripts/automation/registry.md` 追加 1 行

### AC-4: 守门 v25
- [ ] `cargo test -p star-arg --lib -j 4` exit 0
- [ ] workspace 兼容: `cargo check --workspace --lib -j 4` 0 err
- [ ] author = Ulysses

---

## 4. 已知约束

| 约束 | 处理 |
|---|---|
| 守门 #1 v25 单 crate 模式 | `cargo test -p <crate> --lib -j 4` 100% pass |
| 守门 #7 unsafe | 0 unsafe 块 |
| 守门 #10 代签 | Ulysses |
| 守门 #9 RPC | mock / stub |
| 跨 crate 集成 UT | 落在 `crates/<crate>/tests/integration/` (新增目录) |

---

## 5. Deliverable

- 30 新增 UT (5 类 × 各 5-10 UT) 跨 4 crate
- 1 脚本
- 2 文档更新
- 1 报告
- 1 commit

---

## 6. Token 预算

- 软预算 2M, v0.50 估 3-4M
- 实际预期 2-3M
- 触发熔断 5M
- 超出记录 (per 2026-09-10 用户拍板)

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
- self-review
- 跑守门 #1 v3 + v25
- merge 走 `git merge --no-ff wt-arg-06-30ut`
- WBS 升版
- 触发 ARG.7 (10 IT + 8 E2E + 4 PT)

---

## 9. 失败接手

| 失败 | 接手 |
|---|---|
| 子代理 RPC 失败 | task_stop, 父会话接手 |
| 30 UT < 100% pass | 必补到 100% |
| 守门 #7 unsafe | 移除 |
| 跨 crate 循环 import | 必用 trait 抽象拆解 |
| Token 超 5M | task_stop, 报告 |

---

## 10. 引用

- `docs/design/DD-AGENT-RELATIONSHIP-001.md` v0.1.1 §10.1 (32 + 10 + 18 = 60 UT 列表)
- `docs/reports/PHASE-ARG-01-IMPL-REPORT.md` v0.1 (32 UT 已落)
- `docs/reports/PHASE-ARG-02-IMPL-REPORT.md` v0.1 (13 UT 已落)
- `docs/reports/PHASE-ARG-03-IMPL-REPORT.md` v0.1 (47 UT 已落, 含 18 named + 28 lib)
- `docs/reports/STAR-P3-WBS-001.md` v0.50 §14.11
- `crates/arg/` ARG.1 commit `651117e`
- `crates/arg-bridge/` ARG.2 commit `87e1618`
- `crates/arg-effect/` ARG.3 commit `f1207e2`
- `crates/api/src/arg/` ARG.4 commit `1d894ab`
- `frontend/(app)/agent-relationships/` ARG.5 commit `b89391e`
- `docs/briefs/arg-01-arg-crate-skeleton.md` (模板)
- `AGENTS.md` §4 守门 + §3 7 段报告
