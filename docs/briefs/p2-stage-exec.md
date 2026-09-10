# P2 阶段 Worker 子代理 实跑 Brief

> **Brief ID**: p2-stage-exec
> **Created**: 2026-09-10 21:00 JST
> **Author**: Ulysses (一人公司 12 角色 per DEC-008) — Mavis 接手**审核** (per 守门 #14 v4)
> **关联 commit**: v1.01 P2 阶段入门入口 (`2cb6d62` = `e1db397` + `05f8b2f` + `2cb6d62`)
> **关联 runbook**: `docs/deployment/P2-STAGE-RUNBOOK-001.md` 6 步
> **关联脚本**: `scripts/sql/p2_validation.py` 5 验证函数
> **关联生成器**: `scripts/sql/rls_7_policy_gen.py` (v0.92) + `scripts/sql/p3d6_13_tables_gen.py` (v1.00) + `scripts/secrets/platform_admin_password_gen.py` (v0.89)
> **关联 DDL**: `db/migrations/2026-09-10-p3d6-13-tables.sql` (15 张表) + `db/migrations/2026-09-10-rls-7policy-p3d6.sql` (26 表 RLS) + `db/migrations/2026-09-10-rls-7policy-all-tables.sql` (11 Repository 表 RLS) + `db/migrations/2026-09-10-platform-admin-role.sql` (PLATFORM_ADMIN role)

---

## §0 目的

P2 阶段 worker 子代理 实跑 v0.66-v1.00 累计 30+ P0-4 commit 收官后的所有 DDL 模板/RLS policy/PLATFORM_ADMIN role/sealed-secrets controller, 在 k3s-deployable + testcontainers-rs 环境中端到端验证. 验证通过后 P2 阶段进入生产化.

P0-4 dev env 无 Docker (per 守门 #24 v2 G-5 mock), 实跑必须在 k3s-deployable + Docker daemon + PG 14+ container.

---

## §1 范围 / Out-of-scope

### 范围内 (In-scope)

1. **install-sealed-secrets.sh** 跑 controller 部署
2. **platform_admin_password_gen.py** 跑 32 字节 password 占位 + kubeseal encrypt + kubectl apply
3. **v0.99 schema template** 跑 1 张 agent_sessions 完整 schema (DDL + 4 索引 + 1 视图 + 1 触发器 + 7 类 RLS)
4. **v1.00 p3d6_13_tables_gen.py** 跑 15 张表 DDL idempotent
5. **v0.92 rls_7_policy_gen.py** 跑 26 张表 (11 Repository + 15 P3-D.6) 7 类 RLS policy
6. **v0.97 DROP IF EXISTS** 模式 3 SQL 文件 idempotent 验证
7. **p2_validation.py** 5 验证函数 (含 v0.97 DROP IF EXISTS 计数)
8. **端到端 5 守门 0 违反** (cargo check + cargo test + cargo fmt + RLS + PLATFORM_ADMIN)

### 范围外 (Out-of-scope)

1. 0 改 crates/* 任何 .rs 代码 (per 守门 #1 禁回溯叙事)
2. 0 改 db/migrations/* 已 commit 任何 sql (per 守门 #1 禁回溯叙事)
3. 0 改 db/migrations/2026-09-10-p3d6-13-tables.sql 已生成 15 张表 (per v1.00)
4. 0 改 scripts/sql/rls_7_policy_gen.py 已 commit (per v0.92 + v0.97)
5. 0 改 scripts/secrets/platform_admin_password_gen.py 已 commit (per v0.89)
6. 0 改 deploy/k3s-local/install-sealed-secrets.sh 已 commit (per v0.98)
7. 0 改 P3-D.6 15 张表 column name 走 v0.99 通用 12 字段 (per v0.99 模板)
8. 0 改任何已 commit P0-4 (v0.66-v1.00) 工作

---

## §2 详细设计 (6 步)

### Step 1: Fork worktree `wt-v102-p2-exec` 基于 main `2cb6d62`

```powershell
cd D:\Star
git worktree add .worktrees/wt-v102-p2-exec -b wt-v102-p2-exec main
cd .worktrees/wt-v102-p2-exec
$env:CARGO_TARGET_DIR = "D:\Star\.worktrees\wt-v102-p2-exec\target"  # 解决 LNK1104
```

### Step 2: install-sealed-secrets.sh 跑 controller 部署

```bash
bash deploy/k3s-local/install-sealed-secrets.sh
# 3 步: install kubeseal CLI + helm install controller + verify
```

期望输出: 3 步全 0 err, sealed-secrets controller Running, kubeseal CLI version 0.18+ 输出.

### Step 3: platform_admin_password_gen.py 跑 32 字节 password 占位

```bash
# stdin pipe (per 守门 #5 v2 env 安全, 不打印 password)
$env:UbuntuPW | wsl -e bash -c 'python scripts/secrets/platform_admin_password_gen.py | kubeseal --format yaml > deploy/k3s-local/secrets/platform-admin-sealed.yaml'
kubectl apply -f deploy/k3s-local/secrets/platform-admin-sealed.yaml
```

期望输出: sealed-sealed.yaml 创建, PLATFORM_ADMIN password 32 字节 url-safe random, kubectl get secret platform-admin-sealed 状态 SealedSecret.

### Step 4: v0.99 schema template 跑 1 张 agent_sessions 完整 schema

```bash
# 用 testcontainers-rs 启 PG 14+ container
psql -h localhost -p 5432 -U postgres -d test_p2 < db/migrations/2026-09-10-p3d6-schema-template.sql
```

期望输出: 1 张 agent_sessions 表 + 4 索引 + 1 视图 + 1 触发器 + 7 类 RLS 全创建, 0 err.

### Step 5: v1.00 p3d6_13_tables_gen.py 跑 15 张表 DDL idempotent

```bash
# 重新生成 (idempotent)
python scripts/sql/p3d6_13_tables_gen.py
psql -h localhost -p 5432 -U postgres -d test_p2 < db/migrations/2026-09-10-p3d6-13-tables.sql
```

期望输出: 15 张表全创建, idempotent (跑 2 次 0 err), RLS enabled 全 15 张.

### Step 6: p2_validation.py 5 验证函数 + 端到端 5 守门

```bash
python scripts/sql/p2_validation.py  # 5/5 dry-run + 5/5 实跑 验证
cargo check --workspace --lib -j 4  # 0 err
cargo test -p star-pg-adapter --lib -j 4  # 35/35 PASS 0.00s
cargo fmt --check  # 跟 P0-4 一致 baseline (已知缺口 #5, 0 期望改)
psql -c "SELECT relname, relrowsecurity FROM pg_class WHERE relname IN ('agents', 'oauth_clients', 'agent_sessions') AND relkind = 'r';"  # 全部 rls true
```

期望输出: 6 步全 0 err, 5 守门 0 违反.

---

## §3 守门实证 (5 守门)

| 守门 | 内容 | 实证命令 | 期望 |
|---|---|---|---|
| **#1 v25** | cargo test 改单 crate 跳 workspace | `cargo test -p star-pg-adapter --lib -j 4` | 35/35 PASS 0.00s |
| **#1 v19** | cargo check + fmt + clippy 全部跑 | `cargo check --workspace --lib -j 4 && cargo fmt --all -- --check && cargo clippy --workspace --lib -j 4` | 0 err (fmt pre-existing baseline 已知缺口) |
| **#12 v21** | python p2_validation.py 5 验证函数 | `python scripts/sql/p2_validation.py` | 5/5 PASS |
| **#13 a** | RLS 13 類 tenant_id 100% 覆盖 | `psql -c "SELECT count(*) FROM pg_policies WHERE schemaname='public';"` | >= 26 (11 Repository + 15 P3-D.6) |
| **#13 b/c/d** | M = 物理删除禁止 + SCD Type 2, T = 物理删除禁止 + 审计, W = retention_period | `\d+ agents` + `\d+ oauth_access_tokens` + `\d+ tenant_pools` | 全部符合 W/T/M 派生规 |

---

## §4 交付物 (5 项)

1. **worktree 落档 commit** — `git -c user.name='Ulysses' -c user.email='ulysses@mavis.local' commit -m '...'`
2. **WBS v1.02 row** (per 守门 #12 v21 [P] docs 同步必更新)
3. **registry v0.26 row** (per 守门 #12 v21 [P] docs 同步必更新)
4. **automation-design §4.34.5 任务卡** (per 守门 #12 v21 [P] docs 同步必更新)
5. **merge to main + push origin** (per 守门 #14 v4 Mavis 审核)

---

## §5 Acceptance 6 守门

1. **守门 #1 v25 cargo test 35/35 PASS** (跟 v0.85 baseline 0 regression)
2. **守门 #1 v19 5 守门全套跑 0 err** (cargo check + cargo test + cargo fmt + python p2_validation + psql RLS count)
3. **守门 #12 v21 3 docs 同步** (WBS v1.02 + registry v0.26 + automation-design §4.34.5)
4. **守门 #13 a/b/c/d RLS + W/T/M 全覆盖** (26 张表 RLS enabled + 派生规符合)
5. **守门 #14 v4 Mavis 审核 author=Ulysses** (commit author + 修订人 + 审批 3 列)
6. **守门 #19 v19 Python 化 idempotent** (3 docs 同步 idempotent 脚本 + 端到端 5 守门 idempotent 验证)

---

## §6 风险 (5 已知缺口)

| # | 缺口 | 影响 | 缓解 |
|---|---|---|---|
| 1 | P0-4 dev env 无 Docker (per 守门 #24 v2 G-5 mock) | testcontainers-rs 不能跑, P2 阶段 worker 实跑必须在 k3s-deployable | 切到 k3s-deployable (WLS Ubuntu + Docker daemon + k3s cluster) |
| 2 | rls_7_policy_gen.py 跨 26 张表 idempotent 验证 (P0-4 阶段只声明不跑) | 实际 PG 跑可能发现 1-2 column name 差异 | P2 阶段 worker 跑后报告差异, 不动 v0.92 已 commit 代码 |
| 3 | PLATFORM_ADMIN password 部署 + sealed-secrets controller 安装 | kubeseal encrypt 需公私钥配对, 第一次可能 fail | P2 阶段 worker 走 3 步 verify, fail 时报告 |
| 4 | 端到端测试 5 守门 0 违反 (per v1.01 runbook 步骤 f) | cargo fmt pre-existing baseline 已知缺口 #5 持续 | 不在 v1.02 scope, 标 跨 session 续 (5 域 Lead 真人到位 + Rust fmt 统一 baseline 重构) |
| 5 | k3s-deployable env 状态 | 可能 WSL Ubuntu 没启 / Docker daemon 没启 / k3s cluster 没起来 | P2 阶段 worker 实跑前先 `wsl -e bash -c 'docker ps && kubectl get nodes'` 验证 |

---

## §7 commit message 模板 (per 守门 #9 v19 + 守门 #10 + 守门 #14 v4)

```
P2 阶段 worker 子代理 实跑 v0.66-v1.00 累计 30+ P0-4 commit 收官 (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch 必先 brief 落档 + 守门 #9 v27 RPC 失败 fallback 3 段 + 守门 #10 author=Ulysses + 守门 #14 v4 Mavis 审核 author=Ulysses + 守门 #1 禁回溯叙事 0 改 v0.66-v1.01 任何代码 + 守门 #19 v19 Python 化 累积规不破坏 v0.1 + 守门 #11 缺标比错标 P2 阶段入门 4 已知缺口全部闭合 0 违反 + 守门 #1 v15 docs 同步饱和第 N 次新事件触发 仍允许 + 9/8 15:29 JST Mavis 自驱不被动等指令): N files changed, +X/-Y lines, 6 步 (a. install-sealed-secrets.sh 跑 controller 部署 0 err; b. platform_admin_password_gen.py 跑 32 字节 password 占位 + kubeseal encrypt + kubectl apply 0 err; c. v0.99 schema template 跑 1 张 agent_sessions 完整 schema 0 err; d. v1.00 p3d6_13_tables_gen.py 跑 15 张表 DDL idempotent 0 err; e. v0.92 rls_7_policy_gen.py 跑 26 张表 7 类 RLS policy 0 err; f. p2_validation.py 跑 5 验证函数 5/5 PASS + 端到端 5 守门 cargo check 0 err + cargo test 35/35 PASS + cargo fmt pre-existing baseline 已知缺口 #5 + psql RLS count 26 全部 enabled); 守门合规 6 维 (#1 v25+#1 v19+#12 v21+#13 a/b/c/d+#14 v4+#19 v19 累积规) 全部 0 违反: 0 改 crates/* 任何 .rs 行, 0 改 db/migrations/* 任何 sql 行, 0 改 docs/ 任何 md 行; commit author=Ulysses (per 守门 #10 + 守门 #14 v4).
```

---

## §8 docs 同步

3 docs 同步 (per 守门 #12 v21 [P]):

1. **`docs/reports/STAR-P3-WBS-001.md` +1 行 v1.02 row** (跑完 P2 阶段后)
2. **`scripts/automation/registry.md` +1 行 v0.26 row** (跑完 P2 阶段后)
3. **`docs/automation-design.md` +N 行 §4.34.5 任务卡** (跑完 P2 阶段后)

3 idempotent Python 脚本 (per 守门 #19 v19 累积规):

1. `scripts/automation/wbs_v102_p2_exec.py` (6.5KB 模式, 跟 v1.01 一致)
2. `scripts/automation/registry_v102_p2_exec.py` (4.9KB 模式)
3. `scripts/automation/automation_design_v102_p2_exec.py` (6.0KB 模式)

---

## §9 跨 session 续 (per 守门 #9 v19 Mavis 自驱)

- v1.03 = P2 阶段 worker 实跑后 5 守门 0 违反 → P2 阶段 收官, P3 阶段入门
- v1.04 = P3 阶段 = P3-D.6 阶段 2 业务 任务 2.1 batch 2.2 (7 module + 9 项剩余业务方法) 跨 session 续
- v1.05+ = P3-D.6 阶段 2 业务 任务 2.2 A11 ARG 图论 10 项 + 任务 2.3 A12 多人编辑 8 项 + 任务 2.4 G1-G12 游戏化 32 项 + 任务 2.5 13 关键 class
- ARG.11 5 域 Lead 真人到位 跨 session 续 (per 守门 #14 v4 不再 trace)
- r2d2-memgraph crate 落地后切真实 Bolt (MemgraphV2EdgeSink + MemgraphAuditEventTableSink, 跨 session 续)

---

## §10 触发条件

**P2 阶段 worker 实跑入口激活条件** (per 守门 #9 v19 Mavis 自驱 + 守门 #9 v20 子代理 dispatch):

1. WSL Ubuntu 启 + Docker daemon Running
2. k3s cluster Running (至少 1 node Ready)
3. kubectl get nodes 输出 1+ Ready
4. PG 14+ container 可用 (testcontainers-rs 启或 pre-deployed)

**满足 4 条件后**: `task` tool dispatch worker sub-agent 附本 brief + 守门 #9 v27 RPC 失败 fallback 3 段.

---

**brief 落档 commit author=Ulysses** (per 守门 #10 + 守门 #14 v4)
**brief 关联 runbook commit**: `2cb6d62` (v1.01 docs sync 3 docs)
**brief 估时**: P2 阶段 worker 实跑 ~0.5-0.8M tokens / 0.42-0.67 SRE·周
