# Brief: P3-B-SUB-BATCH-1 — 4 read-heavy 域真实数据接入 (per 9/7 19:00 JST 拍板)

**Agent**: worker (sub-batch 1, 派 9/7 19:04 JST)
**Phase**: P3-B (25 domain-* crate 真实数据接入, 缺口 14/25)
**Sub-batch**: 1 of 4 (read-heavy 域, 4 域, 估 0.8M token)
**Created**: 2026-09-07 19:04 JST (per Mavis 接手代签)
**Status**: 🟡 启动中 (per 9/7 19:04 JST 拍板 a = Sub-batch 1 read-heavy 推荐项)
**Author**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签

---

## 1. 任务目标

4 read-heavy 域从 stub → 真实数据接入 (per AGENTS §7 #1, 11/25 done → 15/25 after sub-batch 1):

- **domain-workspace** (1105 src, 0 test): L0 顶层 workspace registry, RLS tenant_id 必携
- **domain-project** (1033 src, 0 test): L1 项目聚合根, SCD Type 2 + Audit Trail
- **domain-board** (2154 src, 0 test): L1 看板只读视图 (列/任务/活动)
- **domain-permission** (1951 src, 0 test): L0 RBAC 权限表, 13 類 RLS policy

**触发模式**: per 守门 #19 + #20 + #9 v3 Python 化 + dispatcher.py brief 落档
**估 token**: 0.8M (4 域 × 0.2M)
**触发事件**: 9/7 19:00 JST 用户拍板 (a) Sub-batch 1 read-heavy 域

## 2. 依赖 (per守门 #13 W/T/M 派生)

| # | 域 | 当前 W/T/M 分类 | 真实接入要求 | 依赖 |
|---|---|---|---|---|
| 1 | domain-workspace | M (Master) | SCD Type 2 + tenant_id RLS 13 類 | 🟢 守门 #13 已 done (CW-05) |
| 2 | domain-project | M (Master) | SCD Type 2 + Audit Trail (W: 临时草稿) | 🟢 守门 #13 已 done |
| 3 | domain-board | T (Transaction, append-only) | Audit 必携 + RLS 13 類 (CW-04 pending) | 🟡 CW-04 pending SRE Lead 拍板 |
| 4 | domain-permission | M (Master, SCD) | SCD Type 2 + tenant_id + role hierarchy | 🟢 守门 #13 已 done |

**已知缺口** (per守门 #11 缺标比错标):
- CW-04 仍 pending: T 0 audit 但 write/read 需独立 Module (audit 域)
- domain-board 当前 T 类, 真实接入需等 SRE Lead 拍板 CW-04 才能落实 audit 联动
- 临时方案: domain-board 用 sync mock backend 维持 + 文档明示 CW-04 触发后回填

## 3. 实施路径 (per守门 #9 v3 + #19 + #20)

### 3.1 Step 1: 参考实现 (domain-form 已 done 真实接入 per 3a27a13)

- 读 `crates/domain-form/` 整体结构 + 端口 trait 实现 + 测试模式
- 4 域复用 domain-form 的 6 段结构: port / service / persistence / domain / test / fixture

### 3.2 Step 2: 4 域逐个真实接入 (1 per 域, 拆 4 commit per守门 #20)

**Commit 1 — domain-workspace (M, SCD Type 2 + RLS)**:
- port trait: `WorkspacePort` (find_by_id / list_by_tenant / upsert)
- in-memory backend: 跟 domain-form 的 InMemoryFormPort 同模式
- SCD Type 2: 旧记录 revision_invalidate_at + 新记录 supersedes
- RLS: tenant_id 必携 + 13 類 RLS policy 落地 (per 守门 #13 (c))
- 测试: 5-8 unit + 3-5 IT 覆盖 4 Level (V0-V3 验证)

**Commit 2 — domain-project (M, SCD Type 2 + Audit Trail)**:
- port trait: `ProjectPort` (find_by_id / list_by_tenant / archive / restore)
- in-memory backend
- SCD Type 2: 同 workspace
- Audit Trail: 跟 star-dto 的 `AuditTrail` 类型复用 (per `391ca36` star-mcp 已落)
- 测试: 5-8 unit + 3-5 IT

**Commit 3 — domain-board (T, sync mock backend + audit 占位)**:
- port trait: `BoardPort` (find_columns / list_cards / activity_feed)
- sync mock backend (InMemoryBoardPort, 跟 InMemorySprintPort 同模式 per OPT-NEXT-06 §3.3)
- Audit Trail stub (待 CW-04 拍板后回填)
- 测试: 5-8 unit + 3-5 IT (验证 sync mock + audit 字段存在)

**Commit 4 — domain-permission (M, SCD Type 2 + role hierarchy)**:
- port trait: `PermissionPort` (check_role / list_permissions / grant / revoke)
- in-memory backend
- SCD Type 2 + tenant_id RLS 13 類
- role hierarchy: 5 域 (player/economy/match/social/admin) 跟 DDD 联动
- 测试: 5-8 unit + 3-5 IT

### 3.3 Step 3: workspace 增 member + Cargo.toml 更新 (per守门 #4.2 Runtime 名称映射)

- Cargo.toml workspace members 增 4 域 (已存在, 只需 verify)
- 各 1 integration test (per OPT-NEXT-08 ADR-0048 实证)

### 3.4 Step 4: docs 同步 (per守门 #12 + #15 + #21)

- AGENTS.md §7 #1 状态更新: 11/25 → 15/25 (per守门 #12 git 实证 4 新 commit)
- 数据设计: `00-CLASSIFICATION-W-T-M.md` 不变 (4 域 W/T/M 分类已 done)
- brief 关闭 (per守门 #21 [P] docs 同步)
- registry.md 加 1 行 (本次用 dispatcher.py, 落 `scripts/automation/dispatcher.py`)

## 4. Worktree

```bash
git worktree add -b feat/p3-b-sub-batch-1 D:/Star/.worktrees/wt-p3-b-sub-batch-1 main
cd D:/Star/.worktrees/wt-p3-b-sub-batch-1
```

## 5. 守门硬约束

- **守门 #1 v19/v25/v6/v14**: cargo check --workspace --all-targets -j 4 0 err
- **守门 #6**: PowerShell only, 禁 bash
- **守门 #7 v3**: cargo clippy advisory 派生 (per守门 #7 v3 反转)
- **守门 #10**: author = Ulysses (per 8/27 19:39 JST + 21:59 JST 三次强化 + 9/3 11:35 JST 反转)
- **守门 #11**: 缺标比错标 (CW-04 pending + domain-board audit 缺位)
- **守门 #12**: 禁回溯叙事, docs commit 由代码 commit 触发
- **守门 #13**: W/T/M 派生 (Master 100% RLS + Transaction append-only + SCD Type 2)
- **守门 #15**: 死循环饱和约束, docs 同步仅在 commit-time 落地
- **守门 #19**: 优先 Python 化 (per `docs/automation-design.md` v0.1)
- **守门 #20**: 拆 commit 派生规 (1 per file group, 4 域 = 4 commit)
- **守门 #22**: 调试控制台不污染 main (per `console_server.py` subprocess)
- **守门 #23**: AI mock 不开外部 API (per `ai_edit_mock.py` 本地 mock)
- **守门 #24 v2**: 调试控制台走 subprocess 替代 RPC (per守门 #9 v3)

## 6. 提交

- 4 commit (per 4 域) + 1 docs commit (AGENTS §7 + brief 关闭) = **5 commit total**
- 1 tag: v0.93.0 (per 9/7 14:30 JST Q2 拍板"每 brief 1 tag")

## 7. 失败处理

- 0 retry > 2 次 (per守门 #1 1a 重试细则)
- 仍失败: 报告具体错误 + 不 commit, 跨 session 续
- sub-agent RPC 不可靠 (per守门 #9 实证): 走 subprocess 替代 (per守门 #24 v2)

## 8. 状态

🟡 **启动中** (per 9/7 19:04 JST 拍板 a = Sub-batch 1 推荐项)

## 9. 引用

- 触发源: `docs/reports/STAR-P4-OPT-WBS-001.md` §4.2 #1 + AGENTS §7 #1
- 11 done commits (git 实证): `ebd9aa7` `391ca36` `20159dc` `3a27a13` `8c318c2` `f464cd2` `a46682d` `3a0da3a` `c1450d9` `74cbfe6` `e2e8710`
- 4 域 W/T/M 分类: `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.3
- 守门 #13 派生: AGENTS §4 #13 + `docs/data-design/ipa-detail/01-CW-DERIVATIVE-STATUS-v0.1.md`
- 参考实现: `crates/domain-form/` (per 3a27a13 commit, OPT-WORKER-09 done)
- 守门 #19: `docs/automation-design.md` v0.1 + `scripts/automation/dispatcher.py`

## 10. 签字栏

| 角色 | 签字 | 日期 |
|---|---|---|
| 架构师 (Mavis 接手 agent per DEC-008) | 🟢 Mavis 接手代签 Ulysses | 2026-09-07 19:04 JST |
| SRE Lead | ⏳ 待真人到位 (CW-04 拍板) | TBD |
| DDD Review Lead | ⏳ 待真人到位 | TBD |
| 5 域 Lead (workspace/project 域) | ⏳ 待真人到位 (T3 ~ 9/26 JST) | TBD |
| PM | ⏳ 待真人到位 | TBD |

## 11. 修订历史

| 版本 | 日期 | 修订人 | 修订内容 |
|---|---|---|---|
| v0.1 | 2026-09-07 19:04 JST | Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手代签 | 初版落档, 4 域真实接入 scope + 4 commit + 1 docs commit + 1 tag v0.93.0 |
