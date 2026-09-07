# UAT 落地手册 (UAT Runbook)

> **Status**: 🟢 Active
> **Created**: 2026-09-07 15:00 JST (per 9/7 14:30 JST 用户发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档")
> **Authority**: 架构师 (Mavis 接手 agent per DEC-008) — Mavis 接手代签 Ulysses (per 8/27 19:39 JST + 21:59 JST 三次强化)
> **For**: 落地 UAT 测试的 SRE / 平台 / 5 域 Lead / 评审主持 / PM 5 角色必读

---

## 0. 目的

本文档是 UAT 测试的落地手册, 覆盖:

- **跑 UAT 流程** (本地 / CI / k3s)
- **50+ fixture 一键生成** (per 守门 #19 Python 化)
- **25 业务场景 一键跑** (5 regression 脚本)
- **结果解读** (per 守门 #13 W/T/M 分类)
- **故障排查** (常见 4 错误)
- **引用** (test-design v0.8 / STAR-P4-OPT-WBS-001)

**触发**: 2026-09-07 14:30 JST Ulysses 发令
**基线**: 当前 main HEAD = `eb5a967` (per 9/7 14:35 JST)
**守门**: #1 v19 `cargo check --workspace --all-targets -j 4` 0 err 32.27s 实证
**关联**: `docs/uat-design.md` v0.1

---

## 1. 前置条件

### 1.1 工具依赖

| 工具 | 版本 | 用途 | 安装 |
|---|---|---|---|
| **Node.js** | ≥ 20 LTS | 跑 Playwright e2e | `nvm install 20` 或 `nodejs.org` |
| **pnpm** | ≥ 8 | frontend 依赖 | `npm install -g pnpm` |
| **Python** | ≥ 3.11 | fixture generator | `python.org` 或 `py -3.11` |
| **Cargo** | ≥ 1.74 | backend 编译 | `rustup.rs` |
| **Playwright** | ≥ 1.45 | 浏览器自动化 | `pnpm install` (frontend) |
| **MSW** | ≥ 2.0 | mock backend | `pnpm install` (frontend) |
| **bash** | ≥ 4.0 (WSL) | 跑 regression | WSL 或 Git Bash |

### 1.2 环境变量

> **守门 #5 (per 8/27 11:06 JST hard ban)**: 禁打印 env secret
> **推荐**: 用 `.env.local` (gitignored) 存 secret, 不直接 `Get-ChildItem env:`

```bash
# .env.local (示例, 永不入 commit)
STAR_ACTOR_SESSION_ID="session-2026-09-07-001"
```

### 1.3 当前 main HEAD 验证

```bash
cd D:\Star
git log --oneline -1
# 期望输出: eb5a967 Merge branch 'feat/opt-stub-impl-47' (per OPT-WORKER-09)
```

---

## 2. 跑 UAT 流程

### 2.1 本地开发

```bash
# Step 1: 拉最新 main
cd D:\Star
git fetch origin
git checkout main
git pull origin main

# Step 2: 跑 cargo check 0 err 守门 (per 守门 #1 v19)
cargo check --workspace --all-targets -j 4
# 期望: 0 err 32.27s (实证 per 9/7 bg_5e16e817)

# Step 3: 跑 frontend 依赖
cd frontend
pnpm install --frozen-lockfile
# 期望: pnpm-lock.yaml 一致, 0 err

# Step 4: 跑 Playwright e2e (6 spec, 含 UAT 件套 1-6)
pnpm test:e2e
# 期望: 12 spec 全过 (6 现有 + 6 新增)
```

### 2.2 CI (GitHub Actions, per PR #12)

```bash
# CI 9/9 pass 实证 (per PR #12 commit 0c447c5 + 76baafb)
# 4 enforced: cargo check + cargo test -p star-context --lib -j 4 + clippy + cargo doc
# 6 advisory: cargo fmt + cargo test --release + frontend typecheck/test/build + ...
# per守门 #26 v2 反转 4 守门 (per 9/5 00:15 JST 拍板)
```

### 2.3 k3s (Phase F+ 5 域 Lead 真人到位后)

> **当前状态**: 5 域 Lead 真人到位前, Mavis 临时代签 (per 9/3 19:35 JST 拍板 D 维持)
> **部署**: k3s + envoy (per 9/1 13:03 JST 偏好 nginx→envoy + 13:05 JST envoy 独立 deployment)

```bash
# 5 域 Lead 真人到位后:
kubectl apply -f tools/star-flash-mock/k3s/uat-deployment.yaml
# 期望: 5 域 Lead 真人 review §6 后启动
```

---

## 3. 50+ fixture 一键生成

> **位置**: `tools/star-flash-mock/mock_data/uat/`
> **守门**: #19 Python 化 (per docs/automation-design.md v0.1)

### 3.1 5 个 _generate_uat_*.py 脚本

| 脚本 | 范围 | 文件数 |
|---|---|---|
| `_generate_uat_s01_s05.py` | S01-S05 (WorkItem/Worktree/Agent/Feedback/Validation) | 25 (5 × 5) |
| `_generate_uat_s06_s10.py` | S06-S10 (Validation-fail/Conflict/Rebase/MR/TMO-merge) | 25 |
| `_generate_uat_s11_s15.py` | S11-S15 (TMO M-N2..M-N6) | 25 |
| `_generate_uat_s16_s20.py` | S16-S20 (TMO M-N7 + Streamable HTTP 4) | 25 |
| `_generate_uat_s21_s25.py` | S21-S25 (5 域 AC + 多租户 + RBAC + 审计 + 配额) | 25 |

### 3.2 一键跑全部

```bash
cd D:\Star
py -3 tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_s01_s05.py
py -3 tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_s06_s10.py
py -3 tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_s11_s15.py
py -3 tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_s16_s20.py
py -3 tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_s21_s25.py
# 期望输出: 5 行 "X-XX fixtures generated (5 scenarios × 5 files = 25 files)"
```

### 3.3 幂等性 (idempotent)

每个 _generate_uat_*.py 跑多次, 输出文件**覆盖**当前内容 (per当前实装, 不检查存在)
**改进 (P2)**: 跑前检查 fixture_version+v1 是否匹配, 匹配则 skip (per `_generate_30_five_domain_fixtures.py` 模式)

---

## 4. 25 业务场景 一键跑

> **位置**: `tools/star-flash-mock/mock_data/uat/regression/`
> **守门**: 5 regression 脚本跨 W/T/M 分类守门

### 4.1 5 个 uat-sXX-sXX.sh 脚本

| 脚本 | 范围 | 守门 |
|---|---|---|
| `uat-s01-s05.sh` | S01-S05 (基础流程) | W/T/M 分类 + 25 fixture 完整性 |
| `uat-s06-s10.sh` | S06-S10 (验证 + TMO 基础) | W/T/M + TMO M-N1 stash_append_only |
| `uat-s11-s15.sh` | S11-S15 (TMO M-N2..M-N6) | TMO 7 节点 + 守门 #13 a L1↔L1 禁止 |
| `uat-s16-s20.sh` | S16-S20 (TMO M-N7 + Streamable) | TMO M-N7 + Streamable HTTP 4 + SSE format |
| `uat-s21-s25.sh` | S21-S25 (5 域 AC + 多租户 + RBAC + 审计 + 配额) | raci_4_dim + WORM |

### 4.2 一键跑全部

```bash
cd D:\Star
bash tools/star-flash-mock/mock_data/uat/regression/uat-s01-s05.sh
bash tools/star-flash-mock/mock_data/uat/regression/uat-s06-s10.sh
bash tools/star-flash-mock/mock_data/uat/regression/uat-s11-s15.sh
bash tools/star-flash-mock/mock_data/uat/regression/uat-s16-s20.sh
bash tools/star-flash-mock/mock_data/uat/regression/uat-s21-s25.sh
# 期望: 5 行 "==== UAT 业务场景 X-X regression test PASSED ===="
```

### 4.3 跑单个场景 (e.g. S10 TMO merge)

```bash
# 仅验 S10 fixture 完整性
ls -la tools/star-flash-mock/mock_data/uat/scenarios/S10-tmo-merge/
# 期望: 5 文件 (request.json / expected_response.json / expected_db_state.json / expected_events.json / ac_mapping.md)

# 仅跑 S06-S10 regression
bash tools/star-flash-mock/mock_data/uat/regression/uat-s06-s10.sh
```

---

## 5. 结果解读 (per 守门 #13 W/T/M 分类)

### 5.1 PASS 输出

```text
==== UAT 业务场景 1-5 (S01-S05) regression test ====

--- 跑 S01-S05 _generate_uat_s01_s05.py ---
S01-S05 fixtures generated (5 scenarios × 5 files = 25 files)

--- 验证 S01-S05 fixture 完整性 (5 场景 × 5 文件 = 25) ---
  [OK] S01-workitem-create: 5 files
  [OK] S02-worktree-create: 5 files
  [OK] S03-agent-running: 5 files
  [OK] S04-feedback-loop: 5 files
  [OK] S05-validation-pass: 5 files

--- W/T/M 分类守门 (per 守门 #13) ---
  [OK] S01-workitem-create: Transaction (append-only)
  [OK] S03-agent-running: Transaction (append-only)
  [OK] S04-feedback-loop: Transaction (append-only)
  [OK] S02-worktree-create: Master (SCD Type 2)
  [OK] S05-validation-pass: Work (短 TTL)

==== UAT 业务场景 1-5 regression test PASSED ====
```

### 5.2 W/T/M 分类解读

| 分类 | 含义 | 实证 (S01-S25) |
|---|---|---|
| **Work** | 短 TTL 作業中, 物理删除 / タイマー失効 | S05 (1d) / S06 (1d) / S07 (1h) / S25 (1d) |
| **Transaction** | 业务事实 / 監査 / Append-only | S01/S03/S04/S08/S09/S10-S15/S17-S22/S24 (17 场景) |
| **Master** | 参考 / 設定 / 慢変 SCD Type 2 | S02/S16/S23 (3 场景) |

> **派生规 (per 守门 #13 a/b/c/d)**:
> - (a) W = 物理删除 / タイマー失効 / 短 TTL 明示 retention
> - (b) T = 物理删除禁止 + 監査必須 + RLS 13 類必携
> - (c) M = 物理删除禁止 + SCD Type 2 + RLS 13 類必携
> - (d) Master 100% RLS / Transaction 100% audit / Work 100% retention_period

### 5.3 TMO 7 节点分类解读

| 节点 | 含义 | 派生规 |
|---|---|---|
| **M-N1 merge** | 合并 2+ 任务卡 | 守门 #13 d stash_append_only |
| **M-N2 split** | 拆分任务卡 | 守门 #13 d + context_half 策略 |
| **M-N3 reorder** | 拓扑排序重排 | cycle_detected=False |
| **M-N4 bulk** | 批量操作 | partial_failure 3-of-5 |
| **M-N5 summarize** | 摘要压缩 | compression_ratio=0.071 |
| **M-N6 reassign** | 跨 SA-01..SA-10 重派 | checkpoint_handoff=True |
| **M-N7 metadata** | 元数据操作 | Master SCD Type 2 + rls_13_classes_attached |

---

## 6. 故障排查 (常见 4 错误)

### 6.1 错误 1: `python: command not found` (WSL / Git Bash)

**症状**:
```text
wsl: ...
N/eA c localhost ...
/bin/bash: line 1: python: command not found
```

**原因**: WSL / Git Bash 默认 `python` 命令不存在
**解决**:
```bash
# 方案 1: 用 python3
which python3  # 应输出 /usr/bin/python3

# 方案 2: 用 py launcher (Windows only)
where py  # 应输出 py.exe 路径

# 方案 3: 设环境变量 PYTHON=python3
export PYTHON=python3
bash tools/star-flash-mock/mock_data/uat/regression/uat-s01-s05.sh
```

### 6.2 错误 2: TMO 节点 fixture 缺 L1↔L1 描述

**症状**:
```text
[FAIL] S15-tmo-reassign: 守门 #13 a L1↔L1 禁止 描述缺失
```

**原因**: TMO fixture ac_mapping.md 漏 "L1↔L1 禁止" 字符串
**解决**:
```bash
# 1. 编辑 S15 fixture
# tools/star-flash-mock/mock_data/uat/scenarios/S15-tmo-reassign/ac_mapping.md
# 加 "L1↔L1 禁止" 描述

# 2. 编辑 _generate_uat_s11_s15.py gen_s15_tmo_reassign() 函数
# 在 ac_mapping.md 字段加 "L1↔L1 禁止"

# 3. 重跑
py -3 tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_s11_s15.py
bash tools/star-flash-mock/mock_data/uat/regression/uat-s11-s15.sh
```

### 6.3 错误 3: 25 业务场景 fixture 数量不足

**症状**:
```text
[FAIL] S10-tmo-merge: 4 files (expected 5)
```

**原因**: 某个 fixture 文件缺失 (request/response/db_state/events/ac_mapping 之一)
**解决**:
```bash
# 1. 看哪些文件缺失
ls -la tools/star-flash-mock/mock_data/uat/scenarios/S10-tmo-merge/

# 2. 重跑对应生成器
py -3 tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_s06_s10.py

# 3. 重跑 regression
bash tools/star-flash-mock/mock_data/uat/regression/uat-s06-s10.sh
```

### 6.4 错误 4: 5 域 Lead CONTENT 4 维守门失败

**症状**:
```text
[FAIL] S21: raci_4_dim 缺 decision_scope
```

**原因**: S21 fixture 漏 raci_4_dim 全 4 维 (decision_scope / raci / timeline / mavis_sign_boundary)
**解决**:
```bash
# 1. 编辑 S21 fixture expected_response.json
# 加 raci_4_dim 全 4 维 (per 守门 #14 v2)

# 2. 重跑生成器
py -3 tools/star-flash-mock/mock_data/uat/scripts/_generate_uat_s21_s25.py

# 3. 重跑 regression
bash tools/star-flash-mock/mock_data/uat/regression/uat-s21-s25.sh
```

---

## 7. 引用

### 7.1 文档引用

| 文档 | 引用章节 |
|---|---|
| `docs/uat-design.md` v0.1 | 5 域 + 6 关键流程 + 25 业务场景 + 守门 0 违反 |
| `docs/test-design.md` v0.8 | §6.1 MVP 测试矩阵 + §7 测试场景 + §22-§27 各类守门 |
| `docs/test-design.md` v0.8 §27.3.1 | 6 关键流程锚定 (UAT 件套 1-6) |
| `docs/test-design.md` v0.8 §22 | 16 MCP tool (per AGENTS §7 #1 + ADR-0032) |
| `docs/test-design.md` v0.8 §23 | Streamable HTTP (per AGENTS §7 #3) |
| `docs/test-design.md` v0.8 §24 | DB W/T/M 強制分類 (per 守门 #13) |
| `docs/test-design.md` v0.8 §25 | 5 域 Lead 4 维 (per 守门 #3 + #14) |
| `docs/architecture/2026-09-03-langgraph/02-basic-design.md` v0.2 | §2.6 TMO 7 节点 + §3 9 SA Type |
| `docs/architecture/2026-08-26-upgrade/adr/0043-audit-onboarding-failed.md` | WORM audit.onboarding.failed |
| `docs/architecture/2026-08-26-upgrade/adr/0046-langgraph-task-management-operations.md` | TMO 决策 |
| `docs/recruitment/5-business-domain-lead-referral.md` v0.1 | 5 域 Lead 真人到位 timeline |
| `docs/reports/STAR-P4-OPT-WBS-001.md` | OPT 阶段 WBS |
| `docs/automation-design.md` v0.1 | 守门 #19 Python 化 + 任务卡自动化档 |
| `docs/data-design/ipa-detail/00-CLASSIFICATION-W-T-M.md` v0.2 | W/T/M 強制分類基线 |
| `docs/briefs/OPT-WORKER-12-uat.md` | 本任务完整 brief |

### 7.2 守门引用

| 守门 | 拍板 | 派生规 |
|---|---|---|
| #1 v3 | 持续 | check+fmt+clippy 不替代 e2e |
| #1 v19 | 2026-08-29 JST | `cargo check --workspace --all-targets -j 4` 0 err 32.27s |
| #3 v2 | 2026-09-03 11:35 JST 拍板 B 反转 | Mavis 临时代签 5 域 Lead 决策 |
| #11 | 2026-08-26 JST | 缺标比错标安全 |
| #12 | 2026-08-26 JST | 禁回溯叙事 |
| #13 | 2026-09-01 18:30 JST | DB 三類横展開強制分類 |
| #14 v2 | 2026-09-03 19:43 JST | 5 域 Lead CONTENT 4 维 |
| #19 | 2026-09-02 00:39 JST | agent 交互 Python 化 |
| #20 | 持续 | 拆 commit 派生规 (1 per 件套, 3 commit total) |
| #10 | 2026-08-27 07:16 JST | commit author = `Ulysses <ulysses@mavis.local>` |

### 7.3 件套引用

| 件套 | 文件 | commit 顺序 |
|---|---|---|
| 件套 1 | `frontend/e2e/{worktree-creation-flow, five-domain-feedback-loop, tmo-merge-task-flow, streamable-http-reconnect, mcp-16-tool-coverage, uat-business-acceptance}.spec.ts` | Commit 1 |
| 件套 2 | `tools/star-flash-mock/mock_data/uat/{scenarios/, scripts/, regression/}` (125 fixture + 5 脚本 + 5 regression) | Commit 2 |
| 件套 3 | `docs/{uat-design.md, uat-runbook.md}` | Commit 3 |

---

## 7. 修订历史

| v | 修订人 | 修订内容 | 触发 |
|---|---|---|---|
| v0.1 | Ulysses（一人公司 12 角色 per DEC-008）— Mavis 接手 (per 守门 #10 + 19:39 JST 授权) | 初始版本: 前置 / 跑流程 / 一键生成 / 一键跑 / 结果解读 / 故障排查 / 引用 7 段 | 2026-09-07 14:30 JST user 发令 "补充更新 playwright 测试脚本, 专门增设 UAT 测试的 mock 项目内容以及配套文档" |
