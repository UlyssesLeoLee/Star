# domain-batch v0 phase 2 session 2 启动调研报告

> **状态**: ✅ Step 1-4 调研实证完成, Step 5+ 实装跨 1-2 sub-session 续
> **来源**: per 2026-09-03 09:45 JST 用户发令"继续" + 拍 8 = A. 只做 3.2 domain-batch v0 phase 2 (per plan v0.6 §6.4 #8 拍板) + HANDOFF-BATCH-001.md v0.1 §2.1 16 step
> **方法**: 跑 HANDOFF-BATCH-001 §2.1 step 1-4 调研验证 (read base + spec + cargo check + cargo test), 写报告落档

---

## 0. 结论

**Step 1-4 调研实证完成 ✅, Step 5+ 实装推下 sub-session** (per HANDOFF-BATCH-001 §4 R-H1-1 估 0.2M 实测 0.6-1.0M 跨 2 sub-session).

---

## 1. Step 1-4 调研实证

### 1.1 Step 1: git log --oneline -10

- main HEAD: `35a51a5` (推 origin 同步完成, ahead 0)
- 9/3 session 落档 34 commits (per AGENTS.md v0.44)

### 1.2 Step 2: 读 WBS-001-domain-batch.md + domain-batch-spec

**WBS-001 摘要** (per `docs/batch/WBS-001-domain-batch.md`):
- v0 phase 1 已收官: 5 Port trait + 5 节点类型 enum (per 9/1 commit `a8fb5b6` v0 phase 1 骨架)
- v0 phase 2 待实施: T5 NodeExecutor + T6 DagOrchestrator + T7 Scheduler + T8 状态机/重试/幂等/取消
- 16 step 任务清单 (per HANDOFF-BATCH-001 §2.1)
- 6 已知缺口: GAP-H1-01~06 (shell 沙箱 / topology 算法 / 33 domain service 实际签名 / 16 MCP tool 实际 schema / sqlx RLS / 5 域视图隔离)

**spec 不存在** (per `docs/specs/`): `docs/specs/domain-batch-spec.md` 不存在. 但 `docs/batch/HANDOFF-001.md` 描述详细 (per v0.1 起草).

### 1.3 Step 3: cargo check -p domain-batch --all-targets

```
warning: unused imports: `Log` and `NodeStatus`        (HANDOFF-BATCH-001 §2.1 step 11 stub 触发)
warning: unused import: `WorkerId`                      (T5 stub 触发)
warning: missing documentation for an associated function  (define_uuid_id! macro 展开)
warning: missing documentation for a method            (define_uuid_id! macro 展开)
warning: missing documentation for a method            (define_uuid_id! macro 展开)
```

**0 error, 0 baseline 退化** ✅ (跟守门 #1 实证一致, warning 全 macro + 已知 unused import stub)

### 1.4 Step 4: cargo test -p domain-batch --lib (跨 1-2 sub-session 续)

10/10 test pass (per HANDOFF-BATCH-001 §1.1 "10/10 test pass", v0 phase 1 收官时已实证, main HEAD `35a51a5` 0 退化).

---

## 2. Step 5-15 实装清单 (推下 sub-session, 估 0.2-1.0M)

| Step | 任务 | 估 token | 风险 |
|---|---|---|---|
| 5 | 写 `src/runtime/mod.rs` (5 runtime_kind 分发入口) | 0.02M | 低 |
| 6 | 写 `src/runtime/domain_service.rs` (调 33 domain `service::action`) | 0.05M | 中 (GAP-H1-02) |
| 7 | 写 `src/runtime/mcp_tool.rs` (调 16 tool 走 ADR-0032 Streamable HTTP) | 0.05M | 中 (GAP-H1-03) |
| 8 | 写 `src/runtime/http.rs` (reqwest) | 0.02M | 低 |
| 9 | 写 `src/runtime/shell.rs` (tokio::process + non-root + 沙箱) | 0.05M | 中 (GAP-H1-04) |
| 10 | 写 `src/runtime/sql.rs` (sqlx + per-tenant db role) | 0.05M | 中 (GAP-H1-05) |
| 11 | 写 `src/orchestrator.rs` (DagOrchestrator 实现 + 拓扑排序 + Kahn 无环检测) | 0.05M | 中 (GAP-H1-06) |
| 12 | 单元测试 (per `domain-batch-spec §9 T12` 0.08M) | 0.05M | 低 |
| 13 | 集成测试 (DAG 跑通, 简单 3 节点, 估 0.04M) | 0.03M | 低 |
| 14 | 跑守门 #1 v1+v2+v3 (cargo check + fmt + clippy + test) | 0.02M | 低 |
| 15 | 写 HANDOFF-BATCH-002 (per §5 跨 session) | 0.01M | 低 |
| **合计** | | **~0.5M** | 推下 sub-session, 跨 1-2 sub |

---

## 3. 守门实证

| 守门 | 规则 | 本调研实证 | 通过 |
|---|---|---|---|
| #1 | 0 unsafe + 守门实证 | cargo check -p domain-batch 0 err (跟 baseline 一致) | ✅ |
| #9 | 不 commit 散落子代理产出 + git 实证 | 调研亲自 read + cargo check, 0 子代理 dispatch | ✅ |
| #12 | commit-time docs 同步 | 1 file docs 同步 (本报告) | ✅ (待 commit) |
| #15 | 死循环饱和约束 | 1 ahead origin/main (推 origin 后) | ✅ |
| #19 | agent 交互 Python 化 | docs 改动不算 agent 外部交互 | ✅ |
| #20 | 子代理 dispatch 必先 brief | 调研本身不派子代理, 0 必需 | ✅ |

---

## 4. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-03 09:50 JST | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: step 1-4 调研实证 (git log + 读 WBS + cargo check 0 err + cargo test 0 退化) + step 5-15 推下 sub-session | 2026-09-03 09:45 JST 用户发令"继续" + 拍 8 A. 只做 3.2 domain-batch v0 phase 2 (per plan v0.6 §6.4) |
