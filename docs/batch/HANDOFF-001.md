# HANDOFF-BATCH-001: domain-batch session 1 收尾任务清单 v0.1

> **Status**: 🟡 Draft v0.1 (2026-09-01 19:43 JST Mavis 起草, 当前 session 收尾时落地)
> **修订人**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses
> **触发**: per [WBS-001 v0.1 §5 跨 session HANDOFF 计划](./WBS-001-domain-batch.md) + 2026-09-01 19:43 JST Ulysses 拍板 next-wbs-detail-now + commit `a8fb5b6` v0 phase 1 骨架 + commit `aeaf213` ADR-0040
>
> **dual-use 警告 (per AGENTS.md §5 v0.6 + 2026-08-31 22:45 JST Q1-D 拍板)**: 5 域 (player/economy/match/social/admin) 是 RGS 仓历史治理命名, Star 仓不建立业务子域↔DDD 映射; domain-batch 5 域视图是**业务层**视图, 跟 22 crate DDD bounded context 正交.

---

## §1 当前 session 收尾 (token 估 ~0.1M)

### 1.1 落地任务 (current session 末)

1. ☐ **H1.1** 写 [WBS-001 v0.1](./WBS-001-domain-batch.md) (估 0.05M, ✅ done per 当前)
2. ☐ **H1.2** 写本 HANDOFF-BATCH-001 (估 0.01M, ✅ done per 当前)
3. ☐ **H1.3** 验证 v0 phase 1 骨架守门 #1 v1+v2+v3 仍 0 err + 10/10 test pass (per `cargo test -p domain-batch --lib` re-run)
4. ☐ **H1.4** git status check: docs/batch/WBS-001-domain-batch.md + HANDOFF-001.md 落地; AGENTS.md / spec / BATCH-REQ-001 review 草稿不 commit
5. ☐ **H1.5** 报告 + ask_user session 收尾 (commit WBS+HANDOFF vs 保持 review 草稿)

### 1.2 跨 session 续做入口 (下次 session 启动时)

1. **WB-2.1** T5 NodeExecutor trait + 5 runtime_kind 实现 (per [domain-batch-spec §9 T5](../specs/domain-batch-spec.md))
2. **WB-2.2** T6 DagOrchestrator trait + 拓扑排序 + 并行/串行
3. **WB-2.3** T7 Scheduler trait + cron + 事件触发
4. **WB-2.4** T8 状态机/重试/幂等/取消 + ADR-0030 Lease 复用

## §2 session 2 启动任务清单 (token 估 ~0.2M, per WBS-001 §5)

### 2.1 续 session 启动 step-by-step

