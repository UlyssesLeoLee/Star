# Implementation Plan: PLAN-031 — v0.99.4 重新评估后续 跨 session 续做 (per WBS-001 v0.99.4 row)

> **RFC**: 无(本 plan 是 v0.99.4 row 落档后续,无新 RFC)
> **Owner**: Mavis 自驱(per 守门 #9 v19 Mavis 自驱第 7 次强化)
> **状态**: 🟡 Draft v0.1 (per 2026-09-10 21:23 JST)
> **创建日期**: 2026-09-10
> **最后更新**: 2026-09-10
> **相关 WBS**: `docs/reports/STAR-P3-WBS-001.md` v0.99.4 row (commit `e92215b`)
> **相关 Plan**: plan-016 ~ plan-030 (前置)
> **关联守门**: v33 / v34 active, v35 / v36 候选, v32 (Mavis 审核 author=Ulysses)

---

## 目标 (Goals)

基于 v0.99.4 重新评估,落地 11 缺口显式列的待办项。共 **7 phase** (按工作量分),估总 **~8-10 SRE·周 / 6-8M tokens** (本 session 后续跨 session)。

1. **轻量 housekeeping** (Phase A, ~15 min / 0.05M tokens)
2. **守门 v33 落地闭环** (Phase B, ~30 min / 0.2M tokens)
3. **PreToolUse v0.2 sandbox** (Phase C, ~2-3h / 1-2M tokens) — **P0 阻塞**
4. **P3-D.6 任务 1.6 + 1.7 worker 子代理实装** (Phase D + E, ~2-3h / 1-2M tokens) — 走 worktree 模式
5. **P2 阶段 worker 子代理实跑** (Phase F, ~2-3h / 2M tokens) — 等守门 #1 v15 + 4 守门满足
6. **v1.01 已知缺口闭合** (Phase G, ~3-4h / 1.5-2M tokens) — dev env Docker + RLS idempotent 验证 + PLATFORM_ADMIN 部署 + 端到端

## 非目标 (Non-Goals)

1. ❌ 真人寻访流程 (per v0.63 反转 obsolete)
2. ❌ 重写 v0.99.1-v0.99.3 / v1.00-v1.03 WBS row (per 守门 #1 禁回溯叙事)
3. ❌ 守门 v30 / v31 重启 (obsolete 保留,跟 v0.62+v0.63 反转共存)
4. ❌ RGS 仓 / Physis / GVPE 双向同步 (per Ulysses 双仓并行守门)
5. ❌ 任何整体方向大转弯 (per 9/8 15:19 第 6 次强化, 不适用 Mavis 全权代理)

## 7 Phase 详细计划

### Phase A: 轻量 housekeeping (~15 min / 0.05M tokens)

**目标**: 清理 STALE worktree, 落档现有 commit 数据基础

| Task | 估时 | 输出 |
|---|---|---|
| A.1 清理 1 STALE worktree (`wt-ex-02-star-mutex` 不存在 path) | 1 min | `git worktree prune + git branch -D wt-ex-02-star-mutex` |
| A.2 批量清理 44 ACTIVE 已 merge worktree | 5 min | PowerShell loop: `git worktree remove --force` + `git branch -D`, 0 残留 |
| A.3 落档 wbs_v0994_reassess.py 到 docs/ (idempotent 工具) | 1 min | move from `scripts/automation/` → `tools/wbs-sync-tools/` |
| A.4 重新跑全 pytest 验证 0 回归 | 1 min | `pytest scripts/automation/guardian/tests/ -q` 0 fail |
| A.5 git status clean 确认 | 1 min | 0 untracked, 0 modified |
| A.6 1 commit author=Ulysses (per 守门 #1 v15) | 1 min | "chore(cleanup): 50 worktree 清理" |
| A.7 (可选) v34 sustained_heartbeat.py 实装 + 5 TC + 1 commit | 5 min | 守门 v34 落地实施 |

**风险**: 大批量 git worktree remove 误删 — 防御: 1 个 1 个做, 每次做 `git worktree list` 验证。

---

### Phase B: 守门 v33 + v35 + v36 落地闭环 (~30 min / 0.2M tokens)

**目标**: 把守门 v33 已知缺口全修复, v35 + v36 候选拍板激活

| Task | 估时 | 输出 |
|---|---|---|
| B.1 守门 v33 v0.4 实施 GPG 签名 (commit `6d06091` 候选 → 实施) | 15 min | `scripts/automation/guardian/comment_gpg.py` + `dispatcher.py comment()` 集成 + 5 TC |
| B.2 守门 v33 v0.5 实施 audit 索引 (commit `4185dc4` 候选 → 实施) | 10 min | `scripts/automation/guardian/index_builder.py` + `count_by_decision()` 走索引 + 5 TC |
| B.3 守门 v35 拍板激活 (🟡 → 🟢) | 1 min | 改 `v35_gpg_signed_comments.md` Status + 修订历史 row |
| B.4 守门 v36 拍板激活 (🟡 → 🟢) | 1 min | 改 `v36_audit_log_index.md` Status + 修订历史 row |
| B.5 守门 v37 候选落地 (v1.01 已知缺口联动, audit 触发) | 3 min | `docs/guardian/v37_*.md` Draft v0.1 |
| B.6 跑全 pytest 验证 0 回归 | 1 min | 270+ tests pass |
| B.7 1 commit author=Ulysses (per 守门 #1 v15) | 1 min | "feat(guardian): 守门 v33 v0.4 GPG + v0.5 audit 索引 + v35/v36 激活" |

**风险**: GPG key 落地 (per v35 已知缺口 #3) — 需先 `gpg --full-generate-key` 生成 Ulysses + Mavis 2 个 key pair。

---

### Phase C: PreToolUse 守门 v0.2 子代理 sandbox (~2-3h / 1-2M tokens) — **P0 阻塞**

**目标**: SRS 已知缺口 #1 修复 (子代理派出去后内部代码越权)

| Task | 估时 | 输出 |
|---|---|---|
| C.1 评估 sandbox 方案: bwrap (Linux) / docker --read-only / Windows Job Objects | 10 min | 决策: STAR Windows + mavis 现状 → 选 Windows Job Objects (per 守门 #6 PowerShell only) |
| C.2 写 `scripts/automation/guardian/subprocess_sandbox.py` (Windows Job Objects API) | 60 min | ~200 LOC, set JobLimits (kill on close, no network, no write to C:\) |
| C.3 集成到 `dispatcher.py invoke()`: 派子代理前 set JobObject + 子代理进程 attach | 30 min | `subprocess.Popen(..., creationflags=CREATE_SUSPENDED)` + `AssignProcessToJobObject` |
| C.4 写 5+ TC: sandbox 拦截 rm -rf / + GitHub PAT leak + ... | 30 min | `test_subprocess_sandbox.py` 5 TC |
| C.5 跑全 pytest 验证 0 回归 | 5 min | 280+ tests pass |
| C.6 1 commit author=Ulysses (per 守门 #1 v15) | 1 min | "feat(sandbox): PreToolUse 守门 v0.2 子代理 sandbox" |

**风险**: Windows Job Objects 跟 mavis CLI 兼容性 (mavis CLI 尚未落地 per SRS 已知缺口 #2) — 软依赖, fail-open (per FR-6.1)。

---

### Phase D: P3-D.6 任务 1.6 worker 子代理实装 (~1-1.5h / 0.5-0.8M tokens)

**目标**: 14+15 张 SQL DDL 落档 (A11 7 + A12 7 + G11 15 = 29 张表, 0 改 V0.1)

| Task | 估时 | 输出 |
|---|---|---|
| D.1 派 worker 子代理: brief `p3-d6-1-6-15-more-tables.md` + `p3-d6-1-6-p3d6-tables-impl.md` | 1 min | worktree `wt-p3-d6-1-6-15-more-tables` 创建 + brief 落档 |
| D.2 worker 撰写 14+15 张 DDL (W/T/M 6M + 5T + 3W + 1T = 14 张新增) | 45 min | `crates/star-pg-adapter/migrations/<version>_p3d6_*.sql` 14 file |
| D.3 worker 跑守门实证: cargo test -p star-pg-adapter --lib -j 4 + rls_7_policy_gen.py 跨 26 表 idempotent | 10 min | 35+ tests pass + 14+15 tables idempotent OK |
| D.4 Mavis 验证 commit + 跑守门 + merge to main | 5 min | merge `--no-ff` 0 conflict + 5 守门实证 PASS |
| D.5 docs 同步 (per 守门 #12 v21): WBS v0.99.5 row + registry v0.29 + automation-design §4.34.7 | 5 min | 3 docs 同步 |
| D.6 1 commit author=Ulysses (per 守门 #1 v15) | 1 min | "feat(sql): P3-D.6 14+15 tables DDL 落档" |

**风险**: worker RPC 不可靠 (per 守门 #9 实证) — 防御: 守门 #27 RPC fallback 3 段。

---

### Phase E: P3-D.6 任务 1.7 worker 子代理实装 (~1-1.5h / 0.5-0.8M tokens)

**目标**: 25 module 跨接口定型 (worktree + work-item + comment + notification + audit + search + setting + ...)

| Task | 估时 | 输出 |
|---|---|---|
| E.1 派 worker 子代理: brief `p3-d6-1-7-25module-cross.md` | 1 min | worktree `wt-p3-d6-1-7-25module-cross` 创建 |
| E.2 worker 撰写 25 module 跨接口定型 (~跨 6-8 crate) | 60 min | 25 file trait def + 25 file impl stub + 1 file 跨接口集成 |
| E.3 worker 跑守门实证: cargo check + cargo test + cargo fmt + cargo clippy | 10 min | 0 err + 0 warn + 0 diff |
| E.4 Mavis 验证 commit + 跑守门 + merge to main | 5 min | merge 0 conflict + 5 守门实证 |
| E.5 docs 同步: WBS v0.99.6 row + registry v0.30 + automation-design §4.34.8 | 5 min | 3 docs 同步 |
| E.6 1 commit author=Ulysses (per 守门 #1 v15) | 1 min | "feat(modules): P3-D.6 25 module 跨接口定型" |

**风险**: 跨 6-8 crate 集成复杂,容易触发 V0.1 已有代码 regression — 防御: 守门 #19 v19 累积规 + 0 改 V0.1 + 5 守门全套跑。

---

### Phase F: P2 阶段 worker 子代理实跑 (~2-3h / 2M tokens)

**目标**: 触发 v1.02 P2 阶段 brief, 闭合 v1.01 4/5 已知缺口 (dev env Docker + RLS idempotent + PLATFORM_ADMIN + 端到端)

| Task | 估时 | 输出 |
|---|---|---|
| F.1 守门 #1 v15 触发条件验证: docs 同步饱和 + 4 守门满足 | 5 min | 跑 docs/automation-design.md 状态 + v1.01 5 已知缺口 |
| F.2 派 worker 子代理: brief `p2-stage-exec-v1.02.md` | 5 min | worktree `wt-p2-stage-exec` 创建 |
| F.3 worker 实证 P2 阶段: dev env Docker setup + rls_7_policy_gen.py 跑跨 26 表 + PLATFORM_ADMIN password 部署 + 端到端测试 5 守门 | 120 min | 4 守门 0 违反 + 4 已知缺口闭合 |
| F.4 Mavis 验证 commit + merge + docs 同步 | 10 min | 5 守门实证 + WBS v1.04 row + registry v0.31 + automation-design §4.34.9 |
| F.5 1 commit author=Ulysses (per 守门 #1 v15) | 1 min | "feat(p2-stage): worker 子代理实跑, 闭合 v1.01 4 已知缺口" |

**风险**: P2 阶段触发守门 4 个全满足需全部实跑 — 防御: 守门 #9 v19 Mavis 自驱, 任何缺守门都 warn log。

---

### Phase G: v1.01 已知缺口 #1-#4 闭合 (~3-4h / 1.5-2M tokens)

**目标**: 闭合 v1.01 剩余 4 已知缺口 (P2 阶段 worker 可能已完成部分)

| Task | 估时 | 输出 |
|---|---|---|
| G.1 验证 v1.01 #1 dev env Docker (P2 阶段输出) | 10 min | 跑 dev env setup, 验证 26 张表 idempotent + RLS 13 政策生效 |
| G.2 验证 v1.01 #2 rls_7_policy_gen.py 跨 26 表 idempotent (P2 阶段输出) | 30 min | python rls_7_policy_gen.py 跨 26 表, 验证 idempotent |
| G.3 验证 v1.01 #3 PLATFORM_ADMIN password 部署 (P2 阶段输出) | 30 min | kubectl apply SealedSecret + verify |
| G.4 验证 v1.01 #4 端到端测试 5 守门 (P2 阶段输出) | 30 min | cargo test --workspace + kubectl kustomize + 端到端 5 守门 |
| G.5 落 v1.04 row (P2 阶段 + 4 已知缺口全部闭合) | 10 min | WBS v1.04 row + registry v0.32 + automation-design §4.34.10 |
| G.6 1 commit author=Ulysses (per 守门 #1 v15) | 1 min | "docs(wbs): v1.01 5 已知缺口全部闭合" |

**风险**: dev env Docker setup 涉及 host 状态永久改变 (per 守门 #9 v19 不适用, 等 Ulysses 授权) — 防御: 软依赖 + 文档化缺口, 实际 setup 需 Ulysses 显式发令。

---

## Owner 矩阵 (5 角色, per Ulysses 12 角色 per DEC-008)

| 角色 | 负责内容 | 真人到位后追溯 |
|---|---|---|
| **Mavis (root session, 自驱)** | Phase A 全部 + Phase B 全部 + Phase D-F 监督 worker 子代理 + Phase G 验证 | Mavis 审核 author=Ulysses (per 守门 #14 v3 + v4) |
| **Worker 子代理 (sub-session)** | Phase C sandbox 实施 + Phase D 1.6 撰写 + Phase E 1.7 撰写 + Phase F P2 阶段实证 | Mavis 永久代签 (per 守门 #14 v3, 真人到位 obsolete per v0.63 反转) |
| **架构师 (Architect)** | Phase C 方案选型 (sandbox 选 Windows Job Objects) | Mavis 审核 (per 守门 #14 v4) |
| **SRE Lead** | Phase F P2 阶段 dev env Docker + Phase G dev env 验证 | Mavis 永久代签 |
| **PM** | Phase A-G 整体 schedule + 8 commit 顺序 + docs 同步守门 #12 v21 触发 | Mavis 永久代签 |

---

## 时间线 (4-6 周, 估)

| 周 | Phase | 累计 |
|---|---|---|
| W1 (本周剩余) | Phase A (15 min) + Phase B (30 min) | 0.25M tokens |
| W2 | Phase C (2-3h) | 1.5M tokens |
| W3 | Phase D (1-1.5h) | 2.2M tokens |
| W4 | Phase E (1-1.5h) | 2.9M tokens |
| W5 | Phase F (2-3h) | 4.5M tokens |
| W6 | Phase G (3-4h) | 6.0M tokens |

**估总**: 6-8M tokens / 4-6 周, per STAR-OLU-001 换算 = 4-6 SRE·周。

---

## 关键守门落地映射

| 守门 | 落地的 Phase |
|---|---|
| **#1 v15 docs 同步饱和** | 全部 Phase (1 commit 多文件) |
| **#9 v20 子代理 brief 必先** | Phase D + E (worker dispatch) |
| **#9 v27 RPC fallback** | Phase D + E + F (worker RPC) |
| **#11 缺标比错标** | v0.99.4 row 11 缺口显式列, 跨 Phase 闭合 |
| **#12 v21 [P] docs 同步** | 每个 Phase 末尾 (WBS + registry + automation-design) |
| **#13 W/T/M 100% 覆盖** | Phase D 任务 1.6 (新 14 张表) |
| **#14 v3 + v4 Mavis 审核 author=Ulysses** | 全部 commit |
| **#19 v19 累积规不破坏 V0.1** | Phase D + E (worker 不改 V0.1) |
| **v33 任务卡留言** | 全部 worker dispatch (Phase D + E + F) |
| **v34 30min 探活** | root session 跨 session 探活 (per 守门 v34 实施, Phase A.7) |
| **v35 GPG 签名** (候选) | Phase B.1 落地 |
| **v36 audit 索引** (候选) | Phase B.2 落地 |

---

## 风险 + 缓解

| 风险 | 严重度 | 缓解 |
|---|---|---|
| Phase D/E worker RPC 不可靠 (per 守门 #9 实证) | P0 | 守门 #27 RPC fallback 3 段 |
| Phase C Windows Job Objects 跟 mavis CLI 兼容性 (mavis CLI 未落地) | P1 | 软依赖 + fail-open (per FR-6.1) |
| Phase F P2 阶段触发守门 4 守门全满足需实跑 | P0 | 守门 #9 v19 Mavis 自驱, 任何缺守门都 warn log |
| Phase G dev env Docker setup 涉及 host 状态永久改变 | P0 | 不适用 Mavis 自驱, 需 Ulysses 显式发令 |
| 50 worktree 清理误删 | P1 | 1 个 1 个做, 每次 git worktree list 验证 |
| v0.99.4 row 11 缺口闭合时间跨度长 | P2 | 守门 v34 30min 探活跨 session 持续跟踪 |

---

## 关联文档

- `docs/reports/STAR-P3-WBS-001.md` v0.99.4 row (本 plan 触发)
- `docs/recruitment/5-business-domain-lead-referral.md` (obsolete per v0.63)
- `docs/guardian/v33_subagent_comments.md` 🟢 active
- `docs/guardian/v34_sustained_heartbeat.md` 🟢 active
- `docs/guardian/v35_gpg_signed_comments.md` 🟡 候选 (Phase B.3 激活)
- `docs/guardian/v36_audit_log_index.md` 🟡 候选 (Phase B.4 激活)
- `docs/automation-design.md` §4.34.6 任务卡 (本 plan 落地)
- `scripts/automation/registry.md` v0.28 row (本 plan 落地)
- `scripts/automation/wbs_v0994_reassess.py` idempotent 工具 (本 plan 落地)
- `docs/briefs/p3-d6-1-6-15-more-tables.md` (Phase D 输入)
- `docs/briefs/p3-d6-1-6-p3d6-tables-impl.md` (Phase D 输入)
- `docs/briefs/p3-d6-1-7-25module-cross.md` (Phase E 输入)
- `docs/briefs/p2-stage-exec-v1.02.md` (Phase F 输入)

---

## 修订履歴

| バージョン | 日付 | 修订人 | 修订内容 | 触发 |
|---|---|---|---|---|
| **v0.1** | 2026-09-10 21:23 JST | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #14 v3) | 初版落档, 7 段 (目标/非目标/7 phase/owner/时间线/守门映射/风险), 7 phase (A housekeeping + B 守门闭环 + C sandbox P0 + D 任务 1.6 + E 任务 1.7 + F P2 阶段 + G 缺口闭合), 估 6-8M tokens / 4-6 周 / 4-6 SRE·周 (per STAR-OLU-001), 11 关联文档 | 2026-09-10 21:22 JST Ulysses 发令"你下一步计划基于 wbs 制作好, 可以根据量分 phase" |