1. **Step 1** `git log --oneline -10` 确认当前 HEAD (per [HANDOFF-ST-001 v0.3 §2.1](../../../..\..\reports\HANDOFF-ST-001.md))
2. **Step 2** 读 [WBS-001 v0.1](./WBS-001-domain-batch.md) §1-§2 确认 phase 1+2 子任务列表
3. **Step 3** 读 [domain-batch-spec v0.1 §4 Port trait + §9 T5~T6](../specs/domain-batch-spec.md) 确认 5 Port trait 签名 + 5 节点类型 enum
4. **Step 4** 跑 `cargo check -p domain-batch --all-targets` 验证 v0 phase 1 骨架 0 err (per 守门 #1 v2 派生)
5. **Step 5** 跑 `cargo test -p domain-batch --lib` 验证 10/10 test pass (per 守门 #1 v3 派生)
6. **Step 6** 写 `src/runtime/mod.rs` (5 runtime_kind 分发入口)
7. **Step 7** 写 `src/runtime/domain_service.rs` (调 33 domain `service::action` 走 `star_context` ActorContext)
8. **Step 8** 写 `src/runtime/mcp_tool.rs` (调 MCP tool 走 [ADR-0032 Streamable HTTP](../architecture/2026-08-26-upgrade/adr/0032-mcp-transport-stdio.md))
9. **Step 9** 写 `src/runtime/http.rs` (reqwest)
10. **Step 10** 写 `src/runtime/shell.rs` (tokio::process + non-root + 沙箱 per [ADR-0025](../architecture/2026-08-26-upgrade/adr/0025-vendor-adapter-anti-contamination.md))
11. **Step 11** 写 `src/runtime/sql.rs` (sqlx + per-tenant db role per [BATCH-REQ-001 §3.6 F-054](../requirements/batch-001.md))
12. **Step 12** 写 `src/orchestrator.rs` (DagOrchestrator 实现 + 拓扑排序 + Kahn 算法无环检测)
13. **Step 13** 单元测试 (per [domain-batch-spec §9 T12](../specs/domain-batch-spec.md) 0.08M)
14. **Step 14** 集成测试 (DAG 跑通, 简单 3 节点, 估 0.04M)
15. **Step 15** 跑守门 #1 v1+v2+v3 三过 (cargo check --workspace --all-targets 0 err + 单元测试 + 集成测试)
16. **Step 16** 写 HANDOFF-BATCH-002 (per §5 跨 session 续)

### 2.2 验证 / 守门 (per 守门 #1 派生 v1+v2+v3)

- [ ] `cargo check --workspace --all-targets` 0 err
- [ ] `cargo fmt -p domain-batch` clean
- [ ] `cargo clippy --workspace --lib -- -D warnings` 0 err
- [ ] `cargo test -p domain-batch --lib` 100% pass
- [ ] `cargo test -p domain-batch --test '*'` 100% pass (集成测试)
- [ ] 5 runtime_kind 各 ≥1 单测
- [ ] DAG 拓扑无环检测 (3 节点 → 1 节点 闭环 → 422 BA-006)
- [ ] `batch_event` append-only 验证 (T9 后续 phase)

### 2.3 commit 守门 (per 守门 #1+#9+#12)

- 守门 #1: 实装后必跑 cargo check 0 err + clippy 0 err + test 100% pass
- 守门 #9: 自己写, 不走子代理 (per 守门 #9 实证 RPC 不可靠)
- 守门 #12: commit author = Ulysses <ulysses@mavis.local> + commit message 完整 + 不沿用旧叙事
- 守门 #15: 距离饱和 100 commit, 安全推进

## §3 已知缺口 (per 缺标比错标安全)

| # | 缺口 | 影响 | 状态 | 触发 |
|---|------|------|------|------|
| GAP-H1-01 | 33 domain `service::action` 实际签名未在 v0 phase 1 spec 列出 | WB-2.1 step 7 写 `domain_service.rs` 时需先 verify 33 domain 各自的 service trait | 🟡 v0 phase 2 启动时 | session 2 step 7 |
| GAP-H1-02 | MCP tool 16 个的实际 name + input schema 未在 v0 phase 1 spec 列出 | WB-2.1 step 8 写 `mcp_tool.rs` 时需先 verify 16 tool | 🟡 v0 phase 2 启动时 | session 2 step 8 |
| GAP-H1-03 | sqlx 0.7+ 跟 [BATCH-REQ-001 §3.6 F-054](../requirements/batch-001.md) per-tenant db role 集成 | WB-2.1 step 11 写 `sql.rs` 时需先 verify sqlx RLS 集成 | 🟡 v0 phase 2 启动时 | session 2 step 11 |
| GAP-H1-04 | shell 沙箱化 (J-BA-03 per [domain-batch-spec §12](../specs/domain-batch-spec.md)): nsjail / runc / bubblewrap 选择 | WB-2.1 step 10 写 `shell.rs` 时需先选 sandbox | 🟡 v0 phase 2 启动时, 估 +0.1M | session 2 step 10 |
| GAP-H1-05 | 拓扑排序 (Kahn vs DFS) 库选择 | WB-2.2 step 12 写 `orchestrator.rs` 时需先选算法 | 🟡 v0 phase 2 启动时 | session 2 step 12 |
| GAP-H1-06 | 5 域 (player/economy/match/social/admin) 视图隔离 (per INV-BA-10) 在 NodeExecutor 如何注入 | WB-2.1 step 6-11 各 runtime 需带 `domain` 字段 (per BATCH-REQ-001 §3.4 F-036) | 🟡 v0 phase 2 启动时 | session 2 step 6 |

## §4 风险 (per R-WB 风险表 + H2-EXT 实证)

| # | 风险 | 缓解 |
|---|---|---|
| R-H1-1 | session 2 估 0.2M, 实际可能 0.6-1.0M (per H2 0.3-0.5M 估 → 1.1-1.6M 实测 3-5x 超支) | 拆 2 sub-session: 2.1-2.8 (runtime) + 2.9-2.15 (orchestrator + test), 走 HANDOFF-BATCH-002 + HANDOFF-BATCH-003 |
| R-H1-2 | GAP-H1-04 shell 沙箱化 (nsjail / runc / bubblewrap) 选型复杂 | v0 phase 2 走 `tokio::process` + 白名单命令 + non-root (per INV-BA-08), 沙箱化推到 v0 phase 2 末期 + GAP |
| R-H1-3 | 33 domain service 调用跨域 API 兼容性 | 走 v0 phase 1 已 commit `star_context::ActorContext`, 端口 trait 模式, 不直连 |

## §5 修订历史

| 版本 | 日期 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| v0.1 | 2026-09-01 | 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses | 初版: §1 当前 session 收尾 5 任务 + §2 session 2 启动 16 step 清单 + §3 6 已知缺口 + §4 3 风险 + 0.2M token 估 | 2026-09-01 19:43 JST Ulysses 拍板 next-wbs-detail-now + WBS-001 v0.1 |

---

> **session 1 收尾触发**: 报告 + ask_user 拍板 (commit WBS-001 + HANDOFF-001 vs 保持 review 草稿)
